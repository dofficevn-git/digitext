// Copyright (c) 2026 DigiText. All rights reserved.
//! Cây XML tối giản, tên phần tử / thuộc tính lấy TÊN CỤC BỘ (bỏ tiền tố namespace): gói của phần mềm khác
//! viết `mets:mets`, `premis:object`, `xlink:href`… — kiểm theo tên cục bộ thì không lệ thuộc tiền tố.
use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

#[derive(Debug, Clone, Default)]
pub struct El {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<El>,
    pub text: String,
}

fn local(b: &[u8]) -> String {
    let s = String::from_utf8_lossy(b);
    match s.rsplit_once(':') {
        Some((_, l)) => l.to_string(),
        None => s.into_owned(),
    }
}

impl El {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.iter().find(|(k, _)| k == name).map(|(_, v)| v.as_str())
    }
    pub fn child(&self, name: &str) -> Option<&El> {
        self.children.iter().find(|c| c.name == name)
    }
    /// Chữ của phần tử con trực tiếp (đã cắt khoảng trắng); không có → None, rỗng → Some("").
    pub fn child_text(&self, name: &str) -> Option<String> {
        self.child(name).map(|c| c.text.trim().to_string())
    }
    /// Mọi phần tử con cháu có tên `name` (kể cả chính nó).
    pub fn all(&self, name: &str) -> Vec<&El> {
        let mut out = Vec::new();
        let mut st = vec![self];
        while let Some(e) = st.pop() {
            if e.name == name {
                out.push(e);
            }
            for c in e.children.iter().rev() {
                st.push(c);
            }
        }
        out
    }
}

fn start(e: &BytesStart<'_>) -> El {
    let mut el = El { name: local(e.name().as_ref()), ..Default::default() };
    for a in e.attributes().flatten() {
        let k = local(a.key.as_ref());
        let v = a.unescape_value().map(|v| v.into_owned()).unwrap_or_else(|_| String::from_utf8_lossy(&a.value).into_owned());
        el.attrs.push((k, v));
    }
    el
}

pub fn parse(bytes: &[u8]) -> Result<El, String> {
    let bytes = bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes);
    let mut r = Reader::from_reader(bytes);
    r.config_mut().check_end_names = true;
    let mut buf = Vec::new();
    let mut stack: Vec<El> = Vec::new();
    let mut root: Option<El> = None;
    loop {
        let ev = r.read_event_into(&mut buf).map_err(|e| format!("vị trí {}: {e}", r.buffer_position()))?;
        match ev {
            Event::Start(e) => stack.push(start(&e)),
            Event::Empty(e) => {
                let el = start(&e);
                match stack.last_mut() {
                    Some(p) => p.children.push(el),
                    None => root = Some(el),
                }
            }
            Event::End(_) => {
                let el = stack.pop().ok_or("thẻ đóng thừa")?;
                match stack.last_mut() {
                    Some(p) => p.children.push(el),
                    None => root = Some(el),
                }
            }
            Event::Text(t) => {
                if let Some(p) = stack.last_mut() {
                    let s = t.unescape().map(|s| s.into_owned()).unwrap_or_else(|_| String::from_utf8_lossy(&t).into_owned());
                    p.text.push_str(&s);
                }
            }
            Event::CData(t) => {
                if let Some(p) = stack.last_mut() {
                    p.text.push_str(&String::from_utf8_lossy(&t));
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }
    if !stack.is_empty() {
        return Err("thiếu thẻ đóng".into());
    }
    root.ok_or_else(|| "tệp rỗng".into())
}

#[cfg(test)]
mod tests {
    #[test]
    fn bo_tien_to_namespace() {
        let x = br#"<?xml version="1.0"?><mets:mets xmlns:mets="m" xmlns:xlink="x" OBJID="a"><mets:file SIZE="3"><mets:FLocat xlink:href="data/a%20b.pdf"/></mets:file><t>A &amp; B</t></mets:mets>"#;
        let r = super::parse(x).unwrap();
        assert_eq!(r.name, "mets");
        assert_eq!(r.attr("OBJID"), Some("a"));
        assert_eq!(r.all("FLocat")[0].attr("href"), Some("data/a%20b.pdf"));
        assert_eq!(r.child_text("t").as_deref(), Some("A & B"));
        assert!(super::parse(b"<a><b></a>").is_err());
    }
}
