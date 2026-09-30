// Copyright (c) 2026 DigiText. All rights reserved.
use crate::fetch;
use crate::model::PackageKind;
use quick_xml::events::Event;
use quick_xml::Reader;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}
#[derive(Debug, Clone)]
pub struct Violation {
    pub code: String,
    pub message: String,
    pub severity: Severity,
}
impl Violation {
    fn error(code: &str, message: String) -> Self {
        Self {
            code: code.into(),
            message,
            severity: Severity::Error,
        }
    }
}
fn f_1jo(r_42m: &str) -> bool {
    r_42m.len() == 36
        && r_42m
            .chars()
            .all(|r_42l| r_42l.is_ascii_digit() || ('A'..='F').contains(&r_42l) || r_42l == '-')
}
fn f_2ou(r_42n: PackageKind, objid: &str) -> bool {
    if r_42n.uses_urn_objid() {
        let r_42o: Vec<&str> = objid.splitn(4, ':').collect();
        r_42o.len() == 4
            && r_42o[0] == "urn"
            && !r_42o[1].is_empty()
            && r_42o[2] == "uuid"
            && f_1jo(r_42o[3])
    } else {
        objid.strip_prefix("uuid-").is_some_and(f_1jo)
    }
}
#[derive(Default)]
struct MetsFacts {
    objid: Option<String>,
    oais_type: Option<String>,
    has_csip_structmap: bool,
    refs: Vec<(String, Option<String>, Option<u64>)>,
    mptrs: Vec<String>,
}
fn f_so(r_42q: &quick_xml::events::BytesStart<'_>, r_42r: &[u8]) -> Option<String> {
    r_42q.attributes().flatten().find_map(|r_42p| {
        (r_42p.key.as_ref() == r_42r).then(|| String::from_utf8_lossy(&r_42p.value).into_owned())
    })
}
fn f_1zh(r_42t: &[u8], r_42x: &mut Vec<Violation>, path: &str) -> Option<MetsFacts> {
    let mut r_42y = Reader::from_reader(r_42t);
    let mut r_42s = Vec::new();
    let mut r_42v = MetsFacts::default();
    let mut r_42u: Option<(Option<String>, Option<u64>)> = None;
    loop {
        match r_42y.read_event_into(&mut r_42s) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(e) | Event::Empty(e)) => {
                let r_42w = e.name().as_ref().to_vec();
                match r_42w.as_slice() {
                    b"mets" => {
                        r_42v.objid = f_so(&e, b"OBJID");
                    }
                    b"metsHdr" => {
                        r_42v.oais_type = f_so(&e, b"csip:OAISPACKAGETYPE");
                    }
                    b"structMap" => {
                        if f_so(&e, b"LABEL").as_deref() == Some("CSIP") {
                            r_42v.has_csip_structmap = true;
                        }
                    }
                    b"mdRef" => {
                        if let Some(href) = f_so(&e, b"xlink:href") {
                            let checksum = f_so(&e, b"CHECKSUM");
                            let size = f_so(&e, b"SIZE").and_then(|s| s.parse::<u64>().ok());
                            r_42v.refs.push((href, checksum, size));
                        }
                    }
                    b"file" => {
                        r_42u = Some((
                            f_so(&e, b"CHECKSUM"),
                            f_so(&e, b"SIZE").and_then(|s| s.parse::<u64>().ok()),
                        ));
                    }
                    b"FLocat" => {
                        if let Some(href) = f_so(&e, b"xlink:href") {
                            let (checksum, size) = r_42u.clone().unwrap_or((None, None));
                            r_42v.refs.push((href, checksum, size));
                        }
                    }
                    b"mptr" => {
                        if let Some(href) = f_so(&e, b"xlink:href") {
                            r_42v.mptrs.push(href);
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(e)) => {
                if e.name().as_ref() == b"file" {
                    r_42u = None;
                }
            }
            Ok(_) => {}
            Err(err) => {
                r_42x.push(Violation::error(
                    "M-XML-MALFORMED",
                    format!("{path}: XML không hợp lệ: {err}"),
                ));
                return None;
            }
        }
        r_42s.clear();
    }
    Some(r_42v)
}
fn f_28t(r_42z: &str, href: &str) -> String {
    let mut r_431: Vec<&str> = if r_42z.is_empty() {
        Vec::new()
    } else {
        r_42z.split('/').collect()
    };
    for r_430 in href.split('/') {
        match r_430 {
            ".." => {
                r_431.pop();
            }
            "." | "" => {}
            p => r_431.push(p),
        }
    }
    r_431.join("/")
}
fn f_y4(
    path: &str,
    r_433: &str,
    r_436: &MetsFacts,
    r_437: &BTreeMap<String, Vec<u8>>,
    r_438: &mut Vec<Violation>,
) {
    for (href, r_435, r_439) in &r_436.refs {
        let resolved = f_28t(r_433, href);
        match r_437.get(&resolved) {
            None => {
                r_438
                    .push(
                        Violation::error(
                            "M-HREF-MISSING",
                            format!(
                                "{path}: tham chiếu '{href}' không tồn tại trong gói"
                            ),
                        ),
                    )
            }
            Some(r_434) => {
                if let Some(expect) = r_435 {
                    let r_432 = format!("{:X}", Sha256::digest(r_434));
                    if !expect.eq_ignore_ascii_case(&r_432) {
                        r_438
                            .push(
                                Violation::error(
                                    "M-CHECKSUM-MISMATCH",
                                    format!("{path}: CHECKSUM lệch cho '{resolved}'"),
                                ),
                            );
                    }
                }
                if let Some(expect) = r_439 {
                    if *expect != r_434.len() as u64 {
                        r_438
                            .push(
                                Violation::error(
                                    "M-SIZE-MISMATCH",
                                    format!("{path}: SIZE lệch cho '{resolved}'"),
                                ),
                            );
                    }
                }
            }
        }
    }
    for href in &r_436.mptrs {
        let resolved = f_28t(r_433, href);
        if !r_437.contains_key(&resolved) {
            r_438.push(Violation::error(
                "M-MPTR-MISSING",
                format!("{path}: mptr '{href}' không tồn tại trong gói"),
            ));
        }
    }
}
pub fn f_2p7(r_43g: PackageKind, r_43e: &BTreeMap<String, Vec<u8>>) -> Vec<Violation> {
    let mut r_43h = Vec::new();
    let r_43f = |p: &str| r_43e.contains_key(p);
    let r_43c = format!("metadata/descriptive/{}", r_43g.package_ead_name());
    for required in [
        "METS.xml",
        r_43c.as_str(),
        "representations/rep1/METS.xml",
        "schemas/METS.xsd",
        "schemas/xlink.xsd",
        "schemas/DILCISExtensionMETS.xsd",
    ] {
        if !r_43f(required) {
            r_43h.push(Violation::error(
                "S-MISSING",
                format!("thiếu tệp bắt buộc: {required}"),
            ));
        }
    }
    if !r_43e
        .keys()
        .any(|k| k.starts_with("schemas/EAD") && k.ends_with(".xsd"))
    {
        r_43h.push(Violation::error(
            "S-MISSING",
            "schemas/: cần tối thiểu 01 tệp EAD*.xsd".into(),
        ));
    }
    let r_43k = r_43e
        .keys()
        .filter(|k| k.starts_with("representations/rep1/metadata/descriptive/EAD_"))
        .count();
    if r_43k == 0 {
        r_43h
            .push(
                Violation::error(
                    "S-MISSING",
                    "representations/rep1/metadata/descriptive/: cần tối thiểu 01 tệp EAD_*_*.xml"
                        .into(),
                ),
            );
    }
    let r_43b: Vec<&String> = r_43e
        .keys()
        .filter(|k| k.starts_with("representations/rep1/data/"))
        .collect();
    if r_43b.is_empty() {
        r_43h.push(Violation::error(
            "S-MISSING",
            "representations/rep1/data/: cần tối thiểu 01 tài liệu"
                .into(),
        ));
    }
    if r_43g == PackageKind::Dip && !r_43f("documentation/VanBanXacThuc.pdf") {
        r_43h.push(Violation::error(
            "S-DIP-CERT",
            "gói DIP thiếu documentation/VanBanXacThuc.pdf (Điều 42.2)"
                .into(),
        ));
    }
    let r_43j = [
        "metadata/preservation/PREMIS.xml",
        "representations/rep1/metadata/preservation/PREMIS_rep1.xml",
    ];
    if r_43g.premis_required() {
        for p in r_43j {
            if !r_43f(p) {
                r_43h.push(Violation::error(
                    "S-PREMIS-MISSING",
                    format!(
                        "gói {} bắt buộc PREMIS: thiếu {p}",
                        r_43g.as_str()
                    ),
                ));
            }
        }
        if !r_43f("schemas/PREMIS.xsd") {
            r_43h.push(Violation::error(
                "S-PREMIS-MISSING",
                "thiếu schemas/PREMIS.xsd".into(),
            ));
        }
    }
    if !r_43g.premis_allowed() {
        for p in r_43e.keys().filter(|k| k.contains("/preservation/")) {
            r_43h.push(Violation::error(
                "S-PREMIS-FORBIDDEN",
                format!("gói SIP không có metadata bảo quản: {p}"),
            ));
        }
    }
    let r_43d: Vec<&String> = r_43b.iter().copied().filter(|k| fetch::f_1j0(k)).collect();
    if !r_43g.allows_fetch() && !r_43d.is_empty() {
        r_43h.push(Violation::error(
            "S-FETCH-FORBIDDEN",
            format!(
                "gói {} không dùng tài liệu liên kết .fetch.txt",
                r_43g.as_str()
            ),
        ));
    }
    let mut r_43i: Option<String> = None;
    if let Some(bytes) = r_43e.get("METS.xml") {
        if let Some(facts) = f_1zh(bytes, &mut r_43h, "METS.xml") {
            match &facts.objid {
                Some(objid) if f_2ou(r_43g, objid) => {
                    if r_43g.uses_urn_objid() {
                        r_43i = objid.split(':').nth(1).map(str::to_owned);
                    }
                }
                Some(objid) => r_43h.push(Violation::error(
                    "M-OBJID-INVALID",
                    format!(
                        "METS.xml: OBJID '{objid}' sai quy tắc cho {}",
                        r_43g.as_str()
                    ),
                )),
                None => r_43h.push(Violation::error(
                    "M-OBJID-MISSING",
                    "METS.xml: thiếu OBJID".into(),
                )),
            }
            if facts.oais_type.as_deref() != Some(r_43g.oais_type()) {
                r_43h
                    .push(
                        Violation::error(
                            "M-OAIS-TYPE",
                            format!(
                                "METS.xml: csip:OAISPACKAGETYPE phải là {} (đang là {:?})",
                                r_43g.oais_type(), facts.oais_type
                            ),
                        ),
                    );
            }
            if !facts.has_csip_structmap {
                r_43h.push(Violation::error(
                    "M-STRUCTMAP",
                    "METS.xml: thiếu structMap LABEL=\"CSIP\"".into(),
                ));
            }
            f_y4("METS.xml", "", &facts, r_43e, &mut r_43h);
        }
    }
    if let Some(bytes) = r_43e.get("representations/rep1/METS.xml") {
        let path = "representations/rep1/METS.xml";
        if let Some(facts) = f_1zh(bytes, &mut r_43h, path) {
            match &facts.objid {
                Some(objid) if objid.strip_prefix("uuid-").is_some_and(f_1jo) => {}
                Some(objid) => {
                    r_43h
                        .push(
                            Violation::error(
                                "M-OBJID-INVALID",
                                format!(
                                    "{path}: OBJID '{objid}' phải theo uuid-{{UUID viết hoa}}"
                                ),
                            ),
                        )
                }
                None => {
                    r_43h
                        .push(
                            Violation::error(
                                "M-OBJID-MISSING",
                                format!("{path}: thiếu OBJID"),
                            ),
                        )
                }
            }
            if facts.oais_type.as_deref() != Some(r_43g.oais_type()) {
                r_43h.push(Violation::error(
                    "M-OAIS-TYPE",
                    format!(
                        "{path}: csip:OAISPACKAGETYPE phải là {}",
                        r_43g.oais_type()
                    ),
                ));
            }
            if !facts.has_csip_structmap {
                r_43h.push(Violation::error(
                    "M-STRUCTMAP",
                    format!("{path}: thiếu structMap LABEL=\"CSIP\""),
                ));
            }
            f_y4(path, "representations/rep1", &facts, r_43e, &mut r_43h);
        }
    }
    for path in r_43d {
        let r_43a = String::from_utf8_lossy(&r_43e[path]);
        let r_43m: Vec<&str> = r_43a.split_whitespace().collect();
        let r_43l =
            r_43m.len() == 4 && r_43m[0].starts_with("urn_") && r_43m[1].parse::<u64>().is_ok();
        if !r_43l {
            r_43h.push(Violation::error(
                "F-FETCH-FORMAT",
                format!("{path}: nội dung .fetch.txt sai khuôn dạng"),
            ));
            continue;
        }
        if let Some(fond) = &r_43i {
            if !r_43m[0].starts_with(&format!("urn_{fond}_")) {
                r_43h
                    .push(
                        Violation::error(
                            "F-FETCH-FOND",
                            format!(
                                "{path}: mã phông trong .fetch.txt khác mã phông của gói ({fond})"
                            ),
                        ),
                    );
            }
        }
    }
    r_43h
}
