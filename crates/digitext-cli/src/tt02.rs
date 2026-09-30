// Copyright (c) 2026 DigiText. All rights reserved.
use digitext_package::tt02::{f_11h, Tos, Tot};
use digitext_package::wire::Tq4;
use std::collections::BTreeMap;
use std::process::ExitCode;
#[derive(Debug, PartialEq, Eq)]
pub struct Toq {
    pub input: String,
    pub fond_code: String,
    pub archive_org_code: String,
    pub fond_name: Option<String>,
    pub archive_org_name: Option<String>,
    pub kind: String,
    pub submission_agreement: Option<String>,
    pub map_file: Option<String>,
    pub out_dir: String,
}
pub fn parse_args(r_1wl: &[String]) -> Result<Toq, String> {
    let mut input = None;
    let mut fond = None;
    let mut org = None;
    let mut fond_name = None;
    let mut org_name = None;
    let mut kind = "aip_hoso".to_string();
    let mut agreement = None;
    let mut map_file = None;
    let mut out_dir = ".".to_string();
    let mut r_1wm = r_1wl.iter();
    while let Some(a) = r_1wm.next() {
        let mut r_1wn = |name: &str| {
            r_1wm
                .next()
                .cloned()
                .ok_or_else(|| format!("{name} thiếu giá trị"))
        };
        match a.as_str() {
            "--fond" => fond = Some(r_1wn("--fond")?),
            "--org" => org = Some(r_1wn("--org")?),
            "--fond-name" => fond_name = Some(r_1wn("--fond-name")?),
            "--org-name" => org_name = Some(r_1wn("--org-name")?),
            "--kind" => kind = r_1wn("--kind")?,
            "--agreement" => agreement = Some(r_1wn("--agreement")?),
            "--map" => map_file = Some(r_1wn("--map")?),
            "--out" => out_dir = r_1wn("--out")?,
            other if other.starts_with("--") => {
                return Err(format!("cờ không rõ: {other}"));
            }
            other => {
                if input.replace(other.to_string()).is_some() {
                    return Err("chỉ nhận MỘT tệp đầu vào".into());
                }
            }
        }
    }
    if kind != "aip_hoso" && kind != "sip_hoso" {
        return Err(format!(
            "--kind {kind:?} — chỉ nhận aip_hoso | sip_hoso"
        ));
    }
    Ok(Toq {
        input: input.ok_or("thiếu tệp đầu vào (<mục-lục>.json)")?,
        fond_code: fond.ok_or("thiếu --fond <mã phông>")?,
        archive_org_code: org.ok_or("thiếu --org <mã cơ quan lưu trữ>")?,
        fond_name,
        archive_org_name: org_name,
        kind,
        submission_agreement: agreement,
        map_file,
        out_dir,
    })
}
fn f_2oa(prog: &str) -> String {
    format!(
        "Cách dùng: {prog} <mục-lục.json> --fond <mã phông> --org <mã cơ quan> [tuỳ chọn]\n\nChuyển hồ sơ cấu trúc TT 02/2019 → yêu cầu đóng gói TT 05/2025 (PK-7, Điều 51.3).\n\n  <mục-lục.json>     Một hồ sơ hoặc mảng hồ sơ theo bảng trường TT 02 (camelCase:\n                     maHoSo, tieuDe, thoiHanBaoQuan, vanBan[{{stt, tenLoai, …}}]…)\n  --fond <mã>        Mã phông lưu trữ (fondCode TT 05)\n  --org <mã>         Mã cơ quan lưu trữ (tiền tố arcFileCode)\n  --fond-name <tên>  Tên cơ quan hình thành phông (agent CREATOR trong METS)\n  --org-name <tên>   Tên cơ quan lưu trữ (agent ARCHIVIST trong METS)\n  --kind <loại>      aip_hoso (mặc định) | sip_hoso\n  --agreement <mã>   Mã thoả thuận nộp — bắt buộc với sip_hoso\n  --map <map.json>   Gán documents.id server cho văn bản: {{\"<mã hồ sơ>\": {{\"<stt>\": \"<uuid>\"}}}}\n  --out <thư mục>    Nơi ghi <mã hồ sơ>.wire.json (mặc định: thư mục hiện tại)\n\nTệp wire nộp bằng desktop (Đóng gói theo tiêu chuẩn) hoặc:\n  curl -X POST …/api/v1/packages -d '{{\"request\": <nội dung wire.json>}}'\n"
    )
}
type DocMap = BTreeMap<String, BTreeMap<String, uuid::Uuid>>;
fn f_17m(r_1wt: &Tq4) -> Vec<String> {
    let r_1wp = vec![Some(b"%PDF".to_vec()); r_1wt.documents.len()];
    let r_1wu = digitext_package::Tlv {
        name: "digitext-tt02".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    };
    let r_1wq = chrono::Utc::now()
        .with_timezone(&chrono::FixedOffset::east_opt(7 * 3600).expect("+07:00 hợp lệ"));
    let r_1wo = match r_1wt.to_build_request(r_1wp, r_1wu, r_1wq, None) {
        Ok(b) => b,
        Err(e) => return vec![e],
    };
    match digitext_package::f_vn(&r_1wo, &mut digitext_package::ids::Tl1(0)) {
        Ok(r_1ws) => {
            let kind = match r_1wt.kind() {
                Ok(r_1wr) => r_1wr,
                Err(e) => return vec![e],
            };
            digitext_package::f_2p7(kind, &r_1ws.files)
                .into_iter()
                .map(|v| format!("[{:?}] {}", v.severity, v.message))
                .collect()
        }
        Err(e) => vec![e.to_string()],
    }
}
pub fn cli_main(prog: &str, r_1wv: &[String]) -> ExitCode {
    if r_1wv.iter().any(|a| a == "--help" || a == "-h") || r_1wv.is_empty() {
        print!("{}", f_2oa(prog));
        return if r_1wv.is_empty() {
            ExitCode::from(2)
        } else {
            ExitCode::SUCCESS
        };
    }
    let r_1ww = match parse_args(r_1wv) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{e}\n\n{}", f_2oa(prog));
            return ExitCode::from(2);
        }
    };
    match run(&r_1ww) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(2),
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
fn run(a: &Toq) -> Result<bool, String> {
    let r_1x5 = std::fs::read_to_string(&a.input)
        .map_err(|e| format!("đọc {} lỗi: {e}", a.input))?;
    let r_1wz: Vec<Tos> = match serde_json::from_str::<serde_json::Value>(&r_1x5)
        .map_err(|e| format!("{} không phải JSON: {e}", a.input))?
    {
        serde_json::Value::Array(v) => serde_json::from_value(serde_json::Value::Array(v))
            .map_err(|e| format!("mảng hồ sơ không đọc được: {e}"))?,
        r_1x1 => {
            vec![serde_json::from_value(r_1x1)
                .map_err(|e| format!("hồ sơ không đọc được: {e}"))?]
        }
    };
    let map: DocMap = match &a.map_file {
        None => DocMap::new(),
        Some(p) => serde_json::from_str(
            &std::fs::read_to_string(p).map_err(|e| format!("đọc {p} lỗi: {e}"))?,
        )
        .map_err(|e| format!("{p} sai khuôn map: {e}"))?,
    };
    std::fs::create_dir_all(&a.out_dir)
        .map_err(|e| format!("tạo {} lỗi: {e}", a.out_dir))?;
    let r_1x2 = Tot {
        fond_code: a.fond_code.clone(),
        archive_org_code: a.archive_org_code.clone(),
        kind: a.kind.clone(),
        submission_agreement: a.submission_agreement.clone(),
        fond_name: a.fond_name.clone(),
        archive_org_name: a.archive_org_name.clone(),
    };
    let mut clean = true;
    for mut r_1x0 in r_1wz {
        if let Some(r_1x4) = map.get(&r_1x0.ma_ho_so) {
            for r_1x6 in &mut r_1x0.van_ban {
                if r_1x6.document_id.is_none() {
                    r_1x6.document_id = r_1x4.get(&r_1x6.stt.to_string()).copied();
                }
            }
        }
        let out = f_11h(&r_1x0, &r_1x2);
        println!("── {} — {}", r_1x0.ma_ho_so, r_1x0.tieu_de);
        for n in &out.report.notes {
            println!("   • {n}");
        }
        for e in &out.report.errors {
            println!("   ✗ {e}");
        }
        if out.report.errors.is_empty() {
            for v in f_17m(&out.request) {
                println!("   ✗ validator TT 05: {v}");
                clean = false;
            }
        } else {
            clean = false;
        }
        let name: String = r_1x0
            .ma_ho_so
            .chars()
            .map(|r_1wy| {
                if r_1wy == '/' || r_1wy == '\\' {
                    '_'
                } else {
                    r_1wy
                }
            })
            .collect();
        if !name.is_empty() {
            let r_1x3 = std::path::Path::new(&a.out_dir).join(format!("{name}.wire.json"));
            let r_1wx = serde_json::to_string_pretty(&out.request)
                .map_err(|e| format!("serialize wire lỗi: {e}"))?;
            std::fs::write(&r_1x3, r_1wx)
                .map_err(|e| format!("ghi {} lỗi: {e}", r_1x3.display()))?;
            println!("   → {}", r_1x3.display());
        }
    }
    if !clean {
        eprintln!(
            r#"
Có hồ sơ lỗi chặn — sửa dữ liệu/map rồi chạy lại (tệp wire sạch vẫn dùng được)."#
        );
    }
    Ok(clean)
}
