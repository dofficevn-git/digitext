// Copyright (c) 2026 DigiText. All rights reserved.
//! Workspace-wide renaming of module-level items.
//!
//! `syn` performs no name resolution, so this pass cannot ask "what does this
//! identifier refer to". It compensates by only touching a name that is
//! **defined exactly once in the whole scope** and cannot plausibly mean
//! anything else. A name is disqualified when it is also:
//!
//! * an impl or trait method -- called as `x.name()`, and for a trait it is the
//!   trait's contract, not ours to rename;
//! * a struct field or enum variant;
//! * a local binding anywhere;
//! * ever written after a `.`, which makes it a field or method somewhere;
//! * a word inside any string literal (sqlx column refs, serde renames,
//!   dynamic lookups);
//! * the name of a `#[tauri::command]` -- that is the IPC wire contract;
//! * a segment of a path rooted at an external crate, e.g. `mem::take`. Renaming
//!   `take` there would rewrite the call into the standard library.
//!
//! Modules are excluded outright: `mod foo;` is bound to the name of `foo.rs`,
//! so renaming one means renaming files.

use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

#[derive(Default)]
pub struct Scope {
    /// Item name -> how many definitions were seen.
    pub defs: HashMap<String, Kind>,
    pub def_counts: HashMap<String, usize>,
    pub disqualified: HashSet<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Func,
    Konst,
    Type,
}

fn is_command(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        a.path()
            .segments
            .last()
            .is_some_and(|s| s.ident == "command")
    })
}

impl Scope {
    fn define(&mut self, ident: &syn::Ident, kind: Kind) {
        let name = ident.to_string();
        *self.def_counts.entry(name.clone()).or_default() += 1;
        self.defs.insert(name, kind);
    }

    fn ban(&mut self, name: impl Into<String>) {
        self.disqualified.insert(name.into());
    }

    fn ban_words(&mut self, text: &str) {
        let mut cur = String::new();
        for ch in text.chars() {
            if ch.is_alphanumeric() || ch == '_' {
                cur.push(ch);
            } else if !cur.is_empty() {
                self.disqualified.insert(std::mem::take(&mut cur));
            }
        }
        if !cur.is_empty() {
            self.disqualified.insert(cur);
        }
    }

    fn ban_tokens(&mut self, stream: proc_macro2::TokenStream) {
        for tok in stream {
            match tok {
                proc_macro2::TokenTree::Literal(l) => {
                    let s = l.to_string();
                    self.ban_words(&s);
                }
                proc_macro2::TokenTree::Group(g) => self.ban_tokens(g.stream()),
                _ => {}
            }
        }
    }
}

/// Crate-relative path roots. Anything else is a foreign API.
const LOCAL_ROOTS: &[&str] = &["crate", "self", "super", "Self"];

/// Module names, gathered before the main pass. A `use inner::thing;` names a
/// local module as its root, and without knowing that up front the foreign-root
/// rule would wrongly treat the whole tree as somebody else's API.
#[derive(Default)]
pub struct ModuleScan {
    pub names: HashSet<String>,
}

impl<'ast> Visit<'ast> for ModuleScan {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        self.names.insert(node.ident.to_string());
        visit::visit_item_mod(self, node);
    }
}

pub struct Collector<'a> {
    pub scope: &'a mut Scope,
    pub local_crates: &'a HashSet<String>,
    pub local_mods: &'a HashSet<String>,
}

