// Copyright (c) 2026 DigiText. All rights reserved.
//! Deciding which local bindings are safe to rename.
//!
//! Rust makes a text-level rename dangerous in ways the compiler will not
//! always catch, so a name qualifies only when every hazard below is absent:
//!
//! * **Shadowing.** `let x = x.trim();` rebinds `x` from the old `x`. Renaming
//!   every `x` in the function would silently move the read onto the new
//!   binding, and it still compiles. Any name bound more than once in a
//!   function is therefore skipped outright.
//! * **Inline format captures.** `format!("{count}")` reads `count` from inside
//!   a string literal. Any name that appears in any string literal in the file
//!   is skipped rather than trying to rewrite format strings.
//! * **Item shadowing.** A local `foo` shadows a `fn foo` for the value
//!   namespace; renaming the local would redirect `foo(..)` to the item and
//!   still compile. Names that match any item or import in the file are skipped.

use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};

/// Names bound by patterns, counted so that shadowing can be detected.
#[derive(Default)]
pub struct Bindings {
    pub counts: HashMap<String, usize>,
}

impl<'ast> Visit<'ast> for Bindings {
    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        *self.counts.entry(node.ident.to_string()).or_default() += 1;
        visit::visit_pat_ident(self, node);
    }
}

/// Every identifier-like word appearing in a string or byte-string literal.
#[derive(Default)]
pub struct StringWords {
    pub words: HashSet<String>,
}

impl StringWords {
    fn add(&mut self, text: &str) {
        let mut cur = String::new();
        for ch in text.chars() {
            if ch.is_alphanumeric() || ch == '_' {
                cur.push(ch);
            } else if !cur.is_empty() {
                self.words.insert(std::mem::take(&mut cur));
            }
        }
        if !cur.is_empty() {
            self.words.insert(cur);
        }
    }
}

impl StringWords {
    /// Recurse into groups: in `assert_eq!(x, format!("{d}"))` the literal sits
    /// inside a nested group, so a top-level-only scan would miss `d` entirely
    /// and the capture would be left dangling by a rename.
    fn scan_tokens(&mut self, stream: proc_macro2::TokenStream) {
        for tok in stream {
            match tok {
                proc_macro2::TokenTree::Literal(lit) => self.add(&lit.to_string()),
                proc_macro2::TokenTree::Group(g) => self.scan_tokens(g.stream()),
                _ => {}
            }
        }
    }
}

impl<'ast> Visit<'ast> for StringWords {
    fn visit_lit_str(&mut self, node: &'ast syn::LitStr) {
        self.add(&node.value());
    }

    fn visit_lit_byte_str(&mut self, node: &'ast syn::LitByteStr) {
        self.add(&String::from_utf8_lossy(&node.value()));
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        // Macro bodies are opaque token streams; their literals still matter.
        self.scan_tokens(node.tokens.clone());
        visit::visit_macro(self, node);
    }
}

/// Names declared as items or brought in by `use`, at any depth in the file.
#[derive(Default)]
pub struct ItemNames {
    pub names: HashSet<String>,
}

impl ItemNames {
    fn add(&mut self, ident: &syn::Ident) {
        self.names.insert(ident.to_string());
    }
}

impl<'ast> Visit<'ast> for ItemNames {
    fn visit_item(&mut self, node: &'ast syn::Item) {
        match node {
            syn::Item::Fn(i) => self.add(&i.sig.ident),
            syn::Item::Const(i) => self.add(&i.ident),
            syn::Item::Static(i) => self.add(&i.ident),
            syn::Item::Struct(i) => self.add(&i.ident),
            syn::Item::Enum(i) => self.add(&i.ident),
            syn::Item::Union(i) => self.add(&i.ident),
            syn::Item::Trait(i) => self.add(&i.ident),
            syn::Item::TraitAlias(i) => self.add(&i.ident),
            syn::Item::Type(i) => self.add(&i.ident),
            syn::Item::Mod(i) => self.add(&i.ident),
            syn::Item::Macro(i) => {
                if let Some(id) = &i.ident {
                    self.add(id);
                }
            }
            _ => {}
        }
        visit::visit_item(self, node);
    }

    fn visit_use_tree(&mut self, node: &'ast syn::UseTree) {
        match node {
            syn::UseTree::Name(n) => self.add(&n.ident),
            syn::UseTree::Rename(n) => self.add(&n.rename),
            _ => {}
        }
        visit::visit_use_tree(self, node);
    }

    fn visit_variant(&mut self, node: &'ast syn::Variant) {
        self.add(&node.ident);
        visit::visit_variant(self, node);
    }

    fn visit_field(&mut self, node: &'ast syn::Field) {
        if let Some(id) = &node.ident {
            self.add(id);
        }
        visit::visit_field(self, node);
    }
}

/// Names this tool already produced; skipping them keeps the pass idempotent.
///
/// The `r_` prefix has to be distinctive: a bare `r` prefix would also match
/// `result`, `rows` and `req`, since base36 digits are alphanumeric too. It also
/// has to stay snake_case, or clippy's `non_snake_case` fails the `-D warnings`
/// build.
pub fn already_obfuscated(name: &str) -> bool {
    match name.strip_prefix("r_") {
        Some(tail) => !tail.is_empty() && tail.chars().all(|c| c.is_ascii_alphanumeric()),
        None => false,
    }
}

/// syn does no name resolution, so a pattern like `None => ..` reaches us as a
/// `Pat::Ident` indistinguishable from a real binding -- renaming it silently
/// turns a unit-variant match into a catch-all. Since the build runs
/// `clippy -D warnings`, `non_snake_case` guarantees every genuine local is
/// snake_case, which separates the two cleanly.
fn is_snake_case(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_lowercase())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// The bindings in one function that are safe to rename.
pub fn safe_names(
    body_bindings: &Bindings,
    strings: &StringWords,
    items: &ItemNames,
) -> HashSet<String> {
    body_bindings
        .counts
        .iter()
        .filter(|(_, count)| **count == 1)
        .map(|(name, _)| name.clone())
        .filter(|name| name != "self" && name != "_")
        .filter(|name| !name.starts_with('_')) // deliberately-unused markers
        .filter(|name| !strings.words.contains(name))
        .filter(|name| !items.names.contains(name))
        .filter(|name| is_snake_case(name))
        .filter(|name| !already_obfuscated(name))
        .collect()
}
