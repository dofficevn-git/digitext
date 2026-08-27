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
        "dsSpbmggdmnhu4Vu" | "vinh vien" => return "01".into(),
        "NzAgbsSDbQ==" => return "02".into(),
        "NTAgbsSDbQ==" => return "03".into(),
        "MzAgbsSDbQ==" => return "04".into(),
        "MjAgbsSDbQ==" => return "05".into(),
        "MTAgbsSDbQ==" => return "06".into(),
        _ => {}
    }
    notes
        .push(
            format!(
                "dGjhu51pIGjhuqFuIGLhuqNvIHF14bqjbiA={v:?}IGtow7RuZyBraOG7m3AgYuG6o25nIG3DoyBUVCAwNSDihpIgMDcgKEtow6FjKQ=="
            ),
        );
    "07".into()
}
fn f_1qw(v: Option<&str>, notes: &mut Vec<String>) -> String {
    let Some(v) = v else { return "01".into() };
    let r_41x = f_1v2(v);
    match r_41x.as_str() {
        "01" | "02" | "03" => return r_41x,
        "Y8O0bmcga2hhaQ==" => return "01".into(),
        "c+G7rSBk4bulbmcgY8OzIMSRaeG7gXUga2nhu4du"
        | "Y8OzIMSRaeG7gXUga2nhu4du"
        | "aOG6oW4gY2jhur8=" => {
            return "02".into();
        }
        "beG6rXQ=" => return "03".into(),
        _ => {}
    }
    notes
        .push(
            format!(
                "Y2jhur8gxJHhu5kgc+G7rSBk4bulbmcg{v:?}IGtow7RuZyBraOG7m3AgYuG6o25nIG3DoyDihpIgMDEgKEPDtG5nIGtoYWkp"
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
                "dGnhur9uZyB2aeG7h3Q=" | "dmnhu4d0" => "01".into(),
                "dGnhur9uZyBhbmg=" | "anh" => "02".into(),
                "dGnhur9uZyBwaMOhcA==" | "cGjDoXA=" => "03".into(),
                "dGnhur9uZyBuZ2E=" | "nga" => "04".into(),
                "dGnhur9uZyB0cnVuZw==" | "dGnhur9uZyB0cnVuZyBxdeG7kWM=" | "trung" => {
                    "05".into()
                }
                "dmnhu4d0IGFuaA==" => "06".into(),
                "dmnhu4d0IG5nYQ==" => "07".into(),
                "dmnhu4d0IHBow6Fw" => "08".into(),
                "aMOhbiBuw7Rt" => "09".into(),
                "dmnhu4d0IHRydW5n" => "10".into(),
                _ => {
                    notes
                        .push(
                            format!(
                                "bmfDtG4gbmfhu68g{v:?}IGtow7RuZyBraOG7m3AgYuG6o25nIG3DoyDihpIgMTEgKEtow6FjKQ=="
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
        ("bmdo4buLIHF1eeG6v3Q=", "01"),
        ("cXV54bq/dCDEkeG7i25o", "02"),
        ("Y2jhu4kgdGjhu4s=", "03"),
        ("cXV5IGNo4bq/", "04"),
        ("cXV5IMSR4buLbmg=", "05"),
        ("dGjDtG5nIGPDoW8=", "06"),
        ("dGjDtG5nIGLDoW8=", "07"),
        ("aMaw4bubbmcgZOG6q24=", "08"),
        ("Y2jGsMahbmcgdHLDrG5o", "09"),
        ("a+G6vyBob+G6oWNo", "10"),
        ("cGjGsMahbmcgw6Fu", "11"),
        ("xJHhu4Egw6Fu", "12"),
        ("ZOG7sSDDoW4=", "13"),
        ("YsOhbyBjw6Fv", "14"),
        ("dOG7nSB0csOsbmg=", "15"),
        ("Z2nhuqV5IOG7p3kgcXV54buBbg==", "16"),
        ("cGhp4bq/dSBn4butaQ==", "17"),
        ("cGhp4bq/dSBjaHV54buDbg==", "18"),
        ("cGhp4bq/dSBiw6Fv", "19"),
        ("YmnDqm4gYuG6o24=", "20"),
        ("aOG7o3AgxJHhu5NuZw==", "21"),
        ("Y8O0bmcgdsSDbg==", "22"),
        ("Y8O0bmcgxJFp4buHbg==", "23"),
        ("YuG6o24gZ2hpIG5o4bub", "24"),
        ("YuG6o24gdGjhu49hIHRodeG6rW4=", "25"),
        ("Z2nhuqV5IG3hu51p", "26"),
        ("Z2nhuqV5IGdp4bubaSB0aGnhu4d1", "27"),
        ("Z2nhuqV5IG5naOG7iSBwaMOpcA==", "28"),
        ("dGjGsCBjw7RuZw==", "29"),
        ("YuG6o24gxJHhu5M=", "30"),
        ("YuG6o24gduG6vSBr4bu5IHRodeG6rXQ=", "31"),
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
                "dMOqbiBsb+G6oWkgdsSDbiBi4bqjbiA={v:?}IGtow7RuZyBraOG7m3AgYuG6o25nIG3DoyDihpIgMzIgKEtow6FjKQ=="
            ),
        );
    "32".into()
}
fn f_1qx(v: Option<&str>, notes: &mut Vec<String>) -> Option<String> {
    let v = v?;
    let r_424 = f_1v2(v);
    match r_424.as_str() {
        "01" | "02" | "03" => Some(r_424),
        "YsOsbmggdGjGsOG7nW5n" => Some("01".into()),
        "aMawIGjhu49uZyBuaOG6uQ==" => Some("02".into()),
        "aMawIGjhu49uZyBu4bq3bmc=" | "aMawIGjhu49uZw==" => Some("03".into()),
        _ => {
            notes
                .push(
                    format!(
                        "dMOsbmggdHLhuqFuZyB24bqtdCBsw70g{v:?}IGtow7RuZyBraOG7m3AgYuG6o25nIG3DoyDihpIgYuG7jyB0cuG7kW5n"
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
            "kind {:?}IOKAlCBjaOG7iSBo4buXIHRy4bujIGFpcF9ob3NvIHwgc2lwX2hvc28=",
            opts.kind
        ));
    }
    if hoso.ma_ho_so.trim().is_empty() {
        report.errors.push("dGhp4bq/dSBtw6MgaOG7kyBzxqE=".into());
    }
    if hoso.tieu_de.trim().is_empty() {
        report
            .errors
            .push("dGhp4bq/dSB0acOqdSDEkeG7gSBo4buTIHPGoQ==".into());
    }
    if hoso.van_ban.is_empty() {
        report
            .errors
            .push("aOG7kyBzxqEga2jDtG5nIGPDsyB2xINuIGLhuqNuIG7DoG8=".into());
    }
    if opts.kind == "sip_hoso" && opts.submission_agreement.is_none() {
        report.errors.push(
            "Z8OzaSBTSVAgYuG6r3QgYnXhu5ljIG3DoyB0aG/huqMgdGh14bqtbiBu4buZcCAoLS1hZ3JlZW1lbnQp"
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
                    "bcOjIGjhu5Mgc8ahIGNoxrBhIG1hbmcgbcOjIGPGoSBxdWFuIGzGsHUgdHLhu68g4oaSIHRow6ptIHRp4buBbiB04buRIA=={:?}",
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
                    "buG7mXAgbMawdSAoU0lQKSBjaOG7iSBuaOG6rW4gdGjhu51pIGjhuqFuIGLhuqNvIHF14bqjbiB2xKluaCB2aeG7hW4g4oCUIGjhu5Mgc8ahIMSRYW5nIGzDoCA={:?}",
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
                        "dOG7lW5nIHPhu5EgdsSDbiBi4bqjbiBraGFpICg={t}KSDiiaAgc+G7kSB2xINuIGLhuqNuIGvDqG0gdGhlbyAo{}KSDigJQgbOG6pXkgdGhlbyBz4buRIHRo4buxYw==",
                        hoso.van_ban.len()
                    ),
                );
        }
    }
    if hoso.so_luong_to.is_none() {
        report.errors.push(
            "aOG7kyBzxqEgc+G7kSBow7NhIGLhuq90IGJ14buZYyBz4buRIGzGsOG7o25nIHThu50gKHNvTHVvbmdUbyk="
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
                        "dsSDbiBi4bqjbiBTVFQg{}OiB0aGnhur91IGRvY3VtZW50SWQgdHLDqm4gc2VydmVyIOKAlCBu4buZcCBQREYgdHLGsOG7m2MgcuG7k2kgxJFp4buBbiBtYXA=",
                        r_42b.stt
                    ),
                );
        }
        let ext = r_42b
            .tep_tin
            .as_deref()
            .and_then(|r_426| r_426.rsplit_once('.').map(|(_, e)| format!(".{e}")))
            .unwrap_or_else(|| ".pdf".into());
        let doc_code = format!("{file_code}.{}", f_1wk(r_42b.stt));
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
        documents: r_425,
    };
    for e in request.shape_errors() {
        if !e.contains("documentId") {
            report.errors.push(e);
        }
    }
    Tor { request, report }
}
