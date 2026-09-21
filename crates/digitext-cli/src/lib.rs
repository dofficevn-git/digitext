// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
pub mod tt02;
pub mod wipe;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Help,
    Version,
    Api,
    Ocr,
    Ingest,
    Dispatch,
    Sign,
    Wipe,
    Tt02,
    ExtImport,
    Unknown(String),
}
pub fn parse(r_1wh: Option<&str>) -> Route {
    match r_1wh {
        None | Some("--help" | "-h" | "help") => Route::Help,
        Some("--version" | "-V" | "version") => Route::Version,
        Some("api") => Route::Api,
        Some("ocr") => Route::Ocr,
        Some("ingest") => Route::Ingest,
        Some("dispatch" | "dispatcher") => Route::Dispatch,
        Some("sign") => Route::Sign,
        Some("wipe") => Route::Wipe,
        Some("tt02") => Route::Tt02,
        Some("ext-import" | "extimport") => Route::ExtImport,
        Some(r_1wi) => Route::Unknown(r_1wi.to_owned()),
    }
}
pub fn help_text() -> String {
    format!(
        "digitext {VERSION} — CLI hợp nhất phía server DigiText (§G/§P)\n\nCách dùng: digitext <lệnh> [tham số…]\n\nLệnh:\n  api       [serve | bootstrap-superadmin --username <tên> | backfill-source-pdfs]\n            API server (axum) + lệnh quản trị (mặc định: serve)\n  ocr       [run | --health | --version | --help]\n            Worker OCR (Triton/gRPC; §P-M0 DoD: --health in JSON)\n  ingest    <thư-mục-bó>\n            Nạp thủ công một bó bundle.json + files → documents + hàng đợi OCR (§7)\n  dispatch  Dispatcher fair-share (LISTEN ocr_jobs + nhịp định kỳ, §8)\n  sign      sign | sign-stamp | selftest[-stamp] | verify | make-lt | make-lta\n            Ký/xác minh PDF PAdES bằng USB token PKCS#11 (chạy `digitext sign` xem chi tiết)\n  wipe      --local | --server [--apply]\n            XOÁ SẠCH dữ liệu (máy user/admin hoặc máy chủ). Mặc định chỉ IN KẾ HOẠCH;\n            thi hành phải có --apply + gõ đúng câu xác nhận. KHÔNG hoàn tác được.\n  tt02      <mục-lục.json> --fond <mã> --org <mã> [--kind aip_hoso|sip_hoso] …\n            Chuyển đổi hồ sơ TT 02/2019 → yêu cầu đóng gói TT 05 (PK-7, Điều 51.3)\n  ext-import check|pack --root <kho> --excel <metadata.xlsx> [--out <thư mục gói>] …\n            Nhập kho tài liệu từ BÊN NGOÀI (thư mục PDF + Excel metadata): đối chiếu thư mục\n            với Excel rồi đóng gói TT 05 theo từng hồ sơ. Không cần máy chủ.\n\n  --help / -h       In trợ giúp này\n  --version / -V    In phiên bản\n\nMỗi lệnh giữ NGUYÊN tham số + biến môi trường DIGITEXT_* + mã thoát của binary gốc\n(digitext-api · digitext-ocr · digitext-ingest · digitext-dispatcher · token-signpdf).\n"
    )
}
