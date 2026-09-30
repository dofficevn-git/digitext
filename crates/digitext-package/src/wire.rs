// Copyright (c) 2026 DigiText. All rights reserved.
use crate::ead::DocEad;
use crate::model::{
    DocumentContent, PackageKind, PackageMeta, PackageSourceEvent, T4n, T4o, T7z, T8s, Tga, Tlv,
    T51,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WireMeta {
    Hoso(crate::ead::T9a),
    GoiTin(T8s),
    Dip(T4o),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tq3 {
    #[serde(default)]
    pub document_id: Option<Uuid>,
    pub file_name: String,
    #[serde(default)]
    pub mimetype: Option<String>,
    pub metadata: DocEad,
    #[serde(default)]
    pub fetch: Option<T7z>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tq4 {
    pub kind: String,
    pub fond_code: String,
    pub archive_org_code: String,
    #[serde(default)]
    pub label: Option<String>,
    pub meta: WireMeta,
    #[serde(default)]
    pub submission_agreement: Option<String>,
    #[serde(default)]
    pub dissemination_request_id: Option<String>,
    #[serde(default)]
    pub related_hoso_mets_id: Option<String>,
    #[serde(default)]
    pub include_premis: bool,
    #[serde(default)]
    pub from_sip: bool,
    #[serde(default)]
    pub zip_ordinal: Option<u32>,
    #[serde(default)]
    pub content_information_type: Option<String>,
    #[serde(default)]
    pub dip_cert: Option<T4n>,
    #[serde(default)]
    pub fond_name: Option<String>,
    #[serde(default)]
    pub archive_org_name: Option<String>,
    #[serde(default)]
    pub packager_name: Option<String>,
    #[serde(default)]
    pub packager_code: Option<String>,
    #[serde(default)]
    pub allow_non_pdfa: bool,
    pub documents: Vec<Tq3>,
}
impl Tq4 {
    pub fn kind(&self) -> Result<PackageKind, String> {
        self.kind.parse()
    }
    pub fn shape_errors(&self) -> Vec<String> {
        let mut r_43q = Vec::new();
        let kind = match self.kind() {
            Ok(r_43s) => Some(r_43s),
            Err(r_43r) => {
                r_43q.push(r_43r);
                None
            }
        };
        if let Some(kind) = kind {
            let r_43t = matches!(
                (&self.meta, kind),
                (
                    WireMeta::Hoso(_),
                    PackageKind::AipHoso | PackageKind::SipHoso
                ) | (
                    WireMeta::GoiTin(_),
                    PackageKind::AipTailieu | PackageKind::SipTailieu
                ) | (WireMeta::Dip(_), PackageKind::Dip)
            );
            if !r_43t {
                r_43q.push(format!(
                    "meta không khớp loại gói {}",
                    kind.as_str()
                ));
            }
        }
        if self.kind() == Ok(PackageKind::Dip) && self.dip_cert.is_none() {
            r_43q
                .push(
                    "dipCert: gói DIP bắt buộc tệp văn bản xác thực 11 trường (Điều 42.2)"
                        .into(),
                );
        }
        if self.documents.is_empty() {
            r_43q.push(
                "documents: gói phải chứa tối thiểu 01 tài liệu"
                    .into(),
            );
        }
        for r_43p in &self.documents {
            match (&r_43p.document_id, &r_43p.fetch) {
                (Some(_), Some(_)) => {
                    r_43q
                        .push(
                            format!(
                                "'{}': documentId và fetch loại trừ lẫn nhau",
                                r_43p.file_name
                            ),
                        )
                }
                (None, None) => {
                    r_43q
                        .push(
                            format!(
                                "'{}': cần documentId (tài liệu server) hoặc fetch (tài liệu liên kết)",
                                r_43p.file_name
                            ),
                        )
                }
                _ => {}
            }
        }
        r_43q
    }
    pub fn to_build_request(
        &self,
        resolved_bytes: Vec<Option<Vec<u8>>>,
        r_440: Tlv,
        r_43w: chrono::DateTime<chrono::FixedOffset>,
        r_43u: Option<Vec<u8>>,
    ) -> Result<Tga, String> {
        let kind = self.kind()?;
        if resolved_bytes.len() != self.documents.len() {
            return Err(format!(
                "resolved_bytes ({}) không khớp số tài liệu ({})",
                resolved_bytes.len(),
                self.documents.len()
            ));
        }
        let mut documents = Vec::with_capacity(self.documents.len());
        for (d, bytes) in self.documents.iter().zip(resolved_bytes) {
            let r_43v = match (&d.fetch, bytes) {
                (Some(r_43x), _) => DocumentContent::Fetch(r_43x.clone()),
                (None, Some(b)) => DocumentContent::Bytes(b),
                (None, None) => {
                    return Err(format!(
                        "'{}': thiếu bytes tài liệu",
                        d.file_name
                    ));
                }
            };
            documents.push(T51 {
                file_name: d.file_name.clone(),
                content: r_43v,
                mimetype: d.mimetype.clone(),
                metadata: d.metadata.clone(),
                attachments: Vec::new(),
            });
        }
        let meta = match &self.meta {
            WireMeta::Hoso(r_43z) => PackageMeta::Hoso(r_43z.clone()),
            WireMeta::GoiTin(r_43y) => PackageMeta::GoiTin(r_43y.clone()),
            WireMeta::Dip(d) => PackageMeta::Dip(d.clone()),
        };
        Ok(Tga {
            kind,
            label: self.label.clone(),
            fond_code: self.fond_code.clone(),
            archive_org_code: self.archive_org_code.clone(),
            software: r_440,
            meta,
            submission_agreement: self.submission_agreement.clone(),
            dissemination_request_id: self.dissemination_request_id.clone(),
            related_hoso_mets_id: self.related_hoso_mets_id.clone(),
            documents,
            include_premis: self.include_premis,
            source_event: if self.from_sip {
                PackageSourceEvent::FromSip
            } else {
                PackageSourceEvent::Digitized
            },
            created_at: r_43w,
            zip_ordinal: self.zip_ordinal,
            content_information_type: self.content_information_type.clone(),
            dip_cert: self.dip_cert.clone(),
            cert_font_ttf: r_43u,
            fond_name: self.fond_name.clone(),
            archive_org_name: self.archive_org_name.clone(),
            packager: self.packager_name.as_ref().filter(|n| !n.trim().is_empty()).map(|n| crate::model::OrgIdent {
                name: n.clone(),
                code: self.packager_code.clone().unwrap_or_default(),
            }),
            user_manual_pdf: None,
            allow_non_pdfa: self.allow_non_pdfa,
        })
    }
    pub fn display_label(&self) -> String {
        self.label.clone().unwrap_or_else(|| match &self.meta {
            WireMeta::Hoso(r_443) => r_443.title.clone(),
            WireMeta::GoiTin(r_442) => r_442.title.clone(),
            WireMeta::Dip(r_441) => format!("DIP {}", r_441.request_id),
        })
    }
    pub fn to_aip(&self) -> Result<Tq4, String> {
        let kind = match self.kind()? {
            PackageKind::SipHoso => PackageKind::AipHoso,
            PackageKind::SipTailieu => PackageKind::AipTailieu,
            r_444 => {
                return Err(
                    format!(
                        "chỉ chuyển được gói SIP thành AIP (đang là {})",
                        r_444.as_str()
                    ),
                );
            }
        };
        let mut r_445 = self.clone();
        r_445.kind = kind.as_str().to_string();
        r_445.submission_agreement = None;
        r_445.zip_ordinal = None;
        r_445.from_sip = true;
        r_445.include_premis = false;
        Ok(r_445)
    }
}
