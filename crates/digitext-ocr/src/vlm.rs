// Copyright (c) 2026 DigiText. All rights reserved.
use crate::vlm_fields::{Tpw, Tpx, VlmError};
use base64::Engine;
use digitext_core::VanBanField;
use serde_json::{json, Value};
use std::future::Future;
use std::str::FromStr;
use std::time::Duration;
pub const SYSTEM_PROMPT: &str = "Bạn là bộ TRÍCH TRƯỜNG từ ảnh văn bản hành chính tiếng Việt. Đọc ảnh trang và trích các trường thông tin. CHỈ chép giá trị ĐỌC ĐƯỢC trên ảnh; TUYỆT ĐỐI không suy đoán, không bịa. Không thấy trường nào thì BỎ trường đó. Với số/ngày/mã định danh: chép ĐÚNG y như trên ảnh, không tự sửa. Khoá trường hợp lệ: loaiVanBan, soKyHieu, ngayThang, coQuanBanHanh, trichYeu, toSo, soTrang, thuTu. Trả DUY NHẤT một JSON: {\"fields\":[{\"key\":\"<khoá>\",\"value\":\"<giá trị>\",\"bbox\":[x1,y1,x2,y2],\"confidence\":<0..1>}]}. bbox là pixel trên ảnh, gốc trên-trái, GÓC TRÁI-TRÊN (x1,y1) và GÓC PHẢI-DƯỚI (x2,y2).";
pub const MAX_COMPLETION_TOKENS: u32 = 1024;
pub const DEFAULT_MAX_IMAGE_SIDE: u32 = 1400;
#[derive(Debug, Clone, PartialEq)]
pub struct Thi {
    pub jpeg: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub scale: f32,
}
pub fn f_23i(jpeg: &[u8], r_3gw: u32) -> Result<Thi, VlmError> {
    let (w, h) = crate::render::f_1hv(jpeg).map_err(|e| {
        VlmError::Backend(format!(
            "đọc kích thước ảnh trang: {e}"
        ))
    })?;
    if r_3gw == 0 || w.max(h) <= r_3gw {
        return Ok(Thi {
            jpeg: jpeg.to_vec(),
            width: w,
            height: h,
            scale: 1.0,
        });
    }
    let r_3gv = image::load_from_memory(jpeg)
        .map_err(|e| VlmError::Backend(format!("giải mã ảnh trang: {e}")))?;
    let scale = r_3gw as f32 / w.max(h) as f32;
    let (r_3gy, r_3gx) = (
        ((w as f32 * scale).round() as u32).max(1),
        ((h as f32 * scale).round() as u32).max(1),
    );
    let r_3h0 = r_3gv.resize_exact(r_3gy, r_3gx, image::imageops::FilterType::Triangle);
    let mut r_3gz = Vec::new();
    r_3h0
        .to_rgb8()
        .write_with_encoder(image::codecs::jpeg::JpegEncoder::new_with_quality(
            &mut r_3gz, 85,
        ))
        .map_err(|e| VlmError::Backend(format!("mã hoá lại ảnh trang: {e}")))?;
    Ok(Thi {
        jpeg: r_3gz,
        width: r_3gy,
        height: r_3gx,
        scale: r_3gy as f32 / w as f32,
    })
}
pub fn f_28r(fields: &mut [Tpx], scale: f32) {
    if scale <= 0.0 || (scale - 1.0).abs() < f32::EPSILON {
        return;
    }
    let r_3h2 = 1.0 / scale;
    for r_3h1 in fields.iter_mut() {
        if let Some(b) = r_3h1.bbox.as_mut() {
            for r_3h3 in b.iter_mut() {
                *r_3h3 *= r_3h2;
            }
        }
    }
}
pub struct VlmFieldExtractor {
    client: reqwest::Client,
    endpoint: String,
    model: String,
    max_image_side: u32,
}
impl VlmFieldExtractor {
    pub fn new(
        endpoint: impl Into<String>,
        model: impl Into<String>,
        r_3h4: Duration,
    ) -> Result<Self, VlmError> {
        let client = reqwest::Client::builder()
            .timeout(r_3h4)
            .build()
            .map_err(|e| VlmError::Backend(e.to_string()))?;
        Ok(VlmFieldExtractor {
            client,
            endpoint: endpoint.into(),
            model: model.into(),
            max_image_side: DEFAULT_MAX_IMAGE_SIDE,
        })
    }
    pub fn with_max_image_side(mut self, r_3h5: u32) -> Self {
        self.max_image_side = r_3h5;
        self
    }
}
pub fn f_1ju(jpeg: &[u8]) -> String {
    let b64 = base64::engine::general_purpose::STANDARD.encode(jpeg);
    format!("data:image/jpeg;base64,{b64}")
}
pub fn f_1ce() -> Value {
    let r_3h7: Vec<&str> = VanBanField::ALL
        .iter()
        .map(|r_3h6| r_3h6.as_str())
        .collect();
    json!(
        { "type" : "object", "required" : ["fields"], "additionalProperties" : false,
        "properties" : { "fields" : { "type" : "array", "items" : { "type" : "object",
        "required" : ["key", "value", "bbox", "confidence"], "additionalProperties" :
        false, "properties" : { "key" : { "type" : "string", "enum" : r_3h7 }, "value" :
        { "type" : "string" }, "bbox" : { "type" : "array", "items" : { "type" : "number"
        }, "minItems" : 4, "maxItems" : 4 }, "confidence" : { "type" : "number" }, }, } }
        }, }
    )
}
pub fn build_request(model: &str, r_3h8: &str, page_w: u32, page_h: u32) -> Value {
    let r_3h9 = format!(
        "Ảnh trang kích thước {page_w}x{page_h} pixel. Trích các trường và bbox theo đúng định dạng JSON đã yêu cầu."
    );
    json!(
        { "model" : model, "temperature" : 0.0, "max_tokens" : MAX_COMPLETION_TOKENS,
        "guided_json" : f_1ce(), "messages" : [{ "role" : "system", "content" :
        SYSTEM_PROMPT }, { "role" : "user", "content" : [{ "type" : "text", "text" :
        r_3h9 }, { "type" : "image_url", "image_url" : { "url" : r_3h8 } }] },], }
    )
}
pub fn f_11k([x1, y1, x2, y2]: [f32; 4]) -> Option<[f32; 4]> {
    let (r_3ha, r) = (x1.min(x2), x1.max(x2));
    let (r_3hb, b) = (y1.min(y2), y1.max(y2));
    let (w, h) = (r - r_3ha, b - r_3hb);
    (w > 0.0 && h > 0.0 && w.is_finite() && h.is_finite()).then_some([r_3ha, r_3hb, w, h])
}
pub fn f_1yy(resp: &Value) -> Vec<Tpx> {
    let Some(content) = resp
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
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
    let Ok(r_3hh) = serde_json::from_str::<Value>(trimmed) else {
        return Vec::new();
    };
    let Some(r_3hc) = r_3hh.get("fields").and_then(Value::as_array) else {
        return Vec::new();
    };
    let mut r_3hi = Vec::new();
    for r_3hd in r_3hc {
        let Some(key) = r_3hd.get("key").and_then(Value::as_str) else {
            continue;
        };
        let Ok(r_3he) = VanBanField::from_str(key) else {
            continue;
        };
        let value = r_3hd
            .get("value")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        if value.trim().is_empty() {
            continue;
        }
        let bbox = r_3hd
            .get("bbox")
            .and_then(Value::as_array)
            .filter(|a| a.len() >= 4)
            .and_then(|a| {
                let r_3hf = |r_3hg: usize| a[r_3hg].as_f64().unwrap_or(0.0) as f32;
                f_11k([r_3hf(0), r_3hf(1), r_3hf(2), r_3hf(3)])
            });
        let confidence = r_3hd
            .get("confidence")
            .and_then(Value::as_f64)
            .unwrap_or(0.5) as f32;
        r_3hi.push(Tpx {
            field: r_3he,
            value,
            bbox,
            confidence: confidence.clamp(0.0, 1.0),
        });
    }
    r_3hi
}
impl Tpw for VlmFieldExtractor {
    #[allow(clippy::manual_async_fn)]
    fn extract(
        &self,
        r_3hj: &[u8],
        page_w: u32,
        page_h: u32,
    ) -> impl Future<Output = Result<Vec<Tpx>, VlmError>> + Send {
        let prepared = f_23i(r_3hj, self.max_image_side);
        let model = self.model.clone();
        let client = self.client.clone();
        let endpoint = self.endpoint.clone();
        async move {
            let prepared = prepared?;
            debug_assert!(
                prepared.scale <= 1.0 && prepared.scale > 0.0,
                "scale phải trong (0,1]"
            );
            if prepared.scale < 1.0 {
                tracing::debug!(
                    goc = format!("{page_w}x{page_h}"), gui = format!("{}x{}", prepared
                    .width, prepared.height), scale = prepared.scale,
                    "VLM: đã thu nhỏ ảnh trang cho vừa ngân sách token"
                );
            }
            let r_3hk = build_request(
                &model,
                &f_1ju(&prepared.jpeg),
                prepared.width,
                prepared.height,
            );
            let resp = client
                .post(&endpoint)
                .json(&r_3hk)
                .send()
                .await
                .map_err(|e| VlmError::Backend(e.to_string()))?;
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                let body = body.trim();
                let detail: String = body.chars().take(400).collect();
                return Err(VlmError::Backend(if detail.is_empty() {
                    format!("vLLM HTTP {status}")
                } else {
                    format!("vLLM HTTP {status}: {detail}")
                }));
            }
            let body: Value = resp
                .json()
                .await
                .map_err(|e| VlmError::Backend(e.to_string()))?;
            if finish_reason(&body) == Some("length") {
                return Err(
                    VlmError::Backend(
                        format!(
                            "output bị cắt (finish_reason=length, max_tokens={MAX_COMPLETION_TOKENS}) — ảnh quá lớn so với ngân sách context, hạ cạnh dài ảnh hoặc nâng --max-model-len"
                        ),
                    ),
                );
            }
            let mut fields = f_1yy(&body);
            f_28r(&mut fields, prepared.scale);
            Ok(fields)
        }
    }
}
pub fn finish_reason(resp: &Value) -> Option<&str> {
    resp.get("choices")?.get(0)?.get("finish_reason")?.as_str()
}
pub struct VlmFieldExtractorV2 {
    client: reqwest::Client,
    endpoint: String,
    model: String,
    profile: crate::vlm_prompt_v2::VlmProfile,
}
impl VlmFieldExtractorV2 {
    pub fn new(
        endpoint: impl Into<String>,
        model: impl Into<String>,
        r_3hl: Duration,
    ) -> Result<Self, VlmError> {
        let client = reqwest::Client::builder()
            .timeout(r_3hl)
            .build()
            .map_err(|e| VlmError::Backend(e.to_string()))?;
        Ok(VlmFieldExtractorV2 {
            client,
            endpoint: endpoint.into(),
            model: model.into(),
            profile: crate::vlm_prompt_v2::VlmProfile::Qwen28,
        })
    }
    pub fn with_profile(mut self, profile: crate::vlm_prompt_v2::VlmProfile) -> Self {
        self.profile = profile;
        self
    }
    pub fn profile(&self) -> crate::vlm_prompt_v2::VlmProfile {
        self.profile
    }
}
impl crate::fields_v2::T80 for VlmFieldExtractorV2 {
    fn extract_v2<'a>(&'a self, r_3hm: &'a [u8]) -> crate::fields_v2::T81<'a> {
        use crate::vlm_prompt_v2 as v2;
        Box::pin(async move {
            let (w, h) = crate::render::f_1hv(r_3hm).map_err(|e| {
                VlmError::Backend(format!(
                    "đọc kích thước ảnh trang: {e}"
                ))
            })?;
            let r_3hq = self.profile.fit_max_side(w, h, v2::MAX_IMAGE_TOKENS_V2)?;
            let r_3ho = f_23i(r_3hm, r_3hq.unwrap_or(0))?;
            let r_3hn = self.profile.image_tokens(r_3ho.width, r_3ho.height);
            let max_tokens = v2::output_budget(r_3hn)?;
            if r_3ho.scale < 1.0 {
                tracing::debug!(
                    goc = format!("{w}x{h}"), gui = format!("{}x{}", r_3ho.width, r_3ho
                    .height), r_3hn, max_tokens,
                    "bóc trường v2: đã thu nhỏ ảnh cho vừa ngân sách token"
                );
            }
            let r_3hp = v2::build_request_v2(&self.model, &f_1ju(&r_3ho.jpeg), max_tokens);
            let resp = self
                .client
                .post(&self.endpoint)
                .json(&r_3hp)
                .send()
                .await
                .map_err(|e| VlmError::Backend(e.to_string()))?;
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                let detail: String = body.trim().chars().take(400).collect();
                return Err(VlmError::Backend(if detail.is_empty() {
                    format!("vLLM HTTP {status}")
                } else {
                    format!("vLLM HTTP {status}: {detail}")
                }));
            }
            let body: Value = resp
                .json()
                .await
                .map_err(|e| VlmError::Backend(e.to_string()))?;
            if finish_reason(&body) == Some("length") {
                return Err(
                    VlmError::Backend(
                        format!(
                            "output bị cắt (finish_reason=length, max_tokens={max_tokens}) — trang quá đặc chữ hoặc ngân sách ảnh sai, kiểm tra --max-model-len"
                        ),
                    ),
                );
            }
            Ok(v2::parse_fields_v2(&body))
        })
    }
}
