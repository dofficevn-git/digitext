// Copyright (c) 2026 DigiText. All rights reserved.
use crate::cms;
use crate::SignError;
use der::Decode;
use lopdf::{content::{Content, Operation}, dictionary, Document, Object, Stream, StringFormat};
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
/// Vị trí khối chữ ký hiển thị do người ký chọn (trên tệp đầu tiên, áp cho cả lô).
/// `x`, `y`: góc TRÊN-TRÁI của khối, tính theo TỈ LỆ bề rộng/bề cao trang (0..1, `y` đo từ
/// mép trên) — cùng một điểm trên mọi khổ giấy. `last_page`: đặt ở trang cuối thay vì trang đầu.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SignPlacement {
    pub last_page: bool,
    pub x: f32,
    pub y: f32,
}
pub fn f_23p(pdf: &[u8], r_4vw: &[u8], r_4vx: u64, r_4vz: Option<&Tpv>) -> Result<Thg, SignError> {
    prepare_at(pdf, r_4vw, r_4vx, r_4vz, None)
}
/// Như `f_23p` nhưng đặt khối chữ ký tại `placement` (None = góc trên-phải trang đầu như cũ).
pub fn prepare_at(
    pdf: &[u8],
    r_4vw: &[u8],
    r_4vx: u64,
    r_4vz: Option<&Tpv>,
    placement: Option<SignPlacement>,
) -> Result<Thg, SignError> {
    let cert = Certificate::from_der(r_4vw).map_err(SignError::der)?;
    let r_4w0 = f_1ig(pdf, r_4vz, placement)?;
    let (r_4vv, r_4vu) = f_2f8(&r_4w0)?;
    let (r_4vr, r_4vq) = f_1cu(&r_4w0, r_4vv, r_4vu).ok_or_else(|| {
        SignError::Pdf("không thấy ô /Contents sau khi chèn".into())
    })?;
    let (r_4vm, r_4vn) = f_ws(&r_4w0, r_4vv, r_4vu).ok_or_else(|| {
        SignError::Pdf("không thấy /ByteRange sau khi chèn".into())
    })?;
    let r_4vy = r_4w0.len();
    let r_4vs = r_4vr - 1;
    let r_4vk = r_4vq + 1;
    let r_4vo = [0usize, r_4vs, r_4vk, r_4vy - r_4vk];
    let mut r_4vp = r_4w0;
    f_2rp(&mut r_4vp, r_4vm, r_4vn, r_4vo)?;
    let r_4vt = f_15c(&r_4vp, r_4vo).ok_or_else(|| {
        SignError::Pdf(
            "ByteRange vừa dựng lại ngoài biên (bug nội bộ)"
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
            "không thấy ô /Contents trong PDF đã chuẩn bị".into(),
        )
    })?;
    let hex = f_1gv(r_4wj);
    if hex.len() > r_4wk - r_4wn {
        return Err(SignError::Pdf(format!(
            "CMS {} byte vượt ô /Contents {} byte (tăng CONTENTS_LEN)",
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
        SignError::Pdf("không thấy ô /Contents của chữ ký".into())
    })?;
    let r_4wt = f_ws(r_4x7, r_4x2, r_4x1)
        .and_then(|(r_4wz, r_4x3)| f_1yg(r_4x7, r_4wz, r_4x3))
        .ok_or_else(|| {
            SignError::Pdf(
                "không đọc được /ByteRange của chữ ký".into(),
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
                            "đuôi sau chữ ký không phải increment DSS/DocTimeStamp hợp lệ: {e}"
                        ),
                    )
                })?,
        )
    } else {
        return Err(
            SignError::Pdf(
                "/ByteRange không phủ trọn file (có nội dung ngoài vùng ký — PDF đã bị sửa sau khi ký)"
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
        .ok_or_else(|| SignError::Pdf("/Contents không phải hex".into()))?;
    let r_4x0 = f_15c(r_4x7, r_4wt)
        .ok_or_else(|| SignError::Pdf("/ByteRange trỏ ra ngoài file".into()))?;
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
/// Thông tin người ký (chữ ký cuối của PDF) — để báo cáo kiểm tra gói tin của phần mềm khác.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignerInfo {
    /// CN (hoặc O) của chứng thư người ký.
    pub signer: String,
    /// Chủ thể đầy đủ (RFC 4514).
    pub subject: String,
    /// CN (hoặc O) của đơn vị cấp chứng thư.
    pub issuer: String,
    pub not_before_unix: i64,
    pub not_after_unix: i64,
}

pub fn signer_info(pdf: &[u8]) -> Result<SignerInfo, SignError> {
    let (a, b) = f_2f8(pdf)?;
    let (s, e) = f_1cu(pdf, a, b).ok_or_else(|| SignError::Pdf("không thấy ô /Contents của chữ ký".into()))?;
    let der = f_1gu(&pdf[s..e]).ok_or_else(|| SignError::Pdf("/Contents không phải hex".into()))?;
    let sd = cms::f_1zy(&der)?;
    let cert = cms::signer_cert(&sd)?;
    let issuer_cn = {
        let tmp = Certificate {
            tbs_certificate: x509_cert::TbsCertificate {
                subject: cert.tbs_certificate.issuer.clone(),
                ..cert.tbs_certificate.clone()
            },
            ..cert.clone()
        };
        f_2fg(&tmp)
    };
    let v = &cert.tbs_certificate.validity;
    Ok(SignerInfo {
        signer: f_2fg(&cert),
        subject: cert.tbs_certificate.subject.to_string(),
        issuer: issuer_cn,
        not_before_unix: v.not_before.to_unix_duration().as_secs() as i64,
        not_after_unix: v.not_after.to_unix_duration().as_secs() as i64,
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
                "không tìm được issuer của cert người ký cho OCSP/CRL"
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
        format!("Ký bởi: {}", f_2fg(cert)),
        format!("Ký ngày: {}", f_1di(r_4y1)),
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
    f_2qb_ten(r_4y8, r_4y9, stamp, None)
}
/// Như `f_2qb` nhưng cho phép thay TÊN HIỂN THỊ trên khối chữ ký ("Ký bởi: …" và chữ trên dấu đỏ) — người ký
/// muốn ghi tên đơn vị gọn hơn tên trong chứng thư (13/10/2026). Chứng thư / chữ ký mật mã không đổi.
pub fn f_2qb_ten(r_4y8: &[u8], r_4y9: u64, stamp: bool, ten: Option<&str>) -> Option<Tpv> {
    let cert = Certificate::from_der(r_4y8).ok()?;
    let ten = ten.map(str::trim).filter(|s| !s.is_empty()).map(str::to_string);
    let mut lines = f_2f7(&cert, r_4y9);
    if let Some(t) = &ten {
        lines[0] = format!("Ký bởi: {t}");
    }
    let r_4yb = stamp.then(|| Tly {
        org: ten.clone().unwrap_or_else(|| f_2fg(&cert)),
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
                "không đọc được font khối chữ ký"
            )
        })
        .ok()?;
    Some(Tpv {
        lines,
        font_ttf,
        stamp: r_4yb,
        stamp_png: None,
    })
}
#[derive(Debug, Clone)]
pub struct Tpv {
    pub lines: Vec<String>,
    pub font_ttf: Vec<u8>,
    pub stamp: Option<Tly>,
    /// Ảnh con dấu (PNG/JPEG, nên nền trong) do người ký chọn — vẽ bên trái dòng chữ. Có ảnh thì
    /// không vẽ dấu đỏ dựng từ chứng thư (`stamp`).
    pub stamp_png: Option<Vec<u8>>,
}
/// Cạnh dài tối đa của ảnh dấu nhúng vào PDF (px) — đủ nét khi in, không làm phình tệp.
const STAMP_MAX_PX: u32 = 600;
/// Nhúng ảnh dấu thành Image XObject (RGB + SMask nếu có vùng trong suốt).
fn add_stamp_image(doc: &mut Document, bytes: &[u8]) -> Result<(lopdf::ObjectId, u32, u32), String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("ảnh dấu không đọc được: {e}"))?;
    let img = if img.width().max(img.height()) > STAMP_MAX_PX {
        img.resize(STAMP_MAX_PX, STAMP_MAX_PX, image::imageops::FilterType::Triangle)
    } else {
        img
    };
    let rgba = img.to_rgba8();
    let (iw, ih) = rgba.dimensions();
    if iw == 0 || ih == 0 {
        return Err("ảnh dấu rỗng".into());
    }
    let mut rgb = Vec::with_capacity((iw * ih * 3) as usize);
    let mut alpha = Vec::with_capacity((iw * ih) as usize);
    for p in rgba.pixels() {
        rgb.extend_from_slice(&p.0[..3]);
        alpha.push(p.0[3]);
    }
    let mut dict = dictionary! {
        "Type" => "XObject", "Subtype" => "Image", "Width" => iw as i64, "Height" => ih as i64,
        "ColorSpace" => "DeviceRGB", "BitsPerComponent" => 8,
    };
    if alpha.iter().any(|&a| a != 255) {
        let mut mask = Stream::new(
            dictionary! {
                "Type" => "XObject", "Subtype" => "Image", "Width" => iw as i64, "Height" => ih as i64,
                "ColorSpace" => "DeviceGray", "BitsPerComponent" => 8,
            },
            alpha,
        );
        let _ = mask.compress();
        dict.set("SMask", doc.add_object(mask));
    }
    let mut s = Stream::new(dict, rgb);
    let _ = s.compress();
    Ok((doc.add_object(s), iw, ih))
}
/// Khối mới = ảnh dấu (trái, cao bằng khối chữ, tối thiểu 60 pt) + khối chữ `form` (phải).
fn wrap_with_stamp(
    doc: &mut Document,
    form: lopdf::ObjectId,
    w: f32,
    h: f32,
    img: lopdf::ObjectId,
    iw: u32,
    ih: u32,
) -> Result<(lopdf::ObjectId, f32, f32), String> {
    let sh = h.max(60.0);
    let sw = sh * iw as f32 / ih as f32;
    let gap = 4.0;
    let (bw, bh) = (sw + gap + w, sh);
    let ops = vec![
        Operation::new("q", vec![]),
        Operation::new("cm", vec![sw.into(), 0.into(), 0.into(), sh.into(), 0.into(), 0.into()]),
        Operation::new("Do", vec![Object::Name(b"Im0".to_vec())]),
        Operation::new("Q", vec![]),
        Operation::new("q", vec![]),
        Operation::new(
            "cm",
            vec![1.into(), 0.into(), 0.into(), 1.into(), (sw + gap).into(), ((bh - h) / 2.0).into()],
        ),
        Operation::new("Do", vec![Object::Name(b"Fm0".to_vec())]),
        Operation::new("Q", vec![]),
    ];
    let bytes = Content { operations: ops }.encode().map_err(|e| e.to_string())?;
    let mut s = Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Form", "FormType" => 1,
            "BBox" => vec![0.into(), 0.into(), bw.into(), bh.into()],
            "Resources" => dictionary! { "XObject" => dictionary! { "Im0" => img, "Fm0" => form } },
        },
        bytes,
    );
    let _ = s.compress();
    Ok((doc.add_object(s), bw, bh))
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
/// Khung [x0, y0, x1, y1] cho khối rộng `w`, cao `h` trên trang có MediaBox `m`.
fn place_rect(m: [f32; 4], w: f32, h: f32, placement: Option<SignPlacement>) -> [f32; 4] {
    // Khối lớn hơn trang (ảnh dấu to, trang nhỏ) → co đều cho vừa, chừa lề — không bao giờ tràn ra
    // ngoài trang (đề nghị chỉnh sửa mục 5).
    let (pw, ph) = ((m[2] - m[0] - 2.0 * C_K5).max(1.0), (m[3] - m[1] - 2.0 * C_K5).max(1.0));
    let k = (pw / w).min(ph / h).min(1.0);
    let (w, h) = (w * k, h * k);
    match placement {
        None => {
            let r_4z4 = m[2] - C_K5;
            let r_4z5 = m[3] - C_K5;
            [(r_4z4 - w).max(m[0]), (r_4z5 - h).max(m[1]), r_4z4, r_4z5]
        }
        Some(p) => {
            let (pw, ph) = (m[2] - m[0], m[3] - m[1]);
            let left = (m[0] + p.x.clamp(0.0, 1.0) * pw).clamp(m[0], (m[2] - w).max(m[0]));
            let top = (m[3] - p.y.clamp(0.0, 1.0) * ph).clamp((m[1] + h).min(m[3]), m[3]);
            [left, top - h, left + w, top]
        }
    }
}
fn f_1ig(pdf: &[u8], r_4z2: Option<&Tpv>, placement: Option<SignPlacement>) -> Result<Vec<u8>, SignError> {
    let mut r_4yq =
        Document::load_mem(pdf).map_err(|e| SignError::Pdf(format!("nạp PDF: {e}")))?;
    let pages = r_4yq.get_pages();
    let chosen = if placement.is_some_and(|p| p.last_page) {
        pages.values().next_back()
    } else {
        pages.values().next()
    };
    let r_4yw = *chosen.ok_or_else(|| SignError::Pdf("PDF không có trang".into()))?;
    let r_4ym = match r_4z2 {
        Some(r_4yn) if r_4yn.stamp_png.is_some() => {
            let png = r_4yn.stamp_png.as_deref().unwrap_or_default();
            let built = digitext_pdf::f_w0(&mut r_4yq, &r_4yn.lines, &r_4yn.font_ttf)
                .map_err(|e| e.to_string())
                .and_then(|(form, w, h)| match add_stamp_image(&mut r_4yq, png) {
                    Ok((img, iw, ih)) => wrap_with_stamp(&mut r_4yq, form, w, h, img, iw, ih),
                    Err(e) => {
                        tracing::warn!(error = %e, "ảnh dấu lỗi — ký chỉ có dòng chữ");
                        Ok((form, w, h))
                    }
                });
            match built {
                Ok((form_id, w, h)) => f_1s2(&r_4yq, r_4yw).map(|m| (form_id, place_rect(m, w, h, placement))),
                Err(e) => {
                    tracing::warn!(error = %e, "không dựng được khối chữ ký hiển thị — ký vô hình");
                    None
                }
            }
        }
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
                Ok((form_id, r_4z3, h)) => f_1s2(&r_4yq, r_4yw)
                    .map(|r_4yt| (form_id, place_rect(r_4yt, r_4z3, h, placement))),
                Err(e) => {
                    tracing::warn!(
                        error = % e,
                        "không dựng được khối chữ ký hiển thị — ký vô hình"
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
        .ok_or_else(|| SignError::Pdf("thiếu /Root".into()))?;
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
    match f_2f8_acroform(r_4z7) {
        Ok(r) => Ok(r),
        // PDF do phần mềm khác ký (iTextSharp…): /AcroForm, /Fields, /V có thể là từ điển trực tiếp hoặc
        // dict chữ ký nằm trong object stream → tìm thẳng khối object chứa /ByteRange cuối cùng.
        Err(e) => f_2f8_scan(r_4z7).ok_or(e),
    }
}
/// Khối `N G obj … endobj` chứa `/ByteRange` cuối cùng của tệp (chữ ký mới nhất).
fn f_2f8_scan(pdf: &[u8]) -> Option<(usize, usize)> {
    let at = pdf.windows(10).rposition(|w| w == b"/ByteRange")?;
    let head = &pdf[..at];
    let obj = head.windows(4).rposition(|w| w == b" obj" || w == b"\nobj" || w == b"\robj")?;
    // Lùi về đầu dòng "N G obj".
    let start = head[..obj].iter().rposition(|b| *b == b'\n' || *b == b'\r').map(|i| i + 1).unwrap_or(0);
    let end = at + find(&pdf[at..], b"endobj")?;
    Some((start, end))
}
fn f_2f8_acroform(r_4z7: &[u8]) -> Result<(usize, usize), SignError> {
    let r_4z8 =
        Document::load_mem(r_4z7).map_err(|e| SignError::Pdf(format!("nạp PDF: {e}")))?;
    let r_4ze = r_4z8
        .trailer
        .get(b"Root")
        .ok()
        .and_then(|o| o.as_reference().ok())
        .ok_or_else(|| SignError::Pdf("thiếu /Root".into()))?;
    let catalog = r_4z8
        .get_object(r_4ze)
        .and_then(|o| o.as_dict())
        .map_err(|e| SignError::Pdf(format!("đọc catalog: {e}")))?;
    let r_4z6 = catalog
        .get(b"AcroForm")
        .ok()
        .and_then(|o| o.as_reference().ok())
        .ok_or_else(|| SignError::Pdf("thiếu /AcroForm".into()))?;
    let r_4zd = r_4z8
        .get_object(r_4z6)
        .and_then(|o| o.as_dict())
        .and_then(|d| d.get(b"Fields"))
        .and_then(|o| o.as_array())
        .map_err(|e| SignError::Pdf(format!("đọc AcroForm/Fields: {e}")))?;
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
        SignError::Pdf("không thấy trường chữ ký (FT=Sig)".into())
    })?;
    let r_4zf = match r_4z8.reference_table.entries.get(&sig_id.0) {
        Some(lopdf::xref::XrefEntry::Normal { offset, .. }) => *offset as usize,
        _ => {
            return Err(
                SignError::Pdf(
                    "dict chữ ký không có mục xref thường (object stream?)"
                        .into(),
                ),
            );
        }
    };
    if r_4zf >= r_4z7.len() {
        return Err(SignError::Pdf(
            "offset xref của chữ ký ngoài file".into(),
        ));
    }
    let r_4z9 = r_4zf
        + find(&r_4z7[r_4zf..], b"endobj").ok_or_else(|| {
            SignError::Pdf("không thấy endobj của dict chữ ký".into())
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
            "giá trị ByteRange dài hơn placeholder".into(),
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

#[cfg(test)]
mod placement_tests {
    use super::{place_rect, SignPlacement};
    const A4: [f32; 4] = [0.0, 0.0, 595.0, 842.0];
    #[test]
    fn mac_dinh_goc_tren_phai() {
        let r = place_rect(A4, 200.0, 60.0, None);
        assert_eq!(r, [595.0 - 14.0 - 200.0, 842.0 - 14.0 - 60.0, 595.0 - 14.0, 842.0 - 14.0]);
    }
    #[test]
    fn theo_ti_le_tu_mep_tren() {
        let r = place_rect(A4, 200.0, 60.0, Some(SignPlacement { last_page: false, x: 0.5, y: 0.25 }));
        assert_eq!(r, [297.5, 842.0 * 0.75 - 60.0, 497.5, 842.0 * 0.75]);
    }
    #[test]
    fn khoi_to_hon_trang_thi_co_lai() {
        for p in [None, Some(SignPlacement { last_page: false, x: 0.9, y: 0.9 })] {
            let r = place_rect(A4, 900.0, 300.0, p);
            assert!(r[0] >= 0.0 && r[2] <= 595.0 && r[1] >= 0.0 && r[3] <= 842.0, "{r:?}");
            let (w, h) = (r[2] - r[0], r[3] - r[1]);
            assert!((w / h - 3.0).abs() < 0.01, "giữ tỉ lệ: {w}x{h}");
        }
    }
    #[test]
    fn khong_tran_ra_ngoai_trang() {
        let r = place_rect(A4, 200.0, 60.0, Some(SignPlacement { last_page: true, x: 0.99, y: 0.99 }));
        assert!(r[0] >= 0.0 && r[2] <= 595.0 && r[1] >= 0.0 && r[3] <= 842.0, "{r:?}");
        let r = place_rect([10.0, 20.0, 610.0, 862.0], 200.0, 60.0, Some(SignPlacement { last_page: false, x: 0.0, y: 0.0 }));
        assert_eq!(r, [10.0, 862.0 - 60.0, 210.0, 862.0]);
    }
}
#[cfg(test)]
mod stamp_tests {
    use super::{f_1ig, SignPlacement, Tpv};
    use lopdf::{dictionary, Document, Object, Stream};
    fn one_page_pdf() -> Vec<u8> {
        let mut doc = Document::with_version("1.7");
        let pages_id = doc.new_object_id();
        let content = doc.add_object(Stream::new(dictionary! {}, b"".to_vec()));
        let page = doc.add_object(dictionary! {
            "Type" => "Page", "Parent" => pages_id, "Contents" => content,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        });
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => vec![page.into()], "Count" => 1 }),
        );
        let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog);
        let mut out = Vec::new();
        doc.save_to(&mut out).unwrap();
        out
    }
    fn png_trong_suot() -> Vec<u8> {
        let img = image::RgbaImage::from_fn(120, 80, |x, _| image::Rgba([200, 0, 0, if x < 60 { 0 } else { 255 }]));
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(img).write_to(&mut out, image::ImageFormat::Png).unwrap();
        out.into_inner()
    }
    #[test]
    fn nhung_anh_dau_co_smask_va_nam_trong_trang() {
        let lines = vec!["Ký bởi: Nguyễn Văn A".to_string(), "Thời gian: 28/09/2026".to_string()];
        let Some(font) = digitext_pdf::f_1cr(&lines.join(" ")).and_then(|p| std::fs::read(p).ok()) else {
            eprintln!("bỏ qua: máy không có font tiếng Việt");
            return;
        };
        let tpv = Tpv { lines, font_ttf: font, stamp: None, stamp_png: Some(png_trong_suot()) };
        let out = f_1ig(&one_page_pdf(), Some(&tpv), Some(SignPlacement { last_page: false, x: 0.98, y: 0.98 })).unwrap();
        let doc = Document::load_mem(&out).unwrap();
        let widget = doc
            .objects
            .values()
            .filter_map(|o| o.as_dict().ok())
            .find(|d| d.get(b"Subtype").ok().and_then(|v| v.as_name().ok()) == Some(b"Widget".as_slice()))
            .expect("có widget chữ ký");
        let r: Vec<f32> = widget.get(b"Rect").unwrap().as_array().unwrap().iter().map(|o| o.as_float().unwrap()).collect();
        assert!(r[0] >= 0.0 && r[1] >= 0.0 && r[2] <= 595.0 && r[3] <= 842.0 && r[2] > r[0], "{r:?}");
        let ap = widget.get(b"AP").unwrap().as_dict().unwrap().get(b"N").unwrap().as_reference().unwrap();
        let form = doc.get_object(ap).unwrap().as_stream().unwrap();
        let xo = form.dict.get(b"Resources").unwrap().as_dict().unwrap().get(b"XObject").unwrap().as_dict().unwrap();
        let im = doc.get_object(xo.get(b"Im0").unwrap().as_reference().unwrap()).unwrap().as_stream().unwrap();
        assert_eq!(im.dict.get(b"Subtype").unwrap().as_name().unwrap(), b"Image");
        assert!(im.dict.get(b"SMask").is_ok(), "ảnh nền trong phải có SMask");
    }
}
