// Copyright (c) 2026 DigiText. All rights reserved.
use chrono::{Datelike, Duration, NaiveDate, Weekday};
use std::collections::BTreeSet;
use std::ops::RangeInclusive;
pub fn f_1d5(r_2fc: i32) -> Vec<NaiveDate> {
    [(1, 1), (4, 30), (5, 1), (9, 2)]
        .into_iter()
        .filter_map(|(m, d)| NaiveDate::from_ymd_opt(r_2fc, m, d))
        .collect()
}
pub fn f_1z4(r_2fe: &str) -> BTreeSet<NaiveDate> {
    r_2fe
        .split(',')
        .filter_map(|r_2fd| NaiveDate::parse_from_str(r_2fd.trim(), "%Y-%m-%d").ok())
        .collect()
}
pub fn f_1hg(r_2ff: &str, r_2fi: RangeInclusive<i32>) -> BTreeSet<NaiveDate> {
    let mut r_2fg = f_1z4(r_2ff);
    for r_2fh in r_2fi {
        r_2fg.extend(f_1d5(r_2fh));
    }
    r_2fg
}
pub fn f_2ri(r_2fk: NaiveDate, r_2fm: NaiveDate, r_2fl: &BTreeSet<NaiveDate>) -> i64 {
    let mut r_2fj = 0i64;
    let mut d = r_2fk;
    while d < r_2fm {
        d += Duration::days(1);
        let r_2fn = matches!(d.weekday(), Weekday::Sat | Weekday::Sun);
        if !r_2fn && !r_2fl.contains(&d) {
            r_2fj += 1;
        }
    }
    r_2fj
}
