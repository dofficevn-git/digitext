// Copyright (c) 2026 DigiText. All rights reserved.
//! Chuẩn hoá + kiểm tra một dòng cl_* trước khi ghi (dùng chung cho route online, nhập Excel, push).
//! Sửa các lỗi của bản cũ: ngày kiểm theo giá trị thật, số tạm trim, STT ≥ 1, mã bảng nhỏ hợp lệ.

use super::merge::Row;
use digitext_core::chinh_ly::{codes, ClDate};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldErr {
    pub field: String,
    pub message: String,
}

impl FieldErr {
    fn new(field: &str, message: impl Into<String>) -> Self {
        FieldErr { field: field.to_owned(), message: message.into() }
    }
}

fn str_of(row: &Row, f: &str) -> Option<String> {
    match row.get(f) {
        Some(Value::String(s)) => {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_owned())
            }
        }
        Some(Value::Number(n)) => Some(n.to_string()),
        Some(Value::Bool(b)) => Some(b.to_string()),
        _ => None,
    }
}

fn int_of(row: &Row, f: &str) -> Result<Option<i64>, FieldErr> {
    match row.get(f) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Number(n)) => n.as_i64().or_else(|| n.as_f64().map(|x| x as i64)).map(Some).ok_or_else(|| FieldErr::new(f, "phải là số nguyên")),
        Some(Value::String(s)) => {
            let t = s.trim();
            if t.is_empty() {
                Ok(None)
            } else {
                t.parse::<i64>().map(Some).map_err(|_| FieldErr::new(f, format!("{t:?} không phải số nguyên")))
            }
        }
        Some(Value::Bool(_)) | Some(_) => Err(FieldErr::new(f, "phải là số nguyên")),
    }
}

/// Trim chuỗi, "" → null, số dạng chuỗi → số (cột int), bool dạng chuỗi → bool.
fn tidy_strings(row: &mut Row) {
    for (_, v) in row.iter_mut() {
        if let Value::String(s) = v {
            let t = s.trim();
            if t.is_empty() {
                *v = Value::Null;
            } else if t.len() != s.len() {
                *v = Value::String(t.to_owned());
            }
        }
    }
}

fn set_int(row: &mut Row, f: &str, errs: &mut Vec<FieldErr>, min: Option<i64>) {
    match int_of(row, f) {
        Ok(Some(n)) => {
            if let Some(m) = min {
                if n < m {
                    errs.push(FieldErr::new(f, format!("phải ≥ {m}")));
                }
            }
            row.insert(f.to_owned(), Value::from(n));
        }
        Ok(None) => {
            if row.contains_key(f) {
                row.insert(f.to_owned(), Value::Null);
            }
        }
        Err(e) => errs.push(e),
    }
}

fn set_bool(row: &mut Row, f: &str, errs: &mut Vec<FieldErr>) {
    let v = match row.get(f) {
        None | Some(Value::Null) => return,
        Some(Value::Bool(b)) => Some(*b),
        Some(Value::Number(n)) => Some(n.as_i64().unwrap_or(0) != 0),
        Some(Value::String(s)) => match s.trim().to_lowercase().as_str() {
            "1" | "true" | "x" | "có" | "co" | "yes" => Some(true),
            "0" | "false" | "" | "không" | "khong" | "no" => Some(false),
            _ => None,
        },
        _ => None,
    };
    match v {
        Some(b) => {
            row.insert(f.to_owned(), Value::Bool(b));
        }
        None => errs.push(FieldErr::new(f, "phải là có/không")),
    }
}

/// Chuẩn hoá ngày dạng chữ (dd/MM/yyyy | MM/yyyy | yyyy); trả `ClDate` để kiểm tra thứ tự.
fn set_date(row: &mut Row, f: &str, errs: &mut Vec<FieldErr>) -> Option<ClDate> {
    let Some(s) = str_of(row, f) else {
        if row.contains_key(f) {
            row.insert(f.to_owned(), Value::Null);
        }
        return None;
    };
    match ClDate::parse(&s) {
        Ok(Some(d)) => {
            row.insert(f.to_owned(), Value::String(d.to_text()));
            Some(d)
        }
        Ok(None) => {
            row.insert(f.to_owned(), Value::Null);
            None
        }
        Err(e) => {
            errs.push(FieldErr::new(f, e.to_string()));
            None
        }
    }
}

