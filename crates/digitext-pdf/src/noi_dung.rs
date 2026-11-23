// Copyright (c) 2026 DigiText. All rights reserved.
//! Đọc **lệnh vẽ** của một trang để biết trang có trắng không, chữ có nghiêng không, trang có bị
//! xoay ngang không — dành cho PDF **chữ thật** (born-digital), loại không có ảnh quét nào để đo.
//!
//! Bộ đo bên [`crate::anh_xam`] nhìn vào ảnh quét: đếm điểm mực, chiếu để đoán góc. Trang chữ thật
//! không có ảnh nào, nên trước nay chỉ ghi "không đo được" — kể cả khi trang trống trơn hay bị
//! xoay ngang 90°, những thứ nhìn vào lệnh vẽ là biết chắc, không phải đoán.
//!
//! Ba câu trả lời lấy được từ đây, và chỉ ba:
//!
//! * **Trắng**: trang không có một lệnh vẽ nào để lại vết — không chữ, không ảnh, không nét, hoặc
//!   chỉ tô trắng lên nền trắng.
//! * **Nghiêng**: ma trận đặt chữ mang thành phần xoay. Đây là số ĐÚNG chứ không phải ước lượng
//!   như khi chiếu điểm mực.
//! * **Ngang**: `/Rotate 90` hoặc `270` trên trang (thừa kế được từ nút `/Pages` cha).
//!
//! Không trả lời được "nhiều mực hay ít mực": muốn biết thì phải vẽ trang ra mới đếm được.

use crate::{ParsedPdf, PdfError};
use lopdf::content::Content;
use lopdf::{Document, Object, ObjectId};

/// Những gì đọc được từ lệnh vẽ của một trang.
#[derive(Debug, Clone, PartialEq)]
pub struct TrangVe {
    /// Có ít nhất một lệnh để lại vết trên giấy.
    pub co_ve: bool,
    /// Góc nghiêng của chữ (độ, dương = nghiêng theo chiều kim đồng hồ, cùng quy ước với
    /// [`crate::anh_xam`]). `None` = trang không có chữ để đo.
    pub goc: Option<f32>,
    /// `/Rotate` đã chuẩn hoá về 0 / 90 / 180 / 270.
    pub xoay: i32,
}

impl TrangVe {
    /// Trang nằm ngang: xoay 90° hoặc 270°.
    pub fn ngang(&self) -> bool {
        self.xoay == 90 || self.xoay == 270
    }
}

/// Lệnh vẽ dài hơn chừng này thì thôi — trang chữ dày nhất cũng không tới, còn tệp hỏng thì có
/// thể khai độ dài vô lý.
const NOI_DUNG_TOI_DA: usize = 32 * 1024 * 1024;
/// Gộp góc về từng 0,1° khi bỏ phiếu, để sai số dấu phẩy động không tách một góc thành hai.
const BUOC_GOC: f32 = 10.0;

type Matran = [f32; 6];
const DON_VI: Matran = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// `a` rồi mới tới `b` (a × b theo thứ tự PDF).
fn nhan(a: Matran, b: Matran) -> Matran {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
        a[4] * b[0] + a[5] * b[2] + b[4],
        a[4] * b[1] + a[5] * b[3] + b[5],
    ]
}

fn so(o: &Object) -> Option<f32> {
    o.as_float().ok().or_else(|| o.as_i64().ok().map(|v| v as f32))
}

fn lay_matran(ops: &[Object]) -> Option<Matran> {
    if ops.len() < 6 {
        return None;
    }
    let mut m = DON_VI;
    for (i, o) in ops.iter().take(6).enumerate() {
        m[i] = so(o)?;
    }
    Some(m)
}

/// `/Rotate` của trang, thừa kế từ `/Pages` cha nếu trang không tự khai.
fn xoay_cua(doc: &Document, trang: ObjectId) -> i32 {
    let mut id = trang;
    for _ in 0..64 {
        let Ok(d) = doc.get_dictionary(id) else { return 0 };
        if let Some(v) = d.get(b"Rotate").ok().and_then(|o| match o {
            Object::Reference(r) => doc.get_object(*r).ok().and_then(|x| x.as_i64().ok()),
            o => o.as_i64().ok(),
        }) {
            return v.rem_euclid(360) as i32;
        }
        let Some(cha) = d.get(b"Parent").ok().and_then(|o| o.as_reference().ok()) else {
            return 0;
        };
        id = cha;
    }
    0
}

/// Chuỗi trong toán tử vẽ chữ có ký tự nào không phải khoảng trắng.
fn co_chu(ops: &[Object]) -> bool {
    fn thuc(b: &[u8]) -> bool {
        b.iter().any(|c| !c.is_ascii_whitespace())
    }
    ops.iter().any(|o| match o {
        Object::String(s, _) => thuc(s),
        Object::Array(a) => a.iter().any(|x| matches!(x, Object::String(s, _) if thuc(s))),
        _ => false,
    })
}

