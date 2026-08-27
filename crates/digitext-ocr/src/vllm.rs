// Copyright (c) 2026 DigiText. All rights reserved.
use crate::corrector::{CorrectorError, Tb4, Tm9};
use serde_json::{json, Value};
use std::future::Future;
use std::time::Duration;
pub const SYSTEM_PROMPT: &str = "Bạn là bộ SỬA LỖI OCR tiếng Việt. CHỈ sửa dấu thanh, dấu mũ/móc, và ký tự nhầm (confusables) trong đoạn text OCR được đưa. TUYỆT ĐỐI không diễn giải, không viết lại câu, không thêm/bớt chữ, KHÔNG đụng chữ số/ngày/mã định danh. Trả DUY NHẤT một JSON: {\"corrected\": \"<text đã sửa>\", \"confidence\": <số thực 0..1>}.";
pub struct VllmCorrector {
    client: reqwest::Client,
    endpoint: String,
    model: String,
}
impl VllmCorrector {
    pub fn new(
        endpoint: impl Into<String>,
        model: impl Into<String>,
        r_3gm: Duration,
    ) -> Result<Self, CorrectorError> {
        let client = reqwest::Client::builder()
            .timeout(r_3gm)
            .build()
            .map_err(|r_3gl| CorrectorError::Backend(r_3gl.to_string()))?;
        Ok(VllmCorrector {
            client,
            endpoint: endpoint.into(),
            model: model.into(),
        })
    }
}
pub fn build_request(model: &str, r_3gn: &str) -> Value {
    json!(
        { "model" : model, "temperature" : 0.0, "max_tokens" : 256, "response_format" : {
        "type" : "json_object" }, "messages" : [{ "role" : "system", "content" :
        SYSTEM_PROMPT }, { "role" : "user", "content" : r_3gn },], }
    )
}
pub fn f_1yk(resp: &Value) -> Option<Tm9> {
    let content = resp
        .get("choices")?
        .get(0)?
        .get("message")?
        .get("content")?
        .as_str()?;
    let r_3go: Value = serde_json::from_str(content).ok()?;
    let corrected = r_3go.get("corrected")?.as_str()?.to_string();
    let confidence = r_3go
        .get("confidence")
        .and_then(Value::as_f64)
        .unwrap_or(0.5) as f32;
    Some(Tm9 {
        corrected,
        confidence: confidence.clamp(0.0, 1.0),
    })
}
impl Tb4 for VllmCorrector {
    #[allow(clippy::manual_async_fn)]
    fn suggest(
        &self,
        r_3gq: &str,
    ) -> impl Future<Output = Result<Option<Tm9>, CorrectorError>> + Send {
        let r_3gr = build_request(&self.model, r_3gq);
        let client = self.client.clone();
        let endpoint = self.endpoint.clone();
        async move {
            let resp = client
                .post(&endpoint)
                .json(&r_3gr)
                .send()
                .await
                .map_err(|e| CorrectorError::Backend(e.to_string()))?;
            if !resp.status().is_success() {
                return Err(CorrectorError::Backend(format!(
                    "vLLM HTTP {}",
                    resp.status()
                )));
            }
            let r_3gp: Value = resp
                .json()
                .await
                .map_err(|e| CorrectorError::Backend(e.to_string()))?;
            Ok(f_1yk(&r_3gp))
        }
    }
}
