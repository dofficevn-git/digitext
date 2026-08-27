// Copyright (c) 2026 DigiText. All rights reserved.
use crate::ead::DocEad;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageKind {
    AipHoso,
    AipTailieu,
    SipHoso,
    SipTailieu,
    Dip,
}
impl PackageKind {
    pub const ALL: [PackageKind; 5] = [
        PackageKind::AipHoso,
        PackageKind::AipTailieu,
        PackageKind::SipHoso,
        PackageKind::SipTailieu,
        PackageKind::Dip,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            PackageKind::AipHoso => "aip_hoso",
            PackageKind::AipTailieu => "aip_tailieu",
            PackageKind::SipHoso => "sip_hoso",
            PackageKind::SipTailieu => "sip_tailieu",
            PackageKind::Dip => "dip",
        }
    }
    pub fn oais_type(self) -> &'static str {
        match self {
            PackageKind::AipHoso | PackageKind::AipTailieu => "AIP",
            PackageKind::SipHoso | PackageKind::SipTailieu => "SIP",
            PackageKind::Dip => "DIP",
        }
    }
    pub fn mets_type(self) -> &'static str {
        match self {
            PackageKind::AipHoso | PackageKind::SipHoso => "Mixed",
            PackageKind::AipTailieu | PackageKind::SipTailieu => "Collection",
            PackageKind::Dip => "MIXED",
        }
    }
    pub fn package_profile(self) -> &'static str {
        match self {
            PackageKind::AipHoso | PackageKind::AipTailieu => "http://www.ra.ee/METS/v01/IP.xml",
            PackageKind::SipHoso | PackageKind::SipTailieu => {
                "https://earkcsip.dilcis.eu/profile/E-ARK-CSIP.xml"
            }
            PackageKind::Dip => "https://earkdip.dilcis.eu/profile/E-ARK-DIP.xml",
        }
    }
    pub const REP_PROFILE: &'static str = "https://earkcsip.dilcis.eu/profile/E-ARK-CSIP.xml";
    pub fn premis_required(self) -> bool {
        matches!(self, PackageKind::AipHoso | PackageKind::AipTailieu)
    }
    pub fn premis_allowed(self) -> bool {
        !matches!(self, PackageKind::SipHoso | PackageKind::SipTailieu)
    }
    pub fn allows_fetch(self) -> bool {
        matches!(self, PackageKind::AipHoso | PackageKind::SipHoso)
    }
    pub fn uses_urn_objid(self) -> bool {
        matches!(self, PackageKind::AipHoso | PackageKind::AipTailieu)
    }
    pub fn package_ead_name(self) -> &'static str {
        match self {
            PackageKind::SipTailieu => "EAD_TL.xml",
            _ => "EAD.xml",
        }
    }
    pub fn package_ead_xsd_name(self) -> &'static str {
        match self {
            PackageKind::SipTailieu => "EAD_TL.xsd",
            _ => "EAD.xsd",
        }
    }
    pub fn hoso_code_tag(self) -> &'static str {
        match self {
            PackageKind::SipHoso | PackageKind::SipTailieu => "fileCode",
            _ => "arcFileCode",
        }
    }
    pub fn doc_code_tag(self) -> &'static str {
        match self {
            PackageKind::SipHoso | PackageKind::SipTailieu => "docCode",
            _ => "arcDocCode",
        }
    }
    pub fn single_document(self) -> bool {
        matches!(self, PackageKind::AipTailieu | PackageKind::SipTailieu)
    }
    pub fn requires_submission_agreement(self) -> bool {
        matches!(self, PackageKind::SipHoso | PackageKind::SipTailieu)
    }
}
impl std::str::FromStr for PackageKind {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|r_40u| r_40u.as_str() == s)
            .ok_or_else(|| format!("unknown package kind: {s}"))
    }
}
#[derive(Debug, Clone)]
pub struct Tlv {
    pub name: String,
    pub version: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageSourceEvent {
    Digitized,
    FromSip,
}
impl PackageSourceEvent {
    pub fn premis_event_type(self) -> &'static str {
        match self {
            PackageSourceEvent::Digitized => "transfer",
            PackageSourceEvent::FromSip => "information package creation",
        }
    }
}
#[derive(Debug, Clone)]
pub enum PackageMeta {
    Hoso(crate::ead::T9a),
    GoiTin(T8s),
    Dip(T4o),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T8s {
    pub file_code: String,
    pub title: String,
    pub source: String,
    pub total_doc: Option<u32>,
    pub description: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T4o {
    pub request_id: String,
    pub request_date: String,
    pub purpose: String,
    pub purpose_content: Option<String>,
    pub fee_object_type: Option<String>,
    pub research_topic: Option<String>,
    pub description: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T4n {
    pub ma_xac_thuc: String,
    pub ten_luu_tru_lich_su: String,
    pub nguoi_nhan_ho_ten: String,
    pub nguoi_nhan_dinh_danh: String,
    pub ma_luu_tru_goc: String,
    pub so_ky_hieu_goc: Option<String>,
    pub ten_loai: String,
    pub trich_yeu: String,
    pub muc_dich: String,
    pub ngay_cap: String,
    pub thoi_han_su_dung: Option<String>,
    pub so_luong_ban: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T7z {
    pub source_package_uuid: String,
    pub source_path: String,
    pub size_bytes: u64,
    pub doc_id: String,
}
#[derive(Debug, Clone)]
pub enum DocumentContent {
    Bytes(Vec<u8>),
    Fetch(T7z),
}
#[derive(Debug, Clone)]
pub struct To {
    pub file_name: String,
    pub bytes: Vec<u8>,
    pub mimetype: Option<String>,
}
#[derive(Debug, Clone)]
pub struct T51 {
    pub file_name: String,
    pub content: DocumentContent,
    pub mimetype: Option<String>,
    pub metadata: DocEad,
    pub attachments: Vec<To>,
}
#[derive(Debug, Clone)]
pub struct Tga {
    pub kind: PackageKind,
    pub label: Option<String>,
    pub fond_code: String,
    pub archive_org_code: String,
    pub software: Tlv,
    pub meta: PackageMeta,
    pub submission_agreement: Option<String>,
    pub dissemination_request_id: Option<String>,
    pub related_hoso_mets_id: Option<String>,
    pub documents: Vec<T51>,
    pub include_premis: bool,
    pub source_event: PackageSourceEvent,
    pub created_at: DateTime<FixedOffset>,
    pub zip_ordinal: Option<u32>,
    pub content_information_type: Option<String>,
    pub dip_cert: Option<T4n>,
    pub cert_font_ttf: Option<Vec<u8>>,
}
