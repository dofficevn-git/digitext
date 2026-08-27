// Copyright (c) 2026 DigiText. All rights reserved.
//! Local-binding obfuscation for the Rust sources.
//!
//! Renaming happens on the syn AST rather than on text, so struct shorthand is
//! handled correctly: `Point { x }` becomes `Point { x: r1 }` instead of the
//! meaningless `Point { r1 }`. See `scope.rs` for the rules that decide which
//! bindings are eligible -- the short version is that anything which could
//! change meaning while still compiling is left alone.
//!
//!     rust-rename <file.rs>...            # dry run, prints a summary
//!     rust-rename --apply <file.rs>...    # rewrite in place

mod items;
mod scope;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use syn::visit::Visit;
use syn::visit_mut::{self, VisitMut};

use scope::{Bindings, ItemNames, StringWords};

/// Renames within one function body.
struct Renamer {
    map: HashMap<String, String>,
    hits: usize,
}

impl Renamer {
    fn lookup(&self, ident: &syn::Ident) -> Option<syn::Ident> {
        self.map
            .get(&ident.to_string())
            .map(|new| syn::Ident::new(new, ident.span()))
    }
}

impl VisitMut for Renamer {
    fn visit_pat_ident_mut(&mut self, node: &mut syn::PatIdent) {
        if let Some(new) = self.lookup(&node.ident) {
            node.ident = new;
            self.hits += 1;
        }
        visit_mut::visit_pat_ident_mut(self, node);
    }

    fn visit_expr_path_mut(&mut self, node: &mut syn::ExprPath) {
        // Only a bare, unqualified `name` can be a local binding.
        if node.qself.is_none() && node.path.segments.len() == 1 {
            let seg = &mut node.path.segments[0];
            if seg.arguments.is_empty() {
                if let Some(new) = self.lookup(&seg.ident) {
                    seg.ident = new;
                    self.hits += 1;
                }
            }
        }
        visit_mut::visit_expr_path_mut(self, node);
    }

    fn visit_field_pat_mut(&mut self, node: &mut syn::FieldPat) {
        // `Point { x }` binds the field `x`; renaming the binding must keep the
        // field name, so the shorthand has to be expanded to `x: r1`.
        if node.colon_token.is_none() {
            if let syn::Pat::Ident(pi) = &*node.pat {
                if self.map.contains_key(&pi.ident.to_string()) {
                    node.colon_token = Some(Default::default());
                }
            }
        }
        visit_mut::visit_field_pat_mut(self, node);
    }

    fn visit_field_value_mut(&mut self, node: &mut syn::FieldValue) {
        // Same shorthand rule on the construction side: `Point { x }`.
        if node.colon_token.is_none() {
            if let syn::Expr::Path(p) = &node.expr {
                if let Some(id) = p.path.get_ident() {
                    if self.map.contains_key(&id.to_string()) {
                        node.colon_token = Some(Default::default());
                    }
                }
            }
        }
        visit_mut::visit_field_value_mut(self, node);
    }

    fn visit_macro_mut(&mut self, node: &mut syn::Macro) {
        node.tokens = self.rewrite_tokens(node.tokens.clone());
        visit_mut::visit_macro_mut(self, node);
    }
}

