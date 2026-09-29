// Copyright (c) 2026 DigiText. All rights reserved.
//! Gộp 3 chiều cho đồng bộ offline: `server` (bản hiện tại trên máy chủ), `base` (bản client đã
//! kéo về trước khi sửa), `new` (bản client muốn ghi). Trường client không đổi → giữ máy chủ;
//! máy chủ không đổi → lấy client; hai bên đổi cùng giá trị → không xung đột; khác → xung đột.

use serde_json::{Map, Value};

pub type Row = Map<String, Value>;

#[derive(Debug, Clone, PartialEq)]
pub enum Merge {
    /// Bản gộp để ghi (chỉ các trường thay đổi so với server).
    Apply(Row),
    Conflict(Vec<String>),
    NoChange,
}

/// Chuẩn hoá để so sánh: chuỗi trim, "" ≡ null, số nguyên ≡ chuỗi số.
pub fn norm(v: Option<&Value>) -> Value {
    match v {
        None | Some(Value::Null) => Value::Null,
        Some(Value::String(s)) => {
            let t = s.trim();
            if t.is_empty() {
                Value::Null
            } else {
                Value::String(t.to_owned())
            }
        }
        Some(Value::Number(n)) => Value::String(n.to_string()),
        Some(Value::Bool(b)) => Value::String(b.to_string()),
        Some(other) => other.clone(),
    }
}

pub fn same(a: Option<&Value>, b: Option<&Value>) -> bool {
    norm(a) == norm(b)
}

/// `fields`: các cột được phép gộp. `new` chỉ cần chứa cột client thay đổi (cột vắng = không đổi).
pub fn three_way(server: &Row, base: &Row, new: &Row, fields: &[&str]) -> Merge {
    let mut out = Row::new();
    let mut conflicts = Vec::new();
    for f in fields {
        let s = server.get(*f);
        let b = base.get(*f);
        let Some(n) = new.get(*f) else { continue };
        let client_changed = !same(Some(n), b);
        if !client_changed {
            continue;
        }
        let server_changed = !same(s, b);
        if !server_changed {
            out.insert((*f).to_owned(), n.clone());
        } else if same(s, Some(n)) {
            // hai bên cùng đổi về một giá trị
        } else {
            conflicts.push((*f).to_owned());
        }
    }
    if !conflicts.is_empty() {
        Merge::Conflict(conflicts)
    } else if out.is_empty() {
        Merge::NoChange
    } else {
        Merge::Apply(out)
    }
}

/// Các trường trong `new` khác với `server` (dùng khi ver khớp — không cần base).
pub fn changed_fields(server: &Row, new: &Row, fields: &[&str]) -> Row {
    let mut out = Row::new();
    for f in fields {
        if let Some(n) = new.get(*f) {
            if !same(server.get(*f), Some(n)) {
                out.insert((*f).to_owned(), n.clone());
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn row(v: Value) -> Row {
        v.as_object().unwrap().clone()
    }
    const F: &[&str] = &["tieu_de", "so_to", "ghi_chu"];

    #[test]
    fn khac_truong_thi_gop() {
        let base = row(json!({"tieu_de": "A", "so_to": "1", "ghi_chu": null}));
        let server = row(json!({"tieu_de": "A2", "so_to": "1", "ghi_chu": null}));
        let new = row(json!({"tieu_de": "A", "so_to": "5", "ghi_chu": ""}));
        assert_eq!(three_way(&server, &base, &new, F), Merge::Apply(row(json!({"so_to": "5"}))));
    }

    #[test]
    fn cung_truong_khac_gia_tri_thi_xung_dot() {
        let base = row(json!({"tieu_de": "A"}));
        let server = row(json!({"tieu_de": "B"}));
        let new = row(json!({"tieu_de": "C"}));
        assert_eq!(three_way(&server, &base, &new, F), Merge::Conflict(vec!["tieu_de".into()]));
    }

    #[test]
    fn cung_gia_tri_hai_ben_khong_xung_dot() {
        let base = row(json!({"tieu_de": "A"}));
        let server = row(json!({"tieu_de": "B"}));
        let new = row(json!({"tieu_de": " B "}));
        assert_eq!(three_way(&server, &base, &new, F), Merge::NoChange);
    }

    #[test]
    fn rong_va_null_tuong_duong_so_bang_chuoi() {
        assert!(same(Some(&json!("")), None));
        assert!(same(Some(&json!("  ")), Some(&json!(null))));
        assert!(same(Some(&json!(5)), Some(&json!("5"))));
        assert!(!same(Some(&json!("a")), Some(&json!("b"))));
    }

    #[test]
    fn client_khong_gui_truong_thi_giu_server() {
        let base = row(json!({"tieu_de": "A", "so_to": "1"}));
        let server = row(json!({"tieu_de": "X", "so_to": "1"}));
        let new = row(json!({"so_to": "2"}));
        assert_eq!(three_way(&server, &base, &new, F), Merge::Apply(row(json!({"so_to": "2"}))));
    }

    #[test]
    fn truong_ngoai_danh_sach_bi_bo_qua() {
        let base = row(json!({"ho_so_so": "1"}));
        let server = row(json!({"ho_so_so": "2"}));
        let new = row(json!({"ho_so_so": "3"}));
        assert_eq!(three_way(&server, &base, &new, F), Merge::NoChange);
    }

    #[test]
    fn changed_fields_theo_server() {
        let server = row(json!({"tieu_de": "A", "so_to": "1"}));
        let new = row(json!({"tieu_de": "A ", "so_to": "2", "ghi_chu": "x"}));
        assert_eq!(changed_fields(&server, &new, F), row(json!({"so_to": "2", "ghi_chu": "x"})));
    }
}
