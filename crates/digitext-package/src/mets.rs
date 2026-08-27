// Copyright (c) 2026 DigiText. All rights reserved.
use crate::ids::{f_2op, T9n};
use crate::model::{PackageKind, Tlv};
use crate::xml::Xml;
pub const METS_NS: &str = "http://www.loc.gov/METS/";
pub const C_2F: &str = "https://DILCIS.eu/XML/METS/CSIPExtensionMETS";
pub const C_K9: &str = "https://DILCIS.eu/XML/METS/SIPExtensionMETS";
pub const C_Q7: &str = "http://www.w3.org/1999/xlink";
pub const C_Q8: &str = "http://www.w3.org/2001/XMLSchema-instance";
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MdKind {
    EadOther,
    Dc,
    Premis,
}
pub struct Td6 {
    pub id: String,
    pub href: String,
    pub size: u64,
    pub checksum_upper: String,
}
pub struct T84 {
    pub id: String,
    pub href: String,
    pub mimetype: String,
    pub size: u64,
    pub checksum_upper: String,
}
pub struct T50 {
    pub file_id: String,
    pub holey: bool,
    pub attachment_file_ids: Vec<String>,
}
pub struct Tir {
    pub role: &'static str,
    pub code: String,
}
pub struct Tgb<'a> {
    pub kind: PackageKind,
    pub objid: &'a str,
    pub label: &'a str,
    pub content_information_type: &'a str,
    pub created: &'a str,
    pub software: &'a Tlv,
    pub submission_agreement: Option<&'a str>,
    pub ead_ref: Td6,
    pub premis_ref: Option<Td6>,
    pub schema_files: Vec<T84>,
    pub rep_files: Vec<T84>,
    pub documentation_files: Vec<T84>,
    pub outer_div_label: &'a str,
}
fn f_1s0<'a>(r_3z7: &'a Td6, kind: MdKind, created: &'a str) -> Vec<(&'static str, String)> {
    let mut r_3z6: Vec<(&'static str, String)> =
        vec![("ID", r_3z7.id.clone()), ("LOCTYPE", "URL".into())];
    match kind {
        MdKind::EadOther => {
            r_3z6.push(("MDTYPE", "OTHER".into()));
            r_3z6.push(("OTHERMDTYPE", "EAD".into()));
            r_3z6.push(("MDTYPEVERSION", "1.0".into()));
        }
        MdKind::Dc => r_3z6.push(("MDTYPE", "DC".into())),
        MdKind::Premis => r_3z6.push(("MDTYPE", "PREMIS".into())),
    }
    r_3z6.push(("xlink:type", "simple".into()));
    r_3z6.push(("xlink:href", r_3z7.href.clone()));
    r_3z6.push(("MIMETYPE", "text/xml".into()));
    r_3z6.push(("SIZE", r_3z7.size.to_string()));
    r_3z6.push(("CREATED", created.into()));
    r_3z6.push(("CHECKSUM", r_3z7.checksum_upper.clone()));
    r_3z6.push(("CHECKSUMTYPE", "SHA-256".into()));
    r_3z6
}
fn f_2rt(r_3zd: &mut Xml, r_3zb: &Td6, kind: MdKind, created: &str) {
    let r_3za = f_1s0(r_3zb, kind, created);
    let r_3z8: Vec<(&str, &str)> = r_3za
        .iter()
        .map(|(r_3z9, r_3zc)| (*r_3z9, r_3zc.as_str()))
        .collect();
    r_3zd.empty("mdRef", &r_3z8);
}
fn f_2rs(r_3zf: &mut Xml, r_3ze: &T84, created: &str) {
    r_3zf.open(
        "file",
        &[
            ("ID", r_3ze.id.as_str()),
            ("MIMETYPE", r_3ze.mimetype.as_str()),
            ("SIZE", r_3ze.size.to_string().as_str()),
            ("CREATED", created),
            ("CHECKSUM", r_3ze.checksum_upper.as_str()),
            ("CHECKSUMTYPE", "SHA-256"),
        ],
    );
    r_3zf.empty(
        "FLocat",
        &[
            ("xlink:type", "simple"),
            ("xlink:href", r_3ze.href.as_str()),
            ("LOCTYPE", "URL"),
        ],
    );
    r_3zf.close("file");
}
fn f_1wc(
    r_3zj: &mut Xml,
    objid: &str,
    label: &str,
    r_3zh: &str,
    r_3zg: &str,
    profile: &str,
    r_3zi: &str,
) {
    r_3zj.open(
        "mets",
        &[
            ("xmlns", METS_NS),
            ("xmlns:xsi", C_Q8),
            ("xmlns:xlink", C_Q7),
            ("xmlns:csip", C_2F),
            ("xmlns:sip", C_K9),
            ("OBJID", objid),
            ("LABEL", label),
            ("TYPE", r_3zh),
            ("csip:CONTENTINFORMATIONTYPE", r_3zg),
            ("PROFILE", profile),
            ("xsi:schemaLocation", r_3zi),
        ],
    );
}
pub fn f_1x0(r_3zv: &Tgb<'_>, r_3zr: &mut dyn T9n) -> String {
    let mut r_403 = Xml::new();
    f_1wc(
        &mut r_403,
        r_3zv.objid,
        r_3zv.label,
        r_3zv.kind.mets_type(),
        r_3zv.content_information_type,
        r_3zv.kind.package_profile(),
        &format!("{METS_NS} schemas/METS.xsd"),
    );
    r_403.open(
        "metsHdr",
        &[
            ("CREATEDATE", r_3zv.created),
            ("LASTMODDATE", r_3zv.created),
            ("RECORDSTATUS", "NEW"),
            ("csip:OAISPACKAGETYPE", r_3zv.kind.oais_type()),
        ],
    );
    r_403.open(
        "agent",
        &[
            ("ROLE", "CREATOR"),
            ("TYPE", "OTHER"),
            ("OTHERTYPE", "SOFTWARE"),
        ],
    );
    r_403.text_elem("name", &r_3zv.software.name);
    r_403.text_elem_attrs(
        "note",
        &[("csip:NOTETYPE", "SOFTWARE VERSION")],
        &r_3zv.software.version,
    );
    r_403.close("agent");
    if let Some(r_3zz) = r_3zv.submission_agreement {
        r_403.text_elem_attrs("altRecordID", &[("TYPE", "SUBMISSIONAGREEMENT")], r_3zz);
    }
    r_403.close("metsHdr");
    let r_3zm = f_2op(r_3zr);
    r_403.open(
        "dmdSec",
        &[
            ("ID", r_3zm.as_str()),
            ("CREATED", r_3zv.created),
            ("STATUS", "CURRENT"),
        ],
    );
    f_2rt(&mut r_403, &r_3zv.ead_ref, MdKind::EadOther, r_3zv.created);
    r_403.close("dmdSec");
    let amd_id = r_3zv.premis_ref.as_ref().map(|r_3zw| {
        let amd_id = f_2op(r_3zr);
        let r_3zl = f_2op(r_3zr);
        r_403.open("amdSec", &[("ID", amd_id.as_str())]);
        r_403.open(
            "digiprovMD",
            &[("ID", r_3zl.as_str()), ("STATUS", "CURRENT")],
        );
        f_2rt(&mut r_403, r_3zw, MdKind::Premis, r_3zv.created);
        r_403.close("digiprovMD");
        r_403.close("amdSec");
        amd_id
    });
    let r_3zq = f_2op(r_3zr);
    let r_401 = f_2op(r_3zr);
    let r_3zy = f_2op(r_3zr);
    r_403.open("fileSec", &[("ID", r_3zq.as_str())]);
    r_403.open("fileGrp", &[("ID", r_401.as_str()), ("USE", "Schemas")]);
    for f in &r_3zv.schema_files {
        f_2rs(&mut r_403, f, r_3zv.created);
    }
    r_403.close("fileGrp");
    r_403.open(
        "fileGrp",
        &[("ID", r_3zy.as_str()), ("USE", "Representations/rep1")],
    );
    for f in &r_3zv.rep_files {
        f_2rs(&mut r_403, f, r_3zv.created);
    }
    r_403.close("fileGrp");
    let r_3zp = if r_3zv.documentation_files.is_empty() {
        None
    } else {
        let id = f_2op(r_3zr);
        r_403.open("fileGrp", &[("ID", id.as_str()), ("USE", "Documentation")]);
        for f in &r_3zv.documentation_files {
            f_2rs(&mut r_403, f, r_3zv.created);
        }
        r_403.close("fileGrp");
        Some(id)
    };
    r_403.close("fileSec");
    let r_402 = f_2op(r_3zr);
    let r_3zu = f_2op(r_3zr);
    r_403.open(
        "structMap",
        &[
            ("ID", r_402.as_str()),
            ("TYPE", "PHYSICAL"),
            ("LABEL", "CSIP"),
        ],
    );
    r_403.open(
        "div",
        &[("ID", r_3zu.as_str()), ("LABEL", r_3zv.outer_div_label)],
    );
    let r_3zt = f_2op(r_3zr);
    let mut r_3zs: Vec<(&str, &str)> = vec![
        ("ID", r_3zt.as_str()),
        ("LABEL", "Metadata"),
        ("DMDID", r_3zm.as_str()),
    ];
    if let Some(r_3zk) = amd_id.as_deref() {
        r_3zs.push(("ADMID", r_3zk));
    }
    r_403.empty("div", &r_3zs);
    let r_400 = f_2op(r_3zr);
    r_403.open("div", &[("ID", r_400.as_str()), ("LABEL", "Schemas")]);
    r_403.empty("fptr", &[("FILEID", r_401.as_str())]);
    r_403.close("div");
    let r_3zx = f_2op(r_3zr);
    r_403.open(
        "div",
        &[("ID", r_3zx.as_str()), ("LABEL", "Representations/rep1")],
    );
    r_403.empty(
        "mptr",
        &[
            ("xlink:type", "simple"),
            ("xlink:href", "representations/rep1/METS.xml"),
            ("xlink:title", r_3zy.as_str()),
            ("LOCTYPE", "URL"),
        ],
    );
    r_403.close("div");
    if let Some(r_3zo) = r_3zp.as_deref() {
        let r_3zn = f_2op(r_3zr);
        r_403.open("div", &[("ID", r_3zn.as_str()), ("LABEL", "Documentation")]);
        r_403.empty("fptr", &[("FILEID", r_3zo)]);
        r_403.close("div");
    }
    r_403.close("div");
    r_403.close("structMap");
    r_403.close("mets");
    r_403.finish()
}
pub struct Tis<'a> {
    pub kind: PackageKind,
    pub rep_objid: &'a str,
    pub label: &'a str,
    pub content_information_type: &'a str,
    pub created: &'a str,
    pub agents: Vec<Tir>,
    pub doc_refs: Vec<(Td6, T50)>,
    pub premis_ref: Option<Td6>,
    pub data_files: Vec<T84>,
    pub holey_files: Vec<T84>,
}
pub fn f_27s(r_40i: &Tis<'_>, r_40d: &mut dyn T9n) -> String {
    let mut r_40n = Xml::new();
    f_1wc(
        &mut r_40n,
        r_40i.rep_objid,
        r_40i.label,
        r_40i.kind.mets_type(),
        r_40i.content_information_type,
        PackageKind::REP_PROFILE,
        &format!("{METS_NS} ../../schemas/METS.xsd"),
    );
    r_40n.open(
        "metsHdr",
        &[
            ("CREATEDATE", r_40i.created),
            ("LASTMODDATE", r_40i.created),
            ("RECORDSTATUS", "NEW"),
            ("csip:OAISPACKAGETYPE", r_40i.kind.oais_type()),
        ],
    );
    for agent in &r_40i.agents {
        r_40n.open("agent", &[("ROLE", agent.role), ("TYPE", "ORGANIZATION")]);
        r_40n.text_elem("name", &agent.code);
        r_40n.text_elem_attrs(
            "note",
            &[("csip:NOTETYPE", "IDENTIFICATIONCODE")],
            &agent.code,
        );
        r_40n.close("agent");
    }
    r_40n.close("metsHdr");
    let mut r_40b = Vec::with_capacity(r_40i.doc_refs.len());
    for (r_40e, _) in &r_40i.doc_refs {
        let dmd_id = f_2op(r_40d);
        r_40n.open(
            "dmdSec",
            &[
                ("ID", dmd_id.as_str()),
                ("CREATED", r_40i.created),
                ("STATUS", "CURRENT"),
            ],
        );
        f_2rt(&mut r_40n, r_40e, MdKind::Dc, r_40i.created);
        r_40n.close("dmdSec");
        r_40b.push(dmd_id);
    }
    let amd_id = r_40i.premis_ref.as_ref().map(|r_40k| {
        let amd_id = f_2op(r_40d);
        let r_40a = f_2op(r_40d);
        r_40n.open("amdSec", &[("ID", amd_id.as_str())]);
        r_40n.open(
            "digiprovMD",
            &[("ID", r_40a.as_str()), ("STATUS", "CURRENT")],
        );
        f_2rt(&mut r_40n, r_40k, MdKind::Premis, r_40i.created);
        r_40n.close("digiprovMD");
        r_40n.close("amdSec");
        amd_id
    });
    let r_40c = f_2op(r_40d);
    let r_409 = f_2op(r_40d);
    r_40n.open("fileSec", &[("ID", r_40c.as_str())]);
    r_40n.open("fileGrp", &[("ID", r_409.as_str()), ("USE", "Data")]);
    for f in &r_40i.data_files {
        f_2rs(&mut r_40n, f, r_40i.created);
    }
    r_40n.close("fileGrp");
    if !r_40i.holey_files.is_empty() {
        let id = f_2op(r_40d);
        r_40n.open("fileGrp", &[("ID", id.as_str()), ("USE", "Holeyfile")]);
        for f in &r_40i.holey_files {
            f_2rs(&mut r_40n, f, r_40i.created);
        }
        r_40n.close("fileGrp");
    }
    r_40n.close("fileSec");
    let r_40l = f_2op(r_40d);
    let r_40h = f_2op(r_40d);
    r_40n.open(
        "structMap",
        &[
            ("ID", r_40l.as_str()),
            ("TYPE", "PHYSICAL"),
            ("LABEL", "CSIP"),
        ],
    );
    r_40n.open("div", &[("ID", r_40h.as_str()), ("LABEL", "rep1")]);
    if let Some(r_40j) = &r_40i.premis_ref {
        let r_40f = f_2op(r_40d);
        r_40n.open("div", &[("ID", r_40f.as_str()), ("LABEL", "Metadata")]);
        r_40n.empty("fptr", &[("FILEID", r_40j.id.as_str())]);
        r_40n.close("div");
    }
    let r_40g = f_2op(r_40d);
    r_40n.open("div", &[("ID", r_40g.as_str()), ("LABEL", "MetadataLink")]);
    for ((_, link), dmd_id) in r_40i.doc_refs.iter().zip(&r_40b) {
        let div_id = f_2op(r_40d);
        let label = if link.holey {
            "MetadataLink/Holey"
        } else {
            "MetadataLink/File"
        };
        let mut r_407: Vec<(&str, &str)> =
            vec![("ID", div_id.as_str()), ("DMDID", dmd_id.as_str())];
        if let Some(r_404) = amd_id.as_deref() {
            r_407.push(("ADMID", r_404));
        }
        r_407.push(("LABEL", label));
        r_40n.open("div", &r_407);
        r_40n.empty("fptr", &[("FILEID", link.file_id.as_str())]);
        r_40n.close("div");
    }
    r_40n.close("div");
    let r_40m: Vec<&T50> = r_40i
        .doc_refs
        .iter()
        .map(|(_, l)| l)
        .filter(|l| !l.attachment_file_ids.is_empty())
        .collect();
    if !r_40m.is_empty() {
        let r_406 = f_2op(r_40d);
        r_40n.open(
            "div",
            &[("ID", r_406.as_str()), ("LABEL", "AttachmentFile")],
        );
        for link in r_40m {
            let div_id = f_2op(r_40d);
            r_40n.open(
                "div",
                &[("ID", div_id.as_str()), ("LABEL", link.file_id.as_str())],
            );
            for r_405 in &link.attachment_file_ids {
                r_40n.empty("fptr", &[("FILEID", r_405.as_str())]);
            }
            r_40n.close("div");
        }
        r_40n.close("div");
    }
    let r_408 = f_2op(r_40d);
    r_40n.open("div", &[("ID", r_408.as_str()), ("LABEL", "Data")]);
    r_40n.empty("fptr", &[("FILEID", r_409.as_str())]);
    r_40n.close("div");
    r_40n.close("div");
    r_40n.close("structMap");
    r_40n.close("mets");
    r_40n.finish()
}
