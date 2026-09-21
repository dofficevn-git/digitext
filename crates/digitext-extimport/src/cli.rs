// Copyright (c) 2026 DigiText. All rights reserved.
//! `digitext ext-import check|pack` — nhập kho tài liệu từ bên ngoài rồi đóng gói TT05.
use crate::{check, excel, pack, report};
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;

fn usage(prog: &str) -> String {
    format!(
"Cách dùng:
  {prog} check --root <thư mục kho> --excel <metadata.xlsx> [--deep] [--report <thư mục>]
  {prog} pack  --root <thư mục kho> --excel <metadata.xlsx> --out <thư mục gói> [tuỳ chọn]

Đưa kho tài liệu ĐÃ QUÉT TỪ BÊN NGOÀI (thư mục Phông/Hộp/Hồ sơ + một tệp Excel metadata,
mỗi dòng một tài liệu) vào đóng gói TT 05/2025. Không cần máy chủ DigiText.

check   Đối chiếu thư mục với Excel, KHÔNG ghi gì vào kho. Báo cáo ra --report
        (mặc định <thư mục Excel>\\_digitext_kiem_tra): tom-tat-kiem-tra.txt,
        kiem-tra-ho-so.csv, kiem-tra-van-de.csv, kiem-tra.json.
        --deep   mở từng PDF: số trang so với Excel + điều kiện Điều 8.2.a TT05 (PDF/A hai lớp,
                 ≥ 200 dpi, màu ≥ 24 bit, có chữ ký số) — không đạt là LỖI.
                 (đọc toàn bộ dữ liệu — kho vài chục GB mất vài chục phút).
pack    Chạy lại bước kiểm, và CHỈ KHI KHÔNG CÒN LỖI NÀO mới bắt đầu đóng gói — mỗi hồ sơ
        một gói. Ngay trước khi dựng, từng PDF được mở ra soi lại (số trang phải khớp Excel).
        Chạy lại được: hồ sơ đã có gói thì bỏ qua.
        --allow-partial            kho còn hồ sơ lỗi vẫn đóng các hồ sơ SẠCH (hồ sơ lỗi bị giữ lại)
        --kind sip_hoso|aip_hoso   loại gói (mặc định aip_hoso)
        --agreement <mã>           mã thoả thuận nộp lưu — bắt buộc với sip_hoso
        --org <mã>                 mã cơ quan lưu trữ (mặc định: đoạn đầu mã phông, vd H49)
        --only <thư mục|mã hồ sơ>  chỉ đóng hồ sơ này (lặp lại được)
        --limit <n>                chỉ đóng n hồ sơ đầu (chạy thử)
        --allow-non-tt05           CHỈ ĐỂ THỬ: đóng cả tệp chưa đạt Điều 8.2.a TT05 (mặc định: tệp
                                   không PDF/A / thiếu lớp chữ / <200 dpi / <24 bit / chưa ký → TỪ CHỐI)
        --force                    dựng lại cả hồ sơ đã có gói
Chung:  --threads <n>  số luồng (mặc định 4 cho check --deep, 2 cho pack)

Mã thoát: 0 = không lỗi · 1 = có hồ sơ lỗi / gói hỏng · 2 = sai tham số hoặc không đọc được Excel.
")
}

struct Args {
    cmd: String,
    root: PathBuf,
    excel: PathBuf,
    report: Option<PathBuf>,
    out: Option<PathBuf>,
    deep: bool,
    kind: String,
    agreement: Option<String>,
    org: Option<String>,
    only: Vec<String>,
    limit: Option<usize>,
    allow_non_tt05: bool,
    allow_partial: bool,
    force: bool,
    threads: Option<usize>,
}

fn parse(argv: &[String]) -> Result<Args, String> {
    let cmd = argv.first().cloned().ok_or("thiếu lệnh con: check | pack")?;
    if cmd != "check" && cmd != "pack" {
        return Err(format!("lệnh con không rõ: {cmd}"));
    }
    let mut a = Args {
        cmd, root: PathBuf::new(), excel: PathBuf::new(), report: None, out: None, deep: false,
        kind: "aip_hoso".into(), agreement: None, org: None, only: Vec::new(), limit: None,
        allow_non_tt05: false, allow_partial: false, force: false, threads: None,
    };
    let mut it = argv[1..].iter();
    while let Some(k) = it.next() {
        let mut val = |name: &str| it.next().cloned().ok_or(format!("{name} cần một giá trị"));
        match k.as_str() {
            "--root" => a.root = val("--root")?.into(),
            "--excel" => a.excel = val("--excel")?.into(),
            "--report" => a.report = Some(val("--report")?.into()),
            "--out" => a.out = Some(val("--out")?.into()),
            "--kind" => a.kind = val("--kind")?,
            "--agreement" => a.agreement = Some(val("--agreement")?),
            "--org" => a.org = Some(val("--org")?),
            "--only" => a.only.push(val("--only")?),
            "--limit" => a.limit = Some(val("--limit")?.parse().map_err(|_| "--limit phải là số")?),
            "--threads" => a.threads = Some(val("--threads")?.parse().map_err(|_| "--threads phải là số")?),
            "--deep" => a.deep = true,
            "--allow-non-tt05" => a.allow_non_tt05 = true,
            "--allow-partial" => a.allow_partial = true,
            "--force" => a.force = true,
            other => return Err(format!("tham số không rõ: {other}")),
        }
    }
    if a.root.as_os_str().is_empty() || a.excel.as_os_str().is_empty() {
        return Err("bắt buộc --root và --excel".into());
    }
    if a.cmd == "pack" {
        if a.out.is_none() {
            return Err("pack bắt buộc --out".into());
        }
        if a.kind != "aip_hoso" && a.kind != "sip_hoso" {
            return Err("--kind chỉ nhận aip_hoso | sip_hoso".into());
        }
        if a.kind == "sip_hoso" && a.agreement.is_none() {
            return Err("gói SIP bắt buộc --agreement <mã thoả thuận nộp lưu>".into());
        }
    }
    Ok(a)
}

fn tick(label: &str, done: usize, total: usize) {
    if done == total || done % 10 == 0 {
        eprint!("\r  {label}: {done}/{total} hồ sơ   ");
        let _ = std::io::stderr().flush();
        if done == total {
            eprintln!();
        }
    }
}

pub fn cli_main(prog: &str, argv: &[String]) -> ExitCode {
    if argv.is_empty() || argv.iter().any(|a| a == "--help" || a == "-h") {
        print!("{}", usage(prog));
        return if argv.is_empty() { ExitCode::from(2) } else { ExitCode::SUCCESS };
    }
    let a = match parse(argv) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{e}\n\n{}", usage(prog));
            return ExitCode::from(2);
        }
    };
    eprintln!("Đọc Excel {} …", a.excel.display());
    let sheet = match excel::read(&a.excel) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    if !sheet.missing_columns.is_empty() {
        eprintln!("Excel thiếu cột bắt buộc: {}", sheet.missing_columns.join(", "));
        return ExitCode::from(2);
    }
    let plan = check::plan(&sheet, &a.root);
    eprintln!("  {} dòng tài liệu · {} hồ sơ · gốc {}", sheet.rows.len(), plan.dossiers.len(), plan.root.display());
    let deep = a.deep && a.cmd == "check";
    let rep = check::run(&sheet, &a.excel, &plan, deep, !a.allow_non_tt05, a.threads.unwrap_or(4), &|d, t| tick("kiểm", d, t));
    let report_dir = a.report.clone().unwrap_or_else(|| match (&a.cmd[..], &a.out) {
        ("pack", Some(o)) => o.clone(),
        _ => a.excel.parent().unwrap_or(&a.root).join("_digitext_kiem_tra"),
    });
    println!("{}", report::check_summary(&rep));
    match report::write_check(&rep, &report_dir) {
        Ok(files) => println!("Báo cáo kiểm: {}", files.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join("\n              ")),
        Err(e) => eprintln!("{e}"),
    }
    if a.cmd == "check" {
        return if rep.errors() == 0 { ExitCode::SUCCESS } else { ExitCode::from(1) };
    }
    if rep.errors() > 0 && !a.allow_partial {
        eprintln!(
            "\nCHƯA ĐÓNG GÓI: còn {} lỗi ở {} hồ sơ — dữ liệu chưa khớp với Excel.\nSửa theo kiem-tra-van-de.csv rồi chạy lại; hoặc thêm --allow-partial để đóng trước {} hồ sơ đã sạch.",
            rep.errors(), rep.dossiers.len() - rep.dossiers_ok(), rep.dossiers_ok()
        );
        return ExitCode::from(1);
    }
    let opts = pack::PackOptions {
        kind: a.kind, org_code: a.org, agreement: a.agreement, out_dir: a.out.expect("đã kiểm"), force: a.force,
        allow_non_tt05: a.allow_non_tt05, allow_partial: a.allow_partial, threads: a.threads.unwrap_or(2), only: a.only, limit: a.limit,
    };
    println!("\nĐóng gói vào {} …", opts.out_dir.display());
    let results = match pack::run(&plan, &rep.packable(), &opts, &|d, t, r| {
        if r.status == "failed" {
            eprintln!("\n  HỎNG {}: {}", r.hs_code, r.error.as_deref().unwrap_or(""));
        }
        tick("đóng gói", d, t)
    }) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    println!("{}", report::pack_summary(&results));
    match report::write_pack(&results, &opts.out_dir) {
        Ok(p) => println!("Kết quả đóng gói: {}", p.display()),
        Err(e) => eprintln!("{e}"),
    }
    let bad = results.iter().any(|r| r.status == "failed" || r.status == "skipped_errors");
    if bad { ExitCode::from(1) } else { ExitCode::SUCCESS }
}