fn check_code(row: &mut Row, f: &str, table: &[(&str, &str)], errs: &mut Vec<FieldErr>) {
    if let Some(s) = str_of(row, f) {
        // Chấp nhận mã ("02") hoặc nhãn ("Sử dụng có điều kiện") → lưu mã.
        if table.iter().any(|(m, _)| *m == s) {
            return;
        }
        let low = s.to_lowercase();
        if let Some((m, _)) = table.iter().find(|(_, l)| l.to_lowercase() == low) {
            row.insert(f.to_owned(), Value::String((*m).to_owned()));
        } else {
            errs.push(FieldErr::new(f, format!("mã {s:?} không có trong bảng mã")));
        }
    }
}

fn require(row: &Row, f: &str, label: &str, errs: &mut Vec<FieldErr>) {
    if str_of(row, f).is_none() {
        errs.push(FieldErr::new(f, format!("thiếu {label}")));
    }
}

fn require_uuid(row: &Row, f: &str, label: &str, errs: &mut Vec<FieldErr>) {
    match str_of(row, f) {
        None => errs.push(FieldErr::new(f, format!("thiếu {label}"))),
        Some(s) if uuid::Uuid::parse_str(&s).is_err() => errs.push(FieldErr::new(f, format!("{label} không phải uuid"))),
        _ => {}
    }
}

fn check_uuid(row: &Row, f: &str, errs: &mut Vec<FieldErr>) {
    if let Some(s) = str_of(row, f) {
        if uuid::Uuid::parse_str(&s).is_err() {
            errs.push(FieldErr::new(f, "không phải uuid"));
        }
    }
}

/// Điền giá trị mặc định cho cột NOT NULL khi INSERT (jsonb_populate_record không áp DEFAULT của bảng).
pub fn fill_defaults(table: &str, row: &mut Row) {
    let put = |row: &mut Row, f: &str, v: Value| {
        if str_of(row, f).is_none() && !matches!(row.get(f), Some(Value::Bool(_)) | Some(Value::Number(_))) {
            row.insert(f.to_owned(), v);
        }
    };
    match table {
        "cl_ho_so" => {
            put(row, "tieu_de", Value::String(String::new()));
            put(row, "trung_thua", Value::Bool(false));
        }
        "cl_mang" => put(row, "ma_mang", Value::String("00".into())),
        "cl_hop" => put(row, "vinh_vien", Value::Bool(false)),
        "cl_thbq" => {
            put(row, "gia_tri", Value::from(0));
            put(row, "ma_tt05", Value::String("07".into()));
        }
        "cl_muon_tra" => {
            put(row, "trang_thai", Value::String("dang_muon".into()));
            put(row, "ngay_muon", Value::String(chrono::Utc::now().date_naive().to_string()));
        }
        _ => {}
    }
}

