// Copyright (c) 2026 DigiText. All rights reserved.
use crate::SignError;
use cms::cert::{CertificateChoices, IssuerAndSerialNumber};
use cms::content_info::ContentInfo;
use cms::signed_data::{
    CertificateSet, EncapsulatedContentInfo, SignedData, SignerIdentifier, SignerInfo, SignerInfos,
};
use const_oid::ObjectIdentifier as Oid;
use der::asn1::{GeneralizedTime, OctetString, SetOfVec, UtcTime};
use der::{Any, Decode, Encode, Sequence};
use sha2::{Digest, Sha256};
use spki::AlgorithmIdentifierOwned;
use x509_cert::attr::Attribute;
use x509_cert::Certificate;
const C_ER: Oid = Oid::new_unwrap("1.2.840.113549.1.7.1");
pub(crate) const OID_SIGNED_DATA: Oid = Oid::new_unwrap("1.2.840.113549.1.7.2");
const OID_CONTENT_TYPE: Oid = Oid::new_unwrap("1.2.840.113549.1.9.3");
const OID_MESSAGE_DIGEST: Oid = Oid::new_unwrap("1.2.840.113549.1.9.4");
const OID_SIGNING_TIME: Oid = Oid::new_unwrap("1.2.840.113549.1.9.5");
const C_F0: Oid = Oid::new_unwrap("1.2.840.113549.1.9.16.2.47");
const C_F2: Oid = Oid::new_unwrap("1.2.840.113549.1.9.16.2.14");
pub(crate) const OID_SHA256: Oid = Oid::new_unwrap("2.16.840.1.101.3.4.2.1");
const C_EY: Oid = Oid::new_unwrap("1.2.840.113549.1.1.1");
const C_F1: Oid = Oid::new_unwrap("2.5.29.14");
#[derive(Sequence)]
struct T6y {
    cert_hash: OctetString,
}
#[derive(Sequence)]
struct T6z {
    cert: T6y,
}
#[derive(Sequence)]
struct Tlr {
    certs: T6z,
}
fn cert_hash(cert: &Certificate) -> Result<Vec<u8>, SignError> {
    let r_4ld = cert.to_der().map_err(SignError::der)?;
    Ok(Sha256::digest(&r_4ld).to_vec())
}
pub(crate) fn alg_sha256() -> AlgorithmIdentifierOwned {
    AlgorithmIdentifierOwned {
        oid: OID_SHA256,
        parameters: None,
    }
}
fn alg_rsa() -> AlgorithmIdentifierOwned {
    AlgorithmIdentifierOwned {
        oid: C_EY,
        parameters: Some(Any::null()),
    }
}
fn attr(r_4le: Oid, r_4lf: Any) -> Result<Attribute, SignError> {
    let mut r_4lg = SetOfVec::new();
    r_4lg.insert(r_4lf).map_err(SignError::der)?;
    Ok(Attribute {
        oid: r_4le,
        values: r_4lg,
    })
}
pub fn f_w2(
    message_digest: &[u8],
    signing_time_unix: u64,
    signer_cert: &Certificate,
) -> Result<SetOfVec<Attribute>, SignError> {
    let r_4li = attr(
        OID_CONTENT_TYPE,
        Any::encode_from(&C_ER).map_err(SignError::der)?,
    )?;
    let r_4lj = OctetString::new(message_digest).map_err(SignError::der)?;
    let message_digest = attr(
        OID_MESSAGE_DIGEST,
        Any::encode_from(&r_4lj).map_err(SignError::der)?,
    )?;
    let r_4lo = UtcTime::from_unix_duration(core::time::Duration::from_secs(signing_time_unix))
        .map_err(SignError::der)?;
    let r_4ln = attr(
        OID_SIGNING_TIME,
        Any::encode_from(&r_4lo).map_err(SignError::der)?,
    )?;
    let r_4lk = Tlr {
        certs: T6z {
            cert: T6y {
                cert_hash: OctetString::new(cert_hash(signer_cert)?).map_err(SignError::der)?,
            },
        },
    };
    let r_4lm = attr(C_F0, Any::encode_from(&r_4lk).map_err(SignError::der)?)?;
    let mut r_4ll = SetOfVec::new();
    for r_4lh in [r_4li, message_digest, r_4ln, r_4lm] {
        r_4ll.insert(r_4lh).map_err(SignError::der)?;
    }
    Ok(r_4ll)
}
pub fn f_2fa(r_4lp: &SetOfVec<Attribute>) -> Result<Vec<u8>, SignError> {
    r_4lp.to_der().map_err(SignError::der)
}
pub fn f_sd(
    r_4lq: SetOfVec<Attribute>,
    r_4lr: &[u8],
    signer_cert: &Certificate,
) -> Result<Vec<u8>, SignError> {
    f_se(r_4lq, r_4lr, signer_cert, &[], None)
}
pub fn f_se(
    r_4ls: SetOfVec<Attribute>,
    r_4m0: &[u8],
    signer_cert: &Certificate,
    r_4lx: &[Certificate],
    r_4m4: Option<&[u8]>,
) -> Result<Vec<u8>, SignError> {
    let r_4lz = SignerIdentifier::IssuerAndSerialNumber(IssuerAndSerialNumber {
        issuer: signer_cert.tbs_certificate.issuer.clone(),
        serial_number: signer_cert.tbs_certificate.serial_number.clone(),
    });
    let r_4m6 = match r_4m4 {
        Some(r_4m5) => {
            let r_4m7 = Any::from_der(r_4m5).map_err(SignError::der)?;
            let mut r_4ly = SetOfVec::new();
            r_4ly.insert(attr(C_F2, r_4m7)?).map_err(SignError::der)?;
            Some(r_4ly)
        }
        None => None,
    };
    let r_4m2 = SignerInfo {
        version: cms::content_info::CmsVersion::V1,
        sid: r_4lz,
        digest_alg: alg_sha256(),
        signed_attrs: Some(r_4ls),
        signature_algorithm: alg_rsa(),
        signature: OctetString::new(r_4m0).map_err(SignError::der)?,
        unsigned_attrs: r_4m6,
    };
    let mut r_4lu = SetOfVec::new();
    r_4lu.insert(alg_sha256()).map_err(SignError::der)?;
    let mut r_4m3 = SetOfVec::new();
    r_4m3.insert(r_4m2).map_err(SignError::der)?;
    let mut certs = SetOfVec::new();
    certs
        .insert(CertificateChoices::Certificate(signer_cert.clone()))
        .map_err(SignError::der)?;
    let mut r_4lv: Vec<&Certificate> = vec![signer_cert];
    for r_4lw in r_4lx {
        if r_4lv.contains(&r_4lw) {
            continue;
        }
        r_4lv.push(r_4lw);
        certs
            .insert(CertificateChoices::Certificate(r_4lw.clone()))
            .map_err(SignError::der)?;
    }
    let r_4m1 = SignedData {
        version: cms::content_info::CmsVersion::V1,
        digest_algorithms: r_4lu,
        encap_content_info: EncapsulatedContentInfo {
            econtent_type: C_ER,
            econtent: None,
        },
        certificates: Some(CertificateSet(certs)),
        crls: None,
        signer_infos: SignerInfos(r_4m3),
    };
    let r_4lt = ContentInfo {
        content_type: OID_SIGNED_DATA,
        content: Any::encode_from(&r_4m1).map_err(SignError::der)?,
    };
    r_4lt.to_der().map_err(SignError::der)
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct T2u {
    pub signature_ok: bool,
    pub digest_ok: bool,
    pub cert_binding_ok: bool,
    pub signer_chain_ok: bool,
    pub timestamp_present: bool,
    pub timestamp_structural_ok: bool,
    pub timestamp_trusted: bool,
    pub timestamp_time_unix: Option<i64>,
    pub signing_time_unix: Option<i64>,
}
impl T2u {
    pub fn is_valid(&self) -> bool {
        self.signature_ok && self.digest_ok && self.cert_binding_ok
    }
}
pub(crate) fn f_1zy(r_4m9: &[u8]) -> Result<SignedData, SignError> {
    let mut r_4ma = der::SliceReader::new(r_4m9).map_err(SignError::der)?;
    let r_4m8 = ContentInfo::decode(&mut r_4ma).map_err(SignError::der)?;
    if r_4m8.content_type != OID_SIGNED_DATA {
        return Err(SignError::Cms(
            "không phải id-signedData".into(),
        ));
    }
    r_4m8.content.decode_as().map_err(SignError::der)
}
pub fn f_2pq(r_4mb: &[u8], r_4mc: &[u8]) -> Result<T2u, SignError> {
    f_2pr(r_4mb, r_4mc, None, None)
}
pub fn f_2ps(
    r_4md: &[u8],
    r_4me: &[u8],
    r_4mf: Option<&crate::chain::TrustStore>,
) -> Result<T2u, SignError> {
    f_2pr(r_4md, r_4me, None, r_4mf)
}
pub fn f_2pr(
    r_4mk: &[u8],
    r_4ml: &[u8],
    r_4mt: Option<&crate::chain::TrustStore>,
    r_4mz: Option<&crate::chain::TrustStore>,
) -> Result<T2u, SignError> {
    let mut r_4mp = der::SliceReader::new(r_4mk).map_err(SignError::der)?;
    let r_4mj = ContentInfo::decode(&mut r_4mp).map_err(SignError::der)?;
    if r_4mj.content_type != OID_SIGNED_DATA {
        return Err(SignError::Cms(
            "không phải id-signedData".into(),
        ));
    }
    let r_4mr: SignedData = r_4mj.content.decode_as().map_err(SignError::der)?;
    let r_4ms = r_4mr
        .signer_infos
        .0
        .as_slice()
        .first()
        .ok_or_else(|| SignError::Cms("thiếu SignerInfo".into()))?;
    let r_4mh = r_4ms
        .signed_attrs
        .as_ref()
        .ok_or_else(|| SignError::Cms("thiếu signedAttrs".into()))?;
    let r_4mo = f_1b2(r_4mh)?;
    let digest_ok = r_4mo == r_4ml;
    let cert = signer_cert(&r_4mr)?;
    let r_4mv = f_2fa(r_4mh)?;
    let signature_ok = f_2pw(&cert, &r_4mv, r_4ms.signature.as_bytes())?;
    let cert_binding_ok = match f_1ba(r_4mh)? {
        Some(r_4mm) => r_4mm == cert_hash(&cert)?,
        None => false,
    };
    let signing_time_unix = f_1bb(r_4mh);
    let signer_chain_ok = match r_4mt {
        None => false,
        Some(r_4mq) => match signing_time_unix {
            Some(r_4mg) => {
                let r_4mn: Vec<Certificate> = f_rl(&r_4mr)
                    .into_iter()
                    .filter(|r_4mi| r_4mi != &cert)
                    .collect();
                r_4mq.verify_chain(&cert, &r_4mn, r_4mg).is_ok()
            }
            None => false,
        },
    };
    let (timestamp_present, timestamp_structural_ok, timestamp_trusted, timestamp_time_unix) =
        match r_4ms
            .unsigned_attrs
            .as_ref()
            .map(f_1bc)
            .transpose()?
            .flatten()
        {
            Some(r_4mw) => {
                let r_4my = crate::tsa::f_2q0(&r_4mw, r_4ms.signature.as_bytes(), r_4mz)?;
                let r_4mu = r_4my.imprint_ok && matches!(r_4my.token_signature_ok, Some(true));
                let r_4mx = r_4mu && matches!(r_4my.chain_ok, Some(true));
                (true, r_4mu, r_4mx, r_4my.gen_time_unix)
            }
            None => (false, false, false, None),
        };
    Ok(T2u {
        signature_ok,
        digest_ok,
        cert_binding_ok,
        signer_chain_ok,
        timestamp_present,
        timestamp_structural_ok,
        timestamp_trusted,
        timestamp_time_unix,
        signing_time_unix,
    })
}
fn f_1bb(r_4n1: &SetOfVec<Attribute>) -> Option<i64> {
    for r_4n0 in r_4n1.as_slice() {
        if r_4n0.oid == OID_SIGNING_TIME {
            let r_4n2 = r_4n0.values.as_slice().first()?;
            if let Ok(t) = r_4n2.decode_as::<UtcTime>() {
                return Some(t.to_unix_duration().as_secs() as i64);
            }
            if let Ok(t) = r_4n2.decode_as::<GeneralizedTime>() {
                return Some(t.to_unix_duration().as_secs() as i64);
            }
            return None;
        }
    }
    None
}
fn f_1bc(r_4n4: &SetOfVec<Attribute>) -> Result<Option<Vec<u8>>, SignError> {
    for r_4n3 in r_4n4.as_slice() {
        if r_4n3.oid == C_F2 {
            let r_4n5 = r_4n3
                .values
                .as_slice()
                .first()
                .ok_or_else(|| SignError::Cms("timeStampToken rỗng".into()))?;
            return Ok(Some(r_4n5.to_der().map_err(SignError::der)?));
        }
    }
    Ok(None)
}
fn f_1ba(r_4n7: &SetOfVec<Attribute>) -> Result<Option<Vec<u8>>, SignError> {
    for r_4n6 in r_4n7.as_slice() {
        if r_4n6.oid == C_F0 {
            let r_4n9 = r_4n6
                .values
                .as_slice()
                .first()
                .ok_or_else(|| SignError::Cms("signingCertificateV2 rỗng".into()))?;
            let r_4n8 = r_4n9.decode_as::<Tlr>().map_err(SignError::der)?;
            return Ok(Some(r_4n8.certs.cert.cert_hash.as_bytes().to_vec()));
        }
    }
    Ok(None)
}
pub(crate) fn f_1b2(r_4nb: &SetOfVec<Attribute>) -> Result<Vec<u8>, SignError> {
    for r_4na in r_4nb.as_slice() {
        if r_4na.oid == OID_MESSAGE_DIGEST {
            let r_4nd = r_4na
                .values
                .as_slice()
                .first()
                .ok_or_else(|| SignError::Cms("messageDigest rỗng".into()))?;
            let r_4nc = r_4nd.decode_as::<OctetString>().map_err(SignError::der)?;
            return Ok(r_4nc.as_bytes().to_vec());
        }
    }
    Err(SignError::Cms("thiếu messageDigest".into()))
}
pub(crate) fn f_rl(r_4nf: &SignedData) -> Vec<Certificate> {
    r_4nf
        .certificates
        .as_ref()
        .map(|r_4ng| {
            r_4ng
                .0
                .as_slice()
                .iter()
                .filter_map(|r_4ne| match r_4ne {
                    CertificateChoices::Certificate(cert) => Some(cert.clone()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}
fn f_xl(cert: &Certificate) -> Option<Vec<u8>> {
    let r_4nh = cert.tbs_certificate.extensions.as_ref()?;
    let e = r_4nh.iter().find(|e| e.extn_id == C_F1)?;
    let r_4ni = OctetString::from_der(e.extn_value.as_bytes()).ok()?;
    Some(r_4ni.as_bytes().to_vec())
}
pub(crate) fn signer_cert(r_4nl: &SignedData) -> Result<Certificate, SignError> {
    let r_4nm = r_4nl
        .signer_infos
        .0
        .as_slice()
        .first()
        .ok_or_else(|| SignError::Cms("thiếu SignerInfo".into()))?;
    let certs = f_rl(r_4nl);
    if certs.is_empty() {
        return Err(SignError::Cms(
            "không có Certificate trong CertificateSet".into(),
        ));
    }
    let r_4nj = match &r_4nm.sid {
        SignerIdentifier::IssuerAndSerialNumber(r_4nk) => certs.into_iter().find(|c| {
            c.tbs_certificate.issuer == r_4nk.issuer
                && c.tbs_certificate.serial_number == r_4nk.serial_number
        }),
        SignerIdentifier::SubjectKeyIdentifier(r_4nn) => {
            let r_4no = r_4nn.0.as_bytes();
            certs
                .into_iter()
                .find(|c| f_xl(c).as_deref() == Some(r_4no))
        }
    };
    r_4nj.ok_or_else(|| {
        SignError::Cms(
            "không thấy cert khớp SignerIdentifier trong CMS".into(),
        )
    })
}
pub(crate) fn f_2pw(cert: &Certificate, r_4np: &[u8], sig: &[u8]) -> Result<bool, SignError> {
    use rsa::pkcs1::DecodeRsaPublicKey;
    use rsa::pkcs1v15::{Signature, VerifyingKey};
    use rsa::signature::Verifier;
    use rsa::RsaPublicKey;
    let r_4nr = cert
        .tbs_certificate
        .subject_public_key_info
        .subject_public_key
        .raw_bytes();
    let r_4nq = RsaPublicKey::from_pkcs1_der(r_4nr).map_err(|e| SignError::Cms(e.to_string()))?;
    let r_4ns = VerifyingKey::<sha2::Sha256>::new(r_4nq);
    let sig = Signature::try_from(sig).map_err(|e| SignError::Cms(e.to_string()))?;
    Ok(r_4ns.verify(r_4np, &sig).is_ok())
}
