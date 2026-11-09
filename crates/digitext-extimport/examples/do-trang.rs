// Copyright (c) 2026 DigiText. All rights reserved.
//! Chạy thử phép đo trang trắng / trang lệch trên PDF thật, để chỉnh ngưỡng trong `trang.rs`.
//!
//!     cargo run -p digitext-extimport --example do-trang -- <tệp hoặc thư mục>…
//!
//! In mỗi tệp một dòng: số trang đo được, trang trắng, trang lệch kèm góc, trang quét ngang.

use std::time::Instant;

fn main() {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    // --xuat <thư mục>: ghi ảnh JPEG của mọi trang bị gắn cờ để xem tận mắt khi chỉnh ngưỡng.
    let mut xuat: Option<String> = None;
    if let Some(i) = args.iter().position(|a| a == "--xuat") {
        if i + 1 < args.len() {
            xuat = Some(args.remove(i + 1));
        }
        args.remove(i);
    }
    if args.is_empty() {
        eprintln!("Dùng: cargo run -p digitext-extimport --example do-trang -- <tệp|thư mục>…");
        std::process::exit(2);
    }
    // --csv <tệp>: chạy đúng đường của app (pdfa::run_opts + write_csv) rồi ghi báo cáo ra đó.
    let mut csv: Option<String> = None;
    if let Some(i) = args.iter().position(|a| a == "--csv") {
        if i + 1 < args.len() {
            csv = Some(args.remove(i + 1));
        }
        args.remove(i);
    }
    // --muc: in lượng mực từng trang, xếp từ ít tới nhiều — để chỉnh ngưỡng trang trắng.
    let in_muc = args.iter().any(|a| a == "--muc");
    args.retain(|a| a != "--muc");
    let mut moi_trang: Vec<(f32, String, u32)> = Vec::new();
    let files = digitext_extimport::pdfa::collect(&args, 5000);
    if files.is_empty() {
        eprintln!("Không thấy PDF nào.");
        std::process::exit(1);
    }
    println!("{} tệp\n", files.len());
    if let Some(ra) = &csv {
        let reports = digitext_extimport::pdfa::run_opts(&files, false, true, &|d, t| {
            if d == t || d % 20 == 0 {
                eprint!("\r  {d}/{t} bước");
            }
        });
        eprintln!();
        match digitext_extimport::pdfa::write_csv(&reports, std::path::Path::new(ra)) {
            Ok(p) => println!("đã ghi báo cáo: {p}"),
            Err(e) => eprintln!("ghi CSV lỗi: {e}"),
        }
        return;
    }
    let bat_dau = Instant::now();
    let (mut tong_trang, mut tong_trang_trang, mut tong_lech, mut tong_ngang) = (0u32, 0usize, 0usize, 0usize);
    for f in &files {
        let Ok(bytes) = std::fs::read(f) else { continue };
        let facts = digitext_extimport::pdfa::inspect(f);
        if facts.pages == 0 {
            println!("{:<50} (0 trang)", ten(f));
            continue;
        }
        let t0 = Instant::now();
        let Some(kq) = digitext_extimport::trang::do_pdf(&bytes, facts.pages) else {
            println!("{:<50} không mở được", ten(f));
            continue;
        };
        tong_trang += kq.da_do;
        tong_trang_trang += kq.trang_trang.len();
        tong_lech += kq.trang_lech.len();
        tong_ngang += kq.trang_ngang.len();
        if in_muc {
            if let Ok(parsed) = digitext_pdf::ParsedPdf::parse(&bytes) {
                for t in 1..=facts.pages {
                    if let Ok((jpeg, _)) = digitext_pdf::page_jpeg(&parsed, t as i32) {
                        if let Some((muc, _, _)) = digitext_extimport::trang::do_mot_trang(&jpeg) {
                            moi_trang.push((muc, ten(f), t));
                        }
                    }
                }
            }
        }
        if let Some(dir) = &xuat {
            let _ = std::fs::create_dir_all(dir);
            let parsed = digitext_pdf::ParsedPdf::parse(&bytes).ok();
            let mut can: Vec<(u32, &str)> = Vec::new();
            can.extend(kq.trang_trang.iter().map(|t| (t.trang, "trang")));
            can.extend(kq.trang_lech.iter().map(|t| (t.trang, "lech")));
            can.extend(kq.trang_ngang.iter().map(|t| (*t, "ngang")));
            for (trang, loai) in can {
                if let Some(p) = &parsed {
                    if let Ok((jpeg, _)) = digitext_pdf::page_jpeg(p, trang as i32) {
                        let ten_ra = format!("{}/{loai}_{}_t{trang}.jpg", dir, ten(f).trim_end_matches(".pdf"));
                        let _ = std::fs::write(&ten_ra, &jpeg);
                    }
                }
            }
        }
        let mo_ta = kq.mo_ta();
        println!(
            "{:<50} {:>4} trang ({} bỏ qua)  {:>6.1}s  lệch lớn nhất {:>5.1}°  {}",
            ten(f),
            kq.da_do,
            kq.khong_do,
            t0.elapsed().as_secs_f32(),
            kq.goc_lon_nhat,
            if mo_ta.is_empty() { "sạch".to_string() } else { mo_ta.join(" | ") }
        );
    }
    if in_muc {
        moi_trang.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
        println!("\n--- 25 trang IT muc nhat (nguong trang trang hien tai {:.4}) ---", digitext_extimport::trang::MUC_TRANG_TRANG);
        for (muc, ten, t) in moi_trang.iter().take(25) {
            println!("  {:>8.5}  {ten} trang {t}", muc);
        }
    }
    println!(
        "\nTổng: {tong_trang} trang đo được, {tong_trang_trang} trang trắng, {tong_lech} trang lệch, {tong_ngang} trang ngang — {:.1}s",
        bat_dau.elapsed().as_secs_f32()
    );
}

fn ten(p: &std::path::Path) -> String {
    p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
}