impl Renamer {
    /// Rewrite idents inside a macro body, leaving `.field` and `::path` alone.
    ///
    /// Both exclusions need the *preceding* punctuation, not just the last
    /// character: `..` is two `.` tokens (a range, whose right side is an
    /// ordinary expression), and a lone `:` is a struct field separator, not a
    /// path. Treating either as a joiner leaves real uses un-renamed while the
    /// binding is renamed -- which is exactly what broke `vec![..code: req..]`
    /// and `&buf[start..at + 4]`.
    fn rewrite_tokens(&mut self, stream: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        use proc_macro2::{TokenStream, TokenTree};

        let mut out = TokenStream::new();
        let mut prev_joins = false;
        let mut last_punct: Option<char> = None;

        for tree in stream {
            let next = match tree {
                TokenTree::Group(g) => {
                    let inner = self.rewrite_tokens(g.stream());
                    let mut ng = proc_macro2::Group::new(g.delimiter(), inner);
                    ng.set_span(g.span());
                    prev_joins = false;
                    last_punct = None;
                    TokenTree::Group(ng)
                }
                TokenTree::Ident(id) => {
                    let t = if prev_joins {
                        TokenTree::Ident(id)
                    } else {
                        match self.lookup(&id) {
                            Some(new) => {
                                self.hits += 1;
                                TokenTree::Ident(new)
                            }
                            None => TokenTree::Ident(id),
                        }
                    };
                    prev_joins = false;
                    last_punct = None;
                    t
                }
                TokenTree::Punct(p) => {
                    let c = p.as_char();
                    prev_joins = match c {
                        // `.field` joins; `..range` does not.
                        '.' => last_punct != Some('.'),
                        // `::path` joins; a single `:` is a struct field.
                        ':' => last_punct == Some(':'),
                        _ => false,
                    };
                    last_punct = Some(c);
                    TokenTree::Punct(p)
                }
                TokenTree::Literal(l) => {
                    prev_joins = false;
                    last_punct = None;
                    TokenTree::Literal(l)
                }
            };
            out.extend(std::iter::once(next));
        }
        out
    }
}

/// Walks the file and renames inside each function-like body.
struct FilePass<'a> {
    strings: &'a StringWords,
    items: &'a ItemNames,
    counter: &'a mut usize,
    renamed: usize,
    hits: usize,
}

/// `#[tauri::command]` turns a parameter list into a wire contract: the
/// frontend calls `invoke("access_get", { args: { id } })` and Tauri matches the
/// payload keys against the parameter *names*. Renaming them compiles fine and
/// breaks every such call at run time, which no unit test here would catch.
fn is_ipc_boundary(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path()
            .segments
            .last()
            .is_some_and(|s| s.ident == "command")
    })
}

impl VisitMut for FilePass<'_> {
    fn visit_item_fn_mut(&mut self, node: &mut syn::ItemFn) {
        let mut sig = node.sig.clone();
        self.process_fn(
            &node.sig.clone(),
            &mut sig,
            &mut node.block,
            is_ipc_boundary(&node.attrs),
        );
        node.sig = sig;
    }

    fn visit_impl_item_fn_mut(&mut self, node: &mut syn::ImplItemFn) {
        let mut sig = node.sig.clone();
        self.process_fn(
            &node.sig.clone(),
            &mut sig,
            &mut node.block,
            is_ipc_boundary(&node.attrs),
        );
        node.sig = sig;
    }
}

impl FilePass<'_> {
    /// Parameters and body share one scope, so they are renamed together.
    fn process_fn(
        &mut self,
        sig: &syn::Signature,
        out_sig: &mut syn::Signature,
        block: &mut syn::Block,
        ipc: bool,
    ) {
        let mut bindings = Bindings::default();
        let mut params = Bindings::default();
        for arg in &sig.inputs {
            if let syn::FnArg::Typed(t) = arg {
                bindings.visit_pat(&t.pat);
                params.visit_pat(&t.pat);
            }
        }
        bindings.visit_block(block);

        let mut safe = scope::safe_names(&bindings, self.strings, self.items);
        if ipc {
            for name in params.counts.keys() {
                safe.remove(name);
            }
        }
        if safe.is_empty() {
            return;
        }

        let mut names: Vec<_> = safe.into_iter().collect();
        names.sort();
        let mut map = HashMap::new();
        for name in names {
            map.insert(name, format!("r_{}", radix36(*self.counter)));
            *self.counter += 1;
        }
        self.renamed += map.len();

        let mut r = Renamer { map, hits: 0 };
        for arg in out_sig.inputs.iter_mut() {
            if let syn::FnArg::Typed(t) = arg {
                r.visit_pat_mut(&mut t.pat);
            }
        }
        r.visit_block_mut(block);
        self.hits += r.hits;
    }
}

fn radix36(mut n: usize) -> String {
    const D: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    if n == 0 {
        return "0".into();
    }
    let mut s = Vec::new();
    while n > 0 {
        s.push(D[n % 36]);
        n /= 36;
    }
    s.reverse();
    String::from_utf8(s).unwrap()
}