/// Chuẩn hoá tại chỗ và trả danh sách lỗi (rỗng = hợp lệ). `full` = dòng đầy đủ (INSERT):
/// kiểm trường bắt buộc; `false` = chỉ các trường có mặt (UPDATE / merge).
pub fn normalize(table: &str, row: &mut Row, full: bool) -> Vec<FieldErr> {
    let mut errs = Vec::new();
    tidy_strings(row);
    for f in ["so_tt", "gia_tri", "trang_thai", "tu_hop", "den_hop", "so", "so_thu_tu", "to_so", "so_trang", "trang_so", "tong_so_vb",
        "so_luong_trang", "ct_danh_so_tu", "ct_den_so", "ct_so_muc_luc_vb", "ct_ngay", "ct_thang", "ct_nam", "size"]
    {
        if row.contains_key(f) {
            set_int(row, f, &mut errs, None);
        }
    }
    for f in ["trung_thua", "vinh_vien"] {
        if row.contains_key(f) {
            set_bool(row, f, &mut errs);
        }
    }
    for f in ["khoi_id", "phong_id", "phong_ban_id", "mang_id", "thbq_id", "hop_id", "bo_id", "tap_id", "gia_id", "kho_id", "ho_so_id",
        "van_ban_id", "document_id", "nguoi_lap_phieu"]
    {
        check_uuid(row, f, &mut errs);
    }
    match table {
        "cl_khoi_phong" | "cl_ngon_ngu" | "cl_loai_van_ban" => {
            if full {
                require(row, "ma", "mã", &mut errs);
                require(row, "ten", "tên", &mut errs);
            }
        }
        "cl_phong" => {
            if full {
                require(row, "ma_du_an", "mã phông", &mut errs);
                require(row, "ten", "tên phông", &mut errs);
            }
        }
        "cl_phong_ban" => {
            if full {
                require_uuid(row, "phong_id", "phông", &mut errs);
                require(row, "ten", "tên phòng ban", &mut errs);
            }
        }
        "cl_mang" => {
            if full {
                require_uuid(row, "phong_ban_id", "phòng ban", &mut errs);
                require(row, "ten", "tên mảng công việc", &mut errs);
                if str_of(row, "ma_mang").is_none() {
                    row.insert("ma_mang".into(), Value::String("00".into()));
                }
            }
        }
        "cl_thbq" => {
            if full {
                require(row, "ten", "tên thời hạn bảo quản", &mut errs);
            }
            if let Ok(Some(tt)) = int_of(row, "trang_thai") {
                if !(1..=4).contains(&tt) {
                    errs.push(FieldErr::new("trang_thai", "phải là 1 (vĩnh viễn), 2 (có thời hạn), 3 (đánh giá lại) hoặc 4 (theo tuổi thọ công trình)"));
                }
            } else if full {
                errs.push(FieldErr::new("trang_thai", "thiếu loại thời hạn"));
            }
            if let (Ok(Some(tt)), Ok(gt)) = (int_of(row, "trang_thai"), int_of(row, "gia_tri")) {
                let gt = gt.unwrap_or(0);
                row.insert("gia_tri".into(), Value::from(gt));
                if str_of(row, "ma_tt05").is_none() {
                    row.insert("ma_tt05".into(), Value::String(codes::thbq_ma_tt05(tt as i16, gt as i32).into()));
                }
            }
        }
        "cl_hop" => {
            if full {
                require_uuid(row, "phong_ban_id", "phòng ban", &mut errs);
                require(row, "hop_so", "số hộp", &mut errs);
            }
        }
        "cl_bo_tap" => {
            if full {
                require_uuid(row, "phong_ban_id", "phòng ban", &mut errs);
                if !matches!(str_of(row, "loai").as_deref(), Some("bo") | Some("tap")) {
                    errs.push(FieldErr::new("loai", "phải là bo (bó) hoặc tap (tập)"));
                }
            }
            if row.contains_key("so") {
                set_int(row, "so", &mut errs, Some(1));
            } else if full {
                errs.push(FieldErr::new("so", "thiếu số bó / tập"));
            }
        }
        "cl_kho" => {
            if full {
                require(row, "ten", "tên kho", &mut errs);
            }
        }
        "cl_gia" => {
            if full {
                require_uuid(row, "kho_id", "kho", &mut errs);
                require(row, "ten", "tên giá", &mut errs);
            }
            if let (Ok(Some(a)), Ok(Some(b))) = (int_of(row, "tu_hop"), int_of(row, "den_hop")) {
                if a > b {
                    errs.push(FieldErr::new("den_hop", "đến hộp phải ≥ từ hộp"));
                }
            }
        }
        "cl_ho_so" => {
            if full {
                require_uuid(row, "phong_id", "phông", &mut errs);
                require(row, "so_tam", "số tạm", &mut errs);
                if str_of(row, "tieu_de").is_none() {
                    errs.push(FieldErr::new("tieu_de", "thiếu tiêu đề hồ sơ"));
                }
            }
            let bd = set_date(row, "ngay_bat_dau", &mut errs);
            let kt = set_date(row, "ngay_ket_thuc", &mut errs);
            if let (Some(a), Some(b)) = (bd, kt) {
                if a.sort_key() > b.sort_key() {
                    errs.push(FieldErr::new("ngay_ket_thuc", "ngày kết thúc trước ngày bắt đầu"));
                }
            }
            check_code(row, "che_do_su_dung", codes::TIEP_CAN, &mut errs);
            check_code(row, "tinh_trang_vat_ly", codes::TINH_TRANG_VAT_LY, &mut errs);
            check_code(row, "muc_do_tin_cay", codes::TIN_CAY, &mut errs);
            for f in ["ct_ngay", "ct_thang"] {
                if let Ok(Some(n)) = int_of(row, f) {
                    let max = if f == "ct_ngay" { 31 } else { 12 };
                    if !(1..=max).contains(&n) {
                        errs.push(FieldErr::new(f, format!("phải trong 1..{max}")));
                    }
                }
            }
        }
        "cl_van_ban" => {
            if full {
                require_uuid(row, "ho_so_id", "hồ sơ", &mut errs);
            }
            if row.contains_key("so_thu_tu") {
                set_int(row, "so_thu_tu", &mut errs, Some(1));
            } else if full {
                errs.push(FieldErr::new("so_thu_tu", "thiếu số thứ tự văn bản"));
            }
            set_date(row, "ngay_thang", &mut errs);
            check_code(row, "che_do_su_dung", codes::TIEP_CAN, &mut errs);
            check_code(row, "tinh_trang_vat_ly", codes::TINH_TRANG_VAT_LY, &mut errs);
            check_code(row, "do_tin_cay", codes::TIN_CAY, &mut errs);
        }
        "cl_van_ban_file" => {
            if full {
                require_uuid(row, "van_ban_id", "văn bản", &mut errs);
                require(row, "name", "tên tệp", &mut errs);
            }
            if let Some(k) = str_of(row, "kho_giay") {
                let up = k.to_uppercase();
                if codes::KHO_GIAY.contains(&up.as_str()) {
                    row.insert("kho_giay".into(), Value::String(up));
                } else {
                    errs.push(FieldErr::new("kho_giay", "khổ giấy phải là A0..A5"));
                }
            }
        }
        "cl_muon_tra" => {
            if full {
                require_uuid(row, "ho_so_id", "hồ sơ", &mut errs);
                require(row, "nguoi_muon", "người mượn", &mut errs);
            }
            for f in ["ngay_muon", "ngay_du_kien_tra", "ngay_thuc_te_tra"] {
                if let Some(s) = str_of(row, f) {
                    match ClDate::parse(&s) {
                        Ok(Some(d)) if d.day.is_some() => {
                            row.insert(f.to_owned(), Value::String(d.to_iso()));
                        }
                        _ => errs.push(FieldErr::new(f, "ngày phải đủ ngày/tháng/năm")),
                    }
                }
            }
            if let Some(tt) = str_of(row, "trang_thai") {
                if tt != "dang_muon" && tt != "da_tra" {
                    errs.push(FieldErr::new("trang_thai", "phải là dang_muon hoặc da_tra"));
                }
            }
        }
        _ => {}
    }
    errs
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn r(v: Value) -> Row {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn ho_so_chuan_hoa_ngay_va_ma() {
        let mut row = r(json!({"phong_id": "018f0000-0000-7000-8000-000000000001", "so_tam": " 12 ", "tieu_de": "A",
            "ngay_bat_dau": "5/3/2020", "ngay_ket_thuc": "2021", "so_to": "15", "trung_thua": "x",
            "che_do_su_dung": "Công khai", "ct_thang": "13"}));
        let errs = normalize("cl_ho_so", &mut row, true);
        assert_eq!(errs, vec![FieldErr::new("ct_thang", "phải trong 1..12")]);
        assert_eq!(row["so_tam"], json!("12"));
        assert_eq!(row["ngay_bat_dau"], json!("05/03/2020"));
        assert_eq!(row["ngay_ket_thuc"], json!("2021"));
        assert_eq!(row["so_to"], json!("15")); // số tờ hồ sơ là chữ (bản cũ cho phép "15a")
        assert_eq!(row["trung_thua"], json!(true));
        assert_eq!(row["che_do_su_dung"], json!("01"));
    }

    #[test]
    fn ho_so_thieu_bat_buoc_va_nguoc_ngay() {
        let mut row = r(json!({"ngay_bat_dau": "2022", "ngay_ket_thuc": "2021", "tieu_de": ""}));
        let errs = normalize("cl_ho_so", &mut row, true);
        let fields: Vec<&str> = errs.iter().map(|e| e.field.as_str()).collect();
        assert_eq!(fields, vec!["phong_id", "so_tam", "tieu_de", "ngay_ket_thuc"]);
    }

    #[test]
    fn van_ban_stt_va_ngay() {
        let mut row = r(json!({"ho_so_id": "018f0000-0000-7000-8000-000000000001", "so_thu_tu": "0", "ngay_thang": "31/02/2020"}));
        let errs = normalize("cl_van_ban", &mut row, true);
        assert_eq!(errs.len(), 2);
        let mut ok = r(json!({"so_thu_tu": 3, "ngay_thang": "3-2020", "do_tin_cay": "02"}));
        assert!(normalize("cl_van_ban", &mut ok, false).is_empty());
        assert_eq!(ok["ngay_thang"], json!("03/2020"));
    }

    #[test]
    fn thbq_tu_dien_ma_tt05() {
        let mut row = r(json!({"ten": "30 năm", "trang_thai": 2, "gia_tri": "30"}));
        assert!(normalize("cl_thbq", &mut row, true).is_empty());
        assert_eq!(row["ma_tt05"], json!("04"));
        let mut bad = r(json!({"ten": "x", "trang_thai": 9}));
        assert_eq!(normalize("cl_thbq", &mut bad, true).len(), 1);
    }

    #[test]
    fn muon_tra_ngay_iso() {
        let mut row = r(json!({"ho_so_id": "018f0000-0000-7000-8000-000000000001", "nguoi_muon": "A", "ngay_muon": "5/3/2026", "ngay_du_kien_tra": "2026"}));
        let errs = normalize("cl_muon_tra", &mut row, true);
        assert_eq!(errs.len(), 1);
        assert_eq!(row["ngay_muon"], json!("2026-03-05"));
    }
}
