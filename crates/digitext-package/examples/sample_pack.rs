// Dựng gói mẫu từ một tệp PDF để soi cấu trúc bằng ml/tools/check_tt05_package.py.
// cargo run -p digitext-package --example sample_pack -- <aip_hoso|sip_hoso> <tệp.pdf> <thư mục ra> [allow_non_pdfa]
use digitext_package::ids::Tic;
use digitext_package::wire::Tq4;
use digitext_package::{f_vn, Tlv};
fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 4 {
        eprintln!("cách dùng: sample_pack <aip_hoso|sip_hoso> <tệp.pdf> <thư mục ra> [allow_non_pdfa]");
        std::process::exit(2);
    }
    let kind = &a[1];
    let pdf = std::fs::read(&a[2]).expect("đọc PDF");
    let out = std::path::PathBuf::from(&a[3]);
    let allow = a.get(4).map(|s| s == "allow_non_pdfa").unwrap_or(false);
    let sip = kind.starts_with("sip");
    let json = serde_json::json!({
        "kind": kind,
        "fondCode": "H49.139.01",
        "archiveOrgCode": "H49.139",
        "fondName": "Phòng Kinh tế và Hạ tầng huyện Đầm Hà",
        "archiveOrgName": "UBND xã Đầm Hà",
        "packagerName": "DigiText JSC",
        "packagerCode": "DGT",
        "allowNonPdfa": allow,
        "label": "Hồ sơ 599",
        "submissionAgreement": if sip { serde_json::Value::String("TT-2026-001".into()) } else { serde_json::Value::Null },
        "meta": {
            "type": "hoso",
            "fileCode": "H49.139.01.01.356.599",
            "title": "Hồ sơ về việc phê duyệt báo cáo kinh tế kỹ thuật công trình đường giao thông nông thôn",
            "maintenance": "01",
            "mode": "01",
            "language": ["01"],
            "startDate": "01/04/2011",
            "endDate": "30/04/2011",
            "totalDoc": 1,
            "numberOfPaper": 12,
            "numberOfPage": 12,
            "confidenceLevel": "01",
            "paperFileCode": "01.356.599"
        },
        "includePremis": true,
        "documents": [{
            "fileName": "H49.139.01.01.356.599.0000001.pdf",
            "metadata": {
                "type": "doc",
                "docCode": "H49.139.01.01.356.599.0000001",
                "maintenance": "01",
                "typeName": "03",
                "codeNumber": "1234",
                "codeNotation": "QĐ-UBND",
                "issuedDate": "12/04/2011",
                "organName": "UBND huyện Đầm Hà",
                "subject": "Về việc phê duyệt báo cáo kinh tế kỹ thuật",
                "language": ["01"],
                "numberOfPage": 12,
                "mode": "01",
                "confidenceLevel": "01",
                "process": "1"
            }
        }]
    });
    let wire: Tq4 = serde_json::from_value(json).expect("wire");
    let now = chrono::Local::now().fixed_offset();
    let software = Tlv { name: "DigiText".into(), version: env!("CARGO_PKG_VERSION").into() };
    let req = match wire.to_build_request(vec![Some(pdf)], software, now, None) {
        Ok(mut r) => {
            // documentation/UserManual.pdf như worker máy chủ: đọc từ DIGITEXT_PACKAGE_MANUAL.
            r.user_manual_pdf = std::env::var_os("DIGITEXT_PACKAGE_MANUAL")
                .and_then(|p| std::fs::read(p).ok())
                .filter(|b| b.starts_with(b"%PDF-"));
            r
        }
        Err(e) => {
            eprintln!("to_build_request: {e}");
            std::process::exit(1);
        }
    };
    let built = match f_vn(&req, &mut Tic) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("f_vn: {e:?}");
            std::process::exit(1);
        }
    };
    std::fs::create_dir_all(&out).unwrap();
    let path = out.join(&built.zip_name);
    std::fs::write(&path, &built.zip_bytes).unwrap();
    println!("{}", path.display());
    for f in built.files.keys() {
        println!("  {f}");
    }
}
