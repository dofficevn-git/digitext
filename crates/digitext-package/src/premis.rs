// Copyright (c) 2026 DigiText. All rights reserved.
// PREMIS 3.0 cho gói AIP (và DIP khi bật): PREMIS.xml ở gốc mô tả thực thể trí tuệ (hồ sơ / gói tin),
// biểu diễn rep1, từng tệp, sự kiện tạo gói + chuyển tệp, tác nhân phần mềm + cơ quan lưu trữ;
// PREMIS_rep1.xml mô tả biểu diễn + tệp + sự kiện kiểm tra. Khuôn theo gói mẫu dự án Đầm Hà (DOffice 1.2.0).
use crate::model::{OrgIdent, PackageKind, Tlv};
use crate::xml::Xml;
const PREMIS_NS: &str = "http://www.loc.gov/premis/v3";
const HASH_AUTHORITY_URI: &str =
    "http://id.loc.gov/vocabulary/preservation/cryptographicHashFunctions";
/// Một tệp dữ liệu trong rep1.
pub struct PremisFile {
    /// ID của `mets:file` tương ứng (định danh chính của object, khớp METS).
    pub mets_file_id: String,
    pub original_name: String,
    pub sha256_lower: String,
    pub size: u64,
    /// Tên định dạng người đọc được ("PDF", "TXT", …).
    pub format_name: String,
    pub format_version: Option<String>,
    pub mimetype: String,
    pub doc_title: Option<String>,
    pub page_count: Option<u32>,
    pub creator: Option<String>,
    /// Đường dẫn trong gói tính từ gốc.
    pub location: String,
    /// true = tài liệu liên kết (.fetch.txt), không phải bản chính.
    pub holey: bool,
}
pub struct PremisCtx<'a> {
    pub kind: PackageKind,
    pub objid: &'a str,
    pub rep_objid: &'a str,
    pub title: &'a str,
    /// Mã hồ sơ / mã gói tin / mã yêu cầu.
    pub identifier: &'a str,
    pub event_type: &'a str,
    pub creation_event_id: String,
    pub validation_event_id: String,
    pub event_datetime: String,
    pub date: String,
    pub software: &'a Tlv,
    pub archive: &'a OrgIdent,
    pub fond_name: &'a str,
}
fn f_1wd(r_40w: &mut Xml, rel_schema: &str) {
    r_40w.open(
        "premis",
        &[
            ("xmlns", PREMIS_NS),
            ("xmlns:premis", PREMIS_NS),
            ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
            ("xsi:schemaLocation", &format!("{PREMIS_NS} {rel_schema}")),
            ("version", "3.0"),
        ],
    );
}
fn ident(x: &mut Xml, elem: &str, ty: &str, value: &str) {
    x.open(elem, &[]);
    x.text_elem(&format!("{elem}Type"), ty);
    x.text_elem(&format!("{elem}Value"), value);
    x.close(elem);
}
fn preservation_level(x: &mut Xml, ty: &str, value: &str, role: &str, rationale: &str, date: &str) {
    x.open("preservationLevel", &[]);
    x.text_elem("preservationLevelType", ty);
    x.text_elem("preservationLevelValue", value);
    x.text_elem("preservationLevelRole", role);
    x.text_elem("preservationLevelRationale", rationale);
    x.text_elem("preservationLevelDateAssigned", date);
    x.close("preservationLevel");
}
fn relationship(x: &mut Xml, sub_type: &str, id_type: &str, id_value: &str) {
    x.open("relationship", &[]);
    x.text_elem("relationshipType", "structural");
    x.text_elem("relationshipSubType", sub_type);
    x.open("relatedObjectIdentification", &[]);
    x.text_elem("relatedObjectIdentifierType", id_type);
    x.text_elem("relatedObjectIdentifierValue", id_value);
    x.close("relatedObjectIdentification");
    x.close("relationship");
}
fn linking_event(x: &mut Xml, id: &str) {
    ident(x, "linkingEventIdentifier", "UUID", id);
}
fn software_id(s: &Tlv) -> String {
    format!(
        "{}_{}",
        s.name.to_lowercase().replace(|c: char| !c.is_ascii_alphanumeric(), "_"),
        s.version
    )
}
fn agent_software(x: &mut Xml, s: &Tlv, function: &str) {
    x.open("agent", &[]);
    ident(x, "agentIdentifier", "LOCAL", &software_id(s));
    x.text_elem("agentName", &format!("{} {}", s.name, s.version));
    x.text_elem("agentType", "software");
    x.text_elem("agentNote", "Phần mềm tự động dựng gói tin và sinh siêu dữ liệu PREMIS");
    x.open("agentExtension", &[]);
    x.text_elem("softwareVersion", &s.version);
    x.text_elem("functionType", function);
    x.close("agentExtension");
    x.close("agent");
}
fn agent_org(x: &mut Xml, org: &OrgIdent) {
    x.open("agent", &[]);
    ident(x, "agentIdentifier", "LOCAL", &org.code);
    x.text_elem("agentName", &org.name);
    x.text_elem("agentType", "organization");
    x.text_elem(
        "agentNote",
        "Cơ quan lưu trữ chịu trách nhiệm bảo quản lâu dài và cho phép tiếp cận",
    );
    x.open("agentExtension", &[]);
    x.text_elem("preservationRole", "repository");
    x.close("agentExtension");
    x.close("agent");
}
fn event(
    x: &mut Xml,
    id_type: &str,
    id: &str,
    ty: &str,
    when: &str,
    detail: &str,
    outcome_note: &str,
    software: &Tlv,
    role: &str,
    linked: &[(&str, &str, &str)],
) {
    x.open("event", &[]);
    ident(x, "eventIdentifier", id_type, id);
    x.text_elem("eventType", ty);
    x.text_elem("eventDateTime", when);
    x.open("eventDetailInformation", &[]);
    x.text_elem("eventDetail", detail);
    x.close("eventDetailInformation");
    x.open("eventOutcomeInformation", &[]);
    x.text_elem("eventOutcome", "success");
    x.open("eventOutcomeDetail", &[]);
    x.text_elem("eventOutcomeDetailNote", outcome_note);
    x.close("eventOutcomeDetail");
    x.close("eventOutcomeInformation");
    x.open("linkingAgentIdentifier", &[]);
    x.text_elem("linkingAgentIdentifierType", "LOCAL");
    x.text_elem("linkingAgentIdentifierValue", &software_id(software));
    x.text_elem("linkingAgentRole", role);
    x.close("linkingAgentIdentifier");
    for (lt, lv, lr) in linked {
        x.open("linkingObjectIdentifier", &[]);
        x.text_elem("linkingObjectIdentifierType", lt);
        x.text_elem("linkingObjectIdentifierValue", lv);
        x.text_elem("linkingObjectRole", lr);
        x.close("linkingObjectIdentifier");
    }
    x.close("event");
}
fn file_object(x: &mut Xml, f: &PremisFile, ctx: &PremisCtx<'_>, xml_id: &str, transfer_event: Option<&str>, full: bool) {
    x.open("object", &[("xsi:type", "premis:file"), ("xmlID", xml_id)]);
    ident(x, "objectIdentifier", "LOCAL", &f.mets_file_id);
    x.text_elem("objectCategory", "file");
    if full {
        preservation_level(x, "bit preservation", "Full", "requirement", "legal mandate", &ctx.date);
    }
    x.open("objectCharacteristics", &[]);
    x.text_elem("compositionLevel", "0");
    x.open("fixity", &[]);
    x.text_elem_attrs(
        "messageDigestAlgorithm",
        &[
            ("authority", "cryptographicHashFunctions"),
            ("authorityURI", HASH_AUTHORITY_URI),
            ("valueURI", &format!("{HASH_AUTHORITY_URI}/sha256")),
        ],
        "SHA-256",
    );
    x.text_elem("messageDigest", &f.sha256_lower);
    x.text_elem("messageDigestOriginator", &ctx.archive.name);
    x.close("fixity");
    x.text_elem("size", &f.size.to_string());
    x.open("format", &[]);
    x.open("formatDesignation", &[]);
    x.text_elem("formatName", &f.format_name);
    if let Some(v) = &f.format_version {
        x.text_elem("formatVersion", v);
    }
    x.close("formatDesignation");
    x.text_elem("formatNote", &f.mimetype);
    x.close("format");
    x.open("objectCharacteristicsExtension", &[]);
    x.text_elem("originalFileName", &f.original_name);
    if let Some(t) = &f.doc_title {
        x.text_elem("documentTitle", t);
    }
    if let Some(p) = f.page_count {
        x.text_elem("pageCount", &p.to_string());
    }
    if let Some(c) = &f.creator {
        x.text_elem("creator", c);
    }
    if f.holey {
        x.text_elem("linkedDocument", "1");
    }
    x.close("objectCharacteristicsExtension");
    x.close("objectCharacteristics");
    x.text_elem("originalName", &f.original_name);
    x.open("storage", &[]);
    x.open("contentLocation", &[]);
    x.text_elem("contentLocationType", "URI");
    x.text_elem("contentLocationValue", &f.location);
    x.close("contentLocation");
    x.close("storage");
    relationship(x, "is part of", "LOCAL", ctx.rep_objid);
    if let Some(ev) = transfer_event {
        linking_event(x, ev);
    }
    x.close("object");
}
/// PREMIS.xml ở gốc gói.
pub fn package_premis(files: &[PremisFile], ctx: &PremisCtx<'_>) -> String {
    let mut x = Xml::new();
    f_1wd(&mut x, "schemas/PREMIS.xsd");
    let kind_label = match ctx.kind.oais_type() {
        "AIP" => "AIP - Archival Information Package",
        "DIP" => "DIP - Dissemination Information Package",
        _ => "SIP - Submission Information Package",
    };
    x.open("object", &[("xsi:type", "premis:intellectualEntity"), ("xmlID", "obj-intellectual-entity")]);
    ident(&mut x, "objectIdentifier", "LOCAL", ctx.objid);
    x.text_elem("objectCategory", "intellectual entity");
    preservation_level(&mut x, "full preservation", "High", "intention", "institutional policy", &ctx.date);
    x.open("significantProperties", &[]);
    x.text_elem("significantPropertiesType", "Archival Record");
    x.text_elem("significantPropertiesValue", ctx.title);
    x.close("significantProperties");
    x.open("objectCharacteristics", &[]);
    x.open("objectCharacteristicsExtension", &[]);
    x.text_elem("title", ctx.title);
    x.text_elem("type", kind_label);
    x.text_elem("identifier", ctx.identifier);
    x.close("objectCharacteristicsExtension");
    x.close("objectCharacteristics");
    relationship(&mut x, "is part of", "FONDS", ctx.fond_name);
    linking_event(&mut x, &ctx.creation_event_id);
    x.close("object");
    x.open("object", &[("xsi:type", "premis:representation"), ("xmlID", "obj-rep1")]);
    ident(&mut x, "objectIdentifier", "LOCAL", ctx.rep_objid);
    x.text_elem("objectCategory", "representation");
    preservation_level(&mut x, "logical preservation", "High", "intention", "institutional policy", &ctx.date);
    x.open("objectCharacteristics", &[]);
    x.open("objectCharacteristicsExtension", &[]);
    x.text_elem("representationType", "Original");
    x.text_elem("digitalFormatName", "PDF");
    x.close("objectCharacteristicsExtension");
    x.close("objectCharacteristics");
    relationship(&mut x, "represents", "LOCAL", ctx.objid);
    linking_event(&mut x, &ctx.creation_event_id);
    x.close("object");
    let transfer_ids: Vec<String> = files
        .iter()
        .map(|f| format!("event-transfer-{}", f.mets_file_id.trim_start_matches("ID-").to_lowercase()))
        .collect();
    for (i, f) in files.iter().enumerate() {
        file_object(&mut x, f, ctx, &format!("obj-file-{}", i + 1), Some(&transfer_ids[i]), true);
    }
    let n = files.len();
    event(
        &mut x,
        "UUID",
        &ctx.creation_event_id,
        ctx.event_type,
        &ctx.event_datetime,
        &format!("Dựng gói {} cho {}", ctx.kind.oais_type(), ctx.identifier),
        &format!("Gói {} dựng thành công với {n} tệp dữ liệu", ctx.kind.oais_type()),
        ctx.software,
        "executor",
        &[("LOCAL", ctx.objid, "outcome")],
    );
    for (f, ev) in files.iter().zip(&transfer_ids) {
        event(
            &mut x,
            "UUID",
            ev,
            "transfer",
            &ctx.event_datetime,
            &format!("Đưa tệp {} vào biểu diễn rep1, ghi SHA-256; PDF/A và chữ ký số được kiểm trước khi đóng gói.", f.original_name),
            "Tệp đã chép vào biểu diễn và ghi giá trị SHA-256",
            ctx.software,
            "executor",
            &[("LOCAL", &f.mets_file_id, "outcome")],
        );
    }
    agent_software(&mut x, ctx.software, "preservation packaging");
    agent_org(&mut x, ctx.archive);
    x.close("premis");
    x.finish()
}
/// PREMIS_rep1.xml trong representations/rep1/metadata/preservation.
pub fn rep_premis(files: &[PremisFile], ctx: &PremisCtx<'_>) -> String {
    let mut x = Xml::new();
    f_1wd(&mut x, "../../../../schemas/PREMIS.xsd");
    x.open("object", &[("xsi:type", "premis:representation"), ("xmlID", "obj-rep1")]);
    ident(&mut x, "objectIdentifier", "LOCAL", ctx.rep_objid);
    x.text_elem("objectCategory", "representation");
    x.open("objectCharacteristics", &[]);
    x.open("objectCharacteristicsExtension", &[]);
    x.text_elem("representationType", "Original");
    x.text_elem("digitalFormatName", "PDF");
    x.text_elem("purpose", "Biểu diễn số gốc của tài liệu lưu trữ");
    x.close("objectCharacteristicsExtension");
    x.close("objectCharacteristics");
    relationship(&mut x, "represents", "LOCAL", ctx.objid);
    x.close("object");
    for (i, f) in files.iter().enumerate() {
        file_object(&mut x, f, ctx, &format!("obj-file-{}", i + 1), None, false);
    }
    agent_software(&mut x, ctx.software, "Representation-level metadata processing");
    event(
        &mut x,
        "UUID",
        &ctx.validation_event_id,
        "validation",
        &ctx.event_datetime,
        "Kiểm tra cấu trúc biểu diễn, tham chiếu, kích thước và SHA-256 từng tệp. PDF/A và chữ ký số được kiểm trước khi đóng gói.",
        &format!("{} tệp dữ liệu qua kiểm tra cấu trúc và fixity ở mức gói", files.len()),
        ctx.software,
        "validator",
        &[("LOCAL", ctx.rep_objid, "outcome")],
    );
    x.close("premis");
    x.finish()
}
