// Copyright (c) 2026 DigiText. All rights reserved.
use crate::config::WorkerConfig;
use crate::error::{RetryPolicy, WorkerError};
use crate::infer::{f_1si, Tgi, Tile, To6, Tok};
use digitext_core::{DocStatus, JobKind};
use digitext_db::ocr_jobs::OcrPageCounts;
use digitext_db::ocr_results::Tdx;
use digitext_db::{documents, ocr_jobs, ocr_results, review_queue};
use sqlx::PgPool;
use std::future::Future;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;
pub const C_CD: u32 = 256;
const C_F5: u32 = 32;
pub trait Tgj: Send + Sync {
    fn load_tiles(
        &self,
        document_id: Uuid,
        page_no: i32,
    ) -> impl Future<Output = Result<Vec<Tile>, WorkerError>> + Send;
    fn load_page_image(
        &self,
        document_id: Uuid,
        page_no: i32,
    ) -> impl Future<Output = Result<crate::render::Tgf, WorkerError>> + Send;
}
fn f_12t(tile: &Tile, r_3l4: u32, r_3l5: u32, r_3l8: u32, r_3l3: u32) -> Tile {
    let mut r_3l6 = Vec::new();
    if !tile.pixels.is_empty() {
        r_3l6.reserve((r_3l8 * r_3l3 * 3) as usize);
        for r_3l9 in r_3l5..r_3l5 + r_3l3 {
            let r_3l7 = ((r_3l9 * tile.width + r_3l4) * 3) as usize;
            let r_3l2 = r_3l7 + (r_3l8 * 3) as usize;
            if r_3l2 <= tile.pixels.len() {
                r_3l6.extend_from_slice(&tile.pixels[r_3l7..r_3l2]);
            }
        }
    }
    Tile {
        x: tile.x + r_3l4,
        y: tile.y + r_3l5,
        width: r_3l8,
        height: r_3l3,
        pixels: r_3l6,
    }
}
fn f_2gw(tile: &Tile) -> Vec<Tile> {
    if tile.width >= tile.height {
        let ov = C_F5.min(tile.width / 4);
        let r_3lb = (tile.width / 2 + ov).min(tile.width);
        let r_3lc = (tile.width / 2).saturating_sub(ov);
        vec![
            f_12t(tile, 0, 0, r_3lb, tile.height),
            f_12t(tile, r_3lc, 0, tile.width - r_3lc, tile.height),
        ]
    } else {
        let ov = C_F5.min(tile.height / 4);
        let r_3ld = (tile.height / 2 + ov).min(tile.height);
        let r_3la = (tile.height / 2).saturating_sub(ov);
        vec![
            f_12t(tile, 0, 0, tile.width, r_3ld),
            f_12t(tile, 0, r_3la, tile.width, tile.height - r_3la),
        ]
    }
}
pub async fn f_1i1<P: Tok>(
    infer: &P,
    tile: &Tile,
    r_3lf: usize,
    r_3lk: u32,
    r_3lg: &mut u32,
) -> Result<To6, WorkerError> {
    match infer.infer_tile(tile, r_3lf).await {
        Ok(ocr) => Ok(ocr),
        Err(e) if e.is_oom() => {
            if tile.width.max(tile.height) <= r_3lk {
                return Err(e);
            }
            *r_3lg += 1;
            let mut r_3lj = To6 {
                boxes: Vec::new(),
                texts: Vec::new(),
                confidences: Vec::new(),
                ink_coverage: None,
            };
            for r_3ll in f_2gw(tile) {
                let r_3lm = Box::pin(f_1i1(infer, &r_3ll, r_3lf, r_3lk, r_3lg)).await?;
                let (r_3lh, r_3li) = ((r_3ll.x - tile.x) as f32, (r_3ll.y - tile.y) as f32);
                for r_3le in &r_3lm.boxes {
                    r_3lj.boxes.push([
                        r_3le[0] + r_3lh,
                        r_3le[1] + r_3li,
                        r_3le[2] + r_3lh,
                        r_3le[3] + r_3li,
                    ]);
                }
                r_3lj.texts.extend(r_3lm.texts);
                r_3lj.confidences.extend(r_3lm.confidences);
                r_3lj.ink_coverage =
                    crate::infer::min_ink_coverage(r_3lj.ink_coverage, r_3lm.ink_coverage);
            }
            Ok(r_3lj)
        }
        Err(e) => Err(e),
    }
}
pub async fn f_242<P: Tok>(
    infer: &P,
    r_3lr: &[Tile],
    r_3ln: usize,
    r_3lq: u32,
    r_3lo: &mut u32,
) -> Result<Tgi, WorkerError> {
    let mut r_3lp = Vec::with_capacity(r_3lr.len());
    for tile in r_3lr {
        let ocr = f_1i1(infer, tile, r_3ln, r_3lq, r_3lo).await?;
        r_3lp.push((tile.x, tile.y, ocr));
    }
    Ok(f_1si(&r_3lp))
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReviewReason {
    LowConfidence,
    EmptyPage,
    LowInkCoverage,
}
impl ReviewReason {
    fn as_str(self) -> &'static str {
        match self {
            ReviewReason::LowConfidence => review_queue::C_I3,
            ReviewReason::EmptyPage => review_queue::C_I0,
            ReviewReason::LowInkCoverage => review_queue::REASON_LOW_INK_COVERAGE,
        }
    }
}
/// Chốt "đã bắt hết chữ chưa": dịch vụ OCR đo được độ phủ mực và nó dưới ngưỡng → còn chữ
/// trên trang chưa có khung. Xét TRƯỚC độ tin cậy vì chữ sót thì không có trong lớp text,
/// nặng hơn chữ đọc chưa chắc. `min <= 0` = tắt; `None` (engine không đo) = không kết luận.
fn ink_coverage_too_low(coverage: Option<f32>, min: f32) -> bool {
    matches!(coverage, Some(c) if min > 0.0 && c < min)
}
fn needs_review(r_3lu: usize, r_3ls: Option<f32>, r_3lv: f32) -> Option<ReviewReason> {
    if r_3lu == 0 {
        return Some(ReviewReason::EmptyPage);
    }
    match r_3ls {
        Some(r_3lt) if r_3lt > 0.0 && r_3lt >= r_3lv => Some(ReviewReason::LowConfidence),
        _ => None,
    }
}
#[derive(Debug)]
pub enum HeartbeatEnd {
    LeaseLost,
    Db(sqlx::Error),
}
pub async fn f_2bg<T, W, R, RF>(r_3lz: W, r_3lw: Duration, mut r_3lx: R) -> Result<T, HeartbeatEnd>
where
    W: Future<Output = T>,
    R: FnMut() -> RF,
    RF: Future<Output = sqlx::Result<bool>>,
{
    tokio::pin!(r_3lz);
    let mut r_3ly = tokio::time::interval(r_3lw);
    r_3ly.tick().await;
    loop {
        tokio::select! {
            biased; out = & mut r_3lz => return Ok(out), _ = r_3ly.tick() => match
            r_3lx(). await { Ok(true) => {} Ok(false) => return
            Err(HeartbeatEnd::LeaseLost), Err(e) => return Err(HeartbeatEnd::Db(e)), },
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunOutcome {
    Idle,
    Done(Uuid),
    PageFailed(Uuid),
    Requeued(Uuid),
    FailedTransient(Uuid),
    LeaseLost(Uuid),
}
/// Kết luận OCR của một tài liệu từ số trang theo trạng thái.
///
/// `page_failed` (lỗi vĩnh viễn, trang bị cô lập) KHÔNG chặn: tài liệu vẫn xong để
/// người dùng thấy và xử lý trang đó. `failed_transient` (hết lượt thử vì dịch vụ
/// chết) PHẢI chặn — trước đây bị bỏ qua nên tài liệu thiếu trang vẫn thành
/// `ocr_done`, PDF 2 lớp dựng thiếu trang mà màn hình báo 100%.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcrDocOutcome {
    StillPending,
    Finished,
    BlockedByTransient(i64),
}
pub fn decide_ocr_outcome(c: &OcrPageCounts) -> OcrDocOutcome {
    if c.pending > 0 {
        OcrDocOutcome::StillPending
    } else if c.failed_transient > 0 {
        OcrDocOutcome::BlockedByTransient(c.failed_transient)
    } else {
        OcrDocOutcome::Finished
    }
}
#[derive(Debug, Default)]
struct HealthGate {
    down_since: Option<Instant>,
    probes: u32,
}
const C_HG_MIN: Duration = Duration::from_secs(1);
const C_HG_MAX: Duration = Duration::from_secs(30);
pub struct Worker<'a, S, I, C = crate::corrector::T4s> {
    pool: &'a PgPool,
    source: &'a S,
    infer: &'a I,
    cfg: &'a WorkerConfig,
    worker_id: &'a str,
    engine_version_id: Uuid,
    corrector: Option<(&'a C, &'a digitext_vi::Lexicon)>,
    store: Option<&'a dyn digitext_storage::ObjectStore>,
    vlm: Option<&'a dyn crate::vlm_fields::T5b>,
    fields_v2: Option<(&'a dyn crate::fields_v2::T80, Uuid)>,
    metrics: Option<&'a crate::metrics::Metrics>,
    health: Mutex<HealthGate>,
    probing: AtomicBool,
}
impl<'a, S: Tgj, I: Tok> Worker<'a, S, I, crate::corrector::T4s> {
    pub fn new(
        pool: &'a PgPool,
        source: &'a S,
        infer: &'a I,
        cfg: &'a WorkerConfig,
        worker_id: &'a str,
        engine_version_id: Uuid,
    ) -> Self {
        Worker {
            pool,
            source,
            infer,
            cfg,
            worker_id,
            engine_version_id,
            corrector: None,
            store: None,
            vlm: None,
            fields_v2: None,
            metrics: None,
            health: Mutex::new(HealthGate::default()),
            probing: AtomicBool::new(false),
        }
    }
}
impl<'a, S: Tgj, I: Tok, C: crate::corrector::Tb4> Worker<'a, S, I, C> {
    #[allow(clippy::too_many_arguments)]
    pub fn with_corrector(
        pool: &'a PgPool,
        source: &'a S,
        infer: &'a I,
        cfg: &'a WorkerConfig,
        worker_id: &'a str,
        engine_version_id: Uuid,
        corrector: &'a C,
        r_3m0: &'a digitext_vi::Lexicon,
    ) -> Self {
        Worker {
            pool,
            source,
            infer,
            cfg,
            worker_id,
            engine_version_id,
            corrector: Some((corrector, r_3m0)),
            store: None,
            vlm: None,
            fields_v2: None,
            metrics: None,
            health: Mutex::new(HealthGate::default()),
            probing: AtomicBool::new(false),
        }
    }
    fn retry_policy(&self) -> RetryPolicy {
        RetryPolicy::new(
            self.cfg.max_retries,
            self.cfg.retry_base_delay,
            self.cfg.retry_max_delay,
        )
    }
    /// Ghi lỗi tạm thời cho job: lùi lịch theo backoff (không thử lại tức thì như
    /// trước), log đủ để thấy trong journal, và trả outcome tương ứng.
    async fn fail_transient(
        &self,
        job: &ocr_jobs::Ta2,
        what: &str,
        err: &str,
        engine_down: bool,
    ) -> sqlx::Result<RunOutcome> {
        let delay = self
            .retry_policy()
            .backoff(job.attempts.max(0) as u32);
        let out = ocr_jobs::f_1bn(
            self.pool,
            job.id,
            self.worker_id,
            err,
            self.cfg.max_retries as i32,
            delay.as_millis() as i64,
        )
        .await?;
        if engine_down {
            self.mark_engine_down(err);
        }
        Ok(match out {
            Some(o) if o.requeued() => {
                tracing::warn!(
                    job = % job.id, doc = % job.document_id, page = job.page_no, kind = % job.kind,
                    attempts = o.attempts, max = self.cfg.max_retries,
                    delay_ms = delay.as_millis() as u64,
                    "{what}: lỗi tạm thời — thử lại sau: {err}"
                );
                RunOutcome::Requeued(job.id)
            }
            Some(o) => {
                tracing::error!(
                    job = % job.id, doc = % job.document_id, page = job.page_no, kind = % job.kind,
                    attempts = o.attempts,
                    "{what}: lỗi tạm thời, HẾT lượt thử — giữ failed_transient, cần bấm chạy lại: {err}"
                );
                RunOutcome::FailedTransient(job.id)
            }
            None => {
                tracing::warn!(
                    job = % job.id,
                    "{what}: mất lease trước khi ghi lỗi — chủ mới sẽ làm lại"
                );
                RunOutcome::LeaseLost(job.id)
            }
        })
    }
    fn mark_engine_down(&self, err: &str) {
        let mut g = self.health.lock().unwrap_or_else(|p| p.into_inner());
        if g.down_since.is_none() {
            g.down_since = Some(Instant::now());
            g.probes = 0;
            tracing::warn!(
                worker_id = self.worker_id,
                "dịch vụ OCR không phản hồi — các lane ngừng nhận job, chờ health: {err}"
            );
            if let Some(m) = self.metrics {
                m.engine_healthy.set(0);
                m.engine_unavailable_total.inc();
            }
        }
    }
    /// Chặn lane khi dịch vụ OCR đang chết: một lane probe theo backoff 1→30 s, các
    /// lane khác ngủ. Không tiêu `attempts` của job nào trong lúc chờ. Trả `None`
    /// khi bị huỷ.
    async fn wait_engine_healthy(&self, cancel: &CancellationToken) -> Option<()> {
        loop {
            let (down_since, probes) = {
                let g = self.health.lock().unwrap_or_else(|p| p.into_inner());
                match g.down_since {
                    None => return Some(()),
                    Some(t) => (t, g.probes),
                }
            };
            let elected = self
                .probing
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_ok();
            if !elected {
                tokio::select! {
                    _ = cancel.cancelled() => return None,
                    _ = tokio::time::sleep(C_HG_MIN) => {}
                }
                continue;
            }
            let wait = RetryPolicy::new(u32::MAX, C_HG_MIN, C_HG_MAX).backoff(probes);
            let cancelled = tokio::select! {
                _ = cancel.cancelled() => true,
                _ = tokio::time::sleep(wait) => false,
            };
            if cancelled {
                self.probing.store(false, Ordering::Release);
                return None;
            }
            let r = self.infer.probe_engine().await;
            {
                let mut g = self.health.lock().unwrap_or_else(|p| p.into_inner());
                match r {
                    Ok(()) => {
                        g.down_since = None;
                        g.probes = 0;
                        tracing::info!(
                            worker_id = self.worker_id,
                            down_secs = down_since.elapsed().as_secs(),
                            "dịch vụ OCR đã trở lại — tiếp tục nhận job"
                        );
                        if let Some(m) = self.metrics {
                            m.engine_healthy.set(1);
                        }
                    }
                    Err(e) => {
                        g.probes = g.probes.saturating_add(1);
                        let next = RetryPolicy::new(u32::MAX, C_HG_MIN, C_HG_MAX).backoff(g.probes);
                        if g.probes % 5 == 1 {
                            tracing::warn!(
                                probes = g.probes, next_wait_s = next.as_secs(),
                                down_secs = down_since.elapsed().as_secs(),
                                "dịch vụ OCR vẫn chưa sẵn sàng: {e}"
                            );
                        }
                    }
                }
            }
            self.probing.store(false, Ordering::Release);
        }
    }
    pub fn with_store(mut self, store: &'a dyn digitext_storage::ObjectStore) -> Self {
        self.store = Some(store);
        self
    }
    pub fn with_metrics(mut self, metrics: &'a crate::metrics::Metrics) -> Self {
        self.metrics = Some(metrics);
        self
    }
    pub fn with_vlm(mut self, vlm: &'a dyn crate::vlm_fields::T5b) -> Self {
        self.vlm = Some(vlm);
        self
    }
    pub fn with_fields_v2(
        mut self,
        extractor: &'a dyn crate::fields_v2::T80,
        engine_version_id: Uuid,
    ) -> Self {
        self.fields_v2 = Some((extractor, engine_version_id));
        self
    }
    pub async fn run_once(&self) -> sqlx::Result<RunOutcome> {
        let lease = self.cfg.lease.as_secs() as i64;
        let r_3m7: &[&str] = if self.fields_v2.is_some() {
            &["ocr", "re_ocr", "audit", "make_pdf2", "extract_fields"]
        } else {
            &["ocr", "re_ocr", "audit", "make_pdf2"]
        };
        let Some(job) = ocr_jobs::f_zj(self.pool, self.worker_id, lease, r_3m7).await? else {
            return Ok(RunOutcome::Idle);
        };
        if job.kind == JobKind::MakePdf2.as_str() {
            return self.run_make_pdf2(&job, lease).await;
        }
        if job.kind == JobKind::ExtractFields.as_str() {
            return self.run_extract_fields(&job, lease).await;
        }
        let r_3t0 = Instant::now();
        let r_3m1 = self.cfg.infer_batch_size;
        let mut r_3m4 = 0u32;
        let r_3md = async {
            match self.source.load_tiles(job.document_id, job.page_no).await {
                Ok(r_3mg) => f_242(self.infer, &r_3mg, r_3m1, C_CD, &mut r_3m4).await,
                Err(e) => Err(e),
            }
        };
        let r_3m5 = (self.cfg.lease / 3).max(Duration::from_secs(1));
        let r_3mc = match f_2bg(r_3md, r_3m5, || {
            ocr_jobs::f_1gk(self.pool, job.id, self.worker_id, lease)
        })
        .await
        {
            Ok(r_3me) => r_3me,
            Err(HeartbeatEnd::LeaseLost) => {
                tracing::warn!(
                    job = % job.id,
                    "mất lease khi xử lý — bỏ (chủ mới sẽ làm lại)"
                );
                return Ok(RunOutcome::LeaseLost(job.id));
            }
            Err(HeartbeatEnd::Db(e)) => return Err(e),
        };
        if r_3m4 > 0 {
            tracing::info!(
                job = % job.id, r_3m4,
                "đã chia nhỏ tile do GPU OOM (§P4)"
            );
            if let Some(m) = self.metrics {
                m.oom_downshift_total.add(r_3m4 as u64);
            }
        }
        match r_3mc {
            Ok(r_3ma) => {
                if let Some(m) = self.metrics {
                    if let Some(c) = r_3ma.mean_confidence() {
                        m.page_confidence.observe(c as f64);
                    }
                }
                let r_3m2 = digitext_vi::normalize(&r_3ma.text());
                ocr_results::upsert(
                    self.pool,
                    Tdx {
                        document_id: job.document_id,
                        page_no: job.page_no,
                        engine_version_id: self.engine_version_id,
                        text_source: "ocr_raw",
                        content: &r_3m2,
                        mean_confidence: r_3ma.mean_confidence(),
                        low_conf_ratio: r_3ma.low_conf_ratio(self.cfg.conf_fallback),
                        boxes: Some(crate::assemble::f_2gl(&r_3ma)),
                        ink_coverage: r_3ma.ink_coverage,
                    },
                )
                .await?;
                let fields = if self.cfg.fields_engine == crate::config::FieldsEngine::V1 {
                    let fields = crate::fields::extract_fields(&r_3ma);
                    crate::fields::f_2ho(
                        self.pool,
                        job.document_id,
                        job.page_no,
                        self.engine_version_id,
                        "ocr_raw",
                        &fields,
                    )
                    .await?;
                    Some(fields)
                } else {
                    None
                };
                let r_ink_low =
                    ink_coverage_too_low(r_3ma.ink_coverage, self.cfg.review_min_ink_coverage);
                if r_ink_low {
                    tracing::warn!(
                        document = % job.document_id, page = job.page_no,
                        ink_coverage = ? r_3ma.ink_coverage,
                        "độ phủ mực thấp — còn chữ chưa có khung → đưa người soát"
                    );
                }
                let r_reason = if r_ink_low && !r_3ma.confidences.is_empty() {
                    Some(ReviewReason::LowInkCoverage)
                } else {
                    needs_review(
                        r_3ma.confidences.len(),
                        r_3ma.low_conf_ratio(self.cfg.conf_fallback),
                        self.cfg.review_low_conf_ratio,
                    )
                };
                if let Some(reason) = r_reason {
                    if reason == ReviewReason::EmptyPage {
                        tracing::warn!(
                            document = % job.document_id, r_3ma = job.page_no,
                            "trang OCR ra KHÔNG có span nào → đưa người soát"
                        );
                    }
                    review_queue::enqueue(self.pool, job.document_id, job.page_no, reason.as_str())
                        .await?;
                }
                if self.cfg.llm_corrector_enabled {
                    if let Some((r_3m3, r_3m8)) = self.corrector {
                        let gate = crate::corrector::T8j {
                            tau: self.cfg.corrector_tau,
                            max_edit: self.cfg.corrector_max_edit,
                            min_llm_conf: self.cfg.corrector_min_conf,
                        };
                        match crate::corrector::f_s4(
                            self.pool,
                            job.document_id,
                            job.page_no,
                            self.engine_version_id,
                            &r_3ma,
                            r_3m3,
                            gate,
                            self.cfg.conf_fallback,
                            |r_3mh| r_3m8.contains(r_3mh),
                        )
                        .await
                        {
                            Ok(c) => {
                                if let Some(m) = self.metrics {
                                    if c.accepted > 0 {
                                        m.llm_edits_applied_total.add(c.accepted as u64);
                                    }
                                    if c.needs_review {
                                        m.llm_edits_rejected_total.inc();
                                    }
                                }
                                if c.accepted > 0 || c.needs_review {
                                    tracing::info!(
                                        job = % job.id, accepted = c.accepted, review = c
                                        .needs_review, "corrector LLM đã áp (§G6)"
                                    );
                                }
                            }
                            Err(crate::corrector::ApplyError::Corrector(e)) => {
                                tracing::warn!(
                                    job = % job.id,
                                    "corrector LLM lỗi, giữ raw: {e}"
                                )
                            }
                            Err(crate::corrector::ApplyError::Db(e)) => return Err(e),
                        }
                    }
                }
                if self.cfg.vlm_fallback_enabled {
                    if let (Some(vlm), Some(fields)) = (self.vlm, fields.as_ref()) {
                        let gate = crate::vlm_fields::Tpy {
                            min_fields: 3,
                            tau: 0.6,
                            max_low_conf: 0.3,
                            keep_heuristic_conf: 0.7,
                        };
                        if crate::vlm_fields::f_2ey(
                            fields.len(),
                            r_3ma.mean_confidence().unwrap_or(1.0),
                            r_3ma.low_conf_ratio(self.cfg.conf_fallback).unwrap_or(0.0),
                            gate,
                        ) {
                            match self
                                .source
                                .load_page_image(job.document_id, job.page_no)
                                .await
                            {
                                Ok(r_3m6) => {
                                    match vlm
                                        .extract_dyn(&r_3m6.jpeg, r_3m6.width_px, r_3m6.height_px)
                                        .await
                                    {
                                        Ok(raw) => {
                                            let r_3mb = crate::vlm_fields::f_1sk(
                                                fields,
                                                raw,
                                                gate.keep_heuristic_conf,
                                            );
                                            let r_3m9 = crate::vlm_fields::f_s5(
                                                self.pool,
                                                job.document_id,
                                                job.page_no,
                                                self.engine_version_id,
                                                &r_3mb,
                                            )
                                            .await?;
                                            if r_3m9 > 0 {
                                                tracing::info!(
                                                    job = % job.id, fields = r_3m9,
                                                    "VLM Mode 2 bóc trường ca khó (§G6) → review"
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            tracing::warn!(
                                                job = % job.id,
                                                "VLM Mode 2 lỗi, giữ heuristic: {e}"
                                            )
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        job = % job.id,
                                        "VLM Mode 2: tải ảnh trang lỗi, bỏ: {e}"
                                    )
                                }
                            }
                        }
                    }
                }
                if ocr_jobs::f_1ra(self.pool, job.id, self.worker_id).await? {
                    self.maybe_enqueue_make_pdf2(job.document_id).await?;
                }
                if let Some(m) = self.metrics {
                    m.pages_processed.inc("done");
                    m.page_latency_seconds.observe(r_3t0.elapsed().as_secs_f64());
                }
                Ok(RunOutcome::Done(job.id))
            }
            Err(e) if e.is_permanent() => {
                tracing::warn!(
                    job = % job.id, doc = % job.document_id, r_3ma = job.page_no,
                    "trang lỗi VĨNH VIỄN — cô lập, không thử lại: {e}"
                );
                if ocr_jobs::f_1re(self.pool, job.id, self.worker_id, &e.to_string()).await? {
                    self.maybe_enqueue_make_pdf2(job.document_id).await?;
                }
                if let Some(m) = self.metrics {
                    m.pages_processed.inc("page_failed");
                }
                Ok(RunOutcome::PageFailed(job.id))
            }
            Err(e) => {
                let engine_down = matches!(
                    e,
                    WorkerError::TritonUnavailable(_) | WorkerError::Timeout(_)
                );
                let out = self
                    .fail_transient(&job, "OCR trang", &e.to_string(), engine_down)
                    .await?;
                if let Some(m) = self.metrics {
                    m.pages_processed.inc(match out {
                        RunOutcome::Requeued(_) => "queued",
                        RunOutcome::FailedTransient(_) => "failed_transient",
                        _ => "lease_lost",
                    });
                }
                Ok(out)
            }
        }
    }
    async fn maybe_enqueue_make_pdf2(&self, r_3mi: Uuid) -> sqlx::Result<()> {
        let counts = ocr_jobs::ocr_page_counts(self.pool, r_3mi).await?;
        match decide_ocr_outcome(&counts) {
            OcrDocOutcome::StillPending => {}
            OcrDocOutcome::BlockedByTransient(n) => {
                tracing::warn!(
                    doc = % r_3mi, failed_transient = n, done = counts.done,
                    page_failed = counts.page_failed,
                    "OCR chưa xong: {n} trang lỗi tạm thời đã hết lượt thử — giữ ocr_pending, cần bấm chạy lại"
                );
            }
            OcrDocOutcome::Finished => {
                if !documents::transition(self.pool, r_3mi, DocStatus::OcrDone).await? {
                    tracing::debug!(
                        doc = % r_3mi,
                        "không chuyển được sang ocr_done (đã ở bước sau hoặc bị huỷ)"
                    );
                }
                if self.cfg.auto_make_pdf2 {
                    ocr_jobs::f_19r(self.pool, r_3mi).await?;
                }
            }
        }
        Ok(())
    }
    async fn run_make_pdf2(&self, job: &ocr_jobs::Ta2, lease: i64) -> sqlx::Result<RunOutcome> {
        let r_3mk = (self.cfg.lease / 3).max(Duration::from_secs(1));
        let r_3mj =
            crate::assemble::f_va(self.pool, self.source, job.document_id, self.cfg.pdf_dpi);
        let r_3mn = match f_2bg(r_3mj, r_3mk, || {
            ocr_jobs::f_1gk(self.pool, job.id, self.worker_id, lease)
        })
        .await
        {
            Ok(r_3mm) => r_3mm,
            Err(HeartbeatEnd::LeaseLost) => {
                tracing::warn!(
                    job = % job.id,
                    "mất lease khi ghép PDF — bỏ (chủ mới sẽ làm lại)"
                );
                return Ok(RunOutcome::LeaseLost(job.id));
            }
            Err(HeartbeatEnd::Db(e)) => return Err(e),
        };
        use crate::assemble::AssembleError;
        match r_3mn {
            Ok(None) => {
                tracing::info!(
                    job = % job.id, doc = % job.document_id,
                    "make_pdf2: không có trang OCR nào — bỏ qua ghép PDF"
                );
                ocr_jobs::f_1ra(self.pool, job.id, self.worker_id).await?;
                Ok(RunOutcome::Done(job.id))
            }
            Ok(Some(r_3ml)) => self.persist_pdf(job, r_3ml).await,
            Err(AssembleError::Db(e)) => Err(e),
            Err(AssembleError::Data(msg)) => {
                tracing::warn!(
                    job = % job.id, doc = % job.document_id,
                    "ghép PDF 2 lớp hỏng VĨNH VIỄN — không thử lại: {msg}"
                );
                ocr_jobs::f_1re(self.pool, job.id, self.worker_id, &msg).await?;
                Ok(RunOutcome::PageFailed(job.id))
            }
            Err(AssembleError::Source(e)) if e.is_permanent() => {
                tracing::warn!(
                    job = % job.id, doc = % job.document_id,
                    "ghép PDF 2 lớp: nguồn ảnh hỏng VĨNH VIỄN — không thử lại: {e}"
                );
                ocr_jobs::f_1re(self.pool, job.id, self.worker_id, &e.to_string()).await?;
                Ok(RunOutcome::PageFailed(job.id))
            }
            Err(AssembleError::Source(e)) => {
                self.fail_transient(job, "ghép PDF 2 lớp: nguồn ảnh", &e.to_string(), false)
                    .await
            }
        }
    }
    async fn run_extract_fields(
        &self,
        job: &ocr_jobs::Ta2,
        lease: i64,
    ) -> sqlx::Result<RunOutcome> {
        use crate::fields_v2::ExtractV2Error;
        let Some((extractor, r_3n0)) = self.fields_v2 else {
            return self
                .fail_transient(
                    job,
                    "bóc trường v2",
                    "worker không có VLM extractor v2 (DIGITEXT_FIELDS_ENGINE)",
                    false,
                )
                .await;
        };
        let Some(r_3mq) = documents::get(self.pool, job.document_id).await? else {
            tracing::warn!(
                job = % job.id, r_3mq = % job.document_id,
                "tài liệu không còn — bỏ bóc trường"
            );
            ocr_jobs::f_1ra(self.pool, job.id, self.worker_id).await?;
            return Ok(RunOutcome::Done(job.id));
        };
        let page_count = r_3mq.page_count.unwrap_or(0);
        if page_count < 1 {
            tracing::warn!(
                job = % job.id, r_3mq = % job.document_id,
                "bóc trường v2: chưa biết số trang (page_count) — bỏ VĨNH VIỄN"
            );
            ocr_jobs::f_1re(
                    self.pool,
                    job.id,
                    self.worker_id,
                    "chưa biết số trang (page_count) — probe số trang rồi chạy lại",
                )
                .await?;
            return Ok(RunOutcome::PageFailed(job.id));
        }
        let r_3mu = (self.cfg.fields_v2_max_pages as i32).min(page_count);
        let r_3mt = (self.cfg.lease / 3).max(Duration::from_secs(1));
        let r_3mr = async {
            let mut per_page: Vec<(i32, Vec<(digitext_core::VanBanField, String)>)> = Vec::new();
            for r_3mx in 1..=r_3mu {
                let img = match self.source.load_page_image(job.document_id, r_3mx).await {
                    Ok(img) => img,
                    Err(e) if e.is_permanent() && r_3mx > 1 => {
                        tracing::warn!(
                            job = % job.id, page = r_3mx,
                            "bóc trường v2: ảnh trang hỏng, bỏ trang: {e}"
                        );
                        continue;
                    }
                    Err(e) => return Err(ExtractV2Error::Source(e)),
                };
                let r_3ms = extractor
                    .extract_v2(&img.jpeg)
                    .await
                    .map_err(ExtractV2Error::Vlm)?;
                per_page.push((r_3mx, r_3ms));
            }
            Ok(per_page)
        };
        let r_3mz = match f_2bg(r_3mr, r_3mt, || {
            ocr_jobs::f_1gk(self.pool, job.id, self.worker_id, lease)
        })
        .await
        {
            Ok(r_3my) => r_3my,
            Err(HeartbeatEnd::LeaseLost) => {
                tracing::warn!(
                    job = % job.id,
                    "mất lease khi bóc trường v2 — bỏ (chủ mới làm lại)"
                );
                return Ok(RunOutcome::LeaseLost(job.id));
            }
            Err(HeartbeatEnd::Db(e)) => return Err(e),
        };
        match r_3mz {
            Ok(per_page) => {
                let r_3mv = crate::fields_v2::f_1sa(&per_page);
                let r_3mp = crate::fields_v2::f_14m(r_3mq.page_count, &r_3mv);
                let r_3mw =
                    crate::fields_v2::f_s3(self.pool, job.document_id, r_3n0, &r_3mv, &r_3mp)
                        .await?;
                tracing::info!(
                    job = % job.id, r_3mq = % job.document_id, pages = r_3mu, r_3ms =
                    r_3mw,
                    "bóc 16 trường v2 (VLM-primary) → lớp gợi ý + review (§G7)"
                );
                ocr_jobs::f_1ra(self.pool, job.id, self.worker_id).await?;
                Ok(RunOutcome::Done(job.id))
            }
            Err(e) if e.is_permanent() => {
                tracing::warn!(
                    job = % job.id, r_3mq = % job.document_id,
                    "bóc trường v2 hỏng VĨNH VIỄN — không thử lại: {e}"
                );
                ocr_jobs::f_1re(self.pool, job.id, self.worker_id, &e.to_string()).await?;
                Ok(RunOutcome::PageFailed(job.id))
            }
            Err(e) => {
                self.fail_transient(job, "bóc trường v2", &e.to_string(), false)
                    .await
            }
        }
    }
    async fn persist_pdf(&self, job: &ocr_jobs::Ta2, r_3n4: Vec<u8>) -> sqlx::Result<RunOutcome> {
        let Some(store) = self.store else {
            tracing::info!(
                job = % job.id, r_3n2 = % job.document_id, bytes = r_3n4.len(),
                "PDF 2 lớp đã dựng (chưa PUT — chưa cấu hình kho DIGITEXT_PDF_STORE_DIR)"
            );
            ocr_jobs::f_1ra(self.pool, job.id, self.worker_id).await?;
            return Ok(RunOutcome::Done(job.id));
        };
        let Some(r_3n2) = documents::get(self.pool, job.document_id).await? else {
            tracing::warn!(
                job = % job.id, r_3n2 = % job.document_id,
                "tài liệu không còn — bỏ PUT PDF"
            );
            ocr_jobs::f_1ra(self.pool, job.id, self.worker_id).await?;
            return Ok(RunOutcome::Done(job.id));
        };
        let r_3n1 = digitext_pdf::f_2p9(&r_3n4, r_3n2.page_count);
        if !r_3n1.ok {
            let reason = r_3n1
                .reason
                .unwrap_or_else(|| "không đạt".into());
            tracing::error!(
                job = % job.id, r_3n2 = % job.document_id, pages = r_3n1.pages,
                page_count = ? r_3n2.page_count, reason = % reason,
                "PDF 2 lớp KHÔNG đạt — không PUT, không chuyển pha ký"
            );
            self.notify_pdf2_rejected(&r_3n2, &reason).await?;
            ocr_jobs::f_1re(
                self.pool,
                job.id,
                self.worker_id,
                &format!("PDF 2 lớp không đạt: {reason}"),
            )
            .await?;
            return Ok(RunOutcome::PageFailed(job.id));
        }
        let r_3n3 = digitext_storage::f_2qs(r_3n2.project_id, job.document_id);
        match store.put(&r_3n3, &r_3n4).await {
            Ok(()) => {
                documents::f_1rf(
                    self.pool,
                    job.document_id,
                    documents::Tgy {
                        key: &r_3n3,
                        pages: r_3n1.pages as i32,
                        pdfa_2b: digitext_pdf::pdfa::f_1rh(&r_3n4).is_pdfa2b(),
                    },
                )
                .await?;
                documents::transition(self.pool, job.document_id, DocStatus::ToSign).await?;
                ocr_jobs::f_19u(self.pool, job.document_id).await?;
                tracing::info!(
                    job = % job.id, r_3n2 = % job.document_id, bytes = r_3n4.len(), r_3n3
                    = % r_3n3,
                    "PDF 2 lớp đã PUT kho warm → to_sign, chờ ký (§S5.4 G)"
                );
                ocr_jobs::f_1ra(self.pool, job.id, self.worker_id).await?;
                Ok(RunOutcome::Done(job.id))
            }
            Err(e) => {
                self.fail_transient(job, "PUT PDF 2 lớp vào kho", &e.to_string(), false)
                    .await
            }
        }
    }
    async fn notify_pdf2_rejected(
        &self,
        r_3n7: &documents::Document,
        reason: &str,
    ) -> sqlx::Result<()> {
        let msg = format!(
            "Tài liệu \"{}\": PDF 2 lớp KHÔNG đạt ({reason}) — chưa chuyển ký. Cần OCR lại trang lỗi rồi ghép lại.",
            r_3n7.filename
        );
        for r_3n6 in digitext_db::projects::f_re(self.pool, r_3n7.project_id).await? {
            digitext_db::notifications::f_1v8(self.pool, r_3n6, None, "pdf2_rejected", &msg)
                .await?;
        }
        Ok(())
    }
    pub async fn run_loop(&self, r_3na: Duration, r_3nf: &CancellationToken) -> sqlx::Result<u64> {
        let r_3nb = self.cfg.worker_concurrency.max(1);
        let r_3ng = r_3nf.child_token();
        let mut r_3ne = Vec::with_capacity(r_3nb);
        {
            let r_3n9: Vec<_> = (0..r_3nb)
                .map(|_| async {
                    let r = self.run_lane(r_3na, &r_3ng).await;
                    if r.is_err() {
                        r_3ng.cancel();
                    }
                    r
                })
                .collect();
            for r in futures_util::future::join_all(r_3n9).await {
                r_3ne.push(r);
            }
        }
        let mut r_3nd = 0u64;
        let mut r_3n8 = None;
        for r in r_3ne {
            match r {
                Ok(r_3nc) => r_3nd += r_3nc,
                Err(e) => {
                    r_3n8.get_or_insert(e);
                }
            }
        }
        tracing::info!(
            worker_id = self.worker_id,
            r_3nd,
            r_3nb,
            "vòng lặp worker dừng"
        );
        if let Some(e) = r_3n8 {
            return Err(e);
        }
        Ok(r_3nd)
    }
    async fn run_lane(&self, r_3nh: Duration, r_3nl: &CancellationToken) -> sqlx::Result<u64> {
        let mut r_3nj = 0u64;
        while !r_3nl.is_cancelled() {
            if self.wait_engine_healthy(r_3nl).await.is_none() {
                break;
            }
            let Some(r_3nk) = f_1u9(self.run_once(), r_3nl).await else {
                break;
            };
            let r_3ni = match r_3nk {
                Ok(o) => o,
                // Lỗi toàn vẹn dữ liệu (SQLSTATE 23xxx: khoá ngoại, trùng khoá…) là của
                // RIÊNG job vừa chạy — điển hình tài liệu bị xoá giữa chừng nên ghi
                // ocr_results vi phạm FK. Không phải lỗi kết nối → không hạ cả worker.
                Err(e) if is_row_level_db_error(&e) => {
                    tracing::warn!(error = %e, "job vi phạm toàn vẹn dữ liệu — bỏ qua, chạy tiếp");
                    continue;
                }
                Err(e) => return Err(e),
            };
            if r_3ni == RunOutcome::Idle {
                tokio::select! {
                    _ = r_3nl.cancelled() => break, _ = tokio::time::sleep(r_3nh) => {}
                }
            } else {
                r_3nj += 1;
            }
        }
        Ok(r_3nj)
    }
}
/// SQLSTATE lớp 23 (integrity constraint violation) — lỗi thuộc về dòng dữ liệu đang
/// xử lý, không phải kết nối/CSDL hỏng. Worker được phép bỏ qua job đó và chạy tiếp.
pub(crate) fn is_row_level_db_error(e: &sqlx::Error) -> bool {
    e.as_database_error()
        .and_then(|d| d.code())
        .map(|c| c.starts_with("23"))
        .unwrap_or(false)
}
async fn f_1u9<F: Future>(r_3nm: F, r_3nn: &CancellationToken) -> Option<F::Output> {
    tokio::select! {
        biased; r = r_3nm => Some(r), _ = r_3nn.cancelled() => None,
    }
}
pub async fn f_2b8(
    pool: &PgPool,
    r_3no: Duration,
    r_3np: i32,
    r_3nq: &CancellationToken,
    metrics: Option<Arc<crate::metrics::Metrics>>,
) -> sqlx::Result<()> {
    while !r_3nq.is_cancelled() {
        tokio::select! {
            _ = r_3nq.cancelled() => break, _ = tokio::time::sleep(r_3no) => { let n =
            ocr_jobs::requeue_expired(pool, r_3np). await ?; if n > 0 {
            tracing::warn!(requeued = n, "requeue job hết hạn lease"); }
            if let Some(m) = metrics.as_ref() {
                let queued = ocr_jobs::count_by_status(pool).await?
                    .into_iter()
                    .filter(|(s, _)| s == "queued")
                    .map(|(_, n)| n)
                    .sum::<i64>();
                m.queue_depth.set(queued);
            } }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn do_phu_muc_thap_thi_dua_nguoi_soat() {
        assert!(ink_coverage_too_low(Some(0.80), 0.93));
        assert!(!ink_coverage_too_low(Some(0.97), 0.93));
        // Engine không đo (Triton / dịch vụ OCR bản cũ) → không kết luận gì.
        assert!(!ink_coverage_too_low(None, 0.93));
        // Ngưỡng 0 = tắt chốt.
        assert!(!ink_coverage_too_low(Some(0.10), 0.0));
        // Gộp tile: lấy giá trị xấu nhất, mảnh không đo thì bỏ qua.
        use crate::infer::min_ink_coverage;
        assert_eq!(min_ink_coverage(Some(0.9), Some(0.7)), Some(0.7));
        assert_eq!(min_ink_coverage(None, Some(0.7)), Some(0.7));
        assert_eq!(min_ink_coverage(Some(0.9), None), Some(0.9));
        assert_eq!(min_ink_coverage(None, None), None);
    }

    #[test]
    fn loi_ket_noi_khong_phai_loi_dong() {
        // Không có mã SQLSTATE → không phải lỗi dòng → vẫn phải hạ worker như cũ.
        assert!(!is_row_level_db_error(&sqlx::Error::PoolTimedOut));
        assert!(!is_row_level_db_error(&sqlx::Error::RowNotFound));
    }
    fn counts(pending: i64, failed_transient: i64, page_failed: i64, done: i64) -> OcrPageCounts {
        OcrPageCounts {
            pending,
            failed_transient,
            page_failed,
            done,
        }
    }
    #[test]
    fn pending_pages_keep_document_pending() {
        assert_eq!(decide_ocr_outcome(&counts(1, 0, 0, 3)), OcrDocOutcome::StillPending);
        assert_eq!(decide_ocr_outcome(&counts(2, 1, 0, 0)), OcrDocOutcome::StillPending);
    }
    #[test]
    fn exhausted_transient_blocks_ocr_done() {
        assert_eq!(
            decide_ocr_outcome(&counts(0, 2, 0, 10)),
            OcrDocOutcome::BlockedByTransient(2)
        );
    }
    #[test]
    fn page_failed_alone_still_finishes() {
        assert_eq!(decide_ocr_outcome(&counts(0, 0, 1, 4)), OcrDocOutcome::Finished);
        assert_eq!(decide_ocr_outcome(&counts(0, 0, 0, 0)), OcrDocOutcome::Finished);
    }
}
