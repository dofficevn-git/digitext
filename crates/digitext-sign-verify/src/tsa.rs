// Copyright (c) 2026 DigiText. All rights reserved.
use crate::SignError;
use cms::content_info::ContentInfo;
use cms::signed_data::SignedData;
use const_oid::ObjectIdentifier as Oid;
use der::asn1::OctetString;
use der::Decode;
use der::{Encode, Sequence};
use sha2::{Digest, Sha256};
use spki::AlgorithmIdentifierOwned;
use x509_cert::Certificate;
const OID_SIGNED_DATA: Oid = Oid::new_unwrap("1.2.840.113549.1.7.2");
#[derive(Sequence)]
struct Tdc {
    hash_algorithm: AlgorithmIdentifierOwned,
    hashed_message: OctetString,
}
#[derive(Sequence)]
struct TimeStampReq {
    version: u8,
    message_imprint: Tdc,
    #[asn1(optional = "true")]
    nonce: Option<u64>,
    #[asn1(default = "bool_false")]
    cert_req: bool,
}
fn bool_false() -> bool {
    false
}
pub fn build_request(
    r_51i: &[u8],
    nonce: Option<u64>,
    cert_req: bool,
) -> Result<Vec<u8>, SignError> {
    let r_51g = Tdc {
        hash_algorithm: crate::cms::alg_sha256(),
        hashed_message: OctetString::new(Sha256::digest(r_51i).to_vec()).map_err(SignError::der)?,
    };
    let r_51h = TimeStampReq {
        version: 1,
        message_imprint: r_51g,
        nonce,
        cert_req,
    };
    r_51h.to_der().map_err(SignError::der)
}
pub fn f_2le(r_51k: &[u8]) -> Result<Vec<u8>, SignError> {
    let r_51j = f_2df(r_51k)
        .ok_or_else(|| SignError::Cms("VGltZVN0YW1wUmVzcCBraMO0bmcgcGjhuqNpIFNFUVVFTkNF".into()))?;
    let r_51l = f_25x(r_51j)
        .ok_or_else(|| SignError::Cms("VGltZVN0YW1wUmVzcCB0aGnhur91IFBLSVN0YXR1c0luZm8=".into()))?;
    if r_51l.tag != 0x30 {
        return Err(SignError::Cms(
            "UEtJU3RhdHVzSW5mbyBraMO0bmcgcGjhuqNpIFNFUVVFTkNF".into(),
        ));
    }
    let r_51m = f_25x(r_51l.content)
        .ok_or_else(|| SignError::Cms("UEtJU3RhdHVzSW5mbyB0aGnhur91IHN0YXR1cw==".into()))?;
    let status = f_1ij(r_51m.content)
        .ok_or_else(|| SignError::Cms("UEtJU3RhdHVzIGtow7RuZyDEkeG7jWMgxJHGsOG7o2M=".into()))?;
    if status != 0 && status != 1 {
        return Err(SignError::Cms(format!(
            "VFNBIHThu6sgY2jhu5FpIGPhuqVwIHRlbSAoUEtJU3RhdHVzPQ=={status})"
        )));
    }
    let token = f_25x(r_51l.rest).ok_or_else(|| {
        SignError::Cms("VGltZVN0YW1wUmVzcCBncmFudGVkIG5oxrBuZyB0aGnhur91IHRva2Vu".into())
    })?;
    if token.tag != 0x30 {
        return Err(SignError::Cms(
            "VGltZVN0YW1wVG9rZW4ga2jDtG5nIHBo4bqjaSBTRVFVRU5DRQ==".into(),
        ));
    }
    Ok(token.whole.to_vec())
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Too {
    pub imprint_ok: bool,
    pub gen_time_unix: Option<i64>,
    pub token_signature_ok: Option<bool>,
    pub nonce: Option<Vec<u8>>,
    pub chain_ok: Option<bool>,
}
impl Too {
    pub fn check_nonce(&self, r_51o: u64) -> bool {
        let Some(got) = self.nonce.as_deref() else {
            return false;
        };
        let got = got.strip_prefix(&[0u8]).unwrap_or(got);
        let want = r_51o.to_be_bytes();
        let want = want.strip_prefix(&[0u8]).unwrap_or(&want);
        let want: Vec<u8> = want.iter().copied().skip_while(|&b| b == 0).collect();
        let r_51n: Vec<u8> = got.iter().copied().skip_while(|&b| b == 0).collect();
        r_51n == want
    }
}
pub fn f_2py(r_51q: &[u8], r_51p: &[u8]) -> Result<Too, SignError> {
    f_2q0(r_51q, r_51p, None)
}
pub fn f_2q0(
    r_51y: &[u8],
    r_51x: &[u8],
    r_51z: Option<&crate::chain::TrustStore>,
) -> Result<Too, SignError> {
    let mut r_51u = der::SliceReader::new(r_51y).map_err(SignError::der)?;
    let r_51r = ContentInfo::decode(&mut r_51u).map_err(SignError::der)?;
    if r_51r.content_type != OID_SIGNED_DATA {
        return Err(SignError::Cms(
            "VGltZVN0YW1wVG9rZW4ga2jDtG5nIHBo4bqjaSBpZC1zaWduZWREYXRh".into(),
        ));
    }
    let r_51w: SignedData = r_51r.content.decode_as().map_err(SignError::der)?;
    let r_51s = r_51w
        .encap_content_info
        .econtent
        .as_ref()
        .ok_or_else(|| SignError::Cms("dG9rZW4gdGhp4bq/dSBlQ29udGVudCAoVFNUSW5mbyk=".into()))?;
    let r_521 = r_51s.decode_as::<OctetString>().map_err(SignError::der)?;
    let r_520 = r_521.as_bytes();
    let r_51t = f_2mq(r_520)
        .ok_or_else(|| SignError::Cms("VFNUSW5mbyB0aGnhur91IG1lc3NhZ2VJbXByaW50".into()))?;
    let imprint_ok = r_51t == Sha256::digest(r_51x).as_slice();
    let gen_time_unix = f_2mp(r_520);
    let token_signature_ok = f_2pz(&r_51w, r_520);
    let nonce = f_2mr(r_520);
    let chain_ok = r_51z.map(|r_51v| f_2mm(&r_51w, gen_time_unix, r_51v));
    Ok(Too {
        imprint_ok,
        gen_time_unix,
        token_signature_ok,
        nonce,
        chain_ok,
    })
}
fn f_2mm(r_526: &SignedData, gen_time_unix: Option<i64>, r_525: &crate::chain::TrustStore) -> bool {
    let Some(r_522) = gen_time_unix else {
        return false;
    };
    let Ok(r_527) = crate::cms::signer_cert(r_526) else {
        return false;
    };
    let r_524: Vec<Certificate> = crate::cms::f_rl(r_526)
        .into_iter()
        .filter(|r_523| r_523 != &r_527)
        .collect();
    r_525.verify_tsa_chain(&r_527, &r_524, r_522).is_ok()
}
fn f_2pz(r_52b: &SignedData, r_52e: &[u8]) -> Option<bool> {
    let r_52c = r_52b.signer_infos.0.as_slice().first()?;
    if r_52c.digest_alg.oid != crate::cms::OID_SHA256 {
        return None;
    }
    let r_529 = crate::cms::signer_cert(r_52b).ok()?;
    match &r_52c.signed_attrs {
        Some(r_528) => {
            let r_52a = crate::cms::f_1b2(r_528).ok()?;
            if r_52a != Sha256::digest(r_52e).as_slice() {
                return Some(false);
            }
            let r_52d = crate::cms::f_2fa(r_528).ok()?;
            crate::cms::f_2pw(&r_529, &r_52d, r_52c.signature.as_bytes()).ok()
        }
        None => crate::cms::f_2pw(&r_529, r_52e, r_52c.signature.as_bytes()).ok(),
    }
}
fn f_2mq(r_52i: &[u8]) -> Option<Vec<u8>> {
    let r_52g = f_2df(r_52i)?;
    let r_52h = nth(r_52g, 2)?;
    if r_52h.tag != 0x30 {
        return None;
    }
    let r_52f = nth(r_52h.content, 1)?;
    if r_52f.tag != 0x04 {
        return None;
    }
    Some(r_52f.content.to_vec())
}
fn f_2mr(r_52m: &[u8]) -> Option<Vec<u8>> {
    let r_52k = f_2df(r_52m)?;
    let mut r_52j = 5;
    while let Some(r_52l) = nth(r_52k, r_52j) {
        match r_52l.tag {
            0x02 => return Some(r_52l.content.to_vec()),
            0xA0 | 0xA1 => return None,
            _ => r_52j += 1,
        }
    }
    None
}
fn f_2mp(r_52p: &[u8]) -> Option<i64> {
    let r_52o = f_2df(r_52p)?;
    let r_52n = nth(r_52o, 4)?;
    if r_52n.tag != 0x18 {
        return None;
    }
    f_1z2(r_52n.content)
}
fn f_1z2(r_52y: &[u8]) -> Option<i64> {
    if r_52y.len() < 14 {
        return None;
    }
    let r_52w = core::str::from_utf8(&r_52y[..14]).ok()?;
    let r_52v = |r_52q: usize, b: usize| {
        r_52w
            .get(r_52q..b)
            .and_then(|r_52z| r_52z.parse::<i64>().ok())
    };
    let (r_530, r_52u, r_52r) = (r_52v(0, 4)?, r_52v(4, 6)?, r_52v(6, 8)?);
    let (r_52s, r_52t, r_52x) = (r_52v(8, 10)?, r_52v(10, 12)?, r_52v(12, 14)?);
    if !(1..=12).contains(&r_52u) || !(1..=31).contains(&r_52r) {
        return None;
    }
    Some(f_136(r_530, r_52u, r_52r) * 86_400 + r_52s * 3600 + r_52t * 60 + r_52x)
}
fn f_136(y: i64, r_535: i64, r_531: i64) -> i64 {
    let y = if r_535 <= 2 { y - 1 } else { y };
    let r_534 = (if y >= 0 { y } else { y - 399 }) / 400;
    let r_536 = y - r_534 * 400;
    let r_533 = (153 * (if r_535 > 2 { r_535 - 3 } else { r_535 + 9 }) + 2) / 5 + r_531 - 1;
    let r_532 = r_536 * 365 + r_536 / 4 - r_536 / 100 + r_533;
    r_534 * 146_097 + r_532 - 719_468
}
struct Tlv<'a> {
    tag: u8,
    content: &'a [u8],
    whole: &'a [u8],
    rest: &'a [u8],
}
fn f_25x(b: &[u8]) -> Option<Tlv<'_>> {
    if b.len() < 2 {
        return None;
    }
    let tag = b[0];
    let (r_539, r_53a) = f_25t(&b[1..])?;
    let r_538 = 1 + r_53a;
    let r_537 = r_538.checked_add(r_539)?;
    if r_537 > b.len() {
        return None;
    }
    Some(Tlv {
        tag,
        content: &b[r_538..r_537],
        whole: &b[..r_537],
        rest: &b[r_537..],
    })
}
fn f_25t(b: &[u8]) -> Option<(usize, usize)> {
    let r_53c = *b.first()?;
    if r_53c < 0x80 {
        return Some((r_53c as usize, 1));
    }
    let r_53e = (r_53c & 0x7f) as usize;
    if r_53e == 0 || r_53e > 4 || b.len() < 1 + r_53e {
        return None;
    }
    let mut r_53d = 0usize;
    for &r_53b in &b[1..1 + r_53e] {
        r_53d = (r_53d << 8) | r_53b as usize;
    }
    Some((r_53d, 1 + r_53e))
}
fn f_2df(b: &[u8]) -> Option<&[u8]> {
    let r_53f = f_25x(b)?;
    (r_53f.tag == 0x30).then_some(r_53f.content)
}
fn nth(mut b: &[u8], r_53g: usize) -> Option<Tlv<'_>> {
    for _ in 0..r_53g {
        b = f_25x(b)?.rest;
    }
    f_25x(b)
}
fn f_1ij(content: &[u8]) -> Option<i64> {
    if content.is_empty() || content.len() > 8 {
        return None;
    }
    let mut r_53h: i64 = if content[0] & 0x80 != 0 { -1 } else { 0 };
    for &b in content {
        r_53h = (r_53h << 8) | b as i64;
    }
    Some(r_53h)
}
