// Copyright (c) 2026 DigiText. All rights reserved.
use crate::vlm_fields::VlmError;
use digitext_core::VanBanField;
use serde_json::{json, Value};
pub const FIELD_SPECS_V2: [(VanBanField, &str); 16] = [
    (
        VanBanField::ThuTu,
        "Số thứ tự của văn bản trong hồ sơ (chỉ chữ số)",
    ),
    (
        VanBanField::LoaiVanBan,
        "Loại văn bản, ví dụ: Quyết định, Công văn, Tờ trình, Báo cáo tài chính",
    ),
    (
        VanBanField::SoKyHieu,
        "Số và ký hiệu văn bản, ví dụ: 9999/QĐ-UBND",
    ),
    (
        VanBanField::NgayThang,
        "Ngày tháng ban hành, định dạng dd/mm/yyyy",
    ),
    (VanBanField::CoQuanBanHanh, "Tên cơ quan ban hành văn bản"),
    (
        VanBanField::TrichYeu,
        "Trích yếu nội dung văn bản (có thể nhiều dòng)",
    ),
    (VanBanField::ToSo, "Tờ số (chỉ chữ số)"),
    (VanBanField::SoTrang, "Số trang (chỉ chữ số)"),
    (VanBanField::TrangSo, "Trang số (chỉ chữ số)"),
    (
        VanBanField::NgonNgu,
        "Ngôn ngữ của văn bản, ví dụ: Tiếng Việt",
    ),
    (VanBanField::CheDoSuDung, "Chế độ sử dụng tài liệu"),
    (VanBanField::DoTinCay, "Độ tin cậy của tài liệu"),
    (VanBanField::ButTich, "Bút tích trên văn bản"),
    (
        VanBanField::TinhTrangVatLy,
        "Tình trạng vật lý của tài liệu",
    ),
    (VanBanField::TaiLieuSo, "Thông tin tài liệu số"),
    (VanBanField::GhiChu, "Ghi chú"),
];
pub fn f_24a() -> String {
    let mut r_3k2 = String::from(
        r#"Bạn là hệ thống bóc tách thông tin văn bản hành chính Việt Nam.
Hãy đọc ảnh trang văn bản và trích xuất 16 trường sau:

"#,
    );
    let r_3k1: Vec<String> = FIELD_SPECS_V2
        .iter()
        .map(|(k, r_3k0)| format!("- {}: {}", k.as_str(), r_3k0))
        .collect();
    r_3k2.push_str(&r_3k1.join("\n"));
    r_3k2
        .push_str(
            r#"

QUY TẮC BẮT BUỘC:
1. CHỈ trích xuất chữ THỰC SỰ nhìn thấy trên trang. TUYỆT ĐỐI KHÔNG suy đoán, không bịa.
2. Trường nào không xuất hiện trên trang thì trả về chuỗi rỗng "". Không tự suy diễn.
3. Giữ NGUYÊN VĂN tiếng Việt có dấu đúng như trên trang.
4. Với số, ký hiệu, ngày tháng: chép chính xác từng ký tự, không chuẩn hoá lại.
5. BỎ QUA hoàn toàn dấu chữ ký số / con dấu ở MÉP trang (dòng bắt đầu bằng "Ký bởi:", "Thời gian:", tên cơ quan quét/lưu trữ). Đó KHÔNG phải nội dung văn bản.

QUY TẮC RIÊNG TỪNG TRƯỜNG (đọc kỹ — hay nhầm nhất):
- coQuanBanHanh: LẤY cơ quan ở GÓC TRÊN BÊN TRÁI (tiêu đề văn bản). TUYỆT ĐỐI KHÔNG lấy cơ quan sau "Kính gửi:" (đó là NƠI NHẬN), KHÔNG lấy "CỘNG HOÀ XÃ HỘI CHỦ NGHĨA VIỆT NAM" (đó là quốc hiệu), KHÔNG lấy tên cơ quan trong dấu chữ ký số ở mép, KHÔNG lấy cơ quan ban hành MẪU BIỂU (vd "của Bộ Tài chính" trong ô Mẫu số). Đơn/hợp đồng cá nhân không có cơ quan ban hành thì để rỗng.
- soKyHieu: LẤY số ở đầu văn bản, ngay dưới cơ quan ban hành (vd "Số: 123/QĐ-UBND"). KHÔNG lấy số của văn bản KHÁC được nhắc trong thân bài ("Căn cứ Công văn số...", "Thực hiện văn bản số...").
- ngayThang: LẤY ngày trên dòng "...., ngày ... tháng ... năm ..." ở ĐẦU văn bản (ngày ban hành). KHÔNG lấy ngày trong dấu "CV ĐẾN / CÔNG VĂN ĐẾN" (ngày nhận), KHÔNG lấy ngày trong thân bài, KHÔNG lấy ngày sinh, ngày cấp CMND, ngày kết thúc năm tài chính, hay thời gian chữ ký số ở mép.
- loaiVanBan: LẤY từ DÒNG TIÊU ĐỀ in đậm giữa trang (QUYẾT ĐỊNH, TỜ TRÌNH, BÁO CÁO, THÔNG BÁO, KẾ HOẠCH, GIẤY PHÉP, GIẤY CHỨNG NHẬN, HỢP ĐỒNG, BIÊN BẢN, CÔNG VĂN...). Nếu không có dòng tiêu đề, suy từ ký hiệu (CV=Công văn, QĐ=Quyết định, TTr=Tờ trình, BC=Báo cáo, TB=Thông báo, KH=Kế hoạch, GP=Giấy phép).
- trichYeu: LẤY dòng "V/v ..." hoặc "Về việc ..." ngay dưới số/ký hiệu (hoặc dòng phụ đề ngay dưới tiêu đề). KHÔNG lấy câu đầu THÂN BÀI ("Căn cứ...", "Thực hiện...", "Kính gửi...", tên cơ quan/công ty). Nếu văn bản không có dòng trích yếu thì để rỗng.

Chỉ trả về duy nhất một đối tượng JSON."#,
        );
    r_3k2
}
pub const QWEN_PATCH: u32 = 28;
pub const MODEL_MAX_LEN_V2: u32 = 4096;
pub const C_G2: u32 = 1122;
pub const C_JE: u32 = 64;
pub const MAX_IMAGE_TOKENS_V2: u32 = 1850;
pub fn image_tokens(width: u32, height: u32) -> u32 {
    ((width as f64 / QWEN_PATCH as f64) * (height as f64 / QWEN_PATCH as f64)) as u32
}
pub fn fit_max_side(width: u32, height: u32, budget: u32) -> Result<Option<u32>, VlmError> {
    if width == 0 || height == 0 {
        return Err(VlmError::Backend(format!(
            "kích thước ảnh vô lý: {width}x{height}"
        )));
    }
    if image_tokens(width, height) <= budget {
        return Ok(None);
    }
    let r_3k3 = (0.99 * budget as f64 * (QWEN_PATCH as f64).powi(2)
        / (width as f64 * height as f64))
        .sqrt();
    Ok(Some(((r_3k3 * width.max(height) as f64) as u32).max(1)))
}
pub fn output_budget(img_tokens: u32) -> Result<u32, VlmError> {
    let budget = MODEL_MAX_LEN_V2 as i64 - img_tokens as i64 - C_G2 as i64 - C_JE as i64;
    if budget <= 0 {
        return Err(
            VlmError::Backend(
                format!(
                    "ảnh {img_tokens} token không còn chỗ cho đầu ra (max_len={MODEL_MAX_LEN_V2}); phải thu nhỏ ảnh trước"
                ),
            ),
        );
    }
    Ok(budget as u32)
}
pub fn guided_schema_v2_json() -> String {
    use std::fmt::Write;
    let mut props = String::new();
    let mut required = String::new();
    for (i, (k, r_3k4)) in FIELD_SPECS_V2.iter().enumerate() {
        if i > 0 {
            props.push(',');
            required.push(',');
        }
        let key =
            serde_json::to_string(k.as_str()).expect("khoá là chuỗi hợp lệ");
        let desc =
            serde_json::to_string(r_3k4).expect("mô tả là chuỗi hợp lệ");
        write!(
            props,
            "{key}:{{\"type\":\"string\",\"description\":{desc}}}"
        )
        .expect("ghi String");
        required.push_str(&key);
    }
    format!(
        "{{\"type\":\"object\",\"properties\":{{{props}}},\
         \"required\":[{required}],\"additionalProperties\":false}}"
    )
}
#[derive(serde::Serialize)]
pub struct Tiz {
    model: String,
    messages: Value,
    temperature: f64,
    max_tokens: u32,
    guided_json: Box<serde_json::value::RawValue>,
}
pub fn build_request_v2(model: &str, r_3k5: &str, max_tokens: u32) -> Tiz {
    Tiz {
        model: model.to_owned(),
        messages: json!(
            [{ "role" : "user", "content" : [{ "type" : "image_url", "image_url" : {
            "url" : r_3k5 } }, { "type" : "text", "text" : f_24a() },] },]
        ),
        temperature: 0.0,
        max_tokens,
        guided_json: serde_json::value::RawValue::from_string(guided_schema_v2_json())
            .expect("guided_schema_v2_json là JSON hợp lệ"),
    }
}
pub fn parse_fields_v2(r_3ka: &Value) -> Vec<(VanBanField, String)> {
    let Some(content) = r_3ka
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|r_3k7| r_3k7.get("content"))
        .and_then(Value::as_str)
    else {
        return Vec::new();
    };
    let trimmed = content.trim();
    let trimmed = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
        .unwrap_or(trimmed)
        .trim_end_matches("```")
        .trim();
    let Ok(r_3k8) = serde_json::from_str::<Value>(trimmed) else {
        return Vec::new();
    };
    let mut r_3k9 = Vec::new();
    for r_3k6 in VanBanField::ALL {
        let Some(value) = r_3k8.get(r_3k6.as_str()).and_then(Value::as_str) else {
            continue;
        };
        let value = value.trim();
        if value.is_empty() {
            continue;
        }
        r_3k9.push((r_3k6, digitext_vi::normalize(value)));
    }
    r_3k9
}
