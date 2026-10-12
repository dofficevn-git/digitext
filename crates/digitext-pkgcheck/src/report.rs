// Copyright (c) 2026 DigiText. All rights reserved.
//! Báo cáo PDF kết quả kiểm tra gói tin: đầu văn bản (cơ quan / quốc hiệu), tổng hợp, chi tiết từng gói
//! (lỗi, cảnh báo, mục đạt, bảng từng tệp PDF), khối ký người kiểm / người duyệt.
use crate::{Level, PackageResult};
use digitext_pdf::manual::{render_manual, ManualBlock};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportHeader {
    /// Cơ quan chủ quản (dòng trên, có thể trống).
    pub parent_org: String,
    /// Đơn vị kiểm.
    pub org: String,
    /// Địa danh ghi ở dòng ngày tháng ("Hà Nội").
    pub place: String,
    pub inspector: String,
    pub inspector_title: String,
    pub approver: String,
    pub approver_title: String,
}

fn s(x: &str) -> String {
    x.to_string()
}
fn n(v: u64) -> String {
    let s = v.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push('.');
        }
        out.push(c);
    }
    out
}
fn mb(b: u64) -> String {
    if b >= 1_000_000_000 {
        format!("{:.2} GB", b as f64 / 1e9).replace('.', ",")
    } else {
        format!("{:.1} MB", b as f64 / 1e6).replace('.', ",")
    }
}

/// Nhãn ngắn cho bảng từng tệp (lời giải thích đầy đủ đã có ở phần lỗi / cảnh báo của gói).
fn short(p: &str) -> String {
    let rules: [(&str, &str); 10] = [
        ("PDF hỏng", "PDF hỏng"),
        ("không phải PDF/A", "không PDF/A"),
        ("không nhúng hồ sơ màu", "thiếu hồ sơ màu"),
        ("chữ ký số KHÔNG", "chữ ký không toàn vẹn"),
        ("chữ ký không gắn", "chưa PAdES (ESS)"),
        ("không có dấu thời gian", "không TSA"),
        ("LTV", "chưa LTV"),
        ("không xác minh được chữ ký", "lỗi chữ ký"),
        ("chưa ký số", "chưa ký"),
        ("độ phân giải", "dpi thấp"),
    ];
    if let Some((_, s)) = rules.iter().find(|(k, _)| p.contains(k)) {
        return s.to_string();
    }
    if let Some(x) = p.split(" trang không có lớp chữ").next().filter(|_| p.contains("lớp chữ")) {
        return format!("{x} trang thiếu lớp chữ");
    }
    if p.starts_with("ảnh ") {
        return p.split(" (").next().unwrap_or(p).to_string();
    }
    p.to_string()
}

