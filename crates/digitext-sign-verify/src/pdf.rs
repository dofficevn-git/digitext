// Copyright (c) 2026 DigiText. All rights reserved.
use crate::cms;
use crate::SignError;
use der::Decode;
use lopdf::{dictionary, Document, Object, StringFormat};
use sha2::{Digest, Sha256};
use x509_cert::Certificate;
const CONTENTS_LEN: usize = 16384;
#[derive(Debug, Clone)]
pub struct Thg {
    pub pdf: Vec<u8>,
    pub tbs_digest: Vec<u8>,
    pub signed_attrs_der: Vec<u8>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tpr {
    pub signature_ok: bool,
    pub digest_ok: bool,
    pub cert_binding_ok: bool,
    pub signer_chain_ok: bool,
    pub timestamp_present: bool,
    pub timestamp_structural_ok: bool,
    pub timestamp_trusted: bool,
    pub timestamp_time_unix: Option<i64>,
    pub has_dss: bool,
    pub revocation_ok: bool,
    pub revocation_trusted: bool,
    pub revocation_reference_unix: Option<i64>,
    pub has_archive_timestamp: bool,
    pub archive_timestamp_trusted: bool,
    pub archive_time_unix: Option<i64>,
}
impl Tpr {
    pub fn is_valid(&self) -> bool {
        self.signature_ok && self.digest_ok && self.cert_binding_ok
    }
    pub fn is_lt(&self) -> bool {
        self.is_valid()
            && self.signer_chain_ok
            && self.has_dss
            && self.revocation_ok
            && self.revocation_trusted
    }
    pub fn is_lta(&self) -> bool {
        self.is_lt() && self.has_archive_timestamp && self.archive_timestamp_trusted
    }
    pub fn is_lta_structural(&self) -> bool {
        self.is_valid()
            && self.signer_chain_ok
            && self.has_dss
            && self.revocation_ok
            && self.has_archive_timestamp
    }
}
pub fn prepare(pdf: &[u8], r_4vi: &[u8], r_4vj: u64) -> Result<Thg, SignError> {
    f_23p(pdf, r_4vi, r_4vj, None)
}
pub fn f_23p(pdf: &[u8], r_4vw: &[u8], r_4vx: u64, r_4vz: Option<&Tpv>) -> Result<Thg, SignError> {
    let cert = Certificate::from_der(r_4vw).map_err(SignError::der)?;
    let r_4w0 = f_1ig(pdf, r_4vz)?;
    let (r_4vv, r_4vu) = f_2f8(&r_4w0)?;
    let (r_4vr, r_4vq) = f_1cu(&r_4w0, r_4vv, r_4vu).ok_or_else(|| {
        SignError::Pdf("a2jDtG5nIHRo4bqleSDDtCAvQ29udGVudHMgc2F1IGtoaSBjaMOobg==".into())
    })?;
    let (r_4vm, r_4vn) = f_ws(&r_4w0, r_4vv, r_4vu).ok_or_else(|| {
        SignError::Pdf("a2jDtG5nIHRo4bqleSAvQnl0ZVJhbmdlIHNhdSBraGkgY2jDqG4=".into())
    })?;
    let r_4vy = r_4w0.len();
    let r_4vs = r_4vr - 1;
    let r_4vk = r_4vq + 1;
    let r_4vo = [0usize, r_4vs, r_4vk, r_4vy - r_4vk];
    let mut r_4vp = r_4w0;
    f_2rp(&mut r_4vp, r_4vm, r_4vn, r_4vo)?;
    let r_4vt = f_15c(&r_4vp, r_4vo).ok_or_else(|| {
        SignError::Pdf(
            "Qnl0ZVJhbmdlIHbhu6thIGThu7FuZyBs4bqhaSBuZ2/DoGkgYmnDqm4gKGJ1ZyBu4buZaSBi4buZKQ=="
                .into(),
        )
    })?;
    let r_4vl = cms::f_w2(&r_4vt, r_4vx, &cert)?;
    let signed_attrs_der = cms::f_2fa(&r_4vl)?;
    let tbs_digest = Sha256::digest(&signed_attrs_der).to_vec();
    Ok(Thg {
        pdf: r_4vp,
        tbs_digest,
        signed_attrs_der,
    })
}
pub fn finalize(r_4w1: &Thg, r_4w2: &[u8], r_4w3: &[u8]) -> Result<Vec<u8>, SignError> {
    f_1cm(&r_4w1.pdf, &r_4w1.signed_attrs_der, r_4w2, r_4w3)
}
pub fn f_1cm(
    r_4w4: &[u8],
    signed_attrs_der: &[u8],
    r_4w5: &[u8],
    r_4w6: &[u8],
) -> Result<Vec<u8>, SignError> {
    f_1co(r_4w4, signed_attrs_der, r_4w5, r_4w6, None)
}
pub fn f_1co(
    r_4w7: &[u8],
    signed_attrs_der: &[u8],
    r_4w8: &[u8],
    r_4w9: &[u8],
    r_4wa: Option<&[u8]>,
) -> Result<Vec<u8>, SignError> {
    f_1cn(r_4w7, signed_attrs_der, r_4w8, r_4w9, &[], r_4wa)
}
pub fn f_1cn(
    r_4wf: &[u8],
    signed_attrs_der: &[u8],
    r_4wg: &[u8],
    r_4wh: &[u8],
    r_4wd: &[Vec<u8>],
    r_4wi: Option<&[u8]>,
) -> Result<Vec<u8>, SignError> {
    let cert = Certificate::from_der(r_4wh).map_err(SignError::der)?;
    let r_4we: Vec<Certificate> = r_4wd
        .iter()
        .map(|d| Certificate::from_der(d).map_err(SignError::der))
        .collect::<Result<_, _>>()?;
    let r_4wb = der::asn1::SetOfVec::<x509_cert::attr::Attribute>::from_der(signed_attrs_der)
        .map_err(SignError::der)?;
    let r_4wc = cms::f_se(r_4wb, r_4wg, &cert, &r_4we, r_4wi)?;
    f_18u(r_4wf, &r_4wc)
}
fn f_18u(r_4wm: &[u8], r_4wj: &[u8]) -> Result<Vec<u8>, SignError> {
    let (r_4wn, r_4wk) = f_1ct(r_4wm).ok_or_else(|| {
        SignError::Pdf(
            "a2jDtG5nIHRo4bqleSDDtCAvQ29udGVudHMgdHJvbmcgUERGIMSRw6MgY2h14bqpbiBi4buL".into(),
        )
    })?;
    let hex = f_1gv(r_4wj);
    if hex.len() > r_4wk - r_4wn {
        return Err(SignError::Pdf(format!(
            "CMS {}IGJ5dGUgdsaw4bujdCDDtCAvQ29udGVudHMg{}IGJ5dGUgKHTEg25nIENPTlRFTlRTX0xFTik=",
            r_4wj.len(),
            (r_4wk - r_4wn) / 2
        )));
    }
    let mut r_4wl = r_4wm.to_vec();
    for b in &mut r_4wl[r_4wn..r_4wk] {
        *b = b'0';
    }
    r_4wl[r_4wn..r_4wn + hex.len()].copy_from_slice(hex.as_bytes());
    Ok(r_4wl)
}
pub fn f_xy(r_4wo: &[u8]) -> Result<(), SignError> {
    Certificate::from_der(r_4wo)
        .map(|_| ())
        .map_err(SignError::der)
}
pub fn verify(r_4wp: &[u8]) -> Result<Tpr, SignError> {
    f_2q1(r_4wp, None, None)
}
pub fn f_2q2(r_4wq: &[u8], r_4wr: Option<&crate::chain::TrustStore>) -> Result<Tpr, SignError> {
    f_2q1(r_4wq, None, r_4wr)
}
pub fn f_2q1(
    r_4x7: &[u8],
    r_4x8: Option<&crate::chain::TrustStore>,
    r_4xa: Option<&crate::chain::TrustStore>,
) -> Result<Tpr, SignError> {
    let (r_4x2, r_4x1) = f_2f8(r_4x7)?;
    let (r_4wx, r_4ww) = f_1cu(r_4x7, r_4x2, r_4x1).ok_or_else(|| {
        SignError::Pdf("a2jDtG5nIHRo4bqleSDDtCAvQ29udGVudHMgY+G7p2EgY2jhu68ga8O9".into())
    })?;
    let r_4wt = f_ws(r_4x7, r_4x2, r_4x1)
        .and_then(|(r_4wz, r_4x3)| f_1yg(r_4x7, r_4wz, r_4x3))
        .ok_or_else(|| {
            SignError::Pdf(
                "a2jDtG5nIMSR4buNYyDEkcaw4bujYyAvQnl0ZVJhbmdlIGPhu6dhIGNo4buvIGvDvQ==".into(),
            )
        })?;
    let r_4x9 = r_4x7.len();
    let r_4wy = if f_wo(r_4wt, r_4wx, r_4ww, r_4x9) {
        None
    } else if f_wp(r_4wt, r_4wx, r_4ww, r_4x9) {
        let r_4x6 = r_4wt[2] + r_4wt[3];
        Some(
            crate::dss::f_2p3(r_4x7, r_4x6, r_4xa)
                .map_err(|e| {
                    SignError::Pdf(
                        format!(
                            "xJF1w7RpIHNhdSBjaOG7ryBrw70ga2jDtG5nIHBo4bqjaSBpbmNyZW1lbnQgRFNTL0RvY1RpbWVTdGFtcCBo4bujcCBs4buHOiA={e}"
                        ),
                    )
                })?,
        )
    } else {
        return Err(
            SignError::Pdf(
                "L0J5dGVSYW5nZSBraMO0bmcgcGjhu6cgdHLhu41uIGZpbGUgKGPDsyBu4buZaSBkdW5nIG5nb8OgaSB2w7luZyBrw70g4oCUIFBERiDEkcOjIGLhu4sgc+G7rWEgc2F1IGtoaSBrw70p"
                    .into(),
            ),
        );
    };
    let has_dss = r_4wy.as_ref().map(|r| r.has_dss).unwrap_or(false);
    let r_4wv = r_4wy.as_ref().filter(|r| r.has_dss).map(|r| &r.material);
    let has_archive_timestamp = r_4wy
        .as_ref()
        .map(|r| r.archive_present && r.archive_structural_ok)
        .unwrap_or(false);
    let archive_timestamp_trusted = r_4wy
        .as_ref()
        .map(|r| r.archive_present && r.archive_timestamp_trusted)
        .unwrap_or(false);
    let archive_time_unix = r_4wy.as_ref().and_then(|r| r.archive_time_unix);
    let r_4wu = f_1gu(&r_4x7[r_4wx..r_4ww])
        .ok_or_else(|| SignError::Pdf("L0NvbnRlbnRzIGtow7RuZyBwaOG6o2kgaGV4".into()))?;
    let r_4x0 = f_15c(r_4x7, r_4wt)
        .ok_or_else(|| SignError::Pdf("L0J5dGVSYW5nZSB0cuG7jyByYSBuZ2/DoGkgZmlsZQ==".into()))?;
    let r_4xb = cms::f_2pr(&r_4wu, &r_4x0, r_4x8, r_4xa)?;
    let (r_4x4, r_4x5) = f_29x(
        r_4xb.timestamp_trusted,
        r_4xb.timestamp_time_unix,
        archive_timestamp_trusted,
        archive_time_unix,
        r_4xb.signing_time_unix,
    );
    let revocation_ok = match (r_4wv, r_4x4) {
        (Some(r_4xc), Some(r_4ws)) if !r_4xc.is_empty() => {
            f_29w(&r_4wu, r_4xc, r_4ws).unwrap_or(false)
        }
        _ => false,
    };
    Ok(Tpr {
        signature_ok: r_4xb.signature_ok,
        digest_ok: r_4xb.digest_ok,
        cert_binding_ok: r_4xb.cert_binding_ok,
        signer_chain_ok: r_4xb.signer_chain_ok,
        timestamp_present: r_4xb.timestamp_present,
        timestamp_structural_ok: r_4xb.timestamp_structural_ok,
        timestamp_trusted: r_4xb.timestamp_trusted,
        timestamp_time_unix: r_4xb.timestamp_time_unix,
        has_dss,
        revocation_ok,
        revocation_trusted: revocation_ok && r_4x5,
        revocation_reference_unix: r_4x4,
        has_archive_timestamp,
        archive_timestamp_trusted,
        archive_time_unix,
    })
}
fn f_29x(
    timestamp_trusted: bool,
    timestamp_time_unix: Option<i64>,
    r_4xd: bool,
    archive_time_unix: Option<i64>,
    r_4xe: Option<i64>,
) -> (Option<i64>, bool) {
    if let (true, Some(t)) = (timestamp_trusted, timestamp_time_unix) {
        return (Some(t), true);
    }
    if let (true, Some(t)) = (r_4xd, archive_time_unix) {
        return (Some(t), true);
    }
    (r_4xe, false)
}
fn f_29w(
    r_4xg: &[u8],
    r_4xl: &crate::dss::ValidationMaterial,
    r_4xi: i64,
) -> Result<bool, SignError> {
    let r_4xj = cms::f_1zy(r_4xg)?;
    let r_4xk = cms::signer_cert(&r_4xj)?;
    let mut r_4xf = cms::f_rl(&r_4xj);
    for r_4xh in &r_4xl.certs {
        if let Ok(c) = Certificate::from_der(r_4xh) {
            r_4xf.push(c);
        }
    }
    let issuer = r_4xf
        .iter()
        .find(|c| crate::chain::f_1j4(&r_4xk, c))
        .ok_or_else(|| {
            SignError::Cms(
                "a2jDtG5nIHTDrG0gxJHGsOG7o2MgaXNzdWVyIGPhu6dhIGNlcnQgbmfGsOG7nWkga8O9IGNobyBPQ1NQL0NSTA=="
                    .into(),
            )
        })?;
    Ok(crate::ocsp::f_1ab(
        issuer,
        &r_4xk,
        &r_4xl.ocsps,
        &r_4xl.crls,
        r_4xi,
    ))
}
const C_K5: f32 = 14.0;
const C_PN: i64 = 7 * 3600;
pub fn f_1di(r_4xt: u64) -> String {
    let r_4xq = r_4xt as i64 + C_PN;
    let r_4xm = r_4xq.div_euclid(86_400);
    let r_4xs = r_4xq.rem_euclid(86_400);
    let (h, mi, s) = (r_4xs / 3600, (r_4xs % 3600) / 60, r_4xs % 60);
    let r_4xv = r_4xm + 719_468;
    let r_4xp = r_4xv.div_euclid(146_097);
    let r_4xn = r_4xv.rem_euclid(146_097);
    let r_4xu = (r_4xn - r_4xn / 1460 + r_4xn / 36_524 - r_4xn / 146_096) / 365;
    let y = r_4xu + r_4xp * 400;
    let r_4xo = r_4xn - (365 * r_4xu + r_4xu / 4 - r_4xu / 100);
    let r_4xr = (5 * r_4xo + 2) / 153;
    let d = r_4xo - (153 * r_4xr + 2) / 5 + 1;
    let m = if r_4xr < 10 { r_4xr + 3 } else { r_4xr - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}+07:00")
}
pub fn f_2fg(cert: &Certificate) -> String {
    let r_4y0 = &cert.tbs_certificate.subject;
    let r_4xy = |r_4xx: const_oid::ObjectIdentifier| -> Option<String> {
        r_4y0.0.iter().find_map(|r_4xz| {
            r_4xz.0.iter().find_map(|r_4xw| {
                (r_4xw.oid == r_4xx).then(|| {
                    r_4xw
                        .value
                        .decode_as::<der::asn1::Utf8StringRef<'_>>()
                        .map(|s| s.as_str().to_string())
                        .or_else(|_| {
                            r_4xw
                                .value
                                .decode_as::<der::asn1::PrintableStringRef<'_>>()
                                .map(|s| s.as_str().to_string())
                        })
                        .unwrap_or_default()
                })
            })
        })
    };
    const CN: const_oid::ObjectIdentifier = const_oid::ObjectIdentifier::new_unwrap("2.5.4.3");
    const O: const_oid::ObjectIdentifier = const_oid::ObjectIdentifier::new_unwrap("2.5.4.10");
    r_4xy(CN)
        .filter(|s| !s.is_empty())
        .or_else(|| r_4xy(O).filter(|s| !s.is_empty()))
        .unwrap_or_else(|| r_4y0.to_string())
}
pub fn f_2f7(cert: &Certificate, r_4y1: u64) -> Vec<String> {
    vec![
        format!("S8O9IGLhu59pOiA={}", f_2fg(cert)),
        format!("S8O9IG5nw6B5OiA={}", f_1di(r_4y1)),
    ]
}
pub fn f_2fh(cert: &Certificate) -> Option<String> {
    const C_JV: const_oid::ObjectIdentifier = const_oid::ObjectIdentifier::new_unwrap("2.5.4.5");
    const C_OW: const_oid::ObjectIdentifier =
        const_oid::ObjectIdentifier::new_unwrap("0.9.2342.19200300.100.1.1");
    let r_4y3 = cert.tbs_certificate.subject.0.iter().find_map(|r_4y4| {
        r_4y4.0.iter().find_map(|r_4y2| {
            (r_4y2.oid == C_JV || r_4y2.oid == C_OW)
                .then(|| {
                    r_4y2
                        .value
                        .decode_as::<der::asn1::Utf8StringRef<'_>>()
                        .map(|s| s.as_str().to_string())
                        .or_else(|_| {
                            r_4y2
                                .value
                                .decode_as::<der::asn1::PrintableStringRef<'_>>()
                                .map(|s| s.as_str().to_string())
                        })
                        .ok()
                })
                .flatten()
        })
    })?;
    let r_4y5 = r_4y3.trim().trim_start_matches("MST:").trim().to_string();
    (!r_4y5.is_empty()).then_some(r_4y5)
}
pub fn f_2qb(r_4y8: &[u8], r_4y9: u64, stamp: bool) -> Option<Tpv> {
    let cert = Certificate::from_der(r_4y8).ok()?;
    let lines = f_2f7(&cert, r_4y9);
    let r_4yb = stamp.then(|| Tly {
        org: f_2fg(&cert),
        center_lines: f_2fh(&cert)
            .map(|mst| vec![format!("MST: {mst}")])
            .unwrap_or_default(),
    });
    let mut r_4y7 = lines.join(" ");
    if let Some(r_4ya) = &r_4yb {
        r_4y7.push(' ');
        r_4y7.push_str(&r_4ya.org);
        r_4y7.push_str(&r_4ya.center_lines.join(" "));
    }
    let r_4y6 = digitext_pdf::f_1cr(&r_4y7)?;
    let font_ttf = std::fs::read(&r_4y6)
        .map_err(|e| {
            tracing::warn!(
                path = ? r_4y6, error = % e,
                "a2jDtG5nIMSR4buNYyDEkcaw4bujYyBmb250IGto4buRaSBjaOG7ryBrw70="
            )
        })
        .ok()?;
    Some(Tpv {
        lines,
        font_ttf,
        stamp: r_4yb,
    })
}
#[derive(Debug, Clone)]
pub struct Tpv {
    pub lines: Vec<String>,
    pub font_ttf: Vec<u8>,
    pub stamp: Option<Tly>,
}
#[derive(Debug, Clone)]
pub struct Tly {
    pub org: String,
    pub center_lines: Vec<String>,
}
fn f_1s2(r_4yd: &Document, r_4yi: lopdf::ObjectId) -> Option<[f32; 4]> {
    let r_4yg = |o: &Object| {
        o.as_float()
            .ok()
            .or(o.as_i64().ok().map(|r_4yj| r_4yj as f32))
    };
    let mut r_4yf = r_4yi;
    for _ in 0..8 {
        let dict = r_4yd.get_dictionary(r_4yf).ok()?;
        if let Ok(r_4yc) = dict.get(b"MediaBox").and_then(|o| match o {
            Object::Reference(r_4ye) => {
                r_4yd.get_object(*r_4ye).and_then(|o| o.as_array()).cloned()
            }
            r_4yh => r_4yh.as_array().cloned(),
        }) {
            if r_4yc.len() == 4 {
                return Some([
                    r_4yg(&r_4yc[0])?,
                    r_4yg(&r_4yc[1])?,
                    r_4yg(&r_4yc[2])?,
                    r_4yg(&r_4yc[3])?,
                ]);
            }
        }
        r_4yf = dict.get(b"Parent").ok()?.as_reference().ok()?;
    }
    None
}
fn f_1ig(pdf: &[u8], r_4z2: Option<&Tpv>) -> Result<Vec<u8>, SignError> {
    let mut r_4yq =
        Document::load_mem(pdf).map_err(|e| SignError::Pdf(format!("buG6oXAgUERGOiA={e}")))?;
    let r_4yw = *r_4yq
        .get_pages()
        .values()
        .next()
        .ok_or_else(|| SignError::Pdf("UERGIGtow7RuZyBjw7MgdHJhbmc=".into()))?;
    let r_4ym = match r_4z2 {
        Some(r_4yn) => {
            let r_4yo = match &r_4yn.stamp {
                Some(r_4z1) => digitext_pdf::f_w1(
                    &mut r_4yq,
                    &digitext_pdf::Tlz {
                        org: r_4z1.org.clone(),
                        center_lines: r_4z1.center_lines.clone(),
                        side_lines: r_4yn.lines.clone(),
                        color: digitext_pdf::Tlz::RED,
                    },
                    &r_4yn.font_ttf,
                ),
                None => digitext_pdf::f_w0(&mut r_4yq, &r_4yn.lines, &r_4yn.font_ttf),
            };
            match r_4yo {
                Ok((form_id, r_4z3, h)) => f_1s2(&r_4yq, r_4yw).map(|r_4yt| {
                    let r_4z4 = r_4yt[2] - C_K5;
                    let r_4z5 = r_4yt[3] - C_K5;
                    (form_id, [r_4z4 - r_4z3, r_4z5 - h, r_4z4, r_4z5])
                }),
                Err(e) => {
                    tracing::warn!(
                        error = % e,
                        "a2jDtG5nIGThu7FuZyDEkcaw4bujYyBraOG7kWkgY2jhu68ga8O9IGhp4buDbiB0aOG7iyDigJQga8O9IHbDtCBow6xuaA=="
                    );
                    None
                }
            }
        }
        None => None,
    };
    let r_4yz = dictionary! {
        "Type" => "Sig", "Filter" => "Adobe.PPKLite", "SubFilter" =>
        "ETSI.CAdES.detached", "ByteRange" => vec![Object::Integer(0),
        Object::Integer(9_999_999_999), Object::Integer(9_999_999_999),
        Object::Integer(9_999_999_999),], "Contents" => Object::String(vec![0u8;
        CONTENTS_LEN], StringFormat::Hexadecimal),
    };
    let r_4z0 = r_4yq.add_object(r_4yz);
    let mut r_4yr = dictionary! {
        "Type" => "Annot", "Subtype" => "Widget", "FT" => "Sig", "T" =>
        Object::String(b"DigiTextSig".to_vec(), StringFormat::Literal), "Rect" => vec![0
        .into(), 0.into(), 0.into(), 0.into()], "F" => Object::Integer(132), "P" =>
        r_4yw, "V" => r_4z0,
    };
    if let Some((form_id, r_4yx)) = r_4ym {
        r_4yr.set(
            "Rect",
            vec![
                r_4yx[0].into(),
                r_4yx[1].into(),
                r_4yx[2].into(),
                r_4yx[3].into(),
            ],
        );
        r_4yr.set(
            "AP",
            dictionary! {
                "N" => form_id
            },
        );
        r_4yr.set("F", Object::Integer(132));
    }
    let r_4ys = r_4yq.add_object(r_4yr);
    if let Ok(Object::Dictionary(d)) = r_4yq.get_object_mut(r_4yw) {
        match d.get_mut(b"Annots") {
            Ok(Object::Array(r_4yk)) => r_4yk.push(r_4ys.into()),
            _ => d.set("Annots", vec![Object::from(r_4ys)]),
        }
    }
    let r_4yl = r_4yq.add_object(dictionary! {
        "Fields" => vec![Object::from(r_4ys)], "SigFlags" => Object::Integer(3),
    });
    let r_4yy = r_4yq
        .trailer
        .get(b"Root")
        .ok()
        .and_then(|r_4yu| r_4yu.as_reference().ok())
        .ok_or_else(|| SignError::Pdf("dGhp4bq/dSAvUm9vdA==".into()))?;
    if let Ok(Object::Dictionary(r_4yp)) = r_4yq.get_object_mut(r_4yy) {
        r_4yp.set("AcroForm", r_4yl);
    }
    let mut r_4yv = Vec::new();
    r_4yq
        .save_to(&mut r_4yv)
        .map_err(|e| SignError::Pdf(format!("ghi PDF: {e}")))?;
    Ok(r_4yv)
}
fn f_2f8(r_4z7: &[u8]) -> Result<(usize, usize), SignError> {
    let r_4z8 =
        Document::load_mem(r_4z7).map_err(|e| SignError::Pdf(format!("buG6oXAgUERGOiA={e}")))?;
    let r_4ze = r_4z8
        .trailer
        .get(b"Root")
        .ok()
        .and_then(|o| o.as_reference().ok())
        .ok_or_else(|| SignError::Pdf("dGhp4bq/dSAvUm9vdA==".into()))?;
    let catalog = r_4z8
        .get_object(r_4ze)
        .and_then(|o| o.as_dict())
        .map_err(|e| SignError::Pdf(format!("xJHhu41jIGNhdGFsb2c6IA=={e}")))?;
    let r_4z6 = catalog
        .get(b"AcroForm")
        .ok()
        .and_then(|o| o.as_reference().ok())
        .ok_or_else(|| SignError::Pdf("dGhp4bq/dSAvQWNyb0Zvcm0=".into()))?;
    let r_4zd = r_4z8
        .get_object(r_4z6)
        .and_then(|o| o.as_dict())
        .and_then(|d| d.get(b"Fields"))
        .and_then(|o| o.as_array())
        .map_err(|e| SignError::Pdf(format!("xJHhu41jIEFjcm9Gb3JtL0ZpZWxkczog{e}")))?;
    let mut sig_id = None;
    for r_4za in r_4zd {
        let Ok(r_4zc) = r_4za.as_reference() else {
            continue;
        };
        let Ok(r_4zb) = r_4z8.get_object(r_4zc).and_then(|o| o.as_dict()) else {
            continue;
        };
        if r_4zb.get(b"FT").and_then(|o| o.as_name()).ok() != Some(b"Sig".as_ref()) {
            continue;
        }
        if let Ok(r_4zg) = r_4zb.get(b"V").and_then(|o| o.as_reference()) {
            sig_id = Some(r_4zg);
            break;
        }
    }
    let sig_id = sig_id.ok_or_else(|| {
        SignError::Pdf("a2jDtG5nIHRo4bqleSB0csaw4budbmcgY2jhu68ga8O9IChGVD1TaWcp".into())
    })?;
    let r_4zf = match r_4z8.reference_table.entries.get(&sig_id.0) {
        Some(lopdf::xref::XrefEntry::Normal { offset, .. }) => *offset as usize,
        _ => {
            return Err(
                SignError::Pdf(
                    "ZGljdCBjaOG7ryBrw70ga2jDtG5nIGPDsyBt4bulYyB4cmVmIHRoxrDhu51uZyAob2JqZWN0IHN0cmVhbT8p"
                        .into(),
                ),
            );
        }
    };
    if r_4zf >= r_4z7.len() {
        return Err(SignError::Pdf(
            "b2Zmc2V0IHhyZWYgY+G7p2EgY2jhu68ga8O9IG5nb8OgaSBmaWxl".into(),
        ));
    }
    let r_4z9 = r_4zf
        + find(&r_4z7[r_4zf..], b"endobj").ok_or_else(|| {
            SignError::Pdf("a2jDtG5nIHRo4bqleSBlbmRvYmogY+G7p2EgZGljdCBjaOG7ryBrw70=".into())
        })?;
    Ok((r_4zf, r_4z9))
}
fn f_1ct(r_4zh: &[u8]) -> Option<(usize, usize)> {
    let (r_4zj, r_4zi) = f_2f8(r_4zh).ok()?;
    f_1cu(r_4zh, r_4zj, r_4zi)
}
fn f_1cu(r_4zm: &[u8], r_4zq: usize, r_4zo: usize) -> Option<(usize, usize)> {
    let r_4zr = r_4zm.get(r_4zq..r_4zo)?;
    let mut r_4zl = 0;
    while let Some(r_4zs) = find(&r_4zr[r_4zl..], b"/Contents") {
        let r_4zk = r_4zl + r_4zs + b"/Contents".len();
        let mut r_4zp = r_4zk;
        while r_4zp < r_4zr.len() && r_4zr[r_4zp].is_ascii_whitespace() {
            r_4zp += 1;
        }
        if r_4zp < r_4zr.len() && r_4zr[r_4zp] == b'<' {
            let r_4zn = r_4zp + 1 + find(&r_4zr[r_4zp + 1..], b">")?;
            return Some((r_4zq + r_4zp + 1, r_4zq + r_4zn));
        }
        r_4zl = r_4zk;
    }
    None
}
fn f_2rp(r_4zu: &mut [u8], r_4zv: usize, r_4zw: usize, r_4zt: [usize; 4]) -> Result<(), SignError> {
    let r_4zx = r_4zw - r_4zv + 1;
    let mut s = format!("[{} {} {} {}", r_4zt[0], r_4zt[1], r_4zt[2], r_4zt[3]);
    if s.len() + 1 > r_4zx {
        return Err(SignError::Pdf(
            "Z2nDoSB0cuG7iyBCeXRlUmFuZ2UgZMOgaSBoxqFuIHBsYWNlaG9sZGVy".into(),
        ));
    }
    while s.len() + 1 < r_4zx {
        s.push(' ');
    }
    s.push(']');
    r_4zu[r_4zv..=r_4zw].copy_from_slice(s.as_bytes());
    Ok(())
}
pub(crate) fn f_1yg(r_4zy: &[u8], r_500: usize, r_502: usize) -> Option<[usize; 4]> {
    let r_4zz = core::str::from_utf8(r_4zy.get(r_500 + 1..r_502)?).ok()?;
    let r_501: Vec<usize> = r_4zz
        .split_whitespace()
        .filter_map(|r_503| r_503.parse::<usize>().ok())
        .collect();
    if r_501.len() == 4 {
        Some([r_501[0], r_501[1], r_501[2], r_501[3]])
    } else {
        None
    }
}
fn f_ws(r_504: &[u8], r_507: usize, r_505: usize) -> Option<(usize, usize)> {
    let r_50a = r_504.get(r_507..r_505)?;
    let r_508 = find(r_50a, b"/ByteRange")?;
    let r_506 = r_508 + find(&r_50a[r_508..], b"[")?;
    let r_509 = r_506 + 1 + find(&r_50a[r_506 + 1..], b"]")?;
    Some((r_507 + r_506, r_507 + r_509))
}
pub(crate) fn f_wo(r_50b: [usize; 4], r_50d: usize, r_50c: usize, r_50f: usize) -> bool {
    let Some(r_50e) = r_50b[2].checked_add(r_50b[3]) else {
        return false;
    };
    r_50b[0] == 0 && f_1e2(r_50b, r_50d, r_50c) && r_50e == r_50f
}
fn f_1e2(r_50g: [usize; 4], r_50j: usize, r_50i: usize) -> bool {
    let Some(r_50m) = r_50g[0].checked_add(r_50g[1]) else {
        return false;
    };
    let r_50h = r_50j
        .checked_sub(1)
        .is_some_and(|r_50l| r_50m == r_50l && r_50g[2] == r_50i + 1);
    let r_50k = r_50m == r_50j && r_50g[2] == r_50i;
    r_50h || r_50k
}
fn f_wp(r_50n: [usize; 4], r_50p: usize, r_50o: usize, r_50r: usize) -> bool {
    let Some(r_50q) = r_50n[2].checked_add(r_50n[3]) else {
        return false;
    };
    r_50n[0] == 0 && f_1e2(r_50n, r_50p, r_50o) && r_50q <= r_50r
}
pub(crate) fn f_15c(r_50t: &[u8], r_50s: [usize; 4]) -> Option<Vec<u8>> {
    let r_50v = r_50s[0].checked_add(r_50s[1])?;
    let r_50x = r_50s[2].checked_add(r_50s[3])?;
    let r_50u = r_50t.get(r_50s[0]..r_50v)?;
    let r_50w = r_50t.get(r_50s[2]..r_50x)?;
    let mut h = Sha256::new();
    h.update(r_50u);
    h.update(r_50w);
    Some(h.finalize().to_vec())
}
fn find(r_50y: &[u8], r_50z: &[u8]) -> Option<usize> {
    r_50y.windows(r_50z.len()).position(|r_510| r_510 == r_50z)
}
pub(crate) fn f_1gv(r_511: &[u8]) -> String {
    let mut s = String::with_capacity(r_511.len() * 2);
    for b in r_511 {
        s.push_str(&format!("{b:02x}"));
    }
    s
}
pub(crate) fn f_1gu(hex: &[u8]) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    fn f_1uw(r_512: u8) -> Option<u8> {
        match r_512 {
            b'0'..=b'9' => Some(r_512 - b'0'),
            b'a'..=b'f' => Some(r_512 - b'a' + 10),
            b'A'..=b'F' => Some(r_512 - b'A' + 10),
            _ => None,
        }
    }
    let mut r_513 = Vec::with_capacity(hex.len() / 2);
    for r_514 in hex.as_chunks::<2>().0 {
        r_513.push((f_1uw(r_514[0])? << 4) | f_1uw(r_514[1])?);
    }
    Some(r_513)
}