/// prettyplease prints from the AST, where comments do not exist -- the license
/// header would vanish and `dev/check-license-header.sh` would fail every file.
fn leading_license(src: &str) -> String {
    src.lines()
        .take_while(|l| l.trim_start().starts_with("//"))
        .take(3)
        .filter(|l| l.contains("Copyright (c) 2026 DigiText"))
        .map(|l| format!("{l}\n"))
        .collect()
}

/// Crate name as it appears in a path root: `crates/digitext-core/..` -> `digitext_core`.
fn crate_name_of(path: &Path) -> Option<String> {
    let s = path.to_string_lossy();
    let idx = s.find("crates/")?;
    let rest = &s[idx + "crates/".len()..];
    let name = rest.split('/').next()?;
    Some(name.replace('-', "_"))
}

/// Workspace-wide item rename: collect across every file first, then rewrite.
/// A single-file view cannot tell a unique definition from one of many.
fn run_items(files: &[PathBuf], apply: bool) -> i32 {
    let local_crates: std::collections::HashSet<String> =
        files.iter().filter_map(|p| crate_name_of(p)).collect();

    // Pre-scan for module names: the use-tree rule needs them before it can
    // tell a local root from a foreign crate.
    let mut mods = items::ModuleScan::default();
    for path in files {
        if let Ok(src) = std::fs::read_to_string(path) {
            if let Ok(f) = syn::parse_file(&src) {
                mods.visit_file(&f);
            }
        }
    }

    let mut scope = items::Scope::default();
    let mut parsed = Vec::new();
    for path in files {
        let src = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                println!("  !! {}: {e}", path.display());
                return 1;
            }
        };
        match syn::parse_file(&src) {
            Ok(file) => {
                items::Collector {
                    scope: &mut scope,
                    local_crates: &local_crates,
                    local_mods: &mods.names,
                }
                .visit_file(&file);
                parsed.push((path.clone(), src, file));
            }
            Err(e) => {
                println!("  !! {}: parse: {e}", path.display());
                return 1;
            }
        }
    }

    if let Ok(name) = std::env::var("EXPLAIN") {
        println!(
            "EXPLAIN {name}: defs={:?} count={:?} disqualified={}",
            scope.defs.get(&name).map(|k| match k {
                items::Kind::Func => "fn",
                items::Kind::Konst => "const",
                items::Kind::Type => "type",
            }),
            scope.def_counts.get(&name),
            scope.disqualified.contains(&name)
        );
    }

    let mut counter = 0usize;
    let map = items::plan(&scope, &mut counter);
    println!("items defined   : {}", scope.def_counts.len());
    println!("disqualified    : {}", scope.disqualified.len());
    println!("items renamed   : {}", map.len());

    let (mut hits, mut changed) = (0usize, 0usize);
    for (path, src, file) in parsed.iter_mut() {
        let mut rw = items::Rewriter { map: &map, hits: 0 };
        rw.visit_file_mut(file);
        if rw.hits == 0 {
            continue;
        }
        hits += rw.hits;
        let out = format!("{}{}", leading_license(src), prettyplease::unparse(file));
        if syn::parse_file(&out).is_err() {
            println!("  !! {}: reparse after print failed", path.display());
            return 1;
        }
        if out != *src {
            changed += 1;
            if apply {
                if let Err(e) = std::fs::write(&*path, out) {
                    println!("  !! {}: {e}", path.display());
                    return 1;
                }
            }
        }
    }
    println!("occurrences     : {hits}");
    println!("files changed   : {changed}");
    if !apply {
        println!("dry run -- pass --apply to write.");
    }
    0
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let apply = args.iter().any(|a| a == "--apply");
    let files: Vec<PathBuf> = args
        .iter()
        .filter(|a| !a.starts_with("--"))
        .map(PathBuf::from)
        .collect();

    if args.iter().any(|a| a == "--drop-tests") {
        let (mut n, mut changed) = (0usize, 0usize);
        for path in &files {
            let src = std::fs::read_to_string(path).unwrap_or_default();
            let mut file = match syn::parse_file(&src) {
                Ok(f) => f,
                Err(e) => {
                    println!("  !! {}: parse: {e}", path.display());
                    std::process::exit(1);
                }
            };
            let mut st = items::TestStripper::default();
            st.visit_file_mut(&mut file);
            if st.removed == 0 {
                continue;
            }
            n += st.removed;
            let out = format!("{}{}", leading_license(&src), prettyplease::unparse(&file));
            if syn::parse_file(&out).is_err() {
                println!("  !! {}: reparse after print failed", path.display());
                std::process::exit(1);
            }
            changed += 1;
            if apply {
                std::fs::write(&*path, out).ok();
            }
        }
        println!("cfg(test) items removed: {n}");
        println!("files changed          : {changed}");
        if !apply {
            println!("dry run -- pass --apply to write.");
        }
        std::process::exit(0);
    }

    if args.iter().any(|a| a == "--strings") {
        let (mut n, mut changed) = (0usize, 0usize);
        for path in &files {
            let src = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(e) => {
                    println!("  !! {}: {e}", path.display());
                    std::process::exit(1);
                }
            };
            let mut file = match syn::parse_file(&src) {
                Ok(f) => f,
                Err(e) => {
                    println!("  !! {}: parse: {e}", path.display());
                    std::process::exit(1);
                }
            };
            let mut enc = items::StringEncoder::default();
            enc.visit_file_mut(&mut file);
            if enc.hits == 0 {
                continue;
            }
            n += enc.hits;
            let out = format!("{}{}", leading_license(&src), prettyplease::unparse(&file));
            if syn::parse_file(&out).is_err() {
                println!("  !! {}: reparse after print failed", path.display());
                std::process::exit(1);
            }
            if out != src {
                changed += 1;
                if apply {
                    std::fs::write(&*path, out).ok();
                }
            }
        }
        println!("literals encoded: {n}");
        println!("files changed   : {changed}");
        if !apply {
            println!("dry run -- pass --apply to write.");
        }
        std::process::exit(0);
    }

    if args.iter().any(|a| a == "--items") {
        std::process::exit(run_items(&files, apply));
    }

    let mut counter = 0usize;
    let (mut n_files, mut n_renamed, mut n_hits, mut n_changed) = (0, 0, 0, 0);
    let mut failures = Vec::new();

    for path in &files {
        match process_file(path, &mut counter, apply) {
            Ok(Some((renamed, hits, changed))) => {
                n_files += 1;
                n_renamed += renamed;
                n_hits += hits;
                n_changed += usize::from(changed);
            }
            Ok(None) => n_files += 1,
            Err(e) => failures.push(format!("{}: {e}", path.display())),
        }
    }

    println!("files parsed    : {n_files}");
    println!("bindings renamed: {n_renamed}");
    println!("occurrences     : {n_hits}");
    println!("files changed   : {n_changed}");
    for f in &failures {
        println!("  !! {f}");
    }
    if !failures.is_empty() {
        std::process::exit(1);
    }
    if !apply {
        println!("dry run -- pass --apply to write.");
    }
}

