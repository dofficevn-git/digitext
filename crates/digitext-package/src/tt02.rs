// Copyright (c) 2026 DigiText. All rights reserved.
use crate::ead::{DocEad, T4y, T9a};
use crate::wire::{Tq3, Tq4, WireMeta};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tos {
    pub ma_ho_so: String,
    pub tieu_de: String,
    pub thoi_han_bao_quan: String,
    #[serde(default)]
    pub che_do_su_dung: Option<String>,
    #[serde(default)]
    pub ngon_ngu: Vec<String>,
    #[serde(default)]
    pub thoi_gian_bat_dau: Option<String>,
    #[serde(default)]
    pub thoi_gian_ket_thuc: Option<String>,
    #[serde(default)]
    pub tong_so_van_ban: Option<u32>,
    #[serde(default)]
    pub chu_giai: Option<String>,
    #[serde(default)]
    pub ky_hieu_thong_tin: Option<String>,
    #[serde(default)]
    pub tu_khoa: Option<String>,
    #[serde(default)]
    pub so_luong_to: Option<u32>,
    #[serde(default)]
    pub so_luong_trang: Option<u32>,
    #[serde(default)]
    pub tinh_trang_vat_ly: Option<String>,
    #[serde(default)]
    pub ma_ho_so_giay: Option<String>,
    pub van_ban: Vec<Tou>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tou {
    pub stt: u32,
    /// Mục lục số nằm giữa số hồ sơ và STT trong tên tệp (`…153.01.0000002`); có thì mã văn bản
    /// = `<mã hồ sơ>.<mục lục>.<STT>` theo quy tắc đặt tên.
    #[serde(default)]
    pub muc_luc: Option<String>,
    #[serde(default)]
    pub ten_loai: Option<String>,
    #[serde(default)]
    pub so_van_ban: Option<String>,
    #[serde(default)]
    pub ky_hieu: Option<String>,
    #[serde(default)]
    pub ngay_thang: Option<String>,
    #[serde(default)]
    pub co_quan_ban_hanh: Option<String>,
    #[serde(default)]
    pub trich_yeu: Option<String>,
    #[serde(default)]
    pub ngon_ngu: Vec<String>,
    #[serde(default)]
    pub so_trang: Option<u32>,
    #[serde(default)]
    pub ghi_chu: Option<String>,
    #[serde(default)]
    pub ky_hieu_thong_tin: Option<String>,
    #[serde(default)]
    pub tu_khoa: Option<String>,
    #[serde(default)]
    pub che_do_su_dung: Option<String>,
    #[serde(default)]
    pub but_tich: Option<String>,
    #[serde(default)]
    pub tinh_trang_vat_ly: Option<String>,
    #[serde(default)]
    pub document_id: Option<uuid::Uuid>,
    #[serde(default)]
    pub tep_tin: Option<String>,
}
#[derive(Debug, Clone)]
pub struct Tot {
    pub fond_code: String,
    pub archive_org_code: String,
    pub kind: String,
    pub submission_agreement: Option<String>,
    pub fond_name: Option<String>,
    pub archive_org_name: Option<String>,
}
#[derive(Debug)]
pub struct Tor {
    pub request: Tq4,
    pub report: Tt02Report,
}
#[derive(Debug, Default)]
pub struct Tt02Report {
    pub notes: Vec<String>,
    pub errors: Vec<String>,
}
fn f_1v2(r_41v: &str) -> String {
    r_41v.trim().to_lowercase()
}
fn f_1qv(v: &str, notes: &mut Vec<String>) -> String {
    let r_41w = f_1v2(v);
    match r_41w.as_str() {
        "01" | "02" | "03" | "04" | "05" | "06" | "07" => return r_41w,
        "vĩnh viễn" | "vinh vien" => return "01".into(),
        "70 năm" => return "02".into(),
        "50 năm" => return "03".into(),
        "30 năm" => return "04".into(),
        "20 năm" => return "05".into(),
        "10 năm" => return "06".into(),
        _ => {}
    }
    notes
        .push(
            format!(
                "thời hạn bảo quản {v:?} không khớp bảng mã TT 05 → 07 (Khác)"
            ),
        );
    "07".into()
}
fn f_1qw(v: Option<&str>, notes: &mut Vec<String>) -> String {
    let Some(v) = v else { return "01".into() };
    let r_41x = f_1v2(v);
    match r_41x.as_str() {
        "01" | "02" | "03" => return r_41x,
        "công khai" => return "01".into(),
        "sử dụng có điều kiện"
        | "có điều kiện"
        | "hạn chế" => {
            return "02".into();
        }
        "mật" => return "03".into(),
        _ => {}
    }
    notes
        .push(
            format!(
                "chế độ sử dụng {v:?} không khớp bảng mã → 01 (Công khai)"
            ),
        );
    "01".into()
}
fn f_1qu(r_41z: &[String], notes: &mut Vec<String>) -> Vec<String> {
    if r_41z.is_empty() {
        return vec!["01".into()];
    }
    r_41z
        .iter()
        .map(|v| {
            let r_41y = f_1v2(v);
            match r_41y.as_str() {
                "01" | "02" | "03" | "04" | "05" | "06" | "07" | "08" | "09" | "10"
                | "11" => r_41y,
                "tiếng việt" | "việt" => "01".into(),
                "tiếng anh" | "anh" => "02".into(),
                "tiếng pháp" | "pháp" => "03".into(),
                "tiếng nga" | "nga" => "04".into(),
                "tiếng trung" | "tiếng trung quốc" | "trung" => {
                    "05".into()
                }
                "việt anh" => "06".into(),
                "việt nga" => "07".into(),
                "việt pháp" => "08".into(),
                "hán nôm" => "09".into(),
                "việt trung" => "10".into(),
                _ => {
                    notes
                        .push(
                            format!(
                                "ngôn ngữ {v:?} không khớp bảng mã → 11 (Khác)"
                            ),
                        );
                    "11".into()
                }
            }
        })
        .collect()
}
fn f_1r1(v: Option<&str>, notes: &mut Vec<String>) -> String {
    const TABLE: [(&str, &str); 31] = [
        ("nghị quyết", "01"),
        ("quyết định", "02"),
        ("chỉ thị", "03"),
        ("quy chế", "04"),
        ("quy định", "05"),
        ("thông cáo", "06"),
        ("thông báo", "07"),
        ("hướng dẫn", "08"),
        ("chương trình", "09"),
        ("kế hoạch", "10"),
        ("phương án", "11"),
        ("đề án", "12"),
        ("dự án", "13"),
        ("báo cáo", "14"),
        ("tờ trình", "15"),
        ("giấy ủy quyền", "16"),
        ("phiếu gửi", "17"),
        ("phiếu chuyển", "18"),
        ("phiếu báo", "19"),
        ("biên bản", "20"),
        ("hợp đồng", "21"),
        ("công văn", "22"),
        ("công điện", "23"),
        ("bản ghi nhớ", "24"),
        ("bản thỏa thuận", "25"),
        ("giấy mời", "26"),
        ("giấy giới thiệu", "27"),
        ("giấy nghỉ phép", "28"),
        ("thư công", "29"),
        ("bản đồ", "30"),
        ("bản vẽ kỹ thuật", "31"),
    ];
    let Some(v) = v else {
        return "32".into();
    };
    let r_422 = f_1v2(v);
    if r_422.len() <= 2
        && r_422
            .parse::<u8>()
            .is_ok_and(|r_420| (1..=32).contains(&r_420))
    {
        return format!("{:02}", r_422.parse::<u8>().unwrap());
    }
    for (r_423, r_421) in TABLE {
        if r_422 == r_423 {
            return r_421.into();
        }
    }
    notes
        .push(
            format!(
                "tên loại văn bản {v:?} không khớp bảng mã → 32 (Khác)"
            ),
        );
    "32".into()
}
fn f_1qx(v: Option<&str>, notes: &mut Vec<String>) -> Option<String> {
    let v = v?;
    let r_424 = f_1v2(v);
    match r_424.as_str() {
        "01" | "02" | "03" => Some(r_424),
        "bình thường" => Some("01".into()),
        "hư hỏng nhẹ" => Some("02".into()),
        "hư hỏng nặng" | "hư hỏng" => Some("03".into()),
        _ => {
            notes
                .push(
                    format!(
                        "tình trạng vật lý {v:?} không khớp bảng mã → bỏ trống"
                    ),
                );
            None
        }
    }
}
fn f_1wk(stt: u32) -> String {
    format!("{stt:07}")
}
pub fn f_11h(hoso: &Tos, opts: &Tot) -> Tor {
    let mut report = Tt02Report::default();
    if opts.kind != "aip_hoso" && opts.kind != "sip_hoso" {
        report.errors.push(format!(
            "kind {:?} — chỉ hỗ trợ aip_hoso | sip_hoso",
            opts.kind
        ));
    }
    if hoso.ma_ho_so.trim().is_empty() {
        report.errors.push("thiếu mã hồ sơ".into());
    }
    if hoso.tieu_de.trim().is_empty() {
        report
            .errors
            .push("thiếu tiêu đề hồ sơ".into());
    }
    if hoso.van_ban.is_empty() {
        report
            .errors
            .push("hồ sơ không có văn bản nào".into());
    }
    if opts.kind == "sip_hoso" && opts.submission_agreement.is_none() {
        report.errors.push(
            "gói SIP bắt buộc mã thoả thuận nộp (--agreement)"
                .into(),
        );
    }
    let file_code = if hoso
        .ma_ho_so
        .starts_with(&format!("{}.", opts.archive_org_code))
    {
        hoso.ma_ho_so.clone()
    } else {
        report
            .notes
            .push(
                format!(
                    "mã hồ sơ chưa mang mã cơ quan lưu trữ → thêm tiền tố {:?}",
                    opts.archive_org_code
                ),
            );
        format!("{}.{}", opts.archive_org_code, hoso.ma_ho_so)
    };
    let r_429 = f_1qv(&hoso.thoi_han_bao_quan, &mut report.notes);
    if opts.kind == "sip_hoso" && r_429 != "01" {
        report
            .errors
            .push(
                format!(
                    "nộp lưu (SIP) chỉ nhận thời hạn bảo quản vĩnh viễn — hồ sơ đang là {:?}",
                    hoso.thoi_han_bao_quan
                ),
            );
    }
    let r_42a = f_1qw(hoso.che_do_su_dung.as_deref(), &mut report.notes);
    let r_427 = f_1qu(&hoso.ngon_ngu, &mut report.notes);
    if let Some(t) = hoso.tong_so_van_ban {
        if t as usize != hoso.van_ban.len() {
            report
                .notes
                .push(
                    format!(
                        "tổng số văn bản khai ({t}) ≠ số văn bản kèm theo ({}) — lấy theo số thực",
                        hoso.van_ban.len()
                    ),
                );
        }
    }
    if hoso.so_luong_to.is_none() {
        report.errors.push(
            "hồ sơ số hóa bắt buộc số lượng tờ (soLuongTo)"
                .into(),
        );
    }
    let meta = T9a {
        file_code: file_code.clone(),
        title: hoso.tieu_de.trim().to_string(),
        maintenance: r_429.clone(),
        mode: r_42a.clone(),
        language: r_427.clone(),
        start_date: hoso.thoi_gian_bat_dau.clone(),
        end_date: hoso.thoi_gian_ket_thuc.clone(),
        keyword: hoso.tu_khoa.clone(),
        total_doc: hoso.van_ban.len() as u32,
        number_of_paper: hoso.so_luong_to,
        number_of_page: hoso.so_luong_trang,
        physical_format: f_1qx(hoso.tinh_trang_vat_ly.as_deref(), &mut report.notes),
        infor_sign: hoso.ky_hieu_thong_tin.clone(),
        confidence_level: crate::codes::C_2A.to_string(),
        paper_file_code: Some(
            hoso.ma_ho_so_giay
                .clone()
                .unwrap_or_else(|| hoso.ma_ho_so.clone()),
        ),
        risk_recovery: false,
        risk_recovery_status: None,
        description: hoso.chu_giai.clone(),
    };
    let mut r_425 = Vec::with_capacity(hoso.van_ban.len());
    for r_42b in &hoso.van_ban {
        if r_42b.document_id.is_none() {
            report
                .errors
                .push(
                    format!(
                        "văn bản STT {}: thiếu documentId trên server — nộp PDF trước rồi điền map",
                        r_42b.stt
                    ),
                );
        }
        let ext = r_42b
            .tep_tin
            .as_deref()
            .and_then(|r_426| r_426.rsplit_once('.').map(|(_, e)| format!(".{e}")))
            .unwrap_or_else(|| ".pdf".into());
        let doc_code = match r_42b.muc_luc.as_deref().map(str::trim).filter(|m| !m.is_empty()) {
            Some(ml) if !file_code.ends_with(&format!(".{ml}")) => format!("{file_code}.{ml}.{}", f_1wk(r_42b.stt)),
            _ => format!("{file_code}.{}", f_1wk(r_42b.stt)),
        };
        r_425.push(Tq3 {
            document_id: r_42b.document_id,
            file_name: format!("{doc_code}{ext}"),
            mimetype: None,
            metadata: DocEad::Doc(T4y {
                doc_id: None,
                doc_code,
                maintenance: r_429.clone(),
                type_name: f_1r1(r_42b.ten_loai.as_deref(), &mut report.notes),
                code_number: r_42b.so_van_ban.clone(),
                code_notation: r_42b.ky_hieu.clone(),
                issued_date: r_42b.ngay_thang.clone(),
                organ_name: r_42b.co_quan_ban_hanh.clone(),
                subject: r_42b.trich_yeu.clone(),
                language: if r_42b.ngon_ngu.is_empty() {
                    r_427.clone()
                } else {
                    f_1qu(&r_42b.ngon_ngu, &mut report.notes)
                },
                number_of_page: r_42b.so_trang,
                infor_sign: r_42b.ky_hieu_thong_tin.clone(),
                keyword: r_42b.tu_khoa.clone(),
                mode: match r_42b.che_do_su_dung.as_deref() {
                    Some(r_428) => f_1qw(Some(r_428), &mut report.notes),
                    None => r_42a.clone(),
                },
                confidence_level: Some(crate::codes::C_2A.to_string()),
                autograph: r_42b.but_tich.clone(),
                physical_format: f_1qx(r_42b.tinh_trang_vat_ly.as_deref(), &mut report.notes),
                process: None,
                risk_recovery: false,
                risk_recovery_status: None,
                description: r_42b.ghi_chu.clone(),
            }),
            fetch: None,
        });
    }
    let request = Tq4 {
        kind: opts.kind.clone(),
        fond_code: opts.fond_code.clone(),
        archive_org_code: opts.archive_org_code.clone(),
        label: None,
        meta: WireMeta::Hoso(meta),
        submission_agreement: opts.submission_agreement.clone(),
        dissemination_request_id: None,
        related_hoso_mets_id: None,
        include_premis: false,
        from_sip: false,
        zip_ordinal: None,
        content_information_type: None,
        dip_cert: None,
        fond_name: opts.fond_name.clone(),
        archive_org_name: opts.archive_org_name.clone(),
        packager_name: None,
        packager_code: None,
        allow_non_pdfa: false,
        documents: r_425,
    };
    for e in request.shape_errors() {
        if !e.contains("documentId") {
            report.errors.push(e);
        }
    }
    Tor { request, report }
}
