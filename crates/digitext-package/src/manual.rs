// Copyright (c) 2026 DigiText. All rights reserved.
// Sinh `documentation/UserManual.pdf` cho từng gói — "Tài liệu hướng dẫn gói tin" theo mẫu gói
// dự án Đầm Hà (DOffice builder): thông tin tổ chức, phần mềm, ngày tạo gói, thông tin hồ sơ,
// cấu trúc thư mục, danh sách văn bản. Ngày giờ lấy từ `created_at` của gói (đồng hồ máy đang
// đóng gói, cùng giá trị với METS CREATEDATE và PREMIS eventDateTime).
use crate::ead::DocEad;
use crate::model::{OrgIdent, PackageKind, PackageMeta, Tga, Tlv};
use chrono::{DateTime, FixedOffset};
use digitext_pdf::manual::ManualBlock;

pub struct ManualCtx<'a> {
    pub req: &'a Tga,
    pub objid: &'a str,
    pub zip_name: &'a str,
    pub fond: &'a OrgIdent,
    pub archive: &'a OrgIdent,
    pub software: &'a Tlv,
    pub created_at: DateTime<FixedOffset>,
    pub include_premis: bool,
}

fn kv(label: &str, value: impl Into<String>) -> (String, String) {
    (label.to_string(), value.into())
}
fn opt(v: &Option<String>) -> String {
    v.as_deref().map(str::trim).filter(|s| !s.is_empty()).unwrap_or("—").to_string()
}
fn org(o: &OrgIdent) -> String {
    if o.code.trim().is_empty() || o.code.trim() == o.name.trim() {
        o.name.clone()
    } else {
        format!("{} ({})", o.name, o.code)
    }
}
pub fn maintenance_name(code: &str) -> &'static str {
    match code {
        "01" => "Vĩnh viễn",
        "02" => "70 năm",
        "03" => "50 năm",
        "04" => "30 năm",
        "05" => "20 năm",
        "06" => "10 năm",
        "07" => "Khác",
        _ => "—",
    }
}
pub fn language_name(code: &str) -> String {
    match code {
        "01" => "Tiếng Việt".into(),
        "02" => "Tiếng Anh".into(),
        "03" => "Tiếng Pháp".into(),
        "04" => "Tiếng Nga".into(),
        "05" => "Tiếng Trung Quốc".into(),
        "06" => "Tiếng Đức".into(),
        "07" => "Tiếng Nhật".into(),
        "08" => "Tiếng Hàn".into(),
        "09" => "Hán Nôm".into(),
        "10" => "Tiếng dân tộc".into(),
        "11" => "Khác".into(),
        other => format!("Mã {other}"),
    }
}
pub fn type_name(code: &str) -> &'static str {
    match code {
        "01" => "Nghị quyết",
        "02" => "Quyết định",
        "03" => "Chỉ thị",
        "04" => "Quy chế",
        "05" => "Quy định",
        "06" => "Thông cáo",
        "07" => "Thông báo",
        "08" => "Hướng dẫn",
        "09" => "Chương trình",
        "10" => "Kế hoạch",
        "11" => "Phương án",
        "12" => "Đề án",
        "13" => "Dự án",
        "14" => "Báo cáo",
        "15" => "Tờ trình",
        "16" => "Giấy ủy quyền",
        "17" => "Phiếu gửi",
        "18" => "Phiếu chuyển",
        "19" => "Phiếu báo",
        "20" => "Biên bản",
        "21" => "Hợp đồng",
        "22" => "Công văn",
        "23" => "Công điện",
        "24" => "Bản ghi nhớ",
        "25" => "Bản thỏa thuận",
        "26" => "Giấy mời",
        "27" => "Giấy giới thiệu",
        "28" => "Giấy nghỉ phép",
        "29" => "Thư công",
        "30" => "Bản đồ",
        "31" => "Bản vẽ kỹ thuật",
        "32" => "Khác",
        _ => "—",
    }
}

