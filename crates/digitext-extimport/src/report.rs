// Copyright (c) 2026 DigiText. All rights reserved.
//! Ghi báo cáo ra tệp: CSV (UTF-8 có BOM — mở thẳng bằng Excel, không vỡ tiếng Việt) + JSON.
use crate::check::{CheckReport, Severity};
use crate::pack::PackResult;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

fn esc(s: &str) -> String {
    // Chuỗi toàn chữ số dài (mã hồ sơ kiểu 20300000000001…) hoặc có số 0 đứng đầu: Excel mở CSV
    // sẽ đổi thành số và hiện 2.03E+13 / mất số 0. Ghi dạng công thức ="…" để Excel giữ là chữ.
    let digits = !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if digits && (s.len() >= 12 || (s.len() > 1 && s.starts_with('0'))) {
        return format!("=\"{s}\"");
    }
    if s.contains([',', '"', '\n', '\r', ';']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
fn write_csv(path: &Path, header: &[&str], rows: Vec<Vec<String>>) -> Result<(), String> {
    let mut out = String::from("\u{feff}");
    out.push_str(&header.join(","));
    out.push_str("\r\n");
    for r in rows {
        out.push_str(&r.iter().map(|c| esc(c)).collect::<Vec<_>>().join(","));
        out.push_str("\r\n");
    }
    std::fs::write(path, out).map_err(|e| format!("không ghi được {}: {e}", path.display()))
}

/// Tóm tắt dạng văn bản — in ra màn hình và ghi `tom-tat-kiem-tra.txt`.
pub fn check_summary(r: &CheckReport) -> String {
    let mut s = String::new();
    let bytes: u64 = r.dossiers.iter().map(|d| d.bytes).sum();
    s.push_str(&format!(
        "KIỂM TRA KHO NGOÀI — {}\n  Excel : {} (sheet «{}», tiêu đề ở dòng {})\n  Thư mục gốc: {}\n  Kiểm sâu nội dung PDF: {}\n\n",
        chrono::Local::now().format("%d/%m/%Y %H:%M"), r.excel, r.sheet, r.header_line, r.root,
        if r.deep { "CÓ" } else { "không (chạy lại với --deep để soi từng PDF)" }
    ));
    s.push_str(&format!(
        "  Dòng Excel (tài liệu) : {}\n  Tệp PDF trên đĩa      : {}\n  Hồ sơ (thư mục)       : {}\n  Dung lượng            : {:.1} GB\n\n",
        r.rows, r.files_on_disk, r.dossiers.len(), bytes as f64 / 1e9
    ));
    let bad = r.dossiers.len() - r.dossiers_ok();
    s.push_str(&format!(
        "  KẾT QUẢ: {} hồ sơ ĐẠT (đóng gói được) · {} hồ sơ CÓ LỖI (bị giữ lại) · {} lỗi · {} cảnh báo\n\n",
        r.dossiers_ok(), bad, r.errors(), r.warnings()
    ));
    if r.deep {
        let docs: usize = r.dossiers.iter().map(|d| d.docs).sum();
        let pdfa: u32 = r.dossiers.iter().filter_map(|d| d.pdfa).sum();
        let signed: u32 = r.dossiers.iter().filter_map(|d| d.signed).sum();
        let pages: u32 = r.dossiers.iter().filter_map(|d| d.pages).sum();
        let maps: u32 = r.dossiers.iter().filter_map(|d| d.maps).sum();
        s.push_str(&format!("  PDF/A: {pdfa}/{docs} tệp · đã ký số: {signed}/{docs} tệp · tổng {pages} trang · bản đồ/bản vẽ (ngưỡng 300 dpi): {maps} tệp\n"));
        let kinds = r.pdfa_flavours.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join(" · ");
        s.push_str(&format!("  Theo loại tệp tự khai: {kinds}\n\n"));
    }
    let mut by: BTreeMap<(Severity, &str), (u32, u32, &str)> = BTreeMap::new();
    for i in &r.issues {
        let e = by.entry((i.severity, i.code)).or_insert((0, 0, i.message.as_str()));
        e.0 += i.count;
        e.1 += 1;
    }
    s.push_str("  Theo loại vấn đề (số lần · số hồ sơ/dòng báo cáo · ví dụ) — THÔNG TIN không phải lỗi:\n");
    for ((sev, code), (n, k, ex)) in by {
        let ex: String = ex.chars().take(110).collect();
        s.push_str(&format!("    {:9} {:26} {:>6} · {:>5} · {}\n", sev.label(), code, n, k, ex));
    }
    s
}

pub fn write_check(r: &CheckReport, out_dir: &Path) -> Result<Vec<PathBuf>, String> {
    std::fs::create_dir_all(out_dir).map_err(|e| format!("không tạo được {}: {e}", out_dir.display()))?;
    let p1 = out_dir.join("kiem-tra-van-de.csv");
    write_csv(
        &p1,
        &["Mức độ", "Vấn đề", "Thư mục hồ sơ", "Mã hồ sơ", "Dòng Excel", "Tệp", "Số lần", "Nội dung", "Cách xử lý", "Mã vấn đề"],
        r.issues.iter().map(|i| {
            let (ten, cach) = crate::check::huong_dan(i.code);
            vec![
                i.severity.label().to_string(), if ten.is_empty() { i.code.to_string() } else { ten.to_string() }, i.dossier.clone(), i.hs_code.clone(),
                i.line.map(|l| l.to_string()).unwrap_or_default(), i.file.clone().unwrap_or_default(),
                i.count.to_string(), i.message.clone(), cach.to_string(), i.code.to_string(),
            ]
        }).collect(),
    )?;
    let p2 = out_dir.join("kiem-tra-ho-so.csv");
    let o = |v: Option<u32>| v.map(|x| x.to_string()).unwrap_or_default();
    write_csv(
        &p2,
        &["Kết luận", "Thư mục hồ sơ", "Mã hồ sơ", "Mã phông", "Tiêu đề", "Số tài liệu", "MB", "Lỗi", "Cảnh báo", "Tệp PDF/A", "Tệp đã ký", "Số trang thật"],
        r.dossiers.iter().map(|d| vec![
            if d.errors == 0 { "ĐẠT".into() } else { "CÓ LỖI".to_string() }, d.dir.clone(), d.hs_code.clone(),
            d.fond_code.clone(), d.title.clone(), d.docs.to_string(), format!("{:.1}", d.bytes as f64 / 1e6),
            d.errors.to_string(), d.warnings.to_string(), o(d.pdfa), o(d.signed), o(d.pages),
        ]).collect(),
    )?;
    let p3 = out_dir.join("kiem-tra.json");
    std::fs::write(&p3, serde_json::to_vec_pretty(r).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let p4 = out_dir.join("tom-tat-kiem-tra.txt");
    std::fs::write(&p4, format!("\u{feff}{}", check_summary(r))).map_err(|e| e.to_string())?;
    Ok(vec![p4, p2, p1, p3])
}

pub fn pack_summary(results: &[PackResult]) -> String {
    let n = |st: &str| results.iter().filter(|r| r.status == st).count();
    let bytes: u64 = results.iter().filter(|r| r.status == "built").filter_map(|r| r.zip_size).sum();
    let not_pdfa: u32 = results.iter().filter(|r| r.status == "built").map(|r| r.not_pdfa).sum();
    let mut s = format!(
        "ĐÓNG GÓI — {}\n  Dựng mới: {} gói ({:.1} GB) · đã có từ trước: {} · giữ lại vì lỗi bước kiểm: {} · dựng hỏng: {}\n",
        chrono::Local::now().format("%d/%m/%Y %H:%M"), n("built"), bytes as f64 / 1e9, n("skipped_existing"),
        n("skipped_errors"), n("failed")
    );
    if not_pdfa > 0 {
        s.push_str(&format!("  LƯU Ý: {not_pdfa} tệp trong các gói vừa dựng KHÔNG phải PDF/A — gói sẽ không qua được veraPDF (TT05 Điều 8.2.a).\n"));
    }
    for r in results.iter().filter(|r| r.status == "failed").take(20) {
        s.push_str(&format!("  HỎNG {} ({}): {}\n", r.hs_code, r.dir, r.error.as_deref().unwrap_or("")));
    }
    s
}

pub fn write_pack(results: &[PackResult], out_dir: &Path) -> Result<PathBuf, String> {
    let p = out_dir.join("ket-qua-dong-goi.csv");
    let label = |s: &str| match s {
        "built" => "ĐÃ ĐÓNG GÓI",
        "skipped_existing" => "ĐÃ CÓ GÓI (bỏ qua)",
        "skipped_errors" => "GIỮ LẠI — còn lỗi bước kiểm",
        _ => "DỰNG HỎNG",
    };
    write_csv(
        &p,
        &["Kết quả", "Thư mục hồ sơ", "Mã hồ sơ", "Số tài liệu", "Tệp gói", "MB", "SHA-256", "OBJID", "Tệp không PDF/A", "Tệp chưa ký", "Ghi chú chuyển mã", "Lỗi"],
        results.iter().map(|r| vec![
            label(&r.status).to_string(), r.dir.clone(), r.hs_code.clone(), r.docs.to_string(),
            r.zip_name.clone().unwrap_or_default(), r.zip_size.map(|b| format!("{:.1}", b as f64 / 1e6)).unwrap_or_default(),
            r.zip_sha256.clone().unwrap_or_default(), r.objid.clone().unwrap_or_default(), r.not_pdfa.to_string(),
            r.unsigned.to_string(), r.notes.join(" | "), r.error.clone().unwrap_or_default(),
        ]).collect(),
    )?;
    Ok(p)
}