fn process_file(
    path: &Path,
    counter: &mut usize,
    apply: bool,
) -> Result<Option<(usize, usize, bool)>, String> {
    let src = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let (out, renamed, hits) = transform(&src, counter)?;
    if renamed == 0 {
        return Ok(None);
    }
    let changed = out != src;
    if changed && apply {
        std::fs::write(path, out).map_err(|e| e.to_string())?;
    }
    Ok(Some((renamed, hits, changed)))
}

/// Parse, rename, and print one file. Split out from `process_file` so the
/// tests can exercise it without touching the filesystem.
pub fn transform(src: &str, counter: &mut usize) -> Result<(String, usize, usize), String> {
    let mut file = syn::parse_file(src).map_err(|e| format!("parse: {e}"))?;

    let mut strings = StringWords::default();
    strings.visit_file(&file);
    let mut items = ItemNames::default();
    items.visit_file(&file);

    let mut pass = FilePass {
        strings: &strings,
        items: &items,
        counter,
        renamed: 0,
        hits: 0,
    };
    pass.visit_file_mut(&mut file);
    let (renamed, hits) = (pass.renamed, pass.hits);

    let out = format!("{}{}", leading_license(src), prettyplease::unparse(&file));
    // A reparse proves the printed text is still the same valid Rust.
    syn::parse_file(&out).map_err(|e| format!("reparse after print: {e}"))?;
    Ok((out, renamed, hits))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(src: &str) -> String {
        let mut c = 0;
        transform(src, &mut c).expect("transform").0
    }

    #[test]
    fn renames_a_plain_local() {
        let out = run("fn f() { let total = 1; let _ = total + 1; }");
        assert!(!out.contains("total"), "{out}");
        assert!(out.contains("r_0"), "{out}");
    }

    #[test]
    fn leaves_unit_variant_patterns_alone() {
        // syn reports `None` as a Pat::Ident; renaming it turns the arm into a
        // catch-all that still compiles but matches everything.
        let out = run("fn f(x: Option<u8>) -> u8 { match x { Some(v) => v, None => 0 } }");
        assert!(out.contains("None"), "{out}");
    }

    #[test]
    fn leaves_shadowed_names_alone() {
        let out = run("fn f(s: &str) -> usize { let s = s.trim(); let s = s.len(); s }");
        assert!(out.contains("let s"), "{out}");
    }

    #[test]
    fn protects_inline_format_captures_nested_in_a_macro() {
        // The literal lives inside `format!(..)`, itself inside `assert_eq!(..)`.
        let src = r#"fn f() { let d = 1; assert_eq!(g(d), format!("{d}.pdf")); }"#;
        let out = run(src);
        assert!(out.contains("let d"), "{out}");
    }

    #[test]
    fn renames_across_a_range_in_a_macro() {
        // `..` is two `.` tokens; treating the second as field access would
        // leave `at` un-renamed while its binding changed.
        let out =
            run(r#"fn f(buf: &[u8]) { let at = 1; assert!(matches!(&buf[0..at + 4], b"abcd")); }"#);
        assert!(!out.contains("at +"), "{out}");
    }

    #[test]
    fn renames_after_a_struct_field_colon_in_a_macro() {
        let src = "struct A { code: u8 } fn f() { let req = 1u8; let _ = vec![A { code: req }]; }";
        let out = run(src);
        assert!(!out.contains("code: req"), "{out}");
    }

    #[test]
    fn leaves_field_access_and_path_segments_alone() {
        // `len` is a local here; the `.len()` call must survive untouched even
        // though the receiver itself gets renamed.
        let src = "fn f(v: Vec<u8>) { let len = 1; assert_eq!(v.len(), len); }";
        let out = run(src);
        assert!(out.contains(".len()"), "{out}");
        assert!(!out.contains(".r_"), "{out}");
    }

    #[test]
    fn expands_struct_pattern_shorthand() {
        // The struct is declared elsewhere, so `x` is not shielded by the
        // same-file field-name rule and the shorthand really does get rewritten.
        let out = run("fn f(p: P) -> u8 { let P { x } = p; x }");
        assert!(out.contains("x: r_"), "{out}");
    }

    #[test]
    fn expands_struct_expression_shorthand() {
        let out = run("fn f() -> P { let x = 1; P { x } }");
        assert!(out.contains("x: r_"), "{out}");
    }

    #[test]
    fn a_same_file_field_name_shields_the_local() {
        // `x` is also a field of `P` here, so renaming it could redirect a
        // shorthand; the conservative rule keeps it.
        let out = run("struct P { x: u8 } fn f(p: P) -> u8 { let P { x } = p; x }");
        assert!(out.contains("let P { x }"), "{out}");
    }

    #[test]
    fn leaves_tauri_command_parameters_alone() {
        // These names are the IPC payload keys the frontend sends.
        let src = "#[tauri::command] pub fn c(args: u8) -> u8 { let acc = args; acc }";
        let out = run(src);
        assert!(out.contains("args: u8"), "{out}");
    }

    #[test]
    fn keeps_the_license_header_that_the_printer_would_drop() {
        let src = "// Copyright (c) 2026 DigiText. All rights reserved.\nfn f() { let a = 1; let _ = a; }";
        let out = run(src);
        assert!(out.starts_with("// Copyright (c) 2026 DigiText"), "{out}");
    }

    #[test]
    fn is_idempotent() {
        let src = "fn f() { let total = 1; let _ = total + 1; }";
        let once = run(&src.to_string());
        let mut c = 0;
        let twice = transform(&once, &mut c).expect("transform").0;
        assert_eq!(once, twice);
    }

    #[test]
    fn skips_underscore_prefixed_markers() {
        let out = run("fn f() { let _unused = 1; }");
        assert!(out.contains("_unused"), "{out}");
    }
}
