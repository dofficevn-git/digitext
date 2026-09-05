// Copyright (c) 2026 DigiText. All rights reserved.
use crate::infer::Tgi;
use digitext_core::VanBanField;
use digitext_db::ocr_fields::{self, Tdw};
use sqlx::PgPool;
use std::cmp::Ordering;
use std::collections::HashSet;
use uuid::Uuid;
#[derive(Debug, Clone, PartialEq)]
pub struct T76 {
    pub field: VanBanField,
    pub value: String,
    pub bbox: Option<[f32; 4]>,
    pub confidence: f32,
}
const C_4C: [&str; 21] = [
    "Giấy chứng nhận đăng ký doanh nghiệp",
    "Giấy chứng nhận đăng ký kinh doanh",
    "Giấy chứng nhận",
    "Giấy phép khai thác khoáng sản",
    "Giấy phép",
    "Thuyết minh dự án",
    "Thuyết minh",
    "Thiết kế cơ sở",
    "Bản đồ",
    "Báo cáo tài chính",
    "Báo cáo",
    "Quyết định",
    "Công văn",
    "Thông báo",
    "Tờ trình",
    "Nghị quyết",
    "Chỉ thị",
    "Kế hoạch",
    "Biên bản",
    "Hướng dẫn",
    "Thông tư",
];
struct Span<'a> {
    text: &'a str,
    lower: String,
    b: Option<[f32; 4]>,
    conf: f32,
}
impl Span<'_> {
    fn cy(&self) -> f32 {
        self.b.map(|b| (b[1] + b[3]) / 2.0).unwrap_or(f32::MAX)
    }
    fn cx(&self) -> f32 {
        self.b.map(|b| (b[0] + b[2]) / 2.0).unwrap_or(0.0)
    }
    fn xywh(&self) -> Option<[f32; 4]> {
        self.b.map(|b| [b[0], b[1], b[2] - b[0], b[3] - b[1]])
    }
}
fn f_1ft(r_35q: &str) -> bool {
    r_35q
        .split(|r_35n: char| !(r_35n.is_ascii_digit() || r_35n == '/'))
        .any(|r_35r| {
            let r_35p: Vec<&str> = r_35r.split('/').collect();
            r_35p.len() == 3
                && r_35p
                    .iter()
                    .all(|r_35o| !r_35o.is_empty() && r_35o.bytes().all(|b| b.is_ascii_digit()))
        })
}
fn f_1pp(lower: &str) -> bool {
    f_1ft(lower)
        || (lower.contains("ngày") && lower.contains("tháng") && lower.contains("năm"))
}
fn f_1fv(text: &str, lower: &str) -> bool {
    if lower.contains("mã số doanh nghiệp") {
        return true;
    }
    text.split(|c: char| c.is_whitespace() || matches!(c, '(' | ')' | ',' | ';'))
        .any(|r_35s| {
            r_35s.contains('/')
                && r_35s.chars().any(|c| c.is_ascii_digit())
                && r_35s.chars().any(|c| c.is_uppercase())
        })
}
fn f_1j7(lower: &str) -> bool {
    [
        "cộng hoà",
        "cộng hòa",
        "độc lập",
        "tự do",
        "hạnh phúc",
        "chủ nghĩa việt nam",
        "bản sao",
        "bản chính",
    ]
    .iter()
    .any(|r_35t| lower.contains(r_35t))
}
fn f_1jk(lower: &str) -> bool {
    lower.contains("ký bởi") || lower.contains("thời gian")
}
fn f_1jg(lower: &str) -> bool {
    let r_35u = lower.trim_start();
    r_35u.starts_with("căn cứ") || r_35u.starts_with("xét ") || r_35u.starts_with("theo")
}
fn f_1j6(text: &str) -> bool {
    let r_35v = text.chars().filter(|c| c.is_alphabetic()).count();
    if r_35v < 3 {
        return false;
    }
    let r_35w = text.chars().filter(|c| c.is_uppercase()).count();
    r_35w as f32 / r_35v as f32 >= 0.6
}
fn f_2ra(r_35x: &str) -> usize {
    r_35x.split_whitespace().count()
}
fn f_1rm(lower: &str) -> Option<&'static str> {
    let r_35y = lower.trim_start();
    C_4C.into_iter()
        .find(|t| r_35y.starts_with(&t.to_lowercase()))
}
fn f_22t<P, S>(r_363: &[Span], r_364: &HashSet<usize>, r_360: P, r_362: S) -> Option<usize>
where
    P: Fn(&Span) -> bool,
    S: Fn(&Span) -> f32,
{
    r_363
        .iter()
        .enumerate()
        .filter(|(i, r_361)| !r_364.contains(i) && r_360(r_361))
        .max_by(|r_35z, b| {
            r_362(r_35z.1)
                .partial_cmp(&r_362(b.1))
                .unwrap_or(Ordering::Equal)
        })
        .map(|(i, _)| i)
}
pub fn extract_fields(page: &Tgi) -> Vec<T76> {
    let mut r_36h: Vec<Span> = Vec::new();
    for i in 0..page.texts.len() {
        let text = page.texts[i].trim();
        if text.is_empty() {
            continue;
        }
        r_36h.push(Span {
            text,
            lower: text.to_lowercase(),
            b: page.boxes.get(i).copied(),
            conf: page.confidences.get(i).copied().unwrap_or(0.0),
        });
    }
    let r_36f = r_36h
        .iter()
        .filter_map(|s| s.b)
        .map(|b| b[3])
        .fold(1.0_f32, f32::max);
    let r_36g = r_36h
        .iter()
        .filter_map(|s| s.b)
        .map(|b| b[2])
        .fold(1.0_f32, f32::max);
    let r_36c = r_36f * 0.32;
    let mut picked: Vec<(VanBanField, String, Option<[f32; 4]>, f32)> = Vec::new();
    let mut used: HashSet<usize> = HashSet::new();
    let r_367 = |i: usize,
                 field: VanBanField,
                 value: String,
                 used: &mut HashSet<usize>,
                 picked: &mut Vec<(VanBanField, String, Option<[f32; 4]>, f32)>| {
        used.insert(i);
        picked.push((field, value, r_36h[i].xywh(), r_36h[i].conf));
    };
    if let Some(i) = f_22t(
        &r_36h,
        &used,
        |s| f_1rm(&s.lower).is_some(),
        |s| -(f_2ra(s.text) as f32),
    ) {
        let v = f_1rm(&r_36h[i].lower).unwrap().to_string();
        r_367(i, VanBanField::LoaiVanBan, v, &mut used, &mut picked);
    }
    if let Some(i) = f_22t(
        &r_36h,
        &used,
        |s| s.cy() < r_36c && !f_1jk(&s.lower) && f_1fv(s.text, &s.lower),
        |s| -s.cy(),
    ) {
        let v = r_36h[i].text.to_string();
        r_367(i, VanBanField::SoKyHieu, v, &mut used, &mut picked);
    }
    if let Some(i) = f_22t(
        &r_36h,
        &used,
        |s| {
            s.cy() < r_36c
                && f_1pp(&s.lower)
                && !f_1jk(&s.lower)
                && !f_1j7(&s.lower)
                && !f_1jg(&s.lower)
        },
        |s| s.cx() / r_36g - s.cy() / r_36f,
    ) {
        let v = r_36h[i].text.to_string();
        r_367(i, VanBanField::NgayThang, v, &mut used, &mut picked);
    }
    if let Some(i) = f_22t(
        &r_36h,
        &used,
        |s| s.cy() < r_36c && f_1j6(s.text) && !f_1j7(&s.lower) && !f_1jk(&s.lower),
        |s| (0.5 - s.cx() / r_36g) - s.cy() / r_36f,
    ) {
        let v = r_36h[i].text.to_string();
        r_367(i, VanBanField::CoQuanBanHanh, v, &mut used, &mut picked);
    }
    let r_36j = r_36f * 0.42;
    let r_36b = |s: &Span| {
        let r_36a = s
            .lower
            .trim_start_matches(|r_365: char| !r_365.is_alphanumeric());
        r_36a.starts_with("về việc") || r_36a.starts_with("v/v")
    };
    let r_366 = |s: &Span| {
        let r_369 = s.lower.split_whitespace().next().unwrap_or("");
        s.cy() < r_36j
            && s.text.chars().count() >= 12
            && r_369.chars().count() <= 6
            && r_369.ends_with(':')
    };
    let r_36i = f_22t(&r_36h, &used, r_36b, |s| -s.cy())
        .or_else(|| f_22t(&r_36h, &used, r_366, |s| -s.cy()))
        .or_else(|| f_22t(&r_36h, &used, |_| true, |s| s.text.chars().count() as f32));
    if let Some(i) = r_36i {
        let v = r_36h[i].text.to_string();
        r_367(i, VanBanField::TrichYeu, v, &mut used, &mut picked);
    }
    let mut r_36e: Vec<T76> = picked
        .into_iter()
        .map(|(field, value, bbox, confidence)| T76 {
            field,
            value,
            bbox,
            confidence,
        })
        .collect();
    r_36e.sort_by_key(|r_368| {
        VanBanField::ALL
            .iter()
            .position(|r_36d| *r_36d == r_368.field)
            .unwrap_or(usize::MAX)
    });
    r_36e
}
pub async fn f_2ho(
    r_36q: &PgPool,
    r_36k: Uuid,
    r_36p: i32,
    r_36l: Uuid,
    r_36r: &str,
    r_36n: &[T76],
) -> sqlx::Result<u64> {
    let mut r_36o = 0u64;
    for r_36m in r_36n {
        let bbox = r_36m
            .bbox
            .map(|b| serde_json::json!([b[0], b[1], b[2], b[3]]));
        ocr_fields::upsert(
            r_36q,
            Tdw {
                document_id: r_36k,
                page_no: r_36p,
                engine_version_id: r_36l,
                text_source: r_36r,
                field_key: r_36m.field.as_str(),
                value: &r_36m.value,
                bbox,
                confidence: Some(r_36m.confidence),
                verified: false,
            },
        )
        .await?;
        r_36o += 1;
    }
    Ok(r_36o)
}