/// Màu tô hiện tại có phải trắng — tô trắng lên nền trắng thì trang vẫn là trang trắng.
fn mau_trang(ten: &str, ops: &[Object]) -> Option<bool> {
    let v: Vec<f32> = ops.iter().filter_map(so).collect();
    match (ten, v.len()) {
        ("g", 1) => Some(v[0] >= 0.99),
        ("rg", 3) => Some(v.iter().all(|x| *x >= 0.99)),
        ("k", 4) => Some(v.iter().all(|x| *x <= 0.01)),
        // Không gian màu khác (/Separation, ICC…): không đoán, coi như có mực.
        ("sc", _) | ("scn", _) | ("cs", _) => Some(false),
        _ => None,
    }
}

/// `/Rotate` của trang (0 / 90 / 180 / 270), thừa kế từ `/Pages` cha. Rẻ hơn [`doc_trang`] vì
/// không đụng tới lệnh vẽ — trang ảnh quét cũng cần biết số này.
pub fn xoay_trang(parsed: &ParsedPdf, page_no: i32) -> i32 {
    let doc = &parsed.0;
    let Ok(so_trang) = u32::try_from(page_no) else { return 0 };
    match doc.get_pages().get(&so_trang).copied() {
        Some(id) => xoay_cua(doc, id),
        None => 0,
    }
}

/// Đọc lệnh vẽ của trang `page_no` (đếm từ 1).
pub fn doc_trang(parsed: &ParsedPdf, page_no: i32) -> Result<TrangVe, PdfError> {
    let doc = &parsed.0;
    let so_trang = u32::try_from(page_no)
        .map_err(|_| PdfError::NoPageImage(format!("page_no {page_no} không hợp lệ")))?;
    let pages = doc.get_pages();
    let id = *pages.get(&so_trang).ok_or_else(|| {
        PdfError::NoPageImage(format!("không có trang {page_no} (PDF {} trang)", pages.len()))
    })?;
    let xoay = xoay_cua(doc, id);
    let data = doc.get_page_content(id).map_err(|e| PdfError::NeedsRender {
        page: so_trang,
        reason: format!("không đọc được lệnh vẽ của trang: {e}"),
    })?;
    if data.len() > NOI_DUNG_TOI_DA {
        return Err(PdfError::NeedsRender { page: so_trang, reason: "lệnh vẽ của trang quá dài".into() });
    }
    let noi_dung = Content::decode(&data).map_err(|e| PdfError::NeedsRender {
        page: so_trang,
        reason: format!("lệnh vẽ của trang không đọc được: {e}"),
    })?;
    Ok(doc_lenh(&noi_dung.operations, xoay))
}