impl<'ast> Visit<'ast> for Collector<'_> {
    fn visit_item(&mut self, node: &'ast syn::Item) {
        match node {
            syn::Item::Fn(i) => {
                if is_command(&i.attrs) {
                    self.scope.ban(i.sig.ident.to_string());
                } else {
                    self.scope.define(&i.sig.ident, Kind::Func);
                }
            }
            syn::Item::Struct(i) => self.scope.define(&i.ident, Kind::Type),
            syn::Item::Enum(i) => self.scope.define(&i.ident, Kind::Type),
            syn::Item::Type(i) => self.scope.define(&i.ident, Kind::Type),
            syn::Item::Trait(i) => self.scope.define(&i.ident, Kind::Type),
            syn::Item::Const(i) => self.scope.define(&i.ident, Kind::Konst),
            syn::Item::Static(i) => self.scope.define(&i.ident, Kind::Konst),
            // A module name is tied to its file name.
            syn::Item::Mod(i) => self.scope.ban(i.ident.to_string()),
            syn::Item::Union(i) => self.scope.ban(i.ident.to_string()),
            _ => {}
        }
        visit::visit_item(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.scope.ban(node.sig.ident.to_string());
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.scope.ban(node.sig.ident.to_string());
        visit::visit_trait_item_fn(self, node);
    }

    fn visit_field(&mut self, node: &'ast syn::Field) {
        if let Some(id) = &node.ident {
            self.scope.ban(id.to_string());
        }
        visit::visit_field(self, node);
    }

    fn visit_variant(&mut self, node: &'ast syn::Variant) {
        self.scope.ban(node.ident.to_string());
        visit::visit_variant(self, node);
    }

    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.scope.ban(node.ident.to_string());
        visit::visit_pat_ident(self, node);
    }

    fn visit_expr_field(&mut self, node: &'ast syn::ExprField) {
        if let syn::Member::Named(id) = &node.member {
            self.scope.ban(id.to_string());
        }
        visit::visit_expr_field(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.scope.ban(node.method.to_string());
        visit::visit_expr_method_call(self, node);
    }

    fn visit_lit_str(&mut self, node: &'ast syn::LitStr) {
        let v = node.value();
        self.scope.ban_words(&v);
    }

    fn visit_attribute(&mut self, node: &'ast syn::Attribute) {
        // A derive helper can name a function as a STRING:
        // `#[asn1(default = "bool_false")]`. Attribute arguments are raw tokens,
        // never parsed into LitStr nodes, so visit_lit_str never sees them.
        if let syn::Meta::List(l) = &node.meta {
            self.scope.ban_tokens(l.tokens.clone());
        }
        visit::visit_attribute(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if let Some(seg) = node.path.segments.last() {
            self.scope.ban(seg.ident.to_string());
        }
        self.scope.ban_tokens(node.tokens.clone());
        visit::visit_macro(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        // `mem::take(..)` reaches into another crate; renaming `take` would
        // rewrite the call target. Ban every segment of a foreign-rooted path.
        if node.segments.len() > 1 {
            let root = node.segments[0].ident.to_string();
            let local = LOCAL_ROOTS.contains(&root.as_str())
                || self.local_crates.contains(&root)
                || self.local_mods.contains(&root);
            if !local {
                for seg in &node.segments {
                    self.scope.disqualified.insert(seg.ident.to_string());
                }
            }
        }
        visit::visit_path(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        // A `use` tree is not a syn::Path, so the foreign-root rule above never
        // reaches it. `use cryptoki::context::Pkcs11` would otherwise let an
        // external module called `context` be renamed.
        fn root(tree: &syn::UseTree) -> Option<String> {
            match tree {
                syn::UseTree::Path(p) => Some(p.ident.to_string()),
                syn::UseTree::Name(n) => Some(n.ident.to_string()),
                syn::UseTree::Rename(r) => Some(r.ident.to_string()),
                _ => None,
            }
        }
        fn ban_all(tree: &syn::UseTree, scope: &mut Scope) {
            match tree {
                syn::UseTree::Path(p) => {
                    scope.ban(p.ident.to_string());
                    ban_all(&p.tree, scope);
                }
                syn::UseTree::Name(n) => scope.ban(n.ident.to_string()),
                syn::UseTree::Rename(r) => {
                    scope.ban(r.ident.to_string());
                    scope.ban(r.rename.to_string());
                }
                syn::UseTree::Group(g) => {
                    for t in &g.items {
                        ban_all(t, scope);
                    }
                }
                syn::UseTree::Glob(_) => {}
            }
        }

        let r = root(&node.tree).unwrap_or_default();
        let local = LOCAL_ROOTS.contains(&r.as_str())
            || self.local_crates.contains(&r)
            || self.local_mods.contains(&r);
        if !local {
            ban_all(&node.tree, self.scope);
        } else if let syn::UseTree::Rename(rn) = &node.tree {
            self.scope.ban(rn.rename.to_string());
        }
        visit::visit_item_use(self, node);
    }

    fn visit_type_param(&mut self, node: &'ast syn::TypeParam) {
        // `fn f<T>(v: Vec<T>)` -- a generic parameter is a path segment too, so
        // an item that happens to share its name would rewrite the parameter.
        self.scope.ban(node.ident.to_string());
        visit::visit_type_param(self, node);
    }

    fn visit_const_param(&mut self, node: &'ast syn::ConstParam) {
        self.scope.ban(node.ident.to_string());
        visit::visit_const_param(self, node);
    }
}

/// Names this pass already produced.
pub fn already_renamed(name: &str) -> bool {
    let rest = name
        .strip_prefix("f_")
        .or_else(|| name.strip_prefix("C_"))
        .or_else(|| name.strip_prefix('T'));
    matches!(rest, Some(t) if !t.is_empty() && t.chars().all(|c| c.is_ascii_alphanumeric()))
        && name.len() <= 6
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

/// Build the rename map. Casing follows Rust lints: `clippy -D warnings` fails a
/// build on `non_snake_case`, `non_upper_case_globals` or `non_camel_case_types`.
pub fn plan(scope: &Scope, counter: &mut usize) -> HashMap<String, String> {
    let mut names: Vec<_> = scope
        .def_counts
        .iter()
        .filter(|(_, c)| **c == 1)
        .map(|(n, _)| n.clone())
        .filter(|n| !scope.disqualified.contains(n))
        .filter(|n| n != "main")
        .filter(|n| !already_renamed(n))
        .collect();
    names.sort(); // deterministic across runs

    let mut map = HashMap::new();
    for name in names {
        let new = match scope.defs.get(&name) {
            Some(Kind::Func) => format!("f_{}", radix36(*counter)),
            Some(Kind::Konst) => format!("C_{}", radix36(*counter).to_uppercase()),
            Some(Kind::Type) => format!("T{}", radix36(*counter)),
            None => continue,
        };
        *counter += 1;
        map.insert(name, new);
    }
    map
}

pub struct Rewriter<'a> {
    pub map: &'a HashMap<String, String>,
    pub hits: usize,
}

impl Rewriter<'_> {
    fn swap(&mut self, ident: &mut syn::Ident) {
        if let Some(new) = self.map.get(&ident.to_string()) {
            *ident = syn::Ident::new(new, ident.span());
            self.hits += 1;
        }
    }

    /// Rewrite idents inside a macro body, leaving `.field` alone. `::` is kept
    /// because a path segment there may well be one of our items.
    fn tokens(&mut self, stream: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        use proc_macro2::{Spacing, TokenStream, TokenTree};

        // Lookahead is needed, so materialise the stream first.
        let trees: Vec<TokenTree> = stream.into_iter().collect();
        let mut out = TokenStream::new();
        let mut after_dot = false;
        let mut last_punct: Option<char> = None;

        for (i, tree) in trees.iter().enumerate() {
            let next = match tree.clone() {
                TokenTree::Group(g) => {
                    let inner = self.tokens(g.stream());
                    let mut ng = proc_macro2::Group::new(g.delimiter(), inner);
                    ng.set_span(g.span());
                    after_dot = false;
                    last_punct = None;
                    TokenTree::Group(ng)
                }
                TokenTree::Ident(mut id) => {
                    // A struct literal inside `vec![Foo { shape: .. }]` reaches
                    // us as raw tokens, where a field name is indistinguishable
                    // from a value reference. An ident directly before a single
                    // `:` is a field name (or a label), never our item.
                    let field_name = matches!(
                        trees.get(i + 1),
                        Some(TokenTree::Punct(p))
                            if p.as_char() == ':' && p.spacing() == Spacing::Alone
                    );
                    if !after_dot && !field_name {
                        self.swap(&mut id);
                    }
                    after_dot = false;
                    last_punct = None;
                    TokenTree::Ident(id)
                }
                TokenTree::Punct(p) => {
                    // `.field` joins; `..range` does not.
                    after_dot = p.as_char() == '.' && last_punct != Some('.');
                    last_punct = Some(p.as_char());
                    TokenTree::Punct(p)
                }
                TokenTree::Literal(l) => {
                    after_dot = false;
                    last_punct = None;
                    TokenTree::Literal(l)
                }
            };
            out.extend(std::iter::once(next));
        }
        out
    }
}

impl VisitMut for Rewriter<'_> {
    fn visit_item_fn_mut(&mut self, node: &mut syn::ItemFn) {
        self.swap(&mut node.sig.ident);
        visit_mut::visit_item_fn_mut(self, node);
    }
    fn visit_item_struct_mut(&mut self, node: &mut syn::ItemStruct) {
        self.swap(&mut node.ident);
        visit_mut::visit_item_struct_mut(self, node);
    }
    fn visit_item_enum_mut(&mut self, node: &mut syn::ItemEnum) {
        self.swap(&mut node.ident);
        visit_mut::visit_item_enum_mut(self, node);
    }
    fn visit_item_type_mut(&mut self, node: &mut syn::ItemType) {
        self.swap(&mut node.ident);
        visit_mut::visit_item_type_mut(self, node);
    }
    fn visit_item_trait_mut(&mut self, node: &mut syn::ItemTrait) {
        self.swap(&mut node.ident);
        visit_mut::visit_item_trait_mut(self, node);
    }
    fn visit_item_const_mut(&mut self, node: &mut syn::ItemConst) {
        self.swap(&mut node.ident);
        visit_mut::visit_item_const_mut(self, node);
    }
    fn visit_item_static_mut(&mut self, node: &mut syn::ItemStatic) {
        self.swap(&mut node.ident);
        visit_mut::visit_item_static_mut(self, node);
    }

    fn visit_path_segment_mut(&mut self, node: &mut syn::PathSegment) {
        self.swap(&mut node.ident);
        visit_mut::visit_path_segment_mut(self, node);
    }

    fn visit_use_tree_mut(&mut self, node: &mut syn::UseTree) {
        // A `use` tree is not made of PathSegments, so `pub use text::{foo}`
        // is invisible to visit_path_segment_mut and the re-export would keep
        // pointing at the old name.
        match node {
            syn::UseTree::Name(n) => self.swap(&mut n.ident),
            // `ident` is the real item; `rename` is an alias, already banned.
            syn::UseTree::Rename(r) => self.swap(&mut r.ident),
            // Intermediate segments are always a module or a crate, and both
            // are excluded from renaming, so nothing here should ever change.
            syn::UseTree::Path(_) => {}
            _ => {}
        }
        visit_mut::visit_use_tree_mut(self, node);
    }

    fn visit_macro_mut(&mut self, node: &mut syn::Macro) {
        node.tokens = self.tokens(node.tokens.clone());
        visit_mut::visit_macro_mut(self, node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::visit::Visit as _;
    use syn::visit_mut::VisitMut as _;

    /// Run the two-phase pass over a set of files, as the driver does.
    fn run(srcs: &[&str]) -> Vec<String> {
        let crates: HashSet<String> = ["mycrate".to_string()].into_iter().collect();
        let mut scope = Scope::default();
        let mut files: Vec<syn::File> = srcs
            .iter()
            .map(|s| syn::parse_file(s).expect("parse"))
            .collect();
        let mut mods = ModuleScan::default();
        for f in &files {
            mods.visit_file(f);
        }
        for f in &files {
            Collector {
                scope: &mut scope,
                local_crates: &crates,
                local_mods: &mods.names,
            }
            .visit_file(f);
        }
        let mut counter = 0usize;
        let map = plan(&scope, &mut counter);
        files
            .iter_mut()
            .map(|f| {
                Rewriter { map: &map, hits: 0 }.visit_file_mut(f);
                prettyplease::unparse(f)
            })
            .collect()
    }

    fn one(src: &str) -> String {
        run(&[src]).remove(0)
    }

    #[test]
    fn renames_a_unique_free_function() {
        let out = one("fn lonely_helper() {}\nfn main() { lonely_helper(); }");
        assert!(!out.contains("lonely_helper"), "{out}");
        assert!(out.contains("fn f_"), "{out}");
    }

    #[test]
    fn keeps_casing_conventions_so_clippy_stays_quiet() {
        let out = one("const LIMIT_X: u8 = 1;\nstruct WidgetX;\nfn helper_x() {}\nfn main() { let _ = (LIMIT_X, WidgetX, helper_x()); }");
        assert!(out.contains("const C_"), "{out}"); // SCREAMING for globals
        assert!(out.contains("struct T"), "{out}"); // CamelCase for types
        assert!(out.contains("fn f_"), "{out}"); // snake_case for fns
    }

    #[test]
    fn rewrites_a_reexport_in_a_use_tree() {
        // `pub use` is a UseTree, not a Path, so it needs its own handling or
        // the re-export keeps naming something that no longer exists.
        let out = one("mod inner { pub fn only_here() {} }\npub use inner::only_here;");
        assert!(!out.contains("only_here"), "{out}");
    }

    #[test]
    fn leaves_a_foreign_use_tree_alone() {
        // `context` is a module of an external crate here, not our item.
        let out = one("fn context() {}\nuse cryptoki::context::Pkcs11;\nfn main() { context(); let _: Option<Pkcs11> = None; }");
        assert!(out.contains("cryptoki::context::Pkcs11"), "{out}");
    }

    #[test]
    fn leaves_generic_parameters_alone() {
        let out = one("const T: u8 = 1;\nfn f<T>(v: Vec<T>) -> Vec<T> { v }\nfn main() { let _ = T; }");
        assert!(out.contains("Vec<T>"), "{out}");
    }

    #[test]
    fn leaves_a_struct_field_named_in_a_macro_alone() {
        // Inside `vec![Foo { shape: .. }]` the field name is a bare token,
        // indistinguishable from a value reference.
        let out = one("fn shape() -> u8 { 1 }\nfn main() { let _ = vec![Tensor { shape: 3 }]; }");
        // prettyplease re-spaces macro tokens, so match on the name alone.
        assert!(out.contains("shape"), "{out}");
        assert!(!out.contains("f_0 :") && !out.contains("f_0:"), "{out}");
    }

    #[test]
    fn leaves_a_function_named_by_an_attribute_string_alone() {
        // `#[asn1(default = "bool_false")]` names a function as text.
        let out = one("fn bool_false() -> bool { false }\nstruct S {\n    #[asn1(default = \"bool_false\")]\n    v: bool,\n}");
        assert!(out.contains("fn bool_false"), "{out}");
    }

    #[test]
    fn leaves_tauri_command_names_alone() {
        // The name is the string the frontend passes to `invoke`.
        let out = one("#[tauri::command]\npub fn access_get() {}");
        assert!(out.contains("fn access_get"), "{out}");
    }

    #[test]
    fn leaves_impl_and_trait_methods_alone() {
        let out = one("struct S;\nimpl S { fn helper(&self) {} }\nfn main() { S.helper(); }");
        assert!(out.contains("fn helper"), "{out}");
    }

    #[test]
    fn leaves_duplicated_definitions_alone() {
        // Two definitions of `parse` mean a rename cannot be resolved safely.
        let out = one("mod a { pub fn parse() {} }\nmod b { pub fn parse() {} }");
        assert!(out.matches("fn parse").count() == 2, "{out}");
    }

    #[test]
    fn renames_consistently_across_files() {
        let outs = run(&[
            "pub fn shared_thing() {}",
            "use mycrate::shared_thing;\nfn main() { shared_thing(); }",
        ]);
        assert!(!outs[0].contains("shared_thing"), "{}", outs[0]);
        assert!(!outs[1].contains("shared_thing"), "{}", outs[1]);
        let new = outs[0]
            .split("fn ")
            .nth(1)
            .unwrap()
            .split('(')
            .next()
            .unwrap()
            .to_string();
        assert!(outs[1].contains(&new), "{} vs {}", outs[1], new);
    }

    #[test]
    fn leaves_a_name_quoted_in_a_string_alone() {
        let out = one("fn column_name() {}\nfn main() { let _ = \"column_name\"; }");
        assert!(out.contains("fn column_name"), "{out}");
    }

    #[test]
    fn is_idempotent() {
        let src = "fn lonely_helper() {}\nfn main() { lonely_helper(); }";
        let once = one(src);
        assert_eq!(one(&once), once);
    }
}

// ------------------------------------------------------------- strings

/// Base64 of the UTF-8 bytes. Deterministic, so the same text always yields the
/// same token and comparisons between two encoded strings still hold.
pub fn b64(s: &str) -> String {
    const T: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

const VIET: &str = "àáâãèéêìíòóôõùúỳăđĩũơưạảấầẩẫậắằẳẵặẹẻẽếềểễệỉịọỏốồổỗộớờởỡợụủứừửữựỵỷỹ";

fn has_viet(s: &str) -> bool {
    s.chars()
        .any(|c| VIET.contains(c) || VIET.contains(c.to_lowercase().next().unwrap_or(c)))
}

fn already_b64(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
}

/// Encode the literal runs of a format string, leaving `{...}` spans and the
/// `{{`/`}}` escapes untouched, so the placeholder structure -- and therefore
/// the macro's argument count -- is preserved exactly.
pub fn encode_format(s: &str) -> String {
    let mut out = String::new();
    let mut run = String::new();
    let mut chars = s.chars().peekable();

    let flush = |run: &mut String, out: &mut String| {
        if !run.is_empty() {
            if has_viet(run) {
                out.push_str(&b64(run));
            } else {
                out.push_str(run);
            }
            run.clear();
        }
    };

    while let Some(c) = chars.next() {
        match c {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                run.push_str("{{");
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                run.push_str("}}");
            }
            '{' => {
                flush(&mut run, &mut out);
                out.push('{');
                for d in chars.by_ref() {
                    out.push(d);
                    if d == '}' {
                        break;
                    }
                }
            }
            _ => run.push(c),
        }
    }
    flush(&mut run, &mut out);
    out
}

fn encodable(s: &str) -> bool {
    has_viet(s) && !already_b64(s)
}

#[derive(Default)]
pub struct StringEncoder {
    pub hits: usize,
}

impl StringEncoder {
    fn tokens(&mut self, stream: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        use proc_macro2::{TokenStream, TokenTree};
        let mut out = TokenStream::new();
        for tree in stream {
            let next = match tree {
                TokenTree::Group(g) => {
                    let inner = self.tokens(g.stream());
                    let mut ng = proc_macro2::Group::new(g.delimiter(), inner);
                    ng.set_span(g.span());
                    TokenTree::Group(ng)
                }
                TokenTree::Literal(l) => {
                    let repr = l.to_string();
                    match syn::parse_str::<syn::LitStr>(&repr) {
                        Ok(ls) if encodable(&ls.value()) => {
                            self.hits += 1;
                            let mut new = proc_macro2::Literal::string(&encode_format(&ls.value()));
                            new.set_span(l.span());
                            TokenTree::Literal(new)
                        }
                        _ => TokenTree::Literal(l),
                    }
                }
                other => other,
            };
            out.extend(std::iter::once(next));
        }
        out
    }
}

impl VisitMut for StringEncoder {
    fn visit_lit_str_mut(&mut self, node: &mut syn::LitStr) {
        let v = node.value();
        if encodable(&v) {
            self.hits += 1;
            *node = syn::LitStr::new(&encode_format(&v), node.span());
        }
    }

    fn visit_macro_mut(&mut self, node: &mut syn::Macro) {
        node.tokens = self.tokens(node.tokens.clone());
        visit_mut::visit_macro_mut(self, node);
    }

    fn visit_attribute_mut(&mut self, node: &mut syn::Attribute) {
        // `#[error("…")]` and friends hold raw tokens, not LitStr nodes.
        if let syn::Meta::List(l) = &mut node.meta {
            l.tokens = self.tokens(l.tokens.clone());
        }
        visit_mut::visit_attribute_mut(self, node);
    }
}

// ----------------------------------------------------------- drop tests

fn is_cfg_test(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        if !a.path().is_ident("cfg") {
            return false;
        }
        // `#[cfg(test)]` and `#[cfg(all(test, ...))]` both count.
        a.meta
            .require_list()
            .map(|l| l.tokens.to_string().split(|c: char| !c.is_alphanumeric()).any(|t| t == "test"))
            .unwrap_or(false)
    })
}

/// Removes every `#[cfg(test)]` item: the `mod tests` blocks and any helper
/// `use`/`fn` that exists only for them.
#[derive(Default)]
pub struct TestStripper {
    pub removed: usize,
}

impl VisitMut for TestStripper {
    fn visit_file_mut(&mut self, node: &mut syn::File) {
        let before = node.items.len();
        node.items.retain(|i| !item_is_cfg_test(i));
        self.removed += before - node.items.len();
        visit_mut::visit_file_mut(self, node);
    }

    fn visit_item_mod_mut(&mut self, node: &mut syn::ItemMod) {
        if let Some((_, items)) = &mut node.content {
            let before = items.len();
            items.retain(|i| !item_is_cfg_test(i));
            self.removed += before - items.len();
        }
        visit_mut::visit_item_mod_mut(self, node);
    }

    fn visit_item_impl_mut(&mut self, node: &mut syn::ItemImpl) {
        let before = node.items.len();
        node.items.retain(|i| match i {
            syn::ImplItem::Fn(f) => !is_cfg_test(&f.attrs),
            syn::ImplItem::Const(c) => !is_cfg_test(&c.attrs),
            _ => true,
        });
        self.removed += before - node.items.len();
        visit_mut::visit_item_impl_mut(self, node);
    }
}

fn item_is_cfg_test(item: &syn::Item) -> bool {
    match item {
        syn::Item::Mod(i) => is_cfg_test(&i.attrs),
        syn::Item::Fn(i) => is_cfg_test(&i.attrs),
        syn::Item::Use(i) => is_cfg_test(&i.attrs),
        syn::Item::Const(i) => is_cfg_test(&i.attrs),
        syn::Item::Static(i) => is_cfg_test(&i.attrs),
        syn::Item::Struct(i) => is_cfg_test(&i.attrs),
        syn::Item::Enum(i) => is_cfg_test(&i.attrs),
        syn::Item::Impl(i) => is_cfg_test(&i.attrs),
        syn::Item::Type(i) => is_cfg_test(&i.attrs),
        syn::Item::Trait(i) => is_cfg_test(&i.attrs),
        syn::Item::Macro(i) => is_cfg_test(&i.attrs),
        _ => false,
    }
}
