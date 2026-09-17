// Copyright (c) 2026 DigiText. All rights reserved.
//
// Engine nhận dạng chữ qua HTTP — nói chuyện với dịch vụ OCR toàn văn ở ocr/.
//
// Đặt cạnh grpc.rs và giữ đúng hình dạng của nó: cùng trait Tok, cùng kiểu vào
// ra, cùng tinh thần ánh xạ lỗi. Khác biệt duy nhất là vận chuyển — HTTP+PNG
// thay cho gRPC+tensor thô.
//
// Vì sao nén PNG trước khi gửi: một ô 1024×1024 RGB8 là 3MB dữ liệu thô. Gửi
// nguyên qua HTTP mỗi lần là lãng phí nghiêm trọng; PNG không mất mát nên chất
// lượng nhận dạng không đổi.
use crate::error::WorkerError;
use crate::infer::{Tile, To6, Tok};
use base64::Engine as _;
use image::ImageEncoder;
use std::time::Duration;

pub struct HttpOcrInfer {
    client: reqwest::Client,
    tile_url: String,
    health_url: String,
    api_key: Option<String>,
}

#[derive(serde::Deserialize)]
struct TileResponse {
    boxes: Vec<[f32; 4]>,
    texts: Vec<String>,
    confidences: Vec<f32>,
}

impl HttpOcrInfer {
    /// `base` là địa chỉ gốc của dịch vụ, ví dụ `http://127.0.0.1:5002`.
    pub fn new(
        base: &str,
        api_key: Option<String>,
        timeout: Duration,
    ) -> Result<Self, WorkerError> {
        let base = base.trim_end_matches('/');
        if base.is_empty() {
            return Err(WorkerError::ConfigInvalid(
                "địa chỉ dịch vụ OCR rỗng".into(),
            ));
        }
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| WorkerError::ConfigInvalid(format!("không dựng được HTTP client: {e}")))?;
        Ok(HttpOcrInfer {
            client,
            tile_url: format!("{base}/api/ocr/tile"),
            health_url: format!("{base}/api/ocr/health"),
            api_key: api_key.filter(|k| !k.is_empty()),
        })
    }

    pub fn health_url(&self) -> &str {
        &self.health_url
    }

    /// Dùng cho /readyz — thay chỗ của GrpcTritonProbe khi chạy engine HTTP.
    pub async fn probe(&self) -> Result<(), WorkerError> {
        let resp = self
            .client
            .get(&self.health_url)
            .send()
            .await
            .map_err(map_reqwest_error)?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(status_to_error(
                resp.status(),
                &format!("healthz trả {}", resp.status()),
            ))
        }
    }
}

/// Ô ảnh RGB8 thô → PNG.
fn encode_png(tile: &Tile) -> Result<Vec<u8>, WorkerError> {
    let expected = tile.width as usize * tile.height as usize * 3;
    if tile.pixels.len() != expected {
        return Err(WorkerError::CorruptInput(format!(
            "ô ảnh {}×{} cần {} byte RGB8 nhưng có {}",
            tile.width,
            tile.height,
            expected,
            tile.pixels.len()
        )));
    }
    let mut out = Vec::with_capacity(expected / 4);
    image::codecs::png::PngEncoder::new(&mut out)
        .write_image(
            &tile.pixels,
            tile.width,
            tile.height,
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|e| WorkerError::CorruptInput(format!("không nén được PNG: {e}")))?;
    Ok(out)
}

/// Lỗi tầng vận chuyển. Hết giờ là tạm thời; không nối được cũng vậy —
/// dịch vụ có thể đang khởi động lại.
fn map_reqwest_error(e: reqwest::Error) -> WorkerError {
    if e.is_timeout() {
        WorkerError::Timeout(e.to_string())
    } else if e.is_decode() || e.is_body() {
        WorkerError::CorruptInput(e.to_string())
    } else {
        WorkerError::TritonUnavailable(e.to_string())
    }
}

