// Copyright (c) 2026 DigiText. All rights reserved.
use crate::model::Tlv;
use crate::xml::Xml;
const PREMIS_NS: &str = "http://www.loc.gov/premis/v3";
const PREMIS_XSD: &str = "http://www.loc.gov/standards/premis/premis-3-0-draft.xsd";
const HASH_AUTHORITY_URI: &str =
    "http://id.loc.gov/vocabulary/preservation/cryptographicHashFunctions";
pub struct Thf {
    pub object_type: &'static str,
    pub id_value: String,
    pub original_name: String,
    pub sha256_lower: String,
    pub size: u64,
    pub format_name: String,
}
pub struct The<'a> {
    pub event_type: &'a str,
    pub event_id: String,
    pub event_datetime: String,
    pub software: &'a Tlv,
    pub archive_org_code: &'a str,
}
fn f_1wd(r_40w: &mut Xml) {
    r_40w.open(
        "premis",
        &[
            ("xmlns", PREMIS_NS),
            ("xmlns:premis", PREMIS_NS),
            ("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"),
            ("xsi:schemaLocation", &format!("{PREMIS_NS} {PREMIS_XSD}")),
            ("version", "3.0"),
        ],
    );
}
fn f_2rr(r_40y: &mut Xml, ctx: &The<'_>) {
    let r_40x = format!("{} {}", ctx.software.name, ctx.software.version);
    r_40y.open("event", &[]);
    r_40y.open("eventIdentifier", &[]);
    r_40y.text_elem("eventIdentifierType", "UUID");
    r_40y.text_elem("eventIdentifierValue", &ctx.event_id);
    r_40y.close("eventIdentifier");
    r_40y.text_elem("eventType", ctx.event_type);
    r_40y.text_elem("eventDateTime", &ctx.event_datetime);
    r_40y.open("eventOutcomeInformation", &[]);
    r_40y.text_elem("eventOutcome", "success");
    r_40y.close("eventOutcomeInformation");
    r_40y.open("linkingAgentIdentifier", &[]);
    r_40y.text_elem("linkingAgentIdentifierType", "software");
    r_40y.text_elem("linkingAgentIdentifierValue", &r_40x);
    r_40y.text_elem("linkingAgentRole", "executing program");
    r_40y.close("linkingAgentIdentifier");
    r_40y.close("event");
    r_40y.open("agent", &[]);
    r_40y.open("agentIdentifier", &[]);
    r_40y.text_elem("agentIdentifierType", "LOCAL");
    r_40y.text_elem("agentIdentifierValue", &r_40x);
    r_40y.close("agentIdentifier");
    r_40y.text_elem("agentName", &ctx.software.name);
    r_40y.text_elem("agentType", "Software");
    r_40y.close("agent");
}
pub fn f_1x3(r_410: &str, r_40z: &str, ctx: &The<'_>) -> String {
    let mut r_411 = Xml::new();
    f_1wd(&mut r_411);
    r_411.open(
        "object",
        &[("xsi:type", "premis:representation"), ("xmlID", r_40z)],
    );
    r_411.open("objectIdentifier", &[]);
    r_411.text_elem("objectIdentifierType", "representation");
    r_411.text_elem("objectIdentifierValue", r_410);
    r_411.close("objectIdentifier");
    r_411.text_elem("objectCategory", "representation");
    r_411.close("object");
    f_2rr(&mut r_411, ctx);
    r_411.close("premis");
    r_411.finish()
}
pub fn f_27u(r_413: &[Thf], ctx: &The<'_>) -> String {
    let mut r_414 = Xml::new();
    f_1wd(&mut r_414);
    for r_412 in r_413 {
        r_414.open(
            "object",
            &[("xsi:type", "premis:file"), ("xmlID", &r_412.id_value)],
        );
        r_414.open("objectIdentifier", &[]);
        r_414.text_elem("objectIdentifierType", r_412.object_type);
        r_414.text_elem("objectIdentifierValue", &r_412.id_value);
        r_414.close("objectIdentifier");
        r_414.text_elem("objectCategory", "file");
        r_414.open("objectCharacteristics", &[]);
        r_414.text_elem("compositionLevel", "0");
        r_414.open("fixity", &[]);
        r_414.text_elem_attrs(
            "messageDigestAlgorithm",
            &[
                ("authority", "cryptographicHashFunctions"),
                ("authorityURI", HASH_AUTHORITY_URI),
                ("valueURI", &format!("{HASH_AUTHORITY_URI}/sha256")),
            ],
            "SHA-256",
        );
        r_414.text_elem("messageDigest", &r_412.sha256_lower);
        r_414.text_elem("messageDigestOriginator", ctx.archive_org_code);
        r_414.close("fixity");
        r_414.text_elem("size", &r_412.size.to_string());
        r_414.open("format", &[]);
        r_414.open("formatDesignation", &[]);
        r_414.text_elem("formatName", &r_412.format_name);
        r_414.close("formatDesignation");
        r_414.close("format");
        r_414.close("objectCharacteristics");
        r_414.text_elem("originalName", &r_412.original_name);
        r_414.close("object");
    }
    f_2rr(&mut r_414, ctx);
    r_414.close("premis");
    r_414.finish()
}
