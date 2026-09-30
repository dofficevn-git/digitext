// Copyright (c) 2026 DigiText. All rights reserved.
// Bộ XSD đi kèm gói: METS / xlink / DILCIS CSIP (+ SIP) / PREMIS chép nguyên từ tệp chuẩn (gói tự đứng,
// kiểm được khi không có mạng); EAD*.xsd sinh theo đúng danh sách trường của TT05 cho từng loại gói.
use crate::model::PackageKind;
const C_I7: &[&str] = &["language"];
const METS_XSD: &str = include_str!("schemas/METS.xsd");
const XLINK_XSD: &str = include_str!("schemas/xlink.xsd");
const CSIP_XSD: &str = include_str!("schemas/DILCISExtensionMETS.xsd");
const SIP_XSD: &str = include_str!("schemas/DILCISExtensionSIPMETS.xsd");
const PREMIS_XSD: &str = include_str!("schemas/PREMIS.xsd");
fn f_2fi(r_41a: &[&str]) -> String {
    let mut r_41b = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <xs:schema xmlns:xs=\"http://www.w3.org/2001/XMLSchema\"\n\
         \x20          elementFormDefault=\"qualified\"\n\
         \x20          attributeFormDefault=\"unqualified\">\n\
         \x20   <xs:group name=\"elementsGroup\">\n\
         \x20       <xs:sequence>\n",
    );
    for el in r_41a {
        let max = if C_I7.contains(el) {
            " maxOccurs=\"unbounded\""
        } else {
            ""
        };
        r_41b.push_str(&format!(
            "            <xs:element name=\"{el}\" type=\"xs:string\" minOccurs=\"0\"{max}/>\n"
        ));
    }
    r_41b.push_str(
        "        </xs:sequence>\n\
         \x20   </xs:group>\n\
         \x20   <xs:complexType name=\"elementContainer\">\n\
         \x20       <xs:choice>\n\
         \x20           <xs:group ref=\"elementsGroup\"/>\n\
         \x20       </xs:choice>\n\
         \x20   </xs:complexType>\n\
         \x20   <xs:element name=\"simpledc\" type=\"elementContainer\"/>\n\
         </xs:schema>\n",
    );
    r_41b
}
pub fn f_1wy(r_41d: PackageKind) -> String {
    match r_41d {
        PackageKind::AipHoso | PackageKind::SipHoso => {
            let r_41c = r_41d.hoso_code_tag();
            f_2fi(&[
                r_41c,
                "title",
                "maintenance",
                "mode",
                "language",
                "startDate",
                "endDate",
                "keyword",
                "totalDoc",
                "numberOfPaper",
                "numberOfPage",
                "format",
                "inforSign",
                "confidenceLevel",
                "paperFileCode",
                "riskRecovery",
                "riskRecoveryStatus",
                "description",
            ])
        }
        PackageKind::AipTailieu => f_2fi(&["arcFileCode", "title", "source", "description"]),
        PackageKind::SipTailieu => {
            f_2fi(&["fileCode", "title", "source", "totalDoc", "description"])
        }
        PackageKind::Dip => f_2fi(&[
            "requestID",
            "requestDate",
            "purpose",
            "purposeContent",
            "feeObjectType",
            "researchTopic",
            "description",
        ]),
    }
}
pub fn f_18j(r_41f: PackageKind) -> String {
    let r_41e = r_41f.doc_code_tag();
    f_2fi(&[
        "docId",
        r_41e,
        "maintenance",
        "typeName",
        "codeNumber",
        "codeNotation",
        "issuedDate",
        "organName",
        "subject",
        "language",
        "numberOfPage",
        "inforSign",
        "keyword",
        "mode",
        "confidenceLevel",
        "autograph",
        "format",
        "process",
        "riskRecovery",
        "riskRecoveryStatus",
        "description",
    ])
}
pub fn f_18l(r_41h: PackageKind) -> String {
    let r_41g = r_41h.doc_code_tag();
    f_2fi(&[
        r_41g,
        "maintenance",
        "typePic",
        "archivesNumber",
        "inforSign",
        "eventName",
        "imageTitle",
        "photographer",
        "photoPlace",
        "photoTime",
        "colour",
        "filmSize",
        "docAttached",
        "mode",
        "format",
        "riskRecovery",
        "riskRecoveryStatus",
        "description",
    ])
}
pub fn f_18k(r_41j: PackageKind) -> String {
    let r_41i = r_41j.doc_code_tag();
    f_2fi(&[
        r_41i,
        "maintenance",
        "typeMedia",
        "archivesNumber",
        "inforSign",
        "eventName",
        "movieTitle",
        "recorder",
        "recordPlace",
        "recordDate",
        "language",
        "playTime",
        "docAttached",
        "mode",
        "quality",
        "format",
        "riskRecovery",
        "riskRecoveryStatus",
        "description",
    ])
}
pub fn f_1sq() -> String {
    METS_XSD.to_string()
}
pub fn f_23g() -> String {
    PREMIS_XSD.to_string()
}
/// Danh sách (tên tệp, nội dung) trong `schemas/`, thứ tự cố định để gói tái tạo được.
pub fn schema_files(r_41l: PackageKind, r_41m: bool) -> Vec<(String, String)> {
    let mut r_41k = vec![
        ("METS.xsd".to_string(), f_1sq()),
        ("xlink.xsd".to_string(), XLINK_XSD.to_string()),
        ("DILCISExtensionMETS.xsd".to_string(), CSIP_XSD.to_string()),
    ];
    if r_41l.is_sip() {
        r_41k.push(("DILCISExtensionSIPMETS.xsd".to_string(), SIP_XSD.to_string()));
    }
    r_41k.push((r_41l.package_ead_xsd_name().to_string(), f_1wy(r_41l)));
    r_41k.push(("EAD_doc.xsd".to_string(), f_18j(r_41l)));
    r_41k.push(("EAD_media.xsd".to_string(), f_18k(r_41l)));
    r_41k.push(("EAD_pic.xsd".to_string(), f_18l(r_41l)));
    if r_41m {
        r_41k.push(("PREMIS.xsd".to_string(), f_23g()));
    }
    r_41k
}