/// Ánh xạ mã HTTP → WorkerError. Đối xứng với f_1qz ở grpc.rs:42-57 và là MỘT
/// PHẦN CỦA HỢP ĐỒNG với dịch vụ Python — xem ocr/ocr_routes.py.
///
/// Phân biệt tạm thời / vĩnh viễn quyết định job được thử lại hay trang bị đánh
/// hỏng. Riêng GpuOom còn kích hoạt cơ chế tự chia đôi ô ở worker.rs:66-103.
fn status_to_error(status: reqwest::StatusCode, body: &str) -> WorkerError {
    use reqwest::StatusCode as S;
    let msg = format!("{status}: {body}");
    match status {
        S::INSUFFICIENT_STORAGE => WorkerError::GpuOom(msg), // 507
        S::GATEWAY_TIMEOUT | S::REQUEST_TIMEOUT => WorkerError::Timeout(msg),
        S::BAD_GATEWAY | S::SERVICE_UNAVAILABLE => WorkerError::TritonUnavailable(msg),
        S::BAD_REQUEST | S::UNPROCESSABLE_ENTITY => WorkerError::CorruptInput(msg),
        S::NOT_IMPLEMENTED | S::NOT_FOUND => WorkerError::ConfigInvalid(msg),
        // Sai khoá API là lỗi cấu hình — thử lại bao nhiêu lần cũng vô ích.
        S::UNAUTHORIZED | S::FORBIDDEN => WorkerError::ConfigInvalid(msg),
        _ => WorkerError::TritonUnavailable(msg),
    }
}

impl Tok for HttpOcrInfer {
    async fn probe_engine(&self) -> Result<(), WorkerError> {
        HttpOcrInfer::probe(self).await
    }
    async fn infer_tile(&self, tile: &Tile, _batch: usize) -> Result<To6, WorkerError> {
        let png = encode_png(tile)?;
        let b64 = base64::engine::general_purpose::STANDARD.encode(&png);

        let mut req = self
            .client
            .post(&self.tile_url)
            .json(&serde_json::json!({ "image": b64 }));
        if let Some(key) = &self.api_key {
            req = req.header("X-API-Key", key);
        }

        let resp = req.send().await.map_err(map_reqwest_error)?;
        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(status_to_error(status, body.trim()));
        }

        let parsed: TileResponse = resp.json().await.map_err(map_reqwest_error)?;

        // Bất biến bắt buộc — worker và tầng ghép PDF đều dựa vào nó. Lệch độ
        // dài là lỗi VĨNH VIỄN: thử lại cũng ra kết quả hỏng y hệt.
        if parsed.boxes.len() != parsed.texts.len()
            || parsed.texts.len() != parsed.confidences.len()
        {
            return Err(WorkerError::CorruptInput(format!(
                "dịch vụ OCR trả ba mảng lệch nhau: boxes={} texts={} confidences={}",
                parsed.boxes.len(),
                parsed.texts.len(),
                parsed.confidences.len()
            )));
        }

        Ok(To6 {
            boxes: parsed.boxes,
            texts: parsed.texts,
            confidences: parsed.confidences,
        })
    }
}

/// Probe cho /readyz khi chạy engine HTTP — thay chỗ `GrpcTritonProbe`.
///
/// Không có khái niệm "model" như Triton: dịch vụ hoặc đã nạp xong PaddleOCR,
/// hoặc chưa. Nên `required_models` bị bỏ qua và `models` luôn rỗng.
pub struct HttpOcrProbe {
    client: reqwest::Client,
    health_url: String,
}

impl HttpOcrProbe {
    pub fn new(base: &str, timeout: Duration) -> Result<Self, WorkerError> {
        let base = base.trim_end_matches('/');
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| WorkerError::ConfigInvalid(format!("không dựng được HTTP client: {e}")))?;
        Ok(HttpOcrProbe {
            client,
            health_url: format!("{base}/api/ocr/health"),
        })
    }
}

impl crate::readiness::Tol for HttpOcrProbe {
    async fn probe(
        &self,
        _required_models: &[String],
    ) -> Result<crate::readiness::Toj, crate::readiness::Thj> {
        let resp = self
            .client
            .get(&self.health_url)
            .send()
            .await
            .map_err(|e| crate::readiness::Thj(e.to_string()))?;
        Ok(crate::readiness::Toj {
            server_ready: resp.status().is_success(),
            models: Vec::new(),
        })
    }
}
