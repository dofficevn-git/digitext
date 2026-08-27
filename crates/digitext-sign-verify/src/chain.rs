// Copyright (c) 2026 DigiText. All rights reserved.
use crate::SignError;
use der::{Decode, Encode};
use x509_cert::ext::pkix::ExtendedKeyUsage;
use x509_cert::Certificate;
const MAX_DEPTH: usize = 6;
const C_EZ: const_oid::ObjectIdentifier =
    const_oid::ObjectIdentifier::new_unwrap("1.2.840.113549.1.1.11");
const C_EU: const_oid::ObjectIdentifier = const_oid::ObjectIdentifier::new_unwrap("2.5.29.15");
const C_EO: const_oid::ObjectIdentifier = const_oid::ObjectIdentifier::new_unwrap("2.5.29.19");
const C_ET: const_oid::ObjectIdentifier = const_oid::ObjectIdentifier::new_unwrap("2.5.29.37");
const C_EW: const_oid::ObjectIdentifier =
    const_oid::ObjectIdentifier::new_unwrap("1.3.6.1.5.5.7.3.8");
#[derive(Debug, Clone, Default)]
pub struct TrustStore {
    roots: Vec<Certificate>,
}
impl TrustStore {
    pub fn from_pem(r_4jw: &str) -> Result<Self, SignError> {
        let mut roots = Vec::new();
        for r_4jv in f_21z(r_4jw) {
            roots.push(Certificate::from_der(&r_4jv).map_err(SignError::der)?);
        }
        if roots.is_empty() {
            return Err(
                SignError::Cms(
                    "trust store rỗng: không có khối CERTIFICATE nào trong PEM"
                        .into(),
                ),
            );
        }
        Ok(Self { roots })
    }
    pub fn len(&self) -> usize {
        self.roots.len()
    }
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }
    fn is_root(&self, cert: &Certificate) -> bool {
        let Ok(r_4jx) = cert.to_der() else {
            return false;
        };
        self.roots
            .iter()
            .any(|r_4jy| r_4jy.to_der().map(|b| b == r_4jx).unwrap_or(false))
    }
    pub fn verify_signer_der(&self, r_4jz: &[u8], at_unix: i64) -> Result<(), SignError> {
        self.verify_signer_chain_der(r_4jz, &[], at_unix)
    }
    pub fn verify_signer_chain_der(
        &self,
        r_4k0: &[u8],
        r_4k2: &[Vec<u8>],
        at_unix: i64,
    ) -> Result<(), SignError> {
        let cert = Certificate::from_der(r_4k0).map_err(SignError::der)?;
        let r_4k3: Vec<Certificate> = r_4k2
            .iter()
            .map(|r_4k1| Certificate::from_der(r_4k1).map_err(SignError::der))
            .collect::<Result<_, _>>()?;
        self.verify_chain(&cert, &r_4k3, at_unix)
    }
    pub fn resolve_issuer_der(&self, r_4k5: &[u8], r_4k7: &[Vec<u8>]) -> Option<Vec<u8>> {
        let r_4k4 = Certificate::from_der(r_4k5).ok()?;
        let r_4k8: Vec<Certificate> = r_4k7
            .iter()
            .filter_map(|r_4k6| Certificate::from_der(r_4k6).ok())
            .collect();
        r_4k8
            .iter()
            .chain(self.roots.iter())
            .find(|c| f_1j4(&r_4k4, c))
            .and_then(|c| c.to_der().ok())
    }
    pub fn verify_tsa_chain(
        &self,
        r_4ka: &Certificate,
        r_4k9: &[Certificate],
        at_unix: i64,
    ) -> Result<(), SignError> {
        self.verify_chain(r_4ka, r_4k9, at_unix)?;
        if !f_1fy(r_4ka) {
            return Err(
                SignError::Cms(
                    "cert TSA thiếu EKU id-kp-timeStamping (không phải cert đóng dấu thời gian)"
                        .into(),
                ),
            );
        }
        Ok(())
    }
    pub fn verify_chain(
        &self,
        r_4ke: &Certificate,
        r_4kc: &[Certificate],
        at_unix: i64,
    ) -> Result<(), SignError> {
        if self.roots.is_empty() {
            return Err(SignError::Cms(
                "trust store chưa được cấu hình".into(),
            ));
        }
        f_ye(r_4ke, at_unix, "cert người ký")?;
        f_yc(r_4ke)?;
        let mut r_4kb = r_4ke.clone();
        for depth in 0..MAX_DEPTH {
            if self.is_root(&r_4kb) {
                return Ok(());
            }
            let r_4kd = self
                .find_issuer(&r_4kb, r_4kc)
                .ok_or_else(|| {
                    SignError::Cms(
                        format!(
                            "không tìm được cấp phát tin cậy cho '{}' (độ sâu {depth})",
                            r_4kb.tbs_certificate.subject
                        ),
                    )
                })?;
            f_ye(&r_4kd, at_unix, "cert cấp phát")?;
            if self.is_root(&r_4kd) {
                return Ok(());
            }
            f_y2(&r_4kd)?;
            r_4kb = r_4kd;
        }
        Err(
            SignError::Cms(
                format!(
                    "chuỗi chứng thư vượt độ sâu tối đa {MAX_DEPTH} mà chưa tới root tin cậy"
                ),
            ),
        )
    }
    fn find_issuer(&self, cert: &Certificate, r_4kg: &[Certificate]) -> Option<Certificate> {
        self.roots
            .iter()
            .chain(r_4kg.iter())
            .find(|r_4kf| {
                r_4kf.tbs_certificate.subject == cert.tbs_certificate.issuer
                    && f_2fb(cert, r_4kf).unwrap_or(false)
            })
            .cloned()
    }
}
pub(crate) fn f_1j4(r_4kh: &Certificate, r_4ki: &Certificate) -> bool {
    r_4ki.tbs_certificate.subject == r_4kh.tbs_certificate.issuer
        && f_2fb(r_4kh, r_4ki).unwrap_or(false)
}
fn f_2fb(r_4kj: &Certificate, r_4kk: &Certificate) -> Result<bool, SignError> {
    if r_4kj.signature_algorithm.oid != C_EZ {
        return Err(
            SignError::Cms(
                format!(
                    "thuật toán ký cert không hỗ trợ: {} (chỉ sha256WithRSA)",
                    r_4kj.signature_algorithm.oid
                ),
            ),
        );
    }
    let r_4kl = r_4kj.tbs_certificate.to_der().map_err(SignError::der)?;
    crate::cms::f_2pw(r_4kk, &r_4kl, r_4kj.signature.raw_bytes())
}
fn f_ye(cert: &Certificate, at_unix: i64, what: &str) -> Result<(), SignError> {
    let nb = cert
        .tbs_certificate
        .validity
        .not_before
        .to_unix_duration()
        .as_secs() as i64;
    let na = cert
        .tbs_certificate
        .validity
        .not_after
        .to_unix_duration()
        .as_secs() as i64;
    if at_unix < nb {
        return Err(
            SignError::Cms(
                format!(
                    "{what} chưa có hiệu lực tại thời điểm ký ({at_unix} < {nb})"
                ),
            ),
        );
    }
    if at_unix > na {
        return Err(SignError::Cms(format!(
            "{what} đã hết hạn tại thời điểm ký ({at_unix} > {na})"
        )));
    }
    Ok(())
}
fn f_yc(cert: &Certificate) -> Result<(), SignError> {
    let Some(r_4kn) = f_1cv(cert, C_EU) else {
        return Ok(());
    };
    let r_4ko = r_4kn;
    let r_4km = r_4ko.get(3).copied().unwrap_or(0);
    if r_4km & 0b1100_0000 == 0 {
        return Err(
            SignError::Cms(
                "cert người ký không có KeyUsage digitalSignature/nonRepudiation"
                    .into(),
            ),
        );
    }
    Ok(())
}
fn f_y2(cert: &Certificate) -> Result<(), SignError> {
    let Some(r_4kp) = f_1cv(cert, C_EO) else {
        return Err(SignError::Cms(
            "cert trung gian thiếu basicConstraints (không phải CA)"
                .into(),
        ));
    };
    let r_4kq = r_4kp.windows(3).any(|r_4kr| r_4kr == [0x01, 0x01, 0xFF]);
    if !r_4kq {
        return Err(SignError::Cms(
            "cert trung gian có basicConstraints nhưng CA=FALSE".into(),
        ));
    }
    Ok(())
}
fn f_1fy(cert: &Certificate) -> bool {
    let Some(r_4kt) = f_1cv(cert, C_ET) else {
        return false;
    };
    match ExtendedKeyUsage::from_der(&r_4kt) {
        Ok(r_4ks) => r_4ks.0.contains(&C_EW),
        Err(_) => false,
    }
}
fn f_1cv(cert: &Certificate, r_4kv: const_oid::ObjectIdentifier) -> Option<Vec<u8>> {
    let r_4ku = cert.tbs_certificate.extensions.as_ref()?;
    r_4ku
        .iter()
        .find(|e| e.extn_id == r_4kv)
        .map(|e| e.extn_value.as_bytes().to_vec())
}
fn f_21z(r_4l3: &str) -> Vec<Vec<u8>> {
    const BEGIN: &str = "-----BEGIN CERTIFICATE-----";
    const END: &str = "-----END CERTIFICATE-----";
    let mut r_4l2 = Vec::new();
    let mut r_4l4 = r_4l3;
    while let Some(r_4l0) = r_4l4.find(BEGIN) {
        let r_4kw = &r_4l4[r_4l0 + BEGIN.len()..];
        let Some(r_4l1) = r_4kw.find(END) else { break };
        let r_4kx: String = r_4kw[..r_4l1]
            .chars()
            .filter(|r_4ky| !r_4ky.is_whitespace())
            .collect();
        if let Some(r_4kz) = f_u1(&r_4kx) {
            r_4l2.push(r_4kz);
        }
        r_4l4 = &r_4kw[r_4l1 + END.len()..];
    }
    r_4l2
}
fn f_u1(r_4l9: &str) -> Option<Vec<u8>> {
    fn val(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a') as u32 + 26),
            b'0'..=b'9' => Some((c - b'0') as u32 + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }
    let b = r_4l9.as_bytes();
    if !b.len().is_multiple_of(4) {
        return None;
    }
    let mut r_4l7 = Vec::with_capacity(b.len() / 4 * 3);
    for c in b.as_chunks::<4>().0 {
        let r_4l8 = c.iter().filter(|&&r_4la| r_4la == b'=').count();
        if r_4l8 > 2 {
            return None;
        }
        let mut r_4l5 = 0u32;
        for &r_4l6 in c {
            r_4l5 = (r_4l5 << 6) | if r_4l6 == b'=' { 0 } else { val(r_4l6)? };
        }
        r_4l7.push((r_4l5 >> 16) as u8);
        if r_4l8 < 2 {
            r_4l7.push((r_4l5 >> 8) as u8);
        }
        if r_4l8 < 1 {
            r_4l7.push(r_4l5 as u8);
        }
    }
    Some(r_4l7)
}
