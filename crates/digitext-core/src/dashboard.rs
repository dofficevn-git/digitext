// Copyright (c) 2026 DigiText. All rights reserved.
//! Toán cho dashboard 3 góc nhìn (22/10/2026): sức khoẻ dự án (RAG), dự kiến ngày xong, trung bình nhóm ẩn danh,
//! mức đạt định mức. Thuần, không sqlx — máy chủ là nguồn sự thật, desktop không tự tính lại.
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// Múi giờ Việt Nam (+07:00) — mọi mốc "ngày" của dashboard tính theo đây.
pub const VN_OFFSET_SECS: i32 = 7 * 3600;

/// Các chỉ số được phép đặt định mức/người/ngày (khoá của `project_kpi_settings.quotas`), trùng tên cột
/// `dashboard_daily`.
pub const QUOTA_METRICS: [&str; 11] = [
    "pages_scanned",
    "docs_uploaded",
    "fields_edited",
    "fields_verified",
    "work_items_done",
    "work_items_checked_ok",
    "docs_reviewed_approved",
    "docs_signed",
    "cl_ho_so_new",
    "cl_van_ban_new",
    "cl_fields_entered",
];

pub fn is_quota_metric(k: &str) -> bool {
    QUOTA_METRICS.contains(&k)
}

fn vn() -> FixedOffset {
    FixedOffset::east_opt(VN_OFFSET_SECS).expect("+07:00 hợp lệ")
}

/// Hôm nay theo giờ Việt Nam.
pub fn vn_today() -> NaiveDate {
    vn_day_of(Utc::now())
}

/// Ngày Việt Nam của một mốc UTC (18:30Z = 01:30 hôm sau giờ VN).
pub fn vn_day_of(t: DateTime<Utc>) -> NaiveDate {
    t.with_timezone(&vn()).date_naive()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rag {
    Green,
    Amber,
    Red,
    Grey,
}

/// Bốn bước tính tiến độ (giữ quy ước 4 vòng tròn của báo cáo Tiến độ dự án).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Step {
    Scan,
    Pdfa,
    Sign,
    Review,
}

