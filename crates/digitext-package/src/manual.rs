// Copyright (c) 2026 DigiText. All rights reserved.
// Sinh `documentation/UserManual.pdf` cho từng gói — "Tài liệu hướng dẫn gói tin" theo mẫu gói
// dự án Đầm Hà (DOffice builder): thông tin tổ chức, phần mềm, ngày tạo gói, thông tin hồ sơ,
// cấu trúc thư mục, danh sách văn bản. Ngày giờ lấy từ `created_at` của gói (đồng hồ máy đang
// đóng gói, cùng giá trị với METS CREATEDATE và PREMIS eventDateTime).
use crate::ead::DocEad;
use crate::model::{OrgIdent, PackageKind, PackageMeta, Tga, Tlv};
use chrono::{DateTime, FixedOffset};
use digitext_pdf::FormBlock;

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

fn field(label: &str, value: impl Into<String>) -> FormBlock {
    FormBlock::Field { label: label.to_string(), value: value.into() }
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

/// Dựng nội dung và kết xuất PDF. Trả `Err` khi font hỏng hoặc không ghi được PDF.
pub fn user_manual(cx: &ManualCtx<'_>, font: &[u8]) -> Result<Vec<u8>, String> {
    let req = cx.req;
    let kind = match req.kind {
        PackageKind::SipHoso | PackageKind::SipTailieu => "SIP",
        PackageKind::AipHoso | PackageKind::AipTailieu => "AIP",
        PackageKind::Dip => "DIP",
    };
    let ngay = cx.created_at.format("%d/%m/%Y %H:%M:%S").to_string();
    let chuan = match kind {
        "SIP" => "E-ARK CSIP/SIP — Thông tư 05/2025/TT-BNV",
        "DIP" => "E-ARK CSIP/DIP — Thông tư 05/2025/TT-BNV",
        _ => "E-ARK CSIP — Thông tư 05/2025/TT-BNV",
    };
    let (ma, tieu_de) = match &req.meta {
        PackageMeta::Hoso(h) => (h.file_code.clone(), h.title.clone()),
        PackageMeta::GoiTin(g) => (g.file_code.clone(), g.title.clone()),
        PackageMeta::Dip(d) => (d.request_id.clone(), format!("Bản sao khai thác theo yêu cầu {}", d.request_id)),
    };
    let phan_mem = format!("{} v{}", cx.software.name, cx.software.version);
    let mut b: Vec<FormBlock> = Vec::new();

    b.push(FormBlock::SectionHeading("1. THÔNG TIN TỔ CHỨC VÀ DỰ ÁN".into()));
    b.push(field("Đơn vị tạo lập", org(req.packager.as_ref().unwrap_or(cx.fond))));
    b.push(field("Cơ quan hình thành phông", org(cx.fond)));
    b.push(field("Cơ quan lưu trữ", org(cx.archive)));
    b.push(field("Phần mềm đóng gói", phan_mem.clone()));
    b.push(field(&format!("Ngày tạo gói {kind}"), ngay.clone()));
    b.push(field("Chuẩn áp dụng", chuan));

    b.push(FormBlock::SectionHeading(match &req.meta {
        PackageMeta::Hoso(_) => "2. THÔNG TIN HỒ SƠ".into(),
        PackageMeta::GoiTin(_) => "2. THÔNG TIN GÓI TIN TÀI LIỆU".into(),
        PackageMeta::Dip(_) => "2. THÔNG TIN YÊU CẦU KHAI THÁC".into(),
    }));
    if let Some(a) = req.submission_agreement.as_deref().filter(|s| !s.trim().is_empty()) {
        b.push(field("Mã yêu cầu nộp", a));
    }
    if let Some(n) = req.zip_ordinal {
        b.push(field("Số thứ tự trong lần nộp", n.to_string()));
    }
    b.push(field(&format!("Tên ZIP {kind}"), cx.zip_name));
    b.push(field("Package ID (OBJID)", cx.objid));
    match &req.meta {
        PackageMeta::Hoso(h) => {
            b.push(field("Mã hồ sơ", &h.file_code));
            b.push(field("Tên cơ quan / Mã phông", format!("{} (Phông: {})", cx.fond.name, req.fond_code)));
            b.push(field("Tiêu đề hồ sơ", &h.title));
            b.push(field("Thời gian", format!("{} — {}", opt(&h.start_date), opt(&h.end_date))));
            b.push(field("Ngôn ngữ", if h.language.is_empty() { "—".to_string() } else { h.language.iter().map(|c| language_name(c)).collect::<Vec<_>>().join(", ") }));
            b.push(field("Thời hạn bảo quản", maintenance_name(&h.maintenance)));
            b.push(field("Chế độ sử dụng", match h.mode.as_str() { "01" => "Công khai", "02" => "Hạn chế", "03" => "Mật", _ => "—" }));
            b.push(field("Số tài liệu", format!("{} tài liệu", req.documents.len())));
            if let Some(p) = h.paper_file_code.as_deref().filter(|s| !s.trim().is_empty()) {
                b.push(field("Mã hồ sơ giấy", p));
            }
        }
        PackageMeta::GoiTin(g) => {
            b.push(field("Mã gói tin", &g.file_code));
            b.push(field("Tên cơ quan / Mã phông", format!("{} (Phông: {})", cx.fond.name, req.fond_code)));
            b.push(field("Tiêu đề", &g.title));
            b.push(field("Nguồn", &g.source));
            b.push(field("Số tài liệu", format!("{} tài liệu", req.documents.len())));
        }
        PackageMeta::Dip(d) => {
            b.push(field("Mã yêu cầu", &d.request_id));
            b.push(field("Ngày yêu cầu", &d.request_date));
            b.push(field("Mục đích", opt(&d.purpose_content)));
            b.push(field("Số tài liệu", format!("{} tài liệu", req.documents.len())));
        }
    }

    b.push(FormBlock::SectionHeading(format!("3. CẤU TRÚC GÓI {kind}")));
    let mut cau_truc = vec![
        format!("METS.xml — File METS gốc mô tả toàn bộ gói {kind}"),
        format!("metadata/descriptive/{} — Metadata mô tả (EAD)", req.kind.package_ead_name()),
    ];
    if cx.include_premis {
        cau_truc.push("metadata/preservation/PREMIS.xml — Metadata bảo quản (PREMIS): đối tượng, sự kiện, agent".into());
    }
    cau_truc.push("representations/rep1/METS.xml — METS mô tả đại diện rep1".into());
    cau_truc.push("representations/rep1/data/ — Các tệp PDF tài liệu".into());
    cau_truc.push("representations/rep1/metadata/descriptive/ — EAD_doc_* của từng tài liệu".into());
    if cx.include_premis {
        cau_truc.push("representations/rep1/metadata/preservation/PREMIS_rep1.xml — PREMIS của đại diện rep1".into());
    }
    cau_truc.push("schemas/ — Các file XSD (METS, xlink, DILCIS CSIP, EAD, EAD_doc, PREMIS)".into());
    cau_truc.push(format!("documentation/ — Tài liệu hướng dẫn gói {kind} (UserManual.pdf){}", if req.kind == PackageKind::Dip { " và văn bản xác thực (VanBanXacThuc.pdf)" } else { "" }));
    b.push(FormBlock::Bullets(cau_truc));

    b.push(FormBlock::SectionHeading("4. DANH SÁCH TÀI LIỆU TRONG HỒ SƠ".into()));
    for (i, d) in req.documents.iter().enumerate() {
        let dong = match &d.metadata {
            DocEad::Doc(x) => format!(
                "{}. {} — Ngày VB: {} — {} trang — {} — {}",
                i + 1,
                opt(&x.subject),
                opt(&x.issued_date),
                x.number_of_page.map(|n| n.to_string()).unwrap_or_else(|| "—".into()),
                opt(&x.organ_name),
                type_name(&x.type_name)
            ),
            DocEad::Pic(x) => format!("{}. {} — Ảnh — {}", i + 1, opt(&x.image_title), opt(&x.photographer)),
            DocEad::Media(x) => format!("{}. {} — Phim/âm thanh — {}", i + 1, opt(&x.movie_title), opt(&x.recorder)),
        };
        b.push(FormBlock::Paragraph(format!("{dong} ({})", d.file_name)));
    }
    b.push(FormBlock::Paragraph(format!("Sinh tự động bởi {phan_mem}  |  {ngay}  |  E-ARK CSIP — TT05/2025/TT-BNV")));

    let title = format!("TÀI LIỆU HƯỚNG DẪN GÓI TIN {kind}");
    let sub = format!("Mã: {ma} — {tieu_de}");
    digitext_pdf::render_form_plain(&title, Some(&sub), &b, font).map_err(|e| e.to_string())
}