pub fn blocks(h: &ReportHeader, results: &[PackageResult]) -> Vec<ManualBlock> {
    let now = chrono::Local::now();
    let mut b = Vec::new();
    let mut left = Vec::new();
    if !h.parent_org.trim().is_empty() {
        left.push(h.parent_org.trim().to_uppercase());
    }
    left.push(format!("*{}", if h.org.trim().is_empty() { "ĐƠN VỊ KIỂM TRA".to_string() } else { h.org.trim().to_uppercase() }));
    let place = if h.place.trim().is_empty() { String::new() } else { format!("{}, ", h.place.trim()) };
    b.push(ManualBlock::OfficialHeader {
        left,
        right: vec![s("*CỘNG HOÀ XÃ HỘI CHỦ NGHĨA VIỆT NAM"), s("*Độc lập - Tự do - Hạnh phúc")],
        dated: format!("{place}ngày {} tháng {} năm {}", now.format("%d"), now.format("%m"), now.format("%Y")),
    });
    b.push(ManualBlock::Gap(4.0));
    b.push(ManualBlock::Title(s("BÁO CÁO KIỂM TRA GÓI TIN HỒ SƠ LƯU TRỮ SỐ")));
    b.push(ManualBlock::Subtitle(s("theo Thông tư 05/2025/TT-BNV và đặc tả gói E-ARK CSIP")));
    b.push(ManualBlock::Gap(8.0));

    let dat = results.iter().filter(|r| r.errors == 0).count();
    let docs: u64 = results.iter().map(|r| r.pdfs.len() as u64).sum();
    let pages: u64 = results.iter().flat_map(|r| r.pdfs.iter()).map(|p| p.pages as u64).sum();
    b.push(ManualBlock::KeyValue(vec![
        (s("Ngày kiểm"), now.format("%d/%m/%Y %H:%M").to_string()),
        (s("Người kiểm"), if h.inspector.trim().is_empty() { s("—") } else { h.inspector.trim().to_string() }),
        (s("Số gói đã kiểm"), format!("{} gói · {} tệp PDF · {} trang", results.len(), n(docs), n(pages))),
        (s("Kết quả chung"), format!("{} gói đạt, {} gói không đạt", dat, results.len() - dat)),
        (
            s("Phạm vi kiểm"),
            s("Cấu trúc gói và METS (tham chiếu, kích thước, mã băm SHA-256, tệp mồ côi); EAD hồ sơ và văn bản \
               (trường bắt buộc, bảng mã TT05, đối chiếu schema đi kèm gói); PREMIS (fixity); từng tệp PDF \
               (khai PDF/A, hồ sơ màu, lớp chữ, độ phân giải, chữ ký số: toàn vẹn, người ký, dấu thời gian, LTV)."),
        ),
    ]));
    b.push(ManualBlock::Paragraph(s(
        "Ghi chú: kiểm PDF/A ở đây dựa trên dấu hiệu khai báo trong tệp (pdfaid, OutputIntent); kết luận tuân thủ \
         ISO 19005 đầy đủ cần chạy thêm veraPDF. Đối chiếu schema là rút gọn (đủ trường, không trường lạ).",
    )));

    b.push(ManualBlock::SectionBar(s("1. TỔNG HỢP")));
    b.push(ManualBlock::Table {
        headers: ["#", "Gói", "Loại", "Hồ sơ", "Tài liệu / trang", "Lỗi", "Cảnh báo", "Kết luận"].iter().map(|x| s(x)).collect(),
        widths: vec![4.0, 25.0, 6.0, 25.0, 11.0, 6.0, 8.0, 15.0],
        rows: results
            .iter()
            .enumerate()
            .map(|(i, r)| {
                vec![
                    (i + 1).to_string(),
                    r.name.clone(),
                    r.kind.clone(),
                    if r.file_code.is_empty() { r.title.clone() } else { format!("{} — {}", r.file_code, r.title) },
                    format!("{} / {}", r.pdfs.len(), r.pdfs.iter().map(|p| p.pages as u64).sum::<u64>()),
                    r.errors.to_string(),
                    r.warnings.to_string(),
                    r.verdict.clone(),
                ]
            })
            .collect(),
    });

    for (i, r) in results.iter().enumerate() {
        b.push(ManualBlock::SectionBar(format!("{}. GÓI {} — {}", i + 2, r.name, r.verdict)));
        let mut kv = vec![
            (s("Tệp / thư mục"), r.source.clone()),
            (s("Loại gói"), r.kind.clone()),
            (s("OBJID"), if r.objid.is_empty() { s("—") } else { r.objid.clone() }),
            (s("Hồ sơ"), if r.file_code.is_empty() { r.title.clone() } else { format!("{} — {}", r.file_code, r.title) }),
            (
                s("Tài liệu"),
                format!(
                    "{} tệp dữ liệu · totalDoc {} · numberOfPage {} · {}",
                    r.data_files,
                    r.total_doc.map(|x| x.to_string()).unwrap_or_else(|| s("—")),
                    r.pages.map(|x| x.to_string()).unwrap_or_else(|| s("—")),
                    mb(r.size)
                ),
            ),
            (s("Đơn vị / phần mềm tạo"), format!("{} · {}", if r.creator_org.is_empty() { "—" } else { &r.creator_org }, if r.creator_software.is_empty() { "—" } else { &r.creator_software })),
        ];
        if !r.created.is_empty() {
            kv.push((s("Ngày tạo gói"), r.created.clone()));
        }
        if !r.profile.is_empty() {
            kv.push((s("PROFILE"), r.profile.clone()));
        }
        if let Some(h) = &r.sha256 {
            kv.push((s("SHA-256 tệp gói"), h.clone()));
        }
        b.push(ManualBlock::KeyValue(kv));

        for (lvl, title) in [(Level::Err, "Lỗi phải sửa"), (Level::Warn, "Cảnh báo (nên sửa)"), (Level::Ok, "Những mục đạt")] {
            let rows: Vec<Vec<String>> = r
                .findings
                .iter()
                .filter(|f| f.level == lvl)
                .enumerate()
                .map(|(k, f)| vec![(k + 1).to_string(), f.group.clone(), f.message.clone()])
                .collect();
            if rows.is_empty() {
                continue;
            }
            b.push(ManualBlock::Paragraph(format!("{title} ({}):", rows.len())));
            b.push(ManualBlock::Table { headers: vec![s("#"), s("Nhóm"), s("Nội dung")], widths: vec![4.0, 13.0, 83.0], rows });
        }

        if !r.pdfs.is_empty() {
            b.push(ManualBlock::Paragraph(format!("Chi tiết {} tệp PDF:", r.pdfs.len())));
            b.push(ManualBlock::Table {
                headers: ["#", "Tệp", "Trang / lớp chữ", "PDF/A", "Dpi", "Chữ ký số", "Kết luận"].iter().map(|x| s(x)).collect(),
                widths: vec![4.0, 22.0, 8.0, 12.0, 5.0, 27.0, 22.0],
                rows: r
                    .pdfs
                    .iter()
                    .enumerate()
                    .map(|(k, p)| {
                        let pdfa = match (&p.pdfa, p.output_intent) {
                            (Some(d), true) => d.clone(),
                            (Some(d), false) => format!("{d}, thiếu hồ sơ màu"),
                            (None, _) => format!("Không (PDF {})", p.pdf_version),
                        };
                        let ky = if !p.signed {
                            s("Chưa ký")
                        } else {
                            let mut v = vec![match p.sig_valid {
                                Some(true) => s("Toàn vẹn"),
                                _ => s("KHÔNG toàn vẹn"),
                            }];
                            if let Some(x) = &p.signer {
                                v.push(x.clone());
                            }
                            if let Some(x) = &p.signing_time {
                                v.push(format!("ký {x}"));
                            }
                            v.push(format!("TSA: {} · LTV: {}", if p.tsa { "có" } else { "không" }, if p.ltv { "có" } else { "không" }));
                            if let Some(x) = &p.cert_until {
                                v.push(format!("CT hết hạn {x}"));
                            }
                            v.join("\n")
                        };
                        let kl = if p.problems.is_empty() {
                            p.verdict.clone()
                        } else {
                            format!("{}\n{}", p.verdict, p.problems.iter().map(|x| short(x)).collect::<Vec<_>>().join(" · "))
                        };
                        vec![
                            (k + 1).to_string(),
                            p.file.clone(),
                            format!("{} / {}", p.pages, p.text_pages),
                            pdfa,
                            p.min_dpi.map(|d| d.to_string()).unwrap_or_else(|| s("—")),
                            ky,
                            kl,
                        ]
                    })
                    .collect(),
            });
        }
    }

    b.push(ManualBlock::Gap(10.0));
    b.push(ManualBlock::Signatures {
        left: (s("NGƯỜI KIỂM TRA"), if h.inspector_title.is_empty() { s("(Ký, ghi rõ họ tên)") } else { h.inspector_title.clone() }, h.inspector.clone()),
        right: (s("NGƯỜI DUYỆT"), if h.approver_title.is_empty() { s("(Ký, ghi rõ họ tên)") } else { h.approver_title.clone() }, h.approver.clone()),
    });
    b.push(ManualBlock::Footer(format!("Báo cáo sinh tự động bởi DigiText · {} · TT 05/2025/TT-BNV — E-ARK CSIP", now.format("%d/%m/%Y %H:%M"))));
    b
}

pub fn render(h: &ReportHeader, results: &[PackageResult], font: &[u8]) -> Result<Vec<u8>, String> {
    render_manual(&blocks(h, results), font).map_err(|e| e.to_string())
}