/// Tốc độ (đơn vị / ngày) của 4 bước.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rates {
    pub pages_scanned: f64,
    pub docs_pdfa: f64,
    pub docs_signed: f64,
    pub docs_approved: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RagThresholds {
    /// lệch = % hoàn thành − % thời gian đã qua; ≤ ngưỡng này → vàng.
    pub amber_lech: f64,
    /// ≤ ngưỡng này → đỏ.
    pub red_lech: f64,
}
impl Default for RagThresholds {
    fn default() -> Self {
        RagThresholds { amber_lech: -5.0, red_lech: -15.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HealthInput {
    pub today: NaiveDate,
    pub signed_on: Option<NaiveDate>,
    pub end_on: Option<NaiveDate>,
    pub deadline_on: Option<NaiveDate>,
    pub contract_pages: Option<i64>,
    pub contract_files: Option<i64>,
    pub pages: i64,
    pub files: i64,
    pub pdfa_files: i64,
    pub signed_files: i64,
    pub approved_files: i64,
    /// Trung bình 7 ngày VN gần nhất (từ dashboard_daily).
    pub rate7d: Rates,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Health {
    pub progress_pct: Option<f64>,
    pub time_pct: Option<f64>,
    pub lech: Option<f64>,
    pub days_remaining: Option<i64>,
    pub run_rate7d: Rates,
    pub required_rate: Rates,
    pub projected_finish_on: Option<NaiveDate>,
    pub bottleneck_step: Option<Step>,
    pub rag: Rag,
}

fn pct(done: i64, total: i64) -> Option<f64> {
    (total > 0).then(|| ((done as f64) / (total as f64) * 100.0).min(100.0))
}

fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

/// Sức khoẻ dự án. `progress_pct` = trung bình 4 bước (quét / PDF/A / ký số / duyệt); `due` = hạn chót, không có thì
/// ngày kết thúc hợp đồng; dự kiến xong = ngày muộn nhất trong 4 bước theo tốc độ 7 ngày; RAG theo ngưỡng lệch và hạn.
pub fn health(i: &HealthInput, t: &RagThresholds) -> Health {
    // Mục tiêu số tệp: hợp đồng ghi, không thì suy từ số trang hợp đồng ÷ số trang trung bình mỗi tệp.
    let avg_pages = if i.files > 0 { (i.pages as f64 / i.files as f64).max(1.0) } else { 1.0 };
    let target_files = match (i.contract_files, i.contract_pages) {
        (Some(f), _) if f > 0 => Some(f),
        (_, Some(p)) if p > 0 => Some(((p as f64) / avg_pages).round().max(1.0) as i64),
        _ => None,
    };
    let steps = [
        (Step::Scan, i.pages, i.contract_pages.filter(|p| *p > 0), i.rate7d.pages_scanned),
        (Step::Pdfa, i.pdfa_files, target_files, i.rate7d.docs_pdfa),
        (Step::Sign, i.signed_files, target_files, i.rate7d.docs_signed),
        (Step::Review, i.approved_files, target_files, i.rate7d.docs_approved),
    ];
    let pcts: Vec<f64> = steps.iter().filter_map(|(_, d, t, _)| t.and_then(|t| pct(*d, t))).collect();
    let progress_pct = (!pcts.is_empty()).then(|| round1(pcts.iter().sum::<f64>() / pcts.len() as f64));

    let due = i.deadline_on.or(i.end_on);
    let time_pct = match (i.signed_on, due) {
        (Some(a), Some(b)) if b > a => {
            let total = (b - a).num_days() as f64;
            let gone = (i.today - a).num_days() as f64;
            Some(round1((gone / total * 100.0).clamp(0.0, 100.0)))
        }
        _ => None,
    };
    let lech = match (progress_pct, time_pct) {
        (Some(p), Some(t)) => Some(round1(p - t)),
        _ => None,
    };
    let days_remaining = due.map(|d| (d - i.today).num_days());

    let mut required = Rates::default();
    let mut projected: Option<NaiveDate> = None;
    let mut bottleneck: Option<Step> = None;
    for (step, done, target, rate) in steps {
        let Some(target) = target else { continue };
        let remaining = (target - done).max(0) as f64;
        let req = remaining / (days_remaining.unwrap_or(1).max(1) as f64);
        let eta = if remaining <= 0.0 {
            Some(i.today)
        } else if rate > 0.0 {
            Some(i.today + Duration::days((remaining / rate).ceil() as i64))
        } else {
            None
        };
        match step {
            Step::Scan => required.pages_scanned = round1(req),
            Step::Pdfa => required.docs_pdfa = round1(req),
            Step::Sign => required.docs_signed = round1(req),
            Step::Review => required.docs_approved = round1(req),
        }
        // Bước nào không dự kiến được (tốc độ 0 mà còn việc) thì toàn dự án không dự kiến được — đó chính là nút nghẽn.
        match (eta, projected, bottleneck) {
            (None, _, _) => {
                projected = None;
                bottleneck = Some(step);
                break;
            }
            (Some(e), None, None) => {
                projected = Some(e);
                bottleneck = Some(step);
            }
            (Some(e), Some(p), _) if e > p => {
                projected = Some(e);
                bottleneck = Some(step);
            }
            _ => {}
        }
    }
    // Không còn việc ở bước nào → không có nút nghẽn.
    if projected == Some(i.today) {
        bottleneck = None;
    }

    let rag = match (due, lech) {
        (None, _) | (_, None) => Rag::Grey,
        (Some(d), Some(l)) => {
            let over = days_remaining.is_some_and(|r| r < 0) && progress_pct.unwrap_or(0.0) < 100.0;
            if l <= t.red_lech || over {
                Rag::Red
            } else if l <= t.amber_lech || projected.is_some_and(|p| p > d) {
                Rag::Amber
            } else {
                Rag::Green
            }
        }
    };
    Health {
        progress_pct,
        time_pct,
        lech,
        days_remaining,
        run_rate7d: i.rate7d,
        required_rate: required,
        projected_finish_on: projected,
        bottleneck_step: bottleneck,
        rag,
    }
}

/// Trung bình nhóm ẩn danh: chỉ trả khi có ít nhất `k` người (k-anonymity), nếu không là None.
pub fn group_average(values: &[f64], k: usize) -> Option<f64> {
    (values.len() >= k.max(1)).then(|| values.iter().sum::<f64>() / values.len() as f64)
}

/// Mức đạt định mức (%): sản lượng trung bình mỗi ngày có làm ÷ định mức/ngày. None khi chưa có định mức hoặc chưa làm ngày nào.
pub fn attainment_pct(done: f64, active_days: i64, quota_per_day: Option<f64>) -> Option<f64> {
    let q = quota_per_day.filter(|q| *q > 0.0)?;
    (active_days > 0).then(|| round1(done / active_days as f64 / q * 100.0))
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KpiThresholds {
    pub excellent_pct: f64,
    pub pass_pct: f64,
    pub excellent_err_pct: f64,
    pub pass_err_pct: f64,
}
impl Default for KpiThresholds {
    fn default() -> Self {
        KpiThresholds { excellent_pct: 100.0, pass_pct: 80.0, excellent_err_pct: 1.0, pass_err_pct: 3.0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rating {
    XuatSac,
    Dat,
    CanhBao,
    ChuaGiao,
}

/// Xếp loại KPI — cùng luật với `kpiOf` của desktop (StaffReportView), ngưỡng lấy từ cài đặt dự án.
pub fn rating(attainment_pct: Option<f64>, err_pct: f64, t: &KpiThresholds) -> Rating {
    match attainment_pct {
        None => {
            if err_pct > t.pass_err_pct {
                Rating::CanhBao
            } else {
                Rating::ChuaGiao
            }
        }
        Some(a) if a >= t.excellent_pct && err_pct <= t.excellent_err_pct => Rating::XuatSac,
        Some(a) if a >= t.pass_pct && err_pct <= t.pass_err_pct => Rating::Dat,
        Some(_) => Rating::CanhBao,
    }
}

/// Số ngày của một khoảng [from, to] (cả hai đầu) — tiện cho "N ngày gần nhất".
pub fn days_back(today: NaiveDate, days: i64) -> NaiveDate {
    today - Duration::days(days.max(1) - 1)
}

/// Thứ trong tuần theo ISO (1 = thứ Hai) — dùng cho nhãn trục ngày.
pub fn weekday_iso(d: NaiveDate) -> u32 {
    d.weekday().number_from_monday()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }
    fn base() -> HealthInput {
        HealthInput {
            today: d("2026-07-02"),
            signed_on: Some(d("2026-01-01")),
            end_on: Some(d("2026-12-31")),
            deadline_on: None,
            contract_pages: Some(100_000),
            contract_files: Some(10_000),
            pages: 40_000,
            files: 4_000,
            pdfa_files: 4_000,
            signed_files: 4_000,
            approved_files: 4_000,
            rate7d: Rates { pages_scanned: 500.0, docs_pdfa: 50.0, docs_signed: 50.0, docs_approved: 50.0 },
        }
    }
    #[test]
    fn time_pct_lech_rag() {
        let h = health(&base(), &RagThresholds::default());
        assert_eq!(h.progress_pct, Some(40.0));
        assert_eq!(h.time_pct, Some(50.0)); // 182/364 ngày
        assert_eq!(h.lech, Some(-10.0));
        assert_eq!(h.rag, Rag::Amber);
        let mut i = base();
        i.pages = 30_000;
        i.pdfa_files = 3_000;
        i.signed_files = 3_000;
        i.approved_files = 3_000;
        assert_eq!(health(&i, &RagThresholds::default()).rag, Rag::Red);
        let mut i = base();
        i.signed_on = None;
        assert_eq!(health(&i, &RagThresholds::default()).rag, Rag::Grey);
        let mut i = base();
        i.today = d("2027-01-05");
        assert_eq!(health(&i, &RagThresholds::default()).rag, Rag::Red, "quá hạn mà chưa xong → đỏ");
    }
    #[test]
    fn projection_picks_slowest_step() {
        let mut i = base();
        // Quét còn 60.000 trang / 500 = 120 ngày; ký còn 6.000 / 50 = 120 ngày; duyệt chậm hơn: 20/ngày → 300 ngày.
        i.rate7d.docs_approved = 20.0;
        let h = health(&i, &RagThresholds::default());
        assert_eq!(h.bottleneck_step, Some(Step::Review));
        assert_eq!(h.projected_finish_on, Some(d("2026-07-02") + Duration::days(300)));
        assert_eq!(h.rag, Rag::Amber, "dự kiến xong sau hạn → ít nhất vàng");
        i.rate7d.docs_signed = 0.0;
        let h = health(&i, &RagThresholds::default());
        assert_eq!(h.projected_finish_on, None, "tốc độ 0 mà còn việc → không dự kiến được");
        assert_eq!(h.bottleneck_step, Some(Step::Sign));
        let mut done = base();
        done.pages = 100_000;
        done.pdfa_files = 10_000;
        done.signed_files = 10_000;
        done.approved_files = 10_000;
        let h = health(&done, &RagThresholds::default());
        assert_eq!(h.progress_pct, Some(100.0));
        assert_eq!(h.projected_finish_on, Some(done.today));
        assert_eq!(h.bottleneck_step, None);
        assert_eq!(h.rag, Rag::Green);
    }
    #[test]
    fn group_average_hidden_below_k() {
        assert_eq!(group_average(&[10.0, 20.0], 3), None);
        assert_eq!(group_average(&[10.0, 20.0, 30.0], 3), Some(20.0));
        assert_eq!(attainment_pct(600.0, 2, Some(300.0)), Some(100.0));
        assert_eq!(attainment_pct(600.0, 0, Some(300.0)), None);
        assert_eq!(attainment_pct(600.0, 2, None), None);
        let t = KpiThresholds::default();
        assert_eq!(rating(Some(100.0), 0.5, &t), Rating::XuatSac);
        assert_eq!(rating(Some(85.0), 2.0, &t), Rating::Dat);
        assert_eq!(rating(Some(85.0), 5.0, &t), Rating::CanhBao);
        assert_eq!(rating(None, 0.0, &t), Rating::ChuaGiao);
        assert_eq!(rating(None, 9.0, &t), Rating::CanhBao);
    }
    #[test]
    fn vn_day_of_crosses_midnight() {
        let t = DateTime::parse_from_rfc3339("2026-10-22T18:30:00Z").unwrap().with_timezone(&Utc);
        assert_eq!(vn_day_of(t), d("2026-10-23"));
        let t = DateTime::parse_from_rfc3339("2026-10-22T16:59:59Z").unwrap().with_timezone(&Utc);
        assert_eq!(vn_day_of(t), d("2026-10-22"));
        assert_eq!(days_back(d("2026-10-22"), 14), d("2026-10-09"));
        assert!(is_quota_metric("pages_scanned") && !is_quota_metric("computed_at"));
    }
}
