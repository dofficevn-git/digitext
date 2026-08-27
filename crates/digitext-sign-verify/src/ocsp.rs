// Copyright (c) 2026 DigiText. All rights reserved.
use crate::SignError;
use der::asn1::ObjectIdentifier as Oid;
use der::{Decode, Encode};
use sha1::Sha1;
use x509_cert::crl::CertificateList;
use x509_cert::ext::pkix::name::GeneralName;
use x509_cert::ext::pkix::{AuthorityInfoAccessSyntax, CrlDistributionPoints};
use x509_cert::Certificate;
use x509_ocsp::builder::OcspRequestBuilder;
use x509_ocsp::ext::Nonce;
use x509_ocsp::{
    BasicOcspResponse, CertId, CertStatus, OcspResponse, OcspResponseStatus, Request, Version,
};
const C_EN: Oid = Oid::new_unwrap("1.3.6.1.5.5.7.1.1");
const C_EM: Oid = Oid::new_unwrap("1.3.6.1.5.5.7.48.1");
const C_EP: Oid = Oid::new_unwrap("2.5.29.31");
const C_ES: Oid = Oid::new_unwrap("2.5.29.37");
const C_EV: Oid = Oid::new_unwrap("1.3.6.1.5.5.7.3.9");
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Revocation {
    Good,
    Revoked,
    Unknown,
}
#[derive(Debug, Clone)]
pub struct Tjb {
    pub status: Revocation,
    pub signature_ok: bool,
    pub this_update_unix: i64,
    pub next_update_unix: Option<i64>,
}
fn f_1an<T>(r_4sx: &Certificate, r_4sz: Oid) -> Option<T>
where
    T: for<'a> Decode<'a>,
{
    let r_4sy = r_4sx.tbs_certificate.extensions.as_ref()?;
    let e = r_4sy.iter().find(|e| e.extn_id == r_4sz)?;
    T::from_der(e.extn_value.as_bytes()).ok()
}
pub fn ocsp_url(r_4t2: &Certificate) -> Option<String> {
    let r_4t1: AuthorityInfoAccessSyntax = f_1an(r_4t2, C_EN)?;
    r_4t1.0.iter().find_map(|r_4t0| {
        if r_4t0.access_method != C_EM {
            return None;
        }
        match &r_4t0.access_location {
            GeneralName::UniformResourceIdentifier(r_4t3) => Some(r_4t3.as_str().to_owned()),
            _ => None,
        }
    })
}
pub fn f_12n(r_4t4: &Certificate) -> Option<String> {
    let r_4t6: CrlDistributionPoints = f_1an(r_4t4, C_EP)?;
    r_4t6.0.iter().find_map(|r_4t5| {
        let r_4t9 = r_4t5.distribution_point.as_ref()?;
        match r_4t9 {
            x509_cert::ext::pkix::name::DistributionPointName::FullName(r_4t8) => {
                r_4t8.iter().find_map(|r_4t7| match r_4t7 {
                    GeneralName::UniformResourceIdentifier(r_4ta) => {
                        Some(r_4ta.as_str().to_owned())
                    }
                    _ => None,
                })
            }
            _ => None,
        }
    })
}
pub fn f_1vw(r_4tb: &[u8]) -> Option<String> {
    ocsp_url(&Certificate::from_der(r_4tb).ok()?)
}
pub fn f_12o(r_4tc: &[u8]) -> Option<String> {
    f_12n(&Certificate::from_der(r_4tc).ok()?)
}
pub fn f_vl(r_4te: &[u8], r_4tg: &[u8]) -> Result<Vec<u8>, SignError> {
    let r_4td = Certificate::from_der(r_4te).map_err(SignError::der)?;
    let r_4tf = Certificate::from_der(r_4tg).map_err(SignError::der)?;
    f_vk(&r_4td, &r_4tf)
}
pub fn f_y7(r_4tj: &[u8], r_4ti: &[u8], r_4tl: &[u8]) -> Result<Tjb, SignError> {
    let r_4th = Certificate::from_der(r_4ti).map_err(SignError::der)?;
    let r_4tk = Certificate::from_der(r_4tl).map_err(SignError::der)?;
    f_y6(r_4tj, &r_4th, &r_4tk)
}
pub fn f_y1(r_4tm: &[u8], r_4to: &[u8], r_4tq: &[u8]) -> Result<Tjb, SignError> {
    let r_4tn = Certificate::from_der(r_4to).map_err(SignError::der)?;
    let r_4tp = Certificate::from_der(r_4tq).map_err(SignError::der)?;
    f_y0(r_4tm, &r_4tn, &r_4tp)
}
const C_EI: usize = 16;
pub fn f_vk(r_4tr: &Certificate, r_4ts: &Certificate) -> Result<Vec<u8>, SignError> {
    let mut nonce = [0u8; C_EI];
    rand_core::RngCore::fill_bytes(&mut rand_core::OsRng, &mut nonce);
    f_vm(r_4tr, r_4ts, Some(&nonce))
}
pub fn f_vm(
    r_4tw: &Certificate,
    r_4ty: &Certificate,
    nonce: Option<&[u8]>,
) -> Result<Vec<u8>, SignError> {
    let r_4tu = CertId::from_cert::<Sha1>(r_4tw, r_4ty).map_err(SignError::der)?;
    let mut r_4tt = OcspRequestBuilder::new(Version::V1).with_request(Request::new(r_4tu));
    if let Some(r_4tx) = nonce {
        let r_4tv = Nonce::new(r_4tx.to_vec()).map_err(SignError::der)?;
        r_4tt = r_4tt
            .with_extension(r_4tv)
            .map_err(|e| SignError::Cms(format!("Z+G6r24gbm9uY2UgT0NTUCBs4buXaTog{e}")))?;
    }
    r_4tt.build().to_der().map_err(SignError::der)
}
pub fn f_1vu(r_4u3: &[u8]) -> Option<Vec<u8>> {
    let r_4u2 = OcspResponse::from_der(r_4u3).ok()?;
    let r_4u0 = r_4u2.response_bytes?;
    let r_4tz = BasicOcspResponse::from_der(r_4u0.response.as_bytes()).ok()?;
    r_4tz.nonce().map(|r_4u1| r_4u1.0.as_bytes().to_vec())
}
fn f_2al(r_4u4: &Certificate, r_4u5: &[u8], r_4u6: &[u8]) -> bool {
    crate::cms::f_2pw(r_4u4, r_4u5, r_4u6).unwrap_or(false)
}
fn f_1fw(r_4u7: &Certificate) -> bool {
    let Some(r_4u9) = r_4u7.tbs_certificate.extensions.as_ref() else {
        return false;
    };
    let Some(e) = r_4u9.iter().find(|e| e.extn_id == C_ES) else {
        return false;
    };
    match x509_cert::ext::pkix::ExtendedKeyUsage::from_der(e.extn_value.as_bytes()) {
        Ok(r_4u8) => r_4u8.0.contains(&C_EV),
        Err(_) => false,
    }
}
fn f_u4(r_4ua: &BasicOcspResponse, r_4uc: &Certificate) -> bool {
    let Ok(r_4uf) = r_4ua.tbs_response_data.to_der() else {
        return false;
    };
    let r_4ue = r_4ua.signature.raw_bytes();
    if f_2al(r_4uc, &r_4uf, r_4ue) {
        return true;
    }
    if let Some(r_4ub) = &r_4ua.certs {
        for r_4ud in r_4ub {
            if f_1fw(r_4ud) && f_1jq(r_4ud, r_4uc) && f_2al(r_4ud, &r_4uf, r_4ue) {
                return true;
            }
        }
    }
    false
}
fn f_1jq(r_4ug: &Certificate, r_4uh: &Certificate) -> bool {
    let Ok(r_4ui) = r_4ug.tbs_certificate.to_der() else {
        return false;
    };
    f_2al(r_4uh, &r_4ui, r_4ug.signature.raw_bytes())
}
pub fn f_y6(r_4un: &[u8], r_4ul: &Certificate, r_4uq: &Certificate) -> Result<Tjb, SignError> {
    let r_4um = OcspResponse::from_der(r_4un).map_err(SignError::der)?;
    if r_4um.response_status != OcspResponseStatus::Successful {
        return Ok(Tjb {
            status: Revocation::Unknown,
            signature_ok: false,
            this_update_unix: 0,
            next_update_unix: None,
        });
    }
    let r_4uk = r_4um.response_bytes.ok_or_else(|| {
        SignError::Cms("T0NTUCBzdWNjZXNzZnVsIG5oxrBuZyB0aGnhur91IHJlc3BvbnNlQnl0ZXM=".into())
    })?;
    let r_4uj = BasicOcspResponse::from_der(r_4uk.response.as_bytes()).map_err(SignError::der)?;
    let signature_ok = f_u4(&r_4uj, r_4ul);
    let r_4us = CertId::from_cert::<Sha1>(r_4ul, r_4uq).map_err(SignError::der)?;
    let r_4up = r_4uj
        .tbs_response_data
        .responses
        .iter()
        .find(|r_4uo| r_4uo.cert_id == r_4us)
        .ok_or_else(|| {
            SignError::Cms(
                "T0NTUCByZXNwb25zZSBraMO0bmcga2jhu5twIENlcnRJRCBuZ8aw4budaSBrw70=".into(),
            )
        })?;
    let status = match &r_4up.cert_status {
        CertStatus::Good(_) => Revocation::Good,
        CertStatus::Revoked(_) => Revocation::Revoked,
        CertStatus::Unknown(_) => Revocation::Unknown,
    };
    Ok(Tjb {
        status,
        signature_ok,
        this_update_unix: r_4up.this_update.0.to_unix_duration().as_secs() as i64,
        next_update_unix: r_4up
            .next_update
            .as_ref()
            .map(|r_4ur| r_4ur.0.to_unix_duration().as_secs() as i64),
    })
}
pub const C_I9: i64 = 24 * 3600;
fn f_1ac(r_4ut: &Tjb, r_4uw: i64, r_4uu: i64) -> bool {
    if r_4ut.this_update_unix <= 0 {
        return false;
    }
    if let Some(r_4uv) = r_4ut.next_update_unix {
        if r_4uv < r_4ut.this_update_unix {
            return false;
        }
    }
    r_4ut.this_update_unix >= r_4uw.saturating_sub(r_4uu)
}
pub fn f_1ab(
    r_4v0: &Certificate,
    r_4v4: &Certificate,
    r_4v2: &[Vec<u8>],
    r_4uz: &[Vec<u8>],
    r_4v3: i64,
) -> bool {
    let mut r_4ux = false;
    let mut r_4v1: Option<(i64, Revocation)> = None;
    let mut r_4uy = |c: Tjb| {
        if !c.signature_ok {
            return;
        }
        if c.status == Revocation::Revoked {
            r_4ux = true;
            return;
        }
        if !f_1ac(&c, r_4v3, C_I9) {
            return;
        }
        if r_4v1.is_none_or(|(r_4v5, _)| c.this_update_unix > r_4v5) {
            r_4v1 = Some((c.this_update_unix, c.status));
        }
    };
    for der in r_4v2 {
        if let Ok(c) = f_y6(der, r_4v0, r_4v4) {
            r_4uy(c);
        }
    }
    for der in r_4uz {
        if let Ok(c) = f_y0(der, r_4v0, r_4v4) {
            r_4uy(c);
        }
    }
    !r_4ux && matches!(r_4v1, Some((_, Revocation::Good)))
}
pub fn f_y0(r_4v7: &[u8], r_4v8: &Certificate, r_4vd: &Certificate) -> Result<Tjb, SignError> {
    let r_4v6 = CertificateList::from_der(r_4v7).map_err(SignError::der)?;
    let r_4vf = r_4v6.tbs_cert_list.to_der().map_err(SignError::der)?;
    let signature_ok = f_2al(r_4v8, &r_4vf, r_4v6.signature.raw_bytes());
    let r_4vc = &r_4vd.tbs_certificate.serial_number;
    let r_4vb = r_4v6
        .tbs_cert_list
        .revoked_certificates
        .as_ref()
        .map(|r_4v9| r_4v9.iter().any(|r_4va| &r_4va.serial_number == r_4vc))
        .unwrap_or(false);
    let this_update_unix = r_4v6.tbs_cert_list.this_update.to_unix_duration().as_secs() as i64;
    let next_update_unix = r_4v6
        .tbs_cert_list
        .next_update
        .map(|r_4ve| r_4ve.to_unix_duration().as_secs() as i64);
    Ok(Tjb {
        status: if r_4vb {
            Revocation::Revoked
        } else {
            Revocation::Good
        },
        signature_ok,
        this_update_unix,
        next_update_unix,
    })
}
