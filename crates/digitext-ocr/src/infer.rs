// Copyright (c) 2026 DigiText. All rights reserved.
use crate::error::WorkerError;
use std::future::Future;
#[derive(Debug, Clone)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}
impl Tile {
    pub fn empty(x: u32, y: u32, width: u32, height: u32) -> Self {
        Tile {
            x,
            y,
            width,
            height,
            pixels: Vec::new(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct To6 {
    pub boxes: Vec<[f32; 4]>,
    pub texts: Vec<String>,
    pub confidences: Vec<f32>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct Tgi {
    pub boxes: Vec<[f32; 4]>,
    pub texts: Vec<String>,
    pub confidences: Vec<f32>,
}
impl Tgi {
    pub fn mean_confidence(&self) -> Option<f32> {
        if self.confidences.is_empty() {
            None
        } else {
            Some(self.confidences.iter().sum::<f32>() / self.confidences.len() as f32)
        }
    }
    pub fn low_conf_ratio(&self, r_39v: f32) -> Option<f32> {
        if self.confidences.is_empty() {
            return None;
        }
        let r_39u = self.confidences.iter().filter(|&&c| c < r_39v).count();
        Some(r_39u as f32 / self.confidences.len() as f32)
    }
    pub fn text(&self) -> String {
        self.texts.join("\n")
    }
}
pub trait Tok: Send + Sync {
    fn infer_tile(
        &self,
        tile: &Tile,
        batch: usize,
    ) -> impl Future<Output = Result<To6, WorkerError>> + Send;
    /// Kiểm dịch vụ nhận dạng còn sống không — worker dùng để mở lại gate sau khi
    /// gặp lỗi kết nối. Mặc định luôn Ok: engine không có health riêng thì gate
    /// mở ngay và backoff trong DB tự điều tiết.
    fn probe_engine(&self) -> impl Future<Output = Result<(), WorkerError>> + Send {
        async { Ok(()) }
    }
}
const C_C8: f32 = 0.5;
struct Span {
    b: [f32; 4],
    t: String,
    c: f32,
}
fn f_1ip(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    let r_39y = a[0].max(b[0]);
    let r_3a0 = a[1].max(b[1]);
    let r_39z = a[2].min(b[2]);
    let r_3a1 = a[3].min(b[3]);
    let r_39x = (r_39z - r_39y).max(0.0) * (r_3a1 - r_3a0).max(0.0);
    let r_39w = |r_3a2: &[f32; 4]| (r_3a2[2] - r_3a2[0]).max(0.0) * (r_3a2[3] - r_3a2[1]).max(0.0);
    let r_3a3 = r_39w(a) + r_39w(b) - r_39x;
    if r_3a3 <= 0.0 {
        0.0
    } else {
        r_39x / r_3a3
    }
}
fn f_1ux(r_3a8: Vec<Span>) -> Vec<Span> {
    let mut r_3a7: Vec<usize> = (0..r_3a8.len()).collect();
    r_3a7.sort_by(|&i, &r_3a4| {
        r_3a8[r_3a4]
            .c
            .partial_cmp(&r_3a8[i].c)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut r_3a6: Vec<Span> = Vec::new();
    for i in r_3a7 {
        if r_3a6
            .iter()
            .all(|r_3a5| f_1ip(&r_3a8[i].b, &r_3a5.b) <= C_C8)
        {
            r_3a6.push(Span {
                b: r_3a8[i].b,
                t: r_3a8[i].t.clone(),
                c: r_3a8[i].c,
            });
        }
    }
    r_3a6
}
fn f_260(mut r_3ae: Vec<Span>) -> Vec<Span> {
    let r_3ag = |b: &[f32; 4]| (b[1] + b[3]) / 2.0;
    r_3ae.sort_by(|a, b| {
        r_3ag(&a.b)
            .partial_cmp(&r_3ag(&b.b))
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(
                a.b[0]
                    .partial_cmp(&b.b[0])
                    .unwrap_or(std::cmp::Ordering::Equal),
            )
    });
    let mut r_3a9: Vec<Vec<Span>> = Vec::new();
    for r_3ac in r_3ae {
        let r_3ad = r_3a9.last().is_some_and(|line| {
            let r_3ab = &line[0].b;
            let r_3af = 0.6 * (r_3ab[3] - r_3ab[1]).max(r_3ac.b[3] - r_3ac.b[1]).max(1.0);
            (r_3ag(&r_3ac.b) - r_3ag(r_3ab)).abs() <= r_3af
        });
        if r_3ad {
            r_3a9.last_mut().unwrap().push(r_3ac);
        } else {
            r_3a9.push(vec![r_3ac]);
        }
    }
    let mut r_3aa = Vec::with_capacity(r_3a9.iter().map(Vec::len).sum());
    for mut line in r_3a9 {
        line.sort_by(|a, b| {
            a.b[0]
                .partial_cmp(&b.b[0])
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        r_3aa.extend(line);
    }
    r_3aa
}
pub fn f_1si(r_3ap: &[(u32, u32, To6)]) -> Tgi {
    let mut boxes = Vec::new();
    let mut texts = Vec::new();
    let mut confidences = Vec::new();
    for (r_3al, r_3am, r_3aj) in r_3ap {
        let (r_3ah, r_3ai) = (*r_3al as f32, *r_3am as f32);
        for b in &r_3aj.boxes {
            boxes.push([b[0] + r_3ah, b[1] + r_3ai, b[2] + r_3ah, b[3] + r_3ai]);
        }
        texts.extend(r_3aj.texts.iter().cloned());
        confidences.extend(r_3aj.confidences.iter().copied());
    }
    if boxes.len() != texts.len() || texts.len() != confidences.len() {
        return Tgi {
            boxes,
            texts,
            confidences,
        };
    }
    let r_3ao: Vec<Span> = boxes
        .into_iter()
        .zip(texts)
        .zip(confidences)
        .map(|((b, t), c)| Span { b, t, c })
        .collect();
    let r_3ak = f_260(f_1ux(r_3ao));
    let mut boxes = Vec::with_capacity(r_3ak.len());
    let mut texts = Vec::with_capacity(r_3ak.len());
    let mut confidences = Vec::with_capacity(r_3ak.len());
    for r_3an in r_3ak {
        boxes.push(r_3an.b);
        texts.push(r_3an.t);
        confidences.push(r_3an.c);
    }
    Tgi {
        boxes,
        texts,
        confidences,
    }
}
/// Bọc các engine nhận dạng để chọn được lúc chạy.
///
/// `Tok` dùng RPITIT nên KHÔNG object-safe — không thể `Box<dyn Tok>`. Cách
/// giải quyết trong repo này là enum bọc, y như `PageBytesSource` ở
/// render.rs:277-290 đã làm với `Tgd`.
pub enum OcrInferSource {
    Triton(crate::grpc::GrpcTritonInfer),
    Http(crate::http_ocr::HttpOcrInfer),
}
impl Tok for OcrInferSource {
    async fn infer_tile(&self, tile: &Tile, batch: usize) -> Result<To6, WorkerError> {
        match self {
            OcrInferSource::Triton(e) => e.infer_tile(tile, batch).await,
            OcrInferSource::Http(e) => e.infer_tile(tile, batch).await,
        }
    }
    async fn probe_engine(&self) -> Result<(), WorkerError> {
        match self {
            OcrInferSource::Triton(e) => e.probe_engine().await,
            OcrInferSource::Http(e) => e.probe_engine().await,
        }
    }
}
