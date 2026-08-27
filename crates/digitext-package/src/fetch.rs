// Copyright (c) 2026 DigiText. All rights reserved.
use crate::model::T7z;
pub fn f_1c1(r_3yi: &str, r_3yj: &T7z, r_3yh: &str) -> String {
    format!(
        "urn_{}_{}_{} {} {} {}",
        r_3yi, r_3yj.source_package_uuid, r_3yj.source_path, r_3yj.size_bytes, r_3yh, r_3yj.doc_id
    )
}
pub fn f_1j0(r_3yk: &str) -> bool {
    r_3yk.to_ascii_lowercase().ends_with(".fetch.txt")
}
pub fn f_1sm(r_3ym: &str) -> &str {
    let r_3yl = r_3ym.strip_suffix(".fetch.txt").unwrap_or(r_3ym);
    match r_3yl.rfind('.') {
        Some(r_3yn) => &r_3yl[..r_3yn],
        None => r_3yl,
    }
}
