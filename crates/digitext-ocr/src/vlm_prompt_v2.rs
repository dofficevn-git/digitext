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
/// Cách model đếm token ảnh — quyết định thu nhỏ ảnh bao nhiêu và còn bao nhiêu
/// chỗ cho đầu ra. Sai profile thì `max_tokens` tính lệch: output bị cắt
/// (finish_reason=length) hoặc ảnh bị thu nhỏ vô ích.
pub const ENV_VLM_PROFILE: &str = "DIGITEXT_VLM_PROFILE";
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VlmProfile {
    /// Qwen2.5-VL: lưới patch 28 px, token ≈ (w/28)·(h/28).
    #[default]
    Qwen28,
    /// InternVL / Vintern: cắt ảnh thành ô 448 px (tối đa 6) + 1 ô thu nhỏ,
    /// mỗi ô 256 token.
    InternVl448,
}
pub const INTERNVL_TILE: u32 = 448;
pub const INTERNVL_MAX_TILES: u32 = 6;
pub const INTERNVL_TOKENS_PER_TILE: u32 = 256;
impl VlmProfile {
    pub fn from_env() -> Result<Self, VlmError> {
        match std::env::var(ENV_VLM_PROFILE) {
            Err(_) => Ok(Self::Qwen28),
            Ok(v) => Self::parse(&v),
        }
    }
    pub fn parse(v: &str) -> Result<Self, VlmError> {
        match v.trim().to_ascii_lowercase().as_str() {
            "" | "qwen" | "qwen28" | "qwen2.5-vl" => Ok(Self::Qwen28),
            "internvl" | "internvl448" | "vintern" => Ok(Self::InternVl448),
            other => Err(VlmError::Backend(format!(
                "{ENV_VLM_PROFILE}={other:?} không hợp lệ — dùng 'qwen' hoặc 'internvl'"
            ))),
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Qwen28 => "qwen",
            Self::InternVl448 => "internvl",
        }
    }
    pub fn image_tokens(self, width: u32, height: u32) -> u32 {
        match self {
            Self::Qwen28 => image_tokens(width, height),
            Self::InternVl448 => {
                let tiles = (width.div_ceil(INTERNVL_TILE) * height.div_ceil(INTERNVL_TILE))
                    .clamp(1, INTERNVL_MAX_TILES);
                let thumb = if tiles > 1 { 1 } else { 0 };
                (tiles + thumb) * INTERNVL_TOKENS_PER_TILE
            }
        }
    }
    /// Cạnh dài cần thu về để lọt ngân sách token; `None` = ảnh đã vừa.
    pub fn fit_max_side(
        self,
        width: u32,
        height: u32,
        budget: u32,
    ) -> Result<Option<u32>, VlmError> {
        if width == 0 || height == 0 {
            return Err(VlmError::Backend(format!(
                "kích thước ảnh vô lý: {width}x{height}"
            )));
        }
        match self {
            Self::Qwen28 => fit_max_side(width, height, budget),
            Self::InternVl448 => {
                if self.image_tokens(width, height) <= budget {
                    return Ok(None);
                }
                // 3×3 ô = 9 > 6 nên bị kẹp về 6 ô + 1 thu nhỏ = 1792 token < 1850.
                // Thu cạnh dài về 3 ô là đủ; nhỏ hơn nữa chỉ mất chữ.
                let cap = INTERNVL_TILE * 3;
                Ok(Some(cap.min(width.max(height)).max(1)))
            }
        }
    }
}
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
#[cfg(test)]
mod profile_tests {
    use super::*;
    #[test]
    fn internvl_tokens_capped_at_six_tiles_plus_thumbnail() {
        let p = VlmProfile::InternVl448;
        assert_eq!(p.image_tokens(448, 448), 256);
        assert_eq!(p.image_tokens(896, 448), 3 * 256);
        assert_eq!(p.image_tokens(1654, 2339), 7 * 256);
        assert!(p.image_tokens(1654, 2339) <= MAX_IMAGE_TOKENS_V2);
    }
    #[test]
    fn internvl_fit_caps_long_side_to_three_tiles() {
        let p = VlmProfile::InternVl448;
        assert_eq!(p.fit_max_side(448, 448, MAX_IMAGE_TOKENS_V2).unwrap(), None);
        assert_eq!(p.fit_max_side(1654, 2339, 1000).unwrap(), Some(1344));
        assert!(p.fit_max_side(0, 10, 1000).is_err());
    }
    #[test]
    fn qwen_profile_matches_free_functions() {
        let p = VlmProfile::Qwen28;
        assert_eq!(p.image_tokens(1400, 1000), image_tokens(1400, 1000));
        assert_eq!(
            p.fit_max_side(3000, 2000, MAX_IMAGE_TOKENS_V2).unwrap(),
            fit_max_side(3000, 2000, MAX_IMAGE_TOKENS_V2).unwrap()
        );
    }
    #[test]
    fn parse_profile() {
        assert_eq!(VlmProfile::parse("qwen").unwrap(), VlmProfile::Qwen28);
        assert_eq!(VlmProfile::parse("VINTERN").unwrap(), VlmProfile::InternVl448);
        assert!(VlmProfile::parse("llava").is_err());
    }
}
/// "Dạy bằng ví dụ": chữ OCR của trang + vài ví dụ đã được người kiểm tra có chữ OCR giống nhất.
/// Đo trên 47 ảnh plakie riêng (28/9/2026): đúng hoàn toàn 44% (chỉ ảnh) → 57% (+ chữ OCR) → 74%
/// (+ chữ OCR + 3 ví dụ).
#[derive(Debug, Clone, Default)]
pub struct FewShot {
    pub ocr_text: Option<String>,
    /// (chữ OCR của ví dụ, {khoá trường: giá trị đúng})
    pub examples: Vec<(String, Value)>,
}
/// Giới hạn ngữ cảnh của vLLM (`--max-model-len`). Đọc `DIGITEXT_VLM_MAX_LEN`, mặc định 4096 —
/// PHẢI khớp máy chủ vLLM, lệch thì vLLM trả 400.
pub fn model_max_len() -> u32 {
    std::env::var("DIGITEXT_VLM_MAX_LEN")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .filter(|&n: &u32| n >= 2048)
        .unwrap_or(MODEL_MAX_LEN_V2)
}
/// Chừa tối thiểu ngần này token cho JSON 16 trường.
const MIN_OUTPUT_V2: u32 = 700;
const EXAMPLE_OCR_CHARS: usize = 600;
const PAGE_OCR_CHARS: usize = 1500;
/// Ước lượng token tiếng Việt (có dấu) cho Qwen: ~2 ký tự/token, cộng biên.
fn est_tokens(s: &str) -> u32 {
    (s.chars().count() as u32).div_ceil(2) + 16
}
fn clip(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}
pub fn hint_text(h: &FewShot) -> String {
    let mut out = String::new();
    for (i, (ocr, fields)) in h.examples.iter().enumerate() {
        let mut ans = serde_json::Map::new();
        for f in VanBanField::ALL {
            if let Some(v) = fields.get(f.as_str()).and_then(Value::as_str).filter(|v| !v.trim().is_empty()) {
                ans.insert(f.as_str().to_owned(), Value::String(v.trim().to_owned()));
            }
        }
        if ans.is_empty() {
            continue;
        }
        out.push_str(&format!(
            "\n\nVÍ DỤ {} (văn bản KHÁC đã được người kiểm tra — bắt chước cách lấy và cách viết, KHÔNG chép giá trị):\nChữ OCR:\n{}\nKết quả đúng (trường không ghi là rỗng):\n{}",
            i + 1,
            clip(ocr, EXAMPLE_OCR_CHARS),
            Value::Object(ans)
        ));
    }
    if let Some(t) = h.ocr_text.as_deref().filter(|t| !t.trim().is_empty()) {
        out.push_str(&format!(
            "\n\nCHỮ OCR CỦA TRANG CẦN BÓC (máy đọc, có thể sai dấu — đối chiếu với ảnh, ưu tiên chữ trên ảnh):\n{}",
            clip(t, PAGE_OCR_CHARS)
        ));
    }
    if !out.is_empty() {
        out.push_str("\n\nTrả JSON cho TRANG trong ảnh đính kèm.");
    }
    out
}
/// Chọn phần gợi ý vừa ngân sách: bớt dần ví dụ, rồi bỏ chữ OCR, nếu không đủ chỗ.
/// Trả (chữ gợi ý, max_tokens đầu ra).
pub fn fit_hint(h: &FewShot, img_tokens: u32) -> Result<(String, u32), VlmError> {
    let max_len = model_max_len() as i64;
    let mut h = h.clone();
    loop {
        let text = hint_text(&h);
        let budget = max_len - img_tokens as i64 - C_G2 as i64 - C_JE as i64 - est_tokens(&text) as i64;
        if text.is_empty() || budget >= MIN_OUTPUT_V2 as i64 {
            if budget <= 0 {
                return Err(VlmError::Backend(format!(
                    "ảnh {img_tokens} token không còn chỗ cho đầu ra (max_len={max_len}); phải thu nhỏ ảnh trước"
                )));
            }
            return Ok((text, budget as u32));
        }
        if h.examples.pop().is_none() {
            h.ocr_text = None;
        }
    }
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
pub fn build_request_v2(model: &str, r_3k5: &str, max_tokens: u32, hint: &str) -> Tiz {
    Tiz {
        model: model.to_owned(),
        messages: json!(
            [{ "role" : "user", "content" : [{ "type" : "image_url", "image_url" : {
            "url" : r_3k5 } }, { "type" : "text", "text" : format!("{}{hint}", f_24a()) },] },]
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
#[cfg(test)]
mod few_shot_tests {
    use super::*;
    fn ex(n: usize) -> FewShot {
        FewShot {
            ocr_text: Some("UBND TỈNH BẮC NINH\nSố: 12/QĐ-UBND ".repeat(40)),
            examples: (0..n)
                .map(|i| ("SỞ TÀI CHÍNH ".repeat(50), serde_json::json!({"soKyHieu": format!("{i}/QĐ"), "khongCo": "x"})))
                .collect(),
        }
    }
    #[test]
    fn du_cho_thi_giu_het_vi_du() {
        std::env::set_var("DIGITEXT_VLM_MAX_LEN", "8192");
        let (t, out) = fit_hint(&ex(3), 1850).unwrap();
        assert_eq!(t.matches("VÍ DỤ").count(), 3);
        assert!(t.contains("CHỮ OCR CỦA TRANG") && out >= MIN_OUTPUT_V2);
        assert!(!t.contains("khongCo"), "chỉ đưa khoá thuộc 16 trường");
    }
    #[test]
    fn thieu_cho_thi_bot_vi_du_roi_bo_ocr() {
        std::env::set_var("DIGITEXT_VLM_MAX_LEN", "4096");
        let (t, out) = fit_hint(&ex(3), 1850).unwrap();
        assert!(t.matches("VÍ DỤ").count() < 3);
        assert!(out >= MIN_OUTPUT_V2 || t.is_empty());
        let (t0, _) = fit_hint(&FewShot::default(), 1850).unwrap();
        assert!(t0.is_empty());
    }
}
