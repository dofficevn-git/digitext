// Copyright (c) 2026 DigiText. All rights reserved.
pub fn esc(r_44p: &str) -> String {
    let mut out = String::with_capacity(r_44p.len());
    for c in r_44p.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}
pub struct Xml {
    out: String,
    stack: Vec<String>,
}
impl Xml {
    pub fn new() -> Self {
        Self {
            out: String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"),
            stack: Vec::new(),
        }
    }
    fn write_attrs(&mut self, r_44q: &[(&str, &str)]) {
        for (r_44r, v) in r_44q {
            self.out.push(' ');
            self.out.push_str(r_44r);
            self.out.push_str("=\"");
            self.out.push_str(&esc(v));
            self.out.push('"');
        }
    }
    pub fn open(&mut self, r_44t: &str, r_44s: &[(&str, &str)]) {
        self.out.push('<');
        self.out.push_str(r_44t);
        self.write_attrs(r_44s);
        self.out.push_str(">\n");
        self.stack.push(r_44t.to_owned());
    }
    pub fn empty(&mut self, r_44v: &str, r_44u: &[(&str, &str)]) {
        self.out.push('<');
        self.out.push_str(r_44v);
        self.write_attrs(r_44u);
        self.out.push_str("/>\n");
    }
    pub fn text_elem(&mut self, r_44w: &str, r_44x: &str) {
        self.out.push('<');
        self.out.push_str(r_44w);
        self.out.push('>');
        self.out.push_str(&esc(r_44x));
        self.out.push_str("</");
        self.out.push_str(r_44w);
        self.out.push_str(">\n");
    }
    pub fn text_elem_attrs(&mut self, r_44z: &str, r_44y: &[(&str, &str)], r_450: &str) {
        self.out.push('<');
        self.out.push_str(r_44z);
        self.write_attrs(r_44y);
        self.out.push('>');
        self.out.push_str(&esc(r_450));
        self.out.push_str("</");
        self.out.push_str(r_44z);
        self.out.push_str(">\n");
    }
    pub fn close(&mut self, r_451: &str) {
        let r_452 = self.stack.pop();
        debug_assert_eq!(r_452.as_deref(), Some(r_451), "unbalanced XML close");
        self.out.push_str("</");
        self.out.push_str(r_451);
        self.out.push_str(">\n");
    }
    pub fn finish(self) -> String {
        debug_assert!(
            self.stack.is_empty(),
            "unclosed XML elements: {:?}",
            self.stack
        );
        self.out
    }
}
