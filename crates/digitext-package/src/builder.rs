// Copyright (c) 2026 DigiText. All rights reserved.
use crate::ead;
use crate::fetch;
use crate::ids::{self, T9n};
use crate::mets::{self, Td6, Tir, T50, T84};
use crate::mime;
use crate::model::*;
use crate::premis::{self, The, Thf};
use crate::schemas;
use crate::PackageError;
use chrono::{DateTime, Datelike, FixedOffset, Timelike};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Write as _;
pub struct T1g {
    pub kind: PackageKind,
    pub objid: String,
    pub rep_objid: String,
    pub folder_name: String,
    pub zip_name: String,
    pub files: BTreeMap<String, Vec<u8>>,
    pub zip_bytes: Vec<u8>,
    pub zip_sha256: String,
}
fn f_2en(r_3uv: &[u8]) -> String {
    format!("{:X}", Sha256::digest(r_3uv))
}
fn sha256_lower(r_3uw: &[u8]) -> String {
    format!("{:x}", Sha256::digest(r_3uw))
}
fn f_1dg(r_3ux: &DateTime<FixedOffset>) -> String {
    r_3ux.format("%Y-%m-%dT%H:%M:%S%.3f%:z").to_string()
}
fn f_2pc(r_3v0: &Tga) -> Vec<String> {
    let mut e = Vec::new();
    if r_3v0.fond_code.is_empty() {
        e.push("Zm9uZF9jb2RlOiBraMO0bmcgxJHGsOG7o2MgcuG7l25n".into());
    }
    if r_3v0.archive_org_code.is_empty() {
        e.push("YXJjaGl2ZV9vcmdfY29kZToga2jDtG5nIMSRxrDhu6NjIHLhu5duZw==".into());
    }
    if r_3v0.documents.is_empty() {
        e.push(
            "ZG9jdW1lbnRzOiBnw7NpIHBo4bqjaSBjaOG7qWEgdOG7kWkgdGhp4buDdSAwMSB0w6BpIGxp4buHdQ=="
                .into(),
        );
    }
    if r_3v0.kind.single_document() && r_3v0.documents.len() > 1 {
        e.push(
            format!(
                "ZG9jdW1lbnRzOiBnw7NpIA=={}IGNo4buJIGNo4bupYSDEkcO6bmcgMDEgdMOgaSBsaeG7h3UgY2jDrW5oICjEkWFuZyBjw7Mg{})",
                r_3v0.kind.as_str(), r_3v0.documents.len()
            ),
        );
    }
    if r_3v0.kind.requires_submission_agreement() && r_3v0.submission_agreement.is_none() {
        e.push(
            "c3VibWlzc2lvbl9hZ3JlZW1lbnQ6IFNJUCBi4bqvdCBideG7mWMgbcOjIHnDqnUgY+G6p3UgbuG7mXAgKGFsdFJlY29yZElEKQ=="
                .into(),
        );
    }
    match (r_3v0.kind, &r_3v0.dip_cert) {
        (PackageKind::Dip, None) => {
            e.push(
                "ZGlwX2NlcnQ6IGfDs2kgRElQIGLhuq90IGJ14buZYyB04buHcCB2xINuIGLhuqNuIHjDoWMgdGjhu7FjICjEkGnhu4F1IDQyLjIp"
                    .into(),
            )
        }
        (PackageKind::Dip, Some(c)) => {
            for (field, r_3v2) in [
                ("maXacThuc", &c.ma_xac_thuc),
                ("tenLuuTruLichSu", &c.ten_luu_tru_lich_su),
                ("nguoiNhanHoTen", &c.nguoi_nhan_ho_ten),
                ("nguoiNhanDinhDanh", &c.nguoi_nhan_dinh_danh),
                ("maLuuTruGoc", &c.ma_luu_tru_goc),
                ("tenLoai", &c.ten_loai),
                ("trichYeu", &c.trich_yeu),
                ("mucDich", &c.muc_dich),
                ("ngayCap", &c.ngay_cap),
            ] {
                if r_3v2.trim().is_empty() {
                    e.push(format!("dip_cert.{field}OiBraMO0bmcgxJHGsOG7o2MgcuG7l25n"));
                }
            }
            if c.so_luong_ban == 0 {
                e.push("ZGlwX2NlcnQuc29MdW9uZ0JhbjogdOG7kWkgdGhp4buDdSAx".into());
            }
            if r_3v0.cert_font_ttf.is_none() {
                e.push(
                    "Y2VydF9mb250X3R0ZjogdGhp4bq/dSBmb250IFRURiDEkeG7gyBzaW5oIHbEg24gYuG6o24geMOhYyB0aOG7sWMgICAgICAgICAgICAgICAgICAgICAgKMSR4bq3dCBESUdJVEVYVF9DRVJUX0ZPTlQgaG/hurdjIGPDoGkgZm9udCBo4buHIHRo4buRbmcp"
                        .into(),
                );
            }
        }
        (_, Some(_)) => e.push("ZGlwX2NlcnQ6IGNo4buJIGTDoG5oIGNobyBnw7NpIERJUA==".into()),
        _ => {}
    }
    match (&r_3v0.meta, r_3v0.kind) {
        (PackageMeta::Hoso(h), PackageKind::AipHoso | PackageKind::SipHoso) => {
            e.extend(ead::f_2p2(h, r_3v0.kind));
        }
        (PackageMeta::GoiTin(g), PackageKind::AipTailieu | PackageKind::SipTailieu) => {
            e.extend(ead::f_2p1(g, r_3v0.kind));
        }
        (PackageMeta::Dip(d), PackageKind::Dip) => {
            e.extend(ead::f_2ox(d));
        }
        _ => e.push(format!(
            "bWV0YTogYmnhur9uIHRo4buDIG1ldGFkYXRhIGtow7RuZyBraOG7m3AgbG/huqFpIGfDs2kg{}",
            r_3v0.kind.as_str()
        )),
    }
    let mut r_3v1 = std::collections::BTreeSet::new();
    for r_3uy in &r_3v0.documents {
        if r_3uy.file_name.is_empty() || r_3uy.file_name.contains('/') {
            e.push(format!(
                "ZmlsZV9uYW1lIGtow7RuZyBo4bujcCBs4buHOiAn{}'",
                r_3uy.file_name
            ));
        }
        if !r_3v1.insert(r_3uy.file_name.clone()) {
            e.push(format!(
                "ZmlsZV9uYW1lIHRyw7luZyBs4bq3cDogJw=={}'",
                r_3uy.file_name
            ));
        }
        for a in &r_3uy.attachments {
            if a.file_name.is_empty() || a.file_name.contains('/') {
                e.push(format!(
                    "YXR0YWNobWVudCBraMO0bmcgaOG7o3AgbOG7hzogJw=={}'",
                    a.file_name
                ));
            }
            if !r_3v1.insert(a.file_name.clone()) {
                e.push(format!(
                    "ZmlsZV9uYW1lIHRyw7luZyBs4bq3cDogJw=={}'",
                    a.file_name
                ));
            }
        }
        let r_3uz = matches!(r_3uy.content, DocumentContent::Fetch(_));
        if r_3uz && !r_3v0.kind.allows_fetch() {
            e.push(
                format!(
                    "dMOgaSBsaeG7h3UgbGnDqm4ga+G6v3QgLmZldGNoLnR4dCBjaOG7iSBkw7luZyBjaG8gZ8OzaSBt4bupYyBo4buTIHPGoSAoa2luZD0={})",
                    r_3v0.kind.as_str()
                ),
            );
        }
        if r_3uz && !fetch::f_1j0(&r_3uy.file_name) {
            e.push(format!(
                "dMOgaSBsaeG7h3UgbGnDqm4ga+G6v3QgcGjhuqNpIGPDsyB0w6puICcqLmZldGNoLnR4dCc6ICc={}'",
                r_3uy.file_name
            ));
        }
        if !r_3uz && fetch::f_1j0(&r_3uy.file_name) {
            e.push(
                format!(
                    "dMOqbiAnKi5mZXRjaC50eHQnIGTDoG5oIHJpw6puZyBjaG8gdMOgaSBsaeG7h3UgbGnDqm4ga+G6v3Q6ICc={}'",
                    r_3uy.file_name
                ),
            );
        }
        e.extend(ead::f_2oy(&r_3uy.metadata, r_3v0.kind));
    }
    e
}
fn f_2sa(r_3v3: &DateTime<FixedOffset>) -> zip::DateTime {
    zip::DateTime::from_date_and_time(
        r_3v3.year() as u16,
        r_3v3.month() as u8,
        r_3v3.day() as u8,
        r_3v3.hour() as u8,
        r_3v3.minute() as u8,
        r_3v3.second() as u8,
    )
    .unwrap_or_default()
}
fn f_w6(
    folder: &str,
    files: &BTreeMap<String, Vec<u8>>,
    r_3v6: &DateTime<FixedOffset>,
) -> Result<Vec<u8>, PackageError> {
    let mut r_3v8 = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    let r_3v7 = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .last_modified_time(f_2sa(r_3v6));
    for (path, r_3v4) in files {
        r_3v8
            .start_file(format!("{folder}/{path}"), r_3v7)
            .map_err(|err| PackageError::Zip(err.to_string()))?;
        r_3v8
            .write_all(r_3v4)
            .map_err(|err| PackageError::Zip(err.to_string()))?;
    }
    let r_3v5 = r_3v8
        .finish()
        .map_err(|err| PackageError::Zip(err.to_string()))?;
    Ok(r_3v5.into_inner())
}
pub fn f_vn(r_3wb: &Tga, r_3vx: &mut dyn T9n) -> Result<T1g, PackageError> {
    let r_3vz = f_2pc(r_3wb);
    if !r_3vz.is_empty() {
        return Err(PackageError::InvalidRequest(r_3vz));
    }
    let r_3vj = f_1dg(&r_3wb.created_at);
    let r_3vh = r_3wb
        .content_information_type
        .clone()
        .unwrap_or_else(|| "MIXED".to_string());
    let r_3we =
        r_3wb.kind.premis_required() || (r_3wb.kind.premis_allowed() && r_3wb.include_premis);
    let objid = ids::f_1x2(r_3wb.kind, &r_3wb.fond_code, r_3vx);
    let rep_objid = ids::f_2op(r_3vx);
    let folder = ids::folder_name(&objid);
    let label = r_3wb.label.clone().unwrap_or_else(|| match &r_3wb.meta {
        PackageMeta::Hoso(h) => h.title.clone(),
        PackageMeta::GoiTin(g) => g.title.clone(),
        PackageMeta::Dip(d) => format!("DIP {}", d.request_id),
    });
    let mut files: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut r_3vn: Vec<(Td6, T50)> = Vec::new();
    let mut r_3vk: Vec<T84> = Vec::new();
    let mut r_3vw: Vec<T84> = Vec::new();
    let mut r_3w6: Vec<Thf> = Vec::new();
    for r_3vm in &r_3wb.documents {
        let r_3vy = matches!(r_3vm.content, DocumentContent::Fetch(_));
        let r_3vl = format!("representations/rep1/data/{}", r_3vm.file_name);
        let bytes: Vec<u8> = match &r_3vm.content {
            DocumentContent::Bytes(b) => b.clone(),
            DocumentContent::Fetch(r_3w7) => {
                fetch::f_1c1(&r_3wb.fond_code, r_3w7, &r_3vl).into_bytes()
            }
        };
        let r_3w0 = r_3vm
            .mimetype
            .clone()
            .unwrap_or_else(|| mime::f_1ds(&r_3vm.file_name).to_string());
        let r_3vu = ids::file_id(r_3vx);
        let r_3vv = T84 {
            id: r_3vu.clone(),
            href: format!("data/{}", r_3vm.file_name),
            mimetype: r_3w0.clone(),
            size: bytes.len() as u64,
            checksum_upper: f_2en(&bytes),
        };
        if r_3we {
            r_3w6.push(Thf {
                object_type: if r_3vy {
                    "File"
                } else {
                    r_3vm.metadata.premis_object_type()
                },
                id_value: ids::f_2op(r_3vx),
                original_name: r_3vm.file_name.clone(),
                sha256_lower: sha256_lower(&bytes),
                size: bytes.len() as u64,
                format_name: r_3w0.clone(),
            });
        }
        files.insert(r_3vl, bytes);
        if r_3vy {
            r_3vw.push(r_3vv);
        } else {
            r_3vk.push(r_3vv);
        }
        let mut r_3ve = Vec::new();
        for r_3va in &r_3vm.attachments {
            let r_3vb = r_3va.bytes.clone();
            let r_3vd = r_3va
                .mimetype
                .clone()
                .unwrap_or_else(|| mime::f_1ds(&r_3va.file_name).to_string());
            let r_3vc = ids::file_id(r_3vx);
            r_3vk.push(T84 {
                id: r_3vc.clone(),
                href: format!("data/{}", r_3va.file_name),
                mimetype: r_3vd.clone(),
                size: r_3vb.len() as u64,
                checksum_upper: f_2en(&r_3vb),
            });
            if r_3we {
                r_3w6.push(Thf {
                    object_type: "File",
                    id_value: ids::f_2op(r_3vx),
                    original_name: r_3va.file_name.clone(),
                    sha256_lower: sha256_lower(&r_3vb),
                    size: r_3vb.len() as u64,
                    format_name: r_3vd,
                });
            }
            files.insert(
                format!("representations/rep1/data/{}", r_3va.file_name),
                r_3vb,
            );
            r_3ve.push(r_3vc);
        }
        let ead_name = format!(
            "EAD_{}_{}.xml",
            r_3vm.metadata.class(),
            fetch::f_1sm(&r_3vm.file_name)
        );
        let r_3vp = ead::f_18x(&r_3vm.metadata, r_3wb.kind.doc_code_tag()).into_bytes();
        let md = Td6 {
            id: ids::f_2op(r_3vx),
            href: format!("metadata/descriptive/{ead_name}"),
            size: r_3vp.len() as u64,
            checksum_upper: f_2en(&r_3vp),
        };
        files.insert(
            format!("representations/rep1/metadata/descriptive/{ead_name}"),
            r_3vp,
        );
        r_3vn.push((
            md,
            T50 {
                file_id: r_3vu,
                holey: r_3vy,
                attachment_file_ids: r_3ve,
            },
        ));
    }
    let r_3vs = r_3wb.source_event.premis_event_type();
    let r_3w9 = if r_3we {
        let ctx = The {
            event_type: r_3vs,
            event_id: r_3vx.uuid(),
            event_datetime: r_3vj.clone(),
            software: &r_3wb.software,
            archive_org_code: &r_3wb.archive_org_code,
        };
        let xml = premis::f_27u(&r_3w6, &ctx).into_bytes();
        let md = Td6 {
            id: ids::f_2op(r_3vx),
            href: "metadata/preservation/PREMIS_rep1.xml".into(),
            size: xml.len() as u64,
            checksum_upper: f_2en(&xml),
        };
        files.insert(
            "representations/rep1/metadata/preservation/PREMIS_rep1.xml".into(),
            xml,
        );
        Some(md)
    } else {
        None
    };
    let r_3v9 = match r_3wb.kind {
        PackageKind::AipHoso | PackageKind::AipTailieu => {
            vec![
                Tir {
                    role: "CREATOR",
                    code: r_3wb.fond_code.clone(),
                },
                Tir {
                    role: "ARCHIVIST",
                    code: r_3wb.archive_org_code.clone(),
                },
            ]
        }
        PackageKind::SipHoso | PackageKind::SipTailieu => {
            vec![Tir {
                role: "CREATOR",
                code: r_3wb.fond_code.clone(),
            }]
        }
        PackageKind::Dip => {
            vec![
                Tir {
                    role: "ARCHIVIST",
                    code: r_3wb.archive_org_code.clone(),
                },
                Tir {
                    role: "DISSEMINATOR",
                    code: r_3wb
                        .dissemination_request_id
                        .clone()
                        .or_else(|| match &r_3wb.meta {
                            PackageMeta::Dip(d) => Some(d.request_id.clone()),
                            _ => None,
                        })
                        .unwrap_or_default(),
                },
            ]
        }
    };
    let r_3wa = mets::f_27s(
        &mets::Tis {
            kind: r_3wb.kind,
            rep_objid: &rep_objid,
            label: &label,
            content_information_type: &r_3vh,
            created: &r_3vj,
            agents: r_3v9,
            doc_refs: r_3vn,
            premis_ref: r_3w9,
            data_files: r_3vk,
            holey_files: r_3vw,
        },
        r_3vx,
    )
    .into_bytes();
    let r_3w8 = T84 {
        id: ids::file_id(r_3vx),
        href: "representations/rep1/METS.xml".into(),
        mimetype: mime::XML.to_string(),
        size: r_3wa.len() as u64,
        checksum_upper: f_2en(&r_3wa),
    };
    files.insert("representations/rep1/METS.xml".into(), r_3wa);
    let r_3w3 = match &r_3wb.meta {
        PackageMeta::Hoso(h) => ead::f_190(h, r_3wb.kind.hoso_code_tag()),
        PackageMeta::GoiTin(g) => ead::f_18z(g, r_3wb.kind),
        PackageMeta::Dip(d) => ead::f_18w(d),
    }
    .into_bytes();
    let ead_name = r_3wb.kind.package_ead_name();
    let r_3vq = Td6 {
        id: ids::f_2op(r_3vx),
        href: format!("metadata/descriptive/{ead_name}"),
        size: r_3w3.len() as u64,
        checksum_upper: f_2en(&r_3w3),
    };
    files.insert(format!("metadata/descriptive/{ead_name}"), r_3w3);
    let r_3w4 = if r_3we {
        let ctx = The {
            event_type: r_3vs,
            event_id: r_3vx.uuid(),
            event_datetime: r_3vj.clone(),
            software: &r_3wb.software,
            archive_org_code: &r_3wb.archive_org_code,
        };
        let xml = premis::f_1x3(&objid, &ids::f_2op(r_3vx), &ctx).into_bytes();
        let md = Td6 {
            id: ids::f_2op(r_3vx),
            href: "metadata/preservation/PREMIS.xml".into(),
            size: xml.len() as u64,
            checksum_upper: f_2en(&xml),
        };
        files.insert("metadata/preservation/PREMIS.xml".into(), xml);
        Some(md)
    } else {
        None
    };
    let mut r_3wc = Vec::new();
    for (name, r_3vi) in schemas::schema_files(r_3wb.kind, r_3we) {
        let bytes = r_3vi.into_bytes();
        r_3wc.push(T84 {
            id: ids::file_id(r_3vx),
            href: format!("schemas/{name}"),
            mimetype: mime::C_EJ.to_string(),
            size: bytes.len() as u64,
            checksum_upper: f_2en(&bytes),
        });
        files.insert(format!("schemas/{name}"), bytes);
    }
    let mut r_3vo = Vec::new();
    if let (Some(r_3vf), Some(font)) = (&r_3wb.dip_cert, &r_3wb.cert_font_ttf) {
        let r_3w1 = format!(
            "{} — {}",
            r_3vf.nguoi_nhan_ho_ten, r_3vf.nguoi_nhan_dinh_danh
        );
        let r_3vt = vec![
            (
                "YSkgTcOjIHjDoWMgdGjhu7FjIGzGsHUgdHLhu68=",
                r_3vf.ma_xac_thuc.clone(),
            ),
            (
                "YikgVMOqbiBsxrB1IHRy4buvIGzhu4tjaCBz4but",
                r_3vf.ten_luu_tru_lich_su.clone(),
            ),
            ("YykgTmfGsOG7nWkgbmjhuq1u", r_3w1),
            (
                "ZCkgTcOjIGzGsHUgdHLhu68gY+G7p2EgdMOgaSBsaeG7h3UgZ+G7kWM=",
                r_3vf.ma_luu_tru_goc.clone(),
            ),
            (
                "xJEpIFPhu5EgdsOgIGvDvSBoaeG7h3UgY+G7p2EgdMOgaSBsaeG7h3UgZ+G7kWM=",
                r_3vf.so_ky_hieu_goc.clone().unwrap_or_else(|| "—".into()),
            ),
            (
                "ZSkgVMOqbiBsb+G6oWkgdMOgaSBsaeG7h3U=",
                r_3vf.ten_loai.clone(),
            ),
            (
                "ZykgVHLDrWNoIHnhur91IG7hu5lpIGR1bmcgLyB0acOqdSDEkeG7gQ==",
                r_3vf.trich_yeu.clone(),
            ),
            (
                "aCkgTeG7pWMgxJHDrWNoIHPhu60gZOG7pW5n",
                r_3vf.muc_dich.clone(),
            ),
            ("aSkgTmfDoHkgY+G6pXA=", r_3vf.ngay_cap.clone()),
            (
                "aykgVGjhu51pIGjhuqFuIHPhu60gZOG7pW5n",
                r_3vf.thoi_han_su_dung.clone().unwrap_or_else(|| "—".into()),
            ),
            (
                "bCkgU+G7kSBsxrDhu6NuZyBi4bqjbg==",
                r_3vf.so_luong_ban.to_string(),
            ),
        ];
        let r_3vg: Vec<digitext_pdf::T2m> = r_3vt
            .into_iter()
            .map(|(label, r_3wd)| digitext_pdf::T2m {
                label: label.to_string(),
                value: r_3wd,
            })
            .collect();
        let pdf = digitext_pdf::f_v8(
            "VsSCTiBC4bqiTiBYw4FDIFRI4buwQyBC4bqiTiBTQU8gVMOASSBMSeG7hlUgTMavVSBUUuG7riBT4buQ",
            &r_3vf.ten_luu_tru_lich_su.to_uppercase(),
            &r_3vg,
            font,
        )
        .map_err(|r_3vr| PackageError::Cert(r_3vr.to_string()))?;
        r_3vo.push(T84 {
            id: ids::file_id(r_3vx),
            href: "documentation/VanBanXacThuc.pdf".into(),
            mimetype: mime::PDF.to_string(),
            size: pdf.len() as u64,
            checksum_upper: f_2en(&pdf),
        });
        files.insert("documentation/VanBanXacThuc.pdf".into(), pdf);
    }
    let r_3w2 = r_3wb
        .related_hoso_mets_id
        .clone()
        .unwrap_or_else(|| rep_objid.clone());
    let r_3w5 = mets::f_1x0(
        &mets::Tgb {
            kind: r_3wb.kind,
            objid: &objid,
            label: &label,
            content_information_type: &r_3vh,
            created: &r_3vj,
            software: &r_3wb.software,
            submission_agreement: r_3wb.submission_agreement.as_deref(),
            ead_ref: r_3vq,
            premis_ref: r_3w4,
            schema_files: r_3wc,
            rep_files: vec![r_3w8],
            documentation_files: r_3vo,
            outer_div_label: &r_3w2,
        },
        r_3vx,
    )
    .into_bytes();
    files.insert("METS.xml".into(), r_3w5);
    let zip_bytes = f_w6(&folder, &files, &r_3wb.created_at)?;
    let zip_sha256 = sha256_lower(&zip_bytes);
    let zip_name = ids::f_2sb(r_3wb.kind, &objid, r_3wb.zip_ordinal);
    Ok(T1g {
        kind: r_3wb.kind,
        objid,
        rep_objid,
        folder_name: folder,
        zip_name,
        files,
        zip_bytes,
        zip_sha256,
    })
}