/// Dựng nội dung và kết xuất PDF theo mẫu gói dự án (tiêu đề xanh, thanh mục, bảng hai cột,
/// bảng danh sách văn bản). Trả `Err` khi font hỏng hoặc không ghi được PDF.
pub fn user_manual(cx: &ManualCtx<'_>, font: &[u8]) -> Result<Vec<u8>, String> {
    let req = cx.req;
    let kind = match req.kind {
        PackageKind::SipHoso | PackageKind::SipTailieu => "SIP",
        PackageKind::AipHoso | PackageKind::AipTailieu => "AIP",
        PackageKind::Dip => "DIP",
    };
    let ngay = cx.created_at.format("%d/%m/%Y %H:%M:%S").to_string();
    let chuan = format!("E-ARK CSIP/{kind} — Thông tư 05/2025/TT-BNV");
    let (ma_nhan, ma, tieu_de) = match &req.meta {
        PackageMeta::Hoso(h) => ("Mã hồ sơ", h.file_code.clone(), h.title.clone()),
        PackageMeta::GoiTin(g) => ("Mã gói tin", g.file_code.clone(), g.title.clone()),
        PackageMeta::Dip(d) => ("Mã yêu cầu", d.request_id.clone(), format!("Bản sao khai thác theo yêu cầu {}", d.request_id)),
    };
    let phan_mem = format!("{} v{}", cx.software.name, cx.software.version);
    let mut b: Vec<ManualBlock> = Vec::new();
    b.push(ManualBlock::Title(format!("TÀI LIỆU HƯỚNG DẪN GÓI TIN {kind}")));
    b.push(ManualBlock::Subtitle(format!("{ma_nhan}: {ma}")));
    b.push(ManualBlock::Subtitle(tieu_de.clone()));
    b.push(ManualBlock::Gap(4.0));

    b.push(ManualBlock::SectionBar("1. THÔNG TIN TỔ CHỨC VÀ DỰ ÁN".into()));
    let mut r1 = vec![kv("Đơn vị tạo lập", org(req.packager.as_ref().unwrap_or(cx.fond)))];
    if req.packager.is_some() {
        r1.push(kv("Cơ quan hình thành phông", org(cx.fond)));
    }
    r1.push(kv("Cơ quan lưu trữ", org(cx.archive)));
    r1.push(kv("Phần mềm đóng gói", phan_mem.clone()));
    r1.push(kv(&format!("Ngày tạo gói {kind}"), ngay.clone()));
    r1.push(kv("Chuẩn áp dụng", chuan));
    b.push(ManualBlock::KeyValue(r1));

    b.push(ManualBlock::SectionBar(match &req.meta {
        PackageMeta::Hoso(_) => "2. THÔNG TIN HỒ SƠ".into(),
        PackageMeta::GoiTin(_) => "2. THÔNG TIN GÓI TIN TÀI LIỆU".into(),
        PackageMeta::Dip(_) => "2. THÔNG TIN YÊU CẦU KHAI THÁC".into(),
    }));
    let mut r2: Vec<(String, String)> = Vec::new();
    if let Some(a) = req.submission_agreement.as_deref().filter(|s| !s.trim().is_empty()) {
        r2.push(kv("Mã yêu cầu nộp", a));
    }
    if let Some(n) = req.zip_ordinal {
        r2.push(kv("Số thứ tự trong lần nộp", n.to_string()));
    }
    r2.push(kv(&format!("Tên ZIP {kind}"), cx.zip_name));
    r2.push(kv("Package ID (OBJID)", cx.objid));
    match &req.meta {
        PackageMeta::Hoso(h) => {
            r2.push(kv("Mã hồ sơ", &h.file_code));
            r2.push(kv("Tên cơ quan / Mã phông", format!("{} (Phông: {})", cx.fond.name, req.fond_code)));
            r2.push(kv("Tiêu đề hồ sơ", &h.title));
            r2.push(kv("Thời gian", format!("{} — {}", opt(&h.start_date), opt(&h.end_date))));
            r2.push(kv("Ngôn ngữ", if h.language.is_empty() { "—".to_string() } else { h.language.iter().map(|c| language_name(c)).collect::<Vec<_>>().join(", ") }));
            r2.push(kv("Thời hạn bảo quản", maintenance_name(&h.maintenance)));
            r2.push(kv("Chế độ sử dụng", match h.mode.as_str() { "01" => "Công khai", "02" => "Hạn chế", "03" => "Mật", _ => "—" }));
            r2.push(kv("Số tài liệu", format!("{} tài liệu", req.documents.len())));
            if let Some(p) = h.paper_file_code.as_deref().filter(|s| !s.trim().is_empty()) {
                r2.push(kv("Mã hồ sơ giấy", p));
            }
        }
        PackageMeta::GoiTin(g) => {
            r2.push(kv("Mã gói tin", &g.file_code));
            r2.push(kv("Tên cơ quan / Mã phông", format!("{} (Phông: {})", cx.fond.name, req.fond_code)));
            r2.push(kv("Tiêu đề", &g.title));
            r2.push(kv("Nguồn", &g.source));
            r2.push(kv("Số tài liệu", format!("{} tài liệu", req.documents.len())));
        }
        PackageMeta::Dip(d) => {
            r2.push(kv("Mã yêu cầu", &d.request_id));
            r2.push(kv("Ngày yêu cầu", &d.request_date));
            r2.push(kv("Mục đích", opt(&d.purpose_content)));
            r2.push(kv("Số tài liệu", format!("{} tài liệu", req.documents.len())));
        }
    }
    b.push(ManualBlock::KeyValue(r2));

    b.push(ManualBlock::SectionBar(format!("3. CẤU TRÚC GÓI {kind}")));
    let ead = req.kind.package_ead_name();
    let mut ct: Vec<(usize, String, String)> = vec![
        (0, "METS.xml".into(), format!("File METS gốc mô tả toàn bộ gói {kind}")),
        (0, "metadata/".into(), "Chứa các file metadata".into()),
        (1, "descriptive/".into(), format!("Metadata mô tả hồ sơ (EAD: {ead})")),
    ];
    if cx.include_premis {
        ct.push((1, "preservation/".into(), "Metadata bảo quản (PREMIS.xml): đối tượng, sự kiện, agent".into()));
    }
    ct.push((0, "representations/".into(), "Các đại diện số của hồ sơ".into()));
    ct.push((1, "rep1/".into(), "Đại diện số cấp 1".into()));
    ct.push((2, "METS.xml".into(), "METS mô tả đại diện rep1".into()));
    ct.push((2, "data/".into(), "Chứa các file PDF tài liệu".into()));
    ct.push((2, "metadata/descriptive/".into(), "Chứa EAD_doc_* của từng tài liệu".into()));
    if cx.include_premis {
        ct.push((2, "metadata/preservation/".into(), "PREMIS_rep1.xml của đại diện rep1".into()));
    }
    ct.push((0, "schemas/".into(), "Các file XSD schema (METS, EAD, EAD_doc, xlink, DILCIS, PREMIS)".into()));
    ct.push((0, "documentation/".into(), format!("Tài liệu hướng dẫn gói {kind} (UserManual.pdf){}", if req.kind == PackageKind::Dip { ", văn bản xác thực (VanBanXacThuc.pdf)" } else { "" })));
    b.push(ManualBlock::PathTable(ct));

    b.push(ManualBlock::SectionBar("4. DANH SÁCH TÀI LIỆU TRONG HỒ SƠ".into()));
    let rows: Vec<Vec<String>> = req
        .documents
        .iter()
        .enumerate()
        .map(|(i, d)| match &d.metadata {
            DocEad::Doc(x) => vec![
                (i + 1).to_string(),
                opt(&x.subject),
                opt(&x.issued_date),
                x.number_of_page.map(|n| n.to_string()).unwrap_or_else(|| "—".into()),
                opt(&x.organ_name),
                type_name(&x.type_name).to_string(),
            ],
            DocEad::Pic(x) => vec![(i + 1).to_string(), opt(&x.image_title), "—".into(), "—".into(), opt(&x.photographer), "Ảnh".into()],
            DocEad::Media(x) => vec![(i + 1).to_string(), opt(&x.movie_title), "—".into(), "—".into(), opt(&x.recorder), "Phim / âm thanh".into()],
        })
        .collect();
    b.push(ManualBlock::Table {
        headers: ["#", "Trích yếu nội dung", "Ngày VB", "Số trang", "Cơ quan ban hành", "Loại văn bản"].iter().map(|s| s.to_string()).collect(),
        widths: vec![5.0, 36.0, 12.0, 9.0, 22.0, 16.0],
        rows,
    });
    b.push(ManualBlock::Footer(format!("Sinh tự động bởi {phan_mem}  |  {ngay}  |  E-ARK CSIP — TT05/2025/TT-BNV")));
    digitext_pdf::manual::render_manual(&b, font).map_err(|e| e.to_string())
}