fn doc_lenh(ops: &[lopdf::content::Operation], xoay: i32) -> TrangVe {
    let mut ctm = DON_VI;
    let mut kho: Vec<Matran> = Vec::new();
    let mut tm = DON_VI;
    let mut to_trang = false;
    let mut co_ve = false;
    // Góc của từng lần vẽ chữ, gộp theo 0,1°: tiêu đề thẳng mà thân bài nghiêng thì lấy góc được
    // vẽ nhiều nhất, không lấy góc đầu tiên gặp.
    let mut phieu: std::collections::HashMap<i32, usize> = std::collections::HashMap::new();

    for op in ops {
        let t = op.operator.as_str();
        match t {
            "q" => kho.push(ctm),
            "Q" => ctm = kho.pop().unwrap_or(DON_VI),
            "cm" => {
                if let Some(m) = lay_matran(&op.operands) {
                    ctm = nhan(m, ctm);
                }
            }
            "BT" => tm = DON_VI,
            "Tm" => {
                if let Some(m) = lay_matran(&op.operands) {
                    tm = m;
                }
            }
            "g" | "rg" | "k" | "sc" | "scn" | "cs" => {
                if let Some(tr) = mau_trang(t, &op.operands) {
                    to_trang = tr;
                }
            }
            "Tj" | "TJ" | "'" | "\"" => {
                if co_chu(&op.operands) {
                    co_ve = true;
                    let m = nhan(tm, ctm);
                    let goc = m[1].atan2(m[0]).to_degrees();
                    *phieu.entry((goc * BUOC_GOC).round() as i32).or_default() += 1;
                }
            }
            // Nét vẽ luôn tính là có mực; mảng tô thì chỉ tính khi màu không phải trắng.
            "S" | "s" | "B" | "B*" | "b" | "b*" | "sh" | "Do" | "BI" | "EI" | "ID" => co_ve = true,
            "f" | "F" | "f*" => co_ve = co_ve || !to_trang,
            _ => {}
        }
    }

    let goc = phieu
        .into_iter()
        .max_by_key(|(g, n)| (*n, -g.abs()))
        // Ma trận xoay của PDF quay NGƯỢC chiều kim đồng hồ (trục y hướng lên), còn bộ đo ảnh
        // quét ghi dương = thuận chiều kim đồng hồ. Đổi dấu để hai bên nói cùng một thứ tiếng.
        .map(|(g, _)| -(g as f32) / BUOC_GOC);
    TrangVe { co_ve, goc, xoay }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lopdf::content::Operation;

    fn op(ten: &str, ops: Vec<Object>) -> Operation {
        Operation { operator: ten.to_string(), operands: ops }
    }
    fn chu() -> Object {
        Object::String(b"xin chao".to_vec(), lopdf::StringFormat::Literal)
    }
    fn f(x: f32) -> Object {
        Object::Real(x)
    }

    #[test]
    fn trang_chi_dat_font_ma_khong_ve_chu_la_trang_trang() {
        // Đúng nội dung pypdf sinh ra cho trang trắng: `BT /F1 12 Tf 14.4 TL ET`.
        let v = doc_lenh(
            &[
                op("cm", vec![f(1.0), f(0.0), f(0.0), f(1.0), f(0.0), f(0.0)]),
                op("BT", vec![]),
                op("Tf", vec![Object::Name(b"F1".to_vec()), f(12.0)]),
                op("TL", vec![f(14.4)]),
                op("ET", vec![]),
            ],
            0,
        );
        assert!(!v.co_ve, "không vẽ gì thì phải là trang trắng");
        assert_eq!(v.goc, None);
    }

    #[test]
    fn chuoi_toan_khoang_trang_khong_tinh_la_co_chu() {
        let v = doc_lenh(
            &[op("Tj", vec![Object::String(b"   ".to_vec(), lopdf::StringFormat::Literal)])],
            0,
        );
        assert!(!v.co_ve);
    }

    /// Ma trận `.990268 .139173 -0.139173 .990268` = xoay 8° — đúng tệp mẫu pypdf.
    #[test]
    fn doc_dung_goc_nghieng_tu_ma_tran() {
        let v = doc_lenh(
            &[
                op("q", vec![]),
                op("cm", vec![f(0.990268), f(0.139173), f(-0.139173), f(0.990268), f(61.0), f(-37.0)]),
                op("BT", vec![]),
                op("Tm", vec![f(1.0), f(0.0), f(0.0), f(1.0), f(75.0), f(746.0)]),
                op("Tj", vec![chu()]),
                op("ET", vec![]),
                op("Q", vec![]),
            ],
            0,
        );
        assert!(v.co_ve);
        let g = v.goc.expect("phải đoán được góc");
        assert!((g + 8.0).abs() < 0.2, "góc đọc được {g}, phải quanh -8°");
    }

    /// Tiêu đề thẳng, thân bài nghiêng: lấy góc được vẽ NHIỀU nhất.
    #[test]
    fn lay_goc_duoc_ve_nhieu_nhat_chu_khong_lay_goc_dau_tien() {
        let mut v = vec![op("BT", vec![]), op("Tj", vec![chu()])];
        for _ in 0..5 {
            v.push(op("q", vec![]));
            v.push(op("cm", vec![f(0.990268), f(0.139173), f(-0.139173), f(0.990268), f(0.0), f(0.0)]));
            v.push(op("Tj", vec![chu()]));
            v.push(op("Q", vec![]));
        }
        let g = doc_lenh(&v, 0).goc.unwrap();
        assert!((g + 8.0).abs() < 0.2, "góc {g}");
    }

    #[test]
    fn to_trang_len_nen_trang_van_la_trang_trang_con_to_den_thi_khong() {
        let tra = |mau: Operation| {
            doc_lenh(&[mau, op("re", vec![f(0.0), f(0.0), f(100.0), f(100.0)]), op("f", vec![])], 0).co_ve
        };
        assert!(!tra(op("rg", vec![f(1.0), f(1.0), f(1.0)])), "tô trắng không phải là mực");
        assert!(tra(op("rg", vec![f(0.0), f(0.0), f(0.0)])), "tô đen là có mực");
        assert!(tra(op("g", vec![f(0.5)])), "tô xám là có mực");
    }

    #[test]
    fn net_ve_tinh_la_co_muc() {
        assert!(doc_lenh(&[op("S", vec![])], 0).co_ve);
        assert!(doc_lenh(&[op("Do", vec![Object::Name(b"Im0".to_vec())])], 0).co_ve);
    }

    #[test]
    fn xoay_90_va_270_la_trang_ngang() {
        assert!(doc_lenh(&[], 90).ngang());
        assert!(doc_lenh(&[], 270).ngang());
        assert!(!doc_lenh(&[], 0).ngang());
        assert!(!doc_lenh(&[], 180).ngang());
    }
}
