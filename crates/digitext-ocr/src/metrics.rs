// Copyright (c) 2026 DigiText. All rights reserved.
use opentelemetry::metrics::{Counter, Gauge, Histogram, Meter};
use opentelemetry::KeyValue;
use std::sync::atomic::{AtomicI64, Ordering};
pub mod names {
    pub const C_FN: &str = "ocr_pages_processed_total";
    pub const C_FP: &str = "ocr_page_latency_seconds";
    pub const C_9J: &str = "ocr_infer_batch_size";
    pub const C_FO: &str = "ocr_page_confidence";
    pub const C_F4: &str = "ocr_oom_downshift_total";
    pub const C_77: &str = "ocr_fallback_vietocr_total";
    pub const C_HZ: &str = "ocr_queue_depth";
    pub const ENGINE_HEALTHY: &str = "ocr_engine_healthy";
    pub const ENGINE_UNAVAILABLE: &str = "ocr_engine_unavailable_total";
    pub const C_AI: &str = "llm_edits_applied_total";
    pub const C_AJ: &str = "llm_edits_rejected_total";
    pub const C_AC: &str = "status";
}
pub struct CounterMetric {
    inner: Counter<u64>,
}
impl CounterMetric {
    fn new(r_3ay: &Meter, name: &'static str, r_3ax: &'static str) -> Self {
        CounterMetric {
            inner: r_3ay.u64_counter(name).with_description(r_3ax).build(),
        }
    }
    pub fn inc(&self) {
        self.inner.add(1, &[]);
    }
    pub fn add(&self, r_3az: u64) {
        self.inner.add(r_3az, &[]);
    }
}
pub struct LabeledCounter {
    inner: Counter<u64>,
    label: &'static str,
}
impl LabeledCounter {
    fn new(r_3b1: &Meter, name: &'static str, r_3b0: &'static str, label: &'static str) -> Self {
        LabeledCounter {
            inner: r_3b1.u64_counter(name).with_description(r_3b0).build(),
            label,
        }
    }
    pub fn inc(&self, r_3b2: &str) {
        self.add(r_3b2, 1);
    }
    pub fn add(&self, r_3b4: &str, r_3b3: u64) {
        self.inner
            .add(r_3b3, &[KeyValue::new(self.label, r_3b4.to_owned())]);
    }
}
pub struct GaugeMetric {
    inner: Gauge<i64>,
    val: AtomicI64,
}
impl GaugeMetric {
    fn new(r_3b6: &Meter, name: &'static str, r_3b5: &'static str) -> Self {
        GaugeMetric {
            inner: r_3b6.i64_gauge(name).with_description(r_3b5).build(),
            val: AtomicI64::new(0),
        }
    }
    pub fn set(&self, r_3b7: i64) {
        self.val.store(r_3b7, Ordering::Relaxed);
        self.inner.record(r_3b7, &[]);
    }
    pub fn inc(&self) {
        let r_3b8 = self.val.fetch_add(1, Ordering::Relaxed) + 1;
        self.inner.record(r_3b8, &[]);
    }
    pub fn dec(&self) {
        let r_3b9 = self.val.fetch_sub(1, Ordering::Relaxed) - 1;
        self.inner.record(r_3b9, &[]);
    }
    pub fn get(&self) -> i64 {
        self.val.load(Ordering::Relaxed)
    }
}
pub struct HistMetric {
    inner: Histogram<f64>,
}
impl HistMetric {
    fn new(
        r_3bd: &Meter,
        name: &'static str,
        r_3bc: &'static str,
        r_3be: &'static str,
        r_3bb: Vec<f64>,
    ) -> Self {
        let mut r_3ba = r_3bd
            .f64_histogram(name)
            .with_description(r_3bc)
            .with_boundaries(r_3bb);
        if !r_3be.is_empty() {
            r_3ba = r_3ba.with_unit(r_3be);
        }
        HistMetric {
            inner: r_3ba.build(),
        }
    }
    fn latency_seconds(r_3bg: &Meter, name: &'static str, r_3bf: &'static str) -> Self {
        Self::new(
            r_3bg,
            name,
            r_3bf,
            "s",
            vec![
                0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
        )
    }
    fn batch_size(r_3bi: &Meter, name: &'static str, r_3bh: &'static str) -> Self {
        Self::new(
            r_3bi,
            name,
            r_3bh,
            "",
            vec![1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0],
        )
    }
    fn confidence(r_3bk: &Meter, name: &'static str, r_3bj: &'static str) -> Self {
        Self::new(
            r_3bk,
            name,
            r_3bj,
            "",
            vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
        )
    }
    pub fn observe(&self, r_3bl: f64) {
        self.inner.record(r_3bl, &[]);
    }
}
pub struct Metrics {
    pub pages_processed: LabeledCounter,
    pub page_latency_seconds: HistMetric,
    pub infer_batch_size: HistMetric,
    pub page_confidence: HistMetric,
    pub oom_downshift_total: CounterMetric,
    pub fallback_vietocr_total: CounterMetric,
    pub queue_depth: GaugeMetric,
    pub engine_healthy: GaugeMetric,
    pub engine_unavailable_total: CounterMetric,
    pub llm_edits_applied_total: CounterMetric,
    pub llm_edits_rejected_total: CounterMetric,
}
impl Metrics {
    pub fn new() -> Self {
        Self::from_meter(&opentelemetry::global::meter("digitext-ocr"))
    }
    pub fn from_meter(r_3bm: &Meter) -> Self {
        let m = Self::build(r_3bm);
        m.engine_healthy.set(1);
        m
    }
    fn build(r_3bm: &Meter) -> Self {
        Metrics {
            pages_processed: LabeledCounter::new(
                r_3bm,
                names::C_FN,
                "Số trang OCR đã xử lý theo trạng thái kết thúc.",
                names::C_AC,
            ),
            page_latency_seconds: HistMetric::latency_seconds(
                r_3bm,
                names::C_FP,
                "Độ trễ xử lý một trang (giây).",
            ),
            infer_batch_size: HistMetric::batch_size(
                r_3bm,
                names::C_9J,
                "Kích thước batch inference thực tế.",
            ),
            page_confidence: HistMetric::confidence(
                r_3bm,
                names::C_FO,
                "Điểm confidence recognition mỗi trang.",
            ),
            oom_downshift_total: CounterMetric::new(
                r_3bm,
                names::C_F4,
                "Số lần hạ batch/tile do GPU OOM.",
            ),
            fallback_vietocr_total: CounterMetric::new(
                r_3bm,
                names::C_77,
                "Số lần gọi fallback VietOCR.",
            ),
            queue_depth: GaugeMetric::new(
                r_3bm,
                names::C_HZ,
                "Độ sâu hàng đợi job cục bộ.",
            ),
            engine_healthy: GaugeMetric::new(
                r_3bm,
                names::ENGINE_HEALTHY,
                "1 khi dịch vụ nhận dạng trả lời, 0 khi worker đang chờ nó trở lại.",
            ),
            engine_unavailable_total: CounterMetric::new(
                r_3bm,
                names::ENGINE_UNAVAILABLE,
                "Số lần worker phát hiện dịch vụ nhận dạng không phản hồi.",
            ),
            llm_edits_applied_total: CounterMetric::new(
                r_3bm,
                names::C_AI,
                "Số sửa LLM corrector được áp dụng.",
            ),
            llm_edits_rejected_total: CounterMetric::new(
                r_3bm,
                names::C_AJ,
                "Số sửa LLM corrector bị gate từ chối.",
            ),
        }
    }
}
impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}
