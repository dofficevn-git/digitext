// Copyright (c) 2026 DigiText. All rights reserved.
use cryptoki::context::{CInitializeArgs, CInitializeFlags, Pkcs11};
use cryptoki::mechanism::Mechanism;
use cryptoki::object::{Attribute, AttributeType, KeyType, ObjectClass, ObjectHandle};
use cryptoki::session::{Session, UserType};
use cryptoki::slot::Slot;
use cryptoki::types::AuthPin;
use der::{oid::AssociatedOid, Decode, Encode};
use digitext_sign_verify::chain::TrustStore;
use digitext_sign_verify::dss::{embed_dss, f_1cp, f_23h, ValidationMaterial};
use digitext_sign_verify::{f_1cn, f_2q1, prepare, verify, Tpr};
use digitext_sign_verify::{ocsp, tsa};
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};
use std::process::ExitCode;
use x509_cert::ext::pkix::{BasicConstraints, KeyUsage};
use x509_cert::Certificate;
const SHA256_DIGESTINFO_PREFIX: [u8; 19] = [
    0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01, 0x05,
    0x00, 0x04, 0x20,
];
fn digestinfo_sha256(digest: &[u8]) -> Vec<u8> {
    let mut r_5i2 = Vec::with_capacity(SHA256_DIGESTINFO_PREFIX.len() + digest.len());
    r_5i2.extend_from_slice(&SHA256_DIGESTINFO_PREFIX);
    r_5i2.extend_from_slice(digest);
    r_5i2
}
fn f_13r() -> Option<String> {
    if let Ok(m) = std::env::var("DIGITEXT_PKCS11_MODULE") {
        return Some(m);
    }
    let r_5i3: &[&str] = if cfg!(target_os = "macos") {
        &[
            "/usr/local/lib/ica_csp11_v1.dylib",
            "/opt/homebrew/lib/opensc-pkcs11.so",
            "/usr/local/lib/opensc-pkcs11.so",
            "/Library/OpenSC/lib/opensc-pkcs11.so",
        ]
    } else if cfg!(target_os = "windows") {
        &[
            "C:\\Windows\\System32\\ica_csp11_v1.dll",
            "C:\\Program Files\\OpenSC Project\\OpenSC\\pkcs11\\opensc-pkcs11.dll",
            "C:\\Windows\\System32\\eps2003csp11v2.dll",
        ]
    } else {
        &[
            "/usr/lib/ica_csp11_v1.so",
            "/usr/lib/x86_64-linux-gnu/opensc-pkcs11.so",
            "/usr/lib/opensc-pkcs11.so",
            "/usr/lib64/opensc-pkcs11.so",
        ]
    };
    r_5i3
        .iter()
        .find(|r_5i4| std::path::Path::new(r_5i4).exists())
        .map(|r_5i5| r_5i5.to_string())
}
fn attr_bytes(r_5i8: &Session, r_5i7: ObjectHandle, r_5i9: AttributeType) -> Option<Vec<u8>> {
    let r_5i6 = r_5i8.get_attributes(r_5i7, &[r_5i9]).ok()?;
    r_5i6.into_iter().find_map(|a| match a {
        Attribute::Value(v) if r_5i9 == AttributeType::Value => Some(v),
        Attribute::Id(v) if r_5i9 == AttributeType::Id => Some(v),
        _ => None,
    })
}
fn context(module: &str) -> Result<Pkcs11, String> {
    let pkcs11 = Pkcs11::new(module)
        .map_err(|e| format!("không nạp được module '{module}': {e}"))?;
    match pkcs11.initialize(CInitializeArgs::new(CInitializeFlags::OS_LOCKING_OK)) {
        // Driver đã được khởi tạo trong tiến trình này (vd. "Dò token" trước đó) — chuẩn
        // PKCS#11 coi là dùng tiếp được, không phải lỗi (iCA trả mã này, 28/9/2026).
        Ok(())
        | Err(cryptoki::error::Error::Pkcs11(
            cryptoki::error::RvError::CryptokiAlreadyInitialized,
            _,
        )) => {}
        Err(e) => return Err(format!("khởi tạo PKCS#11 lỗi: {e}")),
    }
    Ok(pkcs11)
}
fn f_29c(pkcs11: &Pkcs11) -> Result<Vec<Slot>, String> {
    let r_5ic = pkcs11
        .get_slots_with_token()
        .map_err(|e| format!("liệt kê slot lỗi: {e}"))?;
    if !r_5ic.is_empty() {
        return Ok(r_5ic);
    }
    let r_5ia = pkcs11
        .get_all_slots()
        .map_err(|e| format!("liệt kê toàn bộ slot lỗi: {e}"))?;
    let present: Vec<Slot> = r_5ia
        .iter()
        .copied()
        .filter(|r_5ib| {
            pkcs11
                .get_slot_info(*r_5ib)
                .map(|i| i.token_present())
                .unwrap_or(false)
        })
        .collect();
    if !present.is_empty() {
        eprintln!(
            "(chú ý: get_slots_with_token() rỗng — dùng fallback card-present, {} slot)",
            present.len()
        );
        return Ok(present);
    }
    if !r_5ia.is_empty() {
        eprintln!(
            "(chú ý: không slot nào báo token-present — thử trực tiếp {} slot)",
            r_5ia.len()
        );
    }
    Ok(r_5ia)
}
fn f_23u(tag: &str, r_5id: &[u8]) {
    match Certificate::from_der(r_5id) {
        Ok(c) => {
            let tbs = &c.tbs_certificate;
            println!("  [{tag}] {} bytes DER", r_5id.len());
            println!("    subject : {}", tbs.subject);
            println!("    issuer  : {}", tbs.issuer);
            println!("    serial  : {}", tbs.serial_number);
            println!(
                "    validity: {} → {}",
                tbs.validity.not_before, tbs.validity.not_after
            );
        }
        Err(e) => {
            println!(
                "  [{tag}] {} bytes DER (không parse được: {e})",
                r_5id.len()
            )
        }
    }
}
fn slot_index() -> usize {
    std::env::var("DIGITEXT_SLOT")
        .ok()
        .and_then(|r_5ie| r_5ie.parse().ok())
        .unwrap_or(0)
}
fn f_1vc() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
#[derive(Clone, Debug)]
struct CertMeta {
    idx: usize,
    subject_der: Vec<u8>,
    issuer_der: Vec<u8>,
    not_before: i64,
    not_after: i64,
    can_sign: bool,
    is_ca: bool,
}
#[derive(Clone, Debug, PartialEq)]
struct CertSelection {
    leaf: usize,
    intermediates: Vec<usize>,
}
fn cert_extension(cert: &Certificate, r_5ig: der::asn1::ObjectIdentifier) -> Option<Vec<u8>> {
    let r_5if = cert.tbs_certificate.extensions.as_ref()?;
    r_5if
        .iter()
        .find(|e| e.extn_id == r_5ig)
        .map(|e| e.extn_value.as_bytes().to_vec())
}
fn key_usage_allows_signing(cert: &Certificate) -> bool {
    match cert_extension(cert, KeyUsage::OID) {
        None => true,
        Some(r_5ih) => r_5ih.get(3).copied().unwrap_or(0) & 0b1100_0000 != 0,
    }
}
fn basic_constraints_ca(cert: &Certificate) -> bool {
    match cert_extension(cert, BasicConstraints::OID) {
        None => false,
        Some(r_5ii) => r_5ii.windows(3).any(|w| w == [0x01, 0x01, 0xFF]),
    }
}
fn parse_cert_meta(idx: usize, r_5ij: &[u8]) -> Option<CertMeta> {
    let cert = Certificate::from_der(r_5ij).ok()?;
    let tbs = &cert.tbs_certificate;
    Some(CertMeta {
        idx,
        subject_der: tbs.subject.to_der().ok()?,
        issuer_der: tbs.issuer.to_der().ok()?,
        not_before: tbs.validity.not_before.to_unix_duration().as_secs() as i64,
        not_after: tbs.validity.not_after.to_unix_duration().as_secs() as i64,
        can_sign: key_usage_allows_signing(&cert),
        is_ca: basic_constraints_ca(&cert),
    })
}
fn select_signer(r_5il: &[CertMeta], r_5im: i64) -> Option<CertSelection> {
    if r_5il.is_empty() {
        return None;
    }
    let r_5ip = |m: &CertMeta| (r_5im >= m.not_before && r_5im <= m.not_after, m.not_before);
    let r_5ik = |r_5io: &dyn Fn(&CertMeta) -> bool| {
        r_5il
            .iter()
            .filter(|m| r_5io(m))
            .max_by(|a, b| r_5ip(a).cmp(&r_5ip(b)))
    };
    let leaf = r_5ik(&|m| !m.is_ca && m.can_sign)
        .or_else(|| r_5ik(&|m| !m.is_ca))
        .or_else(|| r_5ik(&|_| true))?;
    let mut intermediates = Vec::new();
    let mut r_5iq = vec![leaf.idx];
    let mut r_5ir = leaf.issuer_der.clone();
    while let Some(r_5in) = r_5il
        .iter()
        .find(|m| m.is_ca && m.subject_der == r_5ir && !r_5iq.contains(&m.idx))
    {
        intermediates.push(r_5in.idx);
        r_5iq.push(r_5in.idx);
        if r_5in.subject_der == r_5in.issuer_der {
            break;
        }
        r_5ir = r_5in.issuer_der.clone();
    }
    Some(CertSelection {
        leaf: leaf.idx,
        intermediates,
    })
}
struct Tl0 {
    cert: Vec<u8>,
    intermediates: Vec<Vec<u8>>,
    leaf_id: Option<Vec<u8>>,
}
fn f_2d8(r_5iy: &Session) -> Result<Tl0, String> {
    let r_5is = r_5iy
        .find_objects(&[Attribute::Class(ObjectClass::CERTIFICATE)])
        .map_err(|e| format!("tìm cert lỗi: {e}"))?;
    let mut r_5it: Vec<(Vec<u8>, Option<Vec<u8>>)> = Vec::new();
    for r_5iu in r_5is {
        if let Some(der) = attr_bytes(r_5iy, r_5iu, AttributeType::Value) {
            let r_5iv = attr_bytes(r_5iy, r_5iu, AttributeType::Id);
            r_5it.push((der, r_5iv));
        }
    }
    if r_5it.is_empty() {
        return Err("token không có chứng thư (certificate)".into());
    }
    println!(
        "\n── {} chứng thư trên token ──",
        r_5it.len()
    );
    for (i, (der, _)) in r_5it.iter().enumerate() {
        f_23u(&format!("cert[{i}]"), der);
    }
    let r_5iw: Vec<CertMeta> = r_5it
        .iter()
        .enumerate()
        .filter_map(|(i, (der, _))| parse_cert_meta(i, der))
        .collect();
    let r_5ix = select_signer(&r_5iw, f_1vc())
        .ok_or("không chọn được chứng thư người ký")?;
    let cert = r_5it[r_5ix.leaf].0.clone();
    let leaf_id = r_5it[r_5ix.leaf].1.clone();
    let intermediates: Vec<Vec<u8>> = r_5ix
        .intermediates
        .iter()
        .map(|&i| r_5it[i].0.clone())
        .collect();
    println!(
        "\n── ĐÃ CHỌN người ký (leaf = cert[{}]) ──",
        r_5ix.leaf
    );
    f_23u("leaf", &cert);
    println!(
        "Số cert trung gian nhúng vào CMS: {}",
        intermediates.len()
    );
    for (i, c) in intermediates.iter().enumerate() {
        f_23u(&format!("inter{i}"), c);
    }
    Ok(Tl0 {
        cert,
        intermediates,
        leaf_id,
    })
}
fn f_1cy(r_5j2: &Session, leaf_id: &Option<Vec<u8>>) -> Result<ObjectHandle, String> {
    let r_5j3 = r_5j2
        .find_objects(&[
            Attribute::Class(ObjectClass::PRIVATE_KEY),
            Attribute::Sign(true),
        ])
        .map_err(|e| format!("tìm khoá ký lỗi: {e}"))?;
    let r_5j1 = leaf_id
        .as_deref()
        .and_then(|r_5j4| {
            r_5j3.iter().copied().find(|r_5j0| {
                attr_bytes(r_5j2, *r_5j0, AttributeType::Id).as_deref() == Some(r_5j4)
            })
        })
        .or_else(|| r_5j3.first().copied())
        .ok_or("token không có khoá ký (CKA_SIGN)")?;
    if let Ok(r_5iz) = r_5j2.get_attributes(r_5j1, &[AttributeType::KeyType]) {
        if let Some(Attribute::KeyType(kt)) = r_5iz.into_iter().next() {
            if kt != KeyType::RSA {
                return Err(
                    format!(
                        "khoá không phải RSA ({kt:?}) — công cụ này chỉ ký RSA"
                    ),
                );
            }
        }
    }
    Ok(r_5j1)
}
fn f_23v(r: &Tpr) {
    println!("\n── VERIFY (digitext-sign-verify::verify) ──");
    println!("  signature_ok          : {}", r.signature_ok);
    println!("  digest_ok             : {}", r.digest_ok);
    println!("  cert_binding_ok       : {}", r.cert_binding_ok);
    println!(
        "  signer_chain_ok       : {} (cần trust store iCA để true)",
        r.signer_chain_ok
    );
    println!("  timestamp_present     : {}", r.timestamp_present);
    println!(
        "  timestamp_structural_ok: {} (imprint + chữ ký TSA khớp)",
        r.timestamp_structural_ok
    );
    println!(
        "  timestamp_trusted     : {} (cần --tsa <root TSA> để true)",
        r.timestamp_trusted
    );
    if let Some(t) = r.timestamp_time_unix {
        println!(
            "  timestamp_time        : {} ({})",
            t,
            f_1d8(t.max(0) as u64)
        );
    }
    println!("  has_dss               : {}", r.has_dss);
    println!(
        "  revocation_ok         : {} (OCSP/CRL nói Good tại mốc {:?})",
        r.revocation_ok, r.revocation_reference_unix
    );
    println!(
        "  revocation_trusted    : {} (mốc trên do TSA đóng, không phải signingTime tự khai)",
        r.revocation_trusted
    );
    println!(
        "  has_archive_timestamp : {} (mới ở mức CẤU TRÚC)",
        r.has_archive_timestamp
    );
    println!(
        "  archive_ts_trusted    : {} (cần --tsa <root TSA> để true)",
        r.archive_timestamp_trusted
    );
    println!("  → is_valid() PAdES-B  : {}", r.is_valid());
    println!("  → is_lt()   PAdES-LT  : {}", r.is_lt());
    println!("  → is_lta()  PAdES-LTA : {}", r.is_lta());
}
fn f_1ow(path: &str) -> Result<TrustStore, String> {
    let pem = std::fs::read_to_string(path)
        .map_err(|e| format!("đọc trust store '{path}' lỗi: {e}"))?;
    TrustStore::from_pem(&pem)
        .map_err(|e| format!("nạp trust store '{path}' lỗi: {e}"))
}
fn f_15w(input: &str, roots: Option<&str>, tsa: Option<&str>) -> Result<(), String> {
    let pdf =
        std::fs::read(input).map_err(|e| format!("đọc PDF '{input}' lỗi: {e}"))?;
    println!("PDF     : {input} ({} bytes)", pdf.len());
    let r_5j6 = roots.map(f_1ow).transpose()?;
    let r_5j7 = tsa.map(f_1ow).transpose()?;
    match &r_5j6 {
        Some(s) => {
            println!(
                "Trust store người ký: {} cert (anchor)",
                s.len()
            )
        }
        None => {
            println!(
                "Trust store người ký: (không) → signer_chain_ok chỉ ở mức cấu trúc"
            )
        }
    }
    if let Some(s) = &r_5j7 {
        println!("Trust store TSA     : {} cert", s.len());
    }
    let r_5j5 = f_2q1(&pdf, r_5j6.as_ref(), r_5j7.as_ref())
        .map_err(|e| format!("verify lỗi: {e}"))?;
    f_23v(&r_5j5);
    if r_5j5.is_lt() {
        println!(
            r#"
✅ PAdES-LT — chữ ký hợp lệ, neo tới root tin cậy, có DSS, và không bị thu hồi TẠI MỐC THỜI GIAN DO TSA ĐÓNG."#
        );
    } else if r_5j5.signer_chain_ok && r_5j5.is_valid() && r_5j5.has_dss {
        println!(
            r#"
○ Chữ ký hợp lệ + neo root + có DSS, nhưng CHƯA đủ LT: bằng chứng thu hồi phải đối chiếu"#
        );
        println!(
            "  với mốc thời gian TIN ĐƯỢC (tem TSA neo root — cấp --tsa <bundle root TSA>); chỉ dựa"
        );
        println!(
            "  signingTime do người ký tự khai thì không kết luận được (xem revocation_trusted)."
        );
    } else if r_5j5.signer_chain_ok && r_5j5.is_valid() {
        println!(
            r#"
✅ Chữ ký hợp lệ VÀ neo tới root tin cậy (signer_chain_ok=true). Chưa có DSS ⇒ chưa LT."#
        );
    } else if r_5j5.is_valid() {
        println!(
            r#"
○ Chữ ký hợp lệ (PAdES-B) nhưng CHƯA neo root tin cậy — cấp --roots <bundle iCA> để kiểm."#
        );
    } else {
        return Err(
            "chữ ký KHÔNG hợp lệ — xem các cờ ở trên".into(),
        );
    }
    Ok(())
}
fn f_220(path: &str) -> Result<Vec<Vec<u8>>, String> {
    let r_5ja = std::fs::read(path).map_err(|e| format!("đọc '{path}' lỗi: {e}"))?;
    let r_5j8 = pem::parse_many(&r_5ja).map_err(|e| format!("PEM '{path}' hỏng: {e}"))?;
    let r_5j9: Vec<Vec<u8>> = r_5j8
        .into_iter()
        .filter(|p| p.tag() == "CERTIFICATE")
        .map(|p| p.into_contents())
        .collect();
    if r_5j9.is_empty() {
        return Err(format!(
            "'{path}' không có khối CERTIFICATE"
        ));
    }
    Ok(r_5j9)
}
fn f_xm(r_5jb: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let c = Certificate::from_der(r_5jb).ok()?;
    Some((
        c.tbs_certificate.subject.to_der().ok()?,
        c.tbs_certificate.issuer.to_der().ok()?,
    ))
}
fn f_1hl() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(15))
        .timeout_read(std::time::Duration::from_secs(30))
        .build()
}
fn f_1ho(r_5jc: &ureq::Agent, url: &str, r_5jf: &str, r_5jd: &[u8]) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let r_5jg = r_5jc
        .post(url)
        .set("Content-Type", r_5jf)
        .send_bytes(r_5jd)
        .map_err(|e| format!("POST {url}: {e}"))?;
    let mut r_5je = Vec::new();
    r_5jg
        .into_reader()
        .take(8 * 1024 * 1024)
        .read_to_end(&mut r_5je)
        .map_err(|e| format!("đọc phản hồi {url}: {e}"))?;
    Ok(r_5je)
}
fn f_1hn(r_5jh: &ureq::Agent, url: &str) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let r_5jj = r_5jh
        .get(url)
        .call()
        .map_err(|e| format!("GET {url}: {e}"))?;
    let mut r_5ji = Vec::new();
    r_5jj
        .into_reader()
        .take(32 * 1024 * 1024)
        .read_to_end(&mut r_5ji)
        .map_err(|e| format!("đọc phản hồi {url}: {e}"))?;
    Ok(r_5ji)
}
const C_42: &str = "http://timestamp.digicert.com";
fn fetch_timestamp(
    r_5jk: &ureq::Agent,
    tsa_url: &str,
    signature: &[u8],
) -> Result<Vec<u8>, String> {
    let r_5jl = rand::random::<u64>();
    let r_5jm = tsa::build_request(signature, Some(r_5jl), true)
        .map_err(|e| format!("dựng TimeStampReq lỗi: {e}"))?;
    let r_5jn = f_1ho(r_5jk, tsa_url, "application/timestamp-query", &r_5jm)?;
    tsa::f_2le(&r_5jn).map_err(|e| format!("TSA từ chối/không trả token: {e}"))
}
fn f_1c9(tsa_url: Option<&str>, signature: &[u8]) -> Result<Option<Vec<u8>>, String> {
    match tsa_url {
        None => Ok(None),
        Some(url) => {
            println!("\n── TEM THỜI GIAN (PAdES-T) ──\n  TSA: {url}");
            let token = fetch_timestamp(&f_1hl(), url, signature)?;
            println!("  ✓ token TSA: {} bytes", token.len());
            Ok(Some(token))
        }
    }
}
fn f_15s(
    input: &str,
    output: &str,
    r_5jr: &str,
    r_5jw: &str,
    r_5k3: Option<&str>,
) -> Result<(), String> {
    let pdf =
        std::fs::read(input).map_err(|e| format!("đọc PDF '{input}' lỗi: {e}"))?;
    println!("PDF     : {input} ({} bytes)", pdf.len());
    let leaf = f_220(r_5jw)?.into_iter().next().ok_or("leaf rỗng")?;
    let r_5jq = f_220(r_5jr)?;
    let (mut root, mut intermediate) = (None, None);
    for r_5jv in &r_5jq {
        if let Some((r_5k2, i)) = f_xm(r_5jv) {
            if r_5k2 == i {
                root = Some(r_5jv.clone());
            } else {
                intermediate = Some(r_5jv.clone());
            }
        }
    }
    let intermediate = intermediate
        .ok_or("bundle --chain thiếu cert trung gian (CA cấp phát)")?;
    let ocsp_url = ocsp::f_1vw(&leaf).ok_or("cert leaf không khai OCSP URL (AIA)")?;
    let r_5jt = ocsp::f_12o(&leaf);
    println!("OCSP    : {ocsp_url}");
    if let Some(u) = &r_5jt {
        println!("CRL     : {u}");
    }
    let r_5jo = f_1hl();
    let (mut r_5jx, mut r_5ju) = (Vec::new(), Vec::new());
    match ocsp::f_vl(&intermediate, &leaf) {
        Ok(r_5k0) => match f_1ho(&r_5jo, &ocsp_url, "application/ocsp-request", &r_5k0) {
            Ok(r_5k1) => {
                println!("  ✓ OCSP phản hồi: {} bytes", r_5k1.len());
                r_5jx.push(r_5k1);
            }
            Err(e) => println!("  ⚠ OCSP thất bại: {e}"),
        },
        Err(e) => println!("  ⚠ dựng OCSP request lỗi: {e}"),
    }
    if let Some(u) = &r_5jt {
        match f_1hn(&r_5jo, u) {
            Ok(r_5js) => {
                println!("  ✓ CRL: {} bytes", r_5js.len());
                r_5ju.push(r_5js);
            }
            Err(e) => println!("  ⚠ CRL thất bại: {e}"),
        }
    }
    if r_5jx.is_empty() && r_5ju.is_empty() {
        println!(
            "  ⚠ Không lấy được OCSP lẫn CRL → DSS chỉ có cert, revocation_ok sẽ = false"
        );
    }
    let mut r_5jp = vec![leaf.clone(), intermediate.clone()];
    if let Some(r) = &root {
        r_5jp.push(r.clone());
    }
    let r_5k5 = ValidationMaterial {
        certs: r_5jp,
        crls: r_5ju,
        ocsps: r_5jx,
    };
    let lt = embed_dss(&pdf, &r_5k5)
        .map_err(|e| format!("embed_dss (nhúng /DSS) lỗi: {e}"))?;
    std::fs::write(output, &lt).map_err(|e| format!("ghi '{output}' lỗi: {e}"))?;
    println!("\nĐÃ GHI PAdES-LT: {output} ({} bytes)", lt.len());
    let store = f_1ow(r_5jr)?;
    let r_5k4 = match r_5k3 {
        Some(r_5jy) => Some(f_1ow(r_5jy)?),
        None => None,
    };
    let r_5jz = f_2q1(&lt, Some(&store), r_5k4.as_ref())
        .map_err(|e| format!("verify LT lỗi: {e}"))?;
    f_23v(&r_5jz);
    if r_5jz.is_lt() {
        println!(
            r#"
✅ PAdES-LT — chữ ký hợp lệ, neo root tin cậy, có DSS, không bị thu hồi. Tự chứng minh dài hạn."#
        );
        Ok(())
    } else if r_5jz.has_dss && r_5jz.signer_chain_ok && r_5jz.revocation_ok {
        if r_5k3.is_none() {
            println!(
                r#"
⚠️  CHƯA KẾT LUẬN được mức PAdES-LT: thiếu bundle root TSA nên mốc đối chiếu thu hồi rơi về signingTime NGƯỜI KÝ TỰ KHAI (revocation_trusted=false)."#
            );
            println!(
                r#"   Tệp đã ghi đúng (DSS + neo root + bằng chứng Good). Kết luận bằng:
   digitext sign make-lt <in> <out> --chain <bundle CA> --leaf <leaf.pem> --tsa-roots <bundle root TSA>
   hoặc: digitext sign verify <out> --roots <bundle CA> --tsa <bundle root TSA>"#
            );
            return Ok(());
        }
        Err(
            "đã nhúng DSS + neo root + bằng chứng nói Good, nhưng mốc đối chiếu là signingTime TỰ KHAI (revocation_trusted=false) DÙ ĐÃ đưa root TSA: PDF phải có tem thời gian TSA neo được tới root đó (PAdES-T) mới đạt PAdES-LT"
                .into(),
        )
    } else if r_5jz.has_dss && r_5jz.signer_chain_ok {
        Err(
            "đã nhúng DSS + neo root nhưng revocation_ok=false (OCSP/CRL thiếu, không verify được, hoặc bằng chứng cũ hơn thời điểm ký quá cửa sổ nới) — xem cờ"
                .into(),
        )
    } else {
        Err("chưa đạt PAdES-LT — xem các cờ ở trên".into())
    }
}
fn f_15t(input: &str, output: &str, tsa_url: &str) -> Result<(), String> {
    let lt = std::fs::read(input).map_err(|e| format!("đọc PDF '{input}' lỗi: {e}"))?;
    println!("PDF LT  : {input} ({} bytes)", lt.len());
    let r_5k6 = f_23h(&lt).map_err(|e| format!("prepare DocTimeStamp lỗi: {e}"))?;
    println!(
        "  Vùng archive-TS phủ : {} bytes",
        r_5k6.tbs.len()
    );
    println!("\n── ARCHIVE-TIMESTAMP (PAdES-LTA) ──\n  TSA: {tsa_url}");
    let token = fetch_timestamp(&f_1hl(), tsa_url, &r_5k6.tbs)?;
    println!("  ✓ token TSA: {} bytes", token.len());
    let lta = f_1cp(&r_5k6, &token)
        .map_err(|e| format!("finalize DocTimeStamp lỗi: {e}"))?;
    std::fs::write(output, &lta).map_err(|e| format!("ghi '{output}' lỗi: {e}"))?;
    println!("\nĐÃ GHI PAdES-LTA: {output} ({} bytes)", lta.len());
    let r_5k7 = verify(&lta).map_err(|e| format!("verify LTA lỗi: {e}"))?;
    f_23v(&r_5k7);
    if r_5k7.has_archive_timestamp {
        println!(
            r#"
✅ PAdES-LTA — archive-timestamp phủ trọn tài liệu (gồm /DSS) đúng VỀ CẤU TRÚC; chữ ký gốc + DSS vẫn nguyên."#
        );
        println!(
            "   is_lta()=true còn cần: signer_chain_ok + revocation_trusted + cert TSA của archive-timestamp"
        );
        println!(
            "   neo tới root TSA — kiểm bằng `verify --roots <bundle CA> --tsa <bundle root TSA>`."
        );
        Ok(())
    } else {
        Err(
            "đã nhúng DocTimeStamp nhưng verify không thấy archive-timestamp hợp lệ — xem các cờ"
                .into(),
        )
    }
}
fn f_sa(r_5k8: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    r_5k8
        .nfd()
        .filter(|c| !('\u{0300}'..='\u{036f}').contains(c))
        .map(|c| match c {
            'đ' => 'd',
            'Đ' => 'D',
            _ => c,
        })
        .filter(|c| c.is_ascii() && (*c == '\n' || !c.is_control()))
        .collect()
}
fn f_210(r_5ka: &str) -> String {
    let mut r_5k9 = String::new();
    for c in r_5ka.chars() {
        match c {
            '\\' => r_5k9.push_str("\\\\"),
            '(' => r_5k9.push_str("\\("),
            ')' => r_5k9.push_str("\\)"),
            _ => r_5k9.push(c),
        }
    }
    r_5k9
}
fn f_2ff(r_5kb: &[u8]) -> String {
    let r_5ke = Certificate::from_der(r_5kb)
        .map(|c| c.tbs_certificate.subject.to_string())
        .unwrap_or_default();
    for r_5kd in r_5ke.split(',') {
        if let Some(r_5kc) = r_5kd.trim().strip_prefix("CN=") {
            return r_5kc.trim().to_string();
        }
    }
    r_5ke
}
fn f_1d8(r_5kh: u64) -> String {
    match chrono::DateTime::<chrono::Utc>::from_timestamp(r_5kh as i64, 0) {
        Some(r_5kf) => {
            let r_5kg = chrono::FixedOffset::east_opt(7 * 3600).unwrap();
            r_5kf
                .with_timezone(&r_5kg)
                .format("%d/%m/%Y %H:%M:%S (GMT+7)")
                .to_string()
        }
        None => String::new(),
    }
}
fn f_1vf(r_5ki: &Object) -> f64 {
    match r_5ki {
        Object::Integer(i) => *i as f64,
        Object::Real(r) => *r as f64,
        _ => 0.0,
    }
}
fn f_28z(r_5kl: &Document, r_5ko: ObjectId) -> [f64; 4] {
    let mut r_5kk = Some(r_5ko);
    while let Some(r_5km) = r_5kk {
        let d = match r_5kl.get_dictionary(r_5km) {
            Ok(d) => d,
            Err(_) => break,
        };
        if let Ok(r_5kn) = d.get(b"MediaBox") {
            let r_5kj = match r_5kn {
                Object::Array(a) => Some(a.clone()),
                Object::Reference(r_5kp) => r_5kl
                    .get_object(*r_5kp)
                    .ok()
                    .and_then(|o| o.as_array().ok().cloned()),
                _ => None,
            };
            if let Some(a) = r_5kj {
                if a.len() == 4 {
                    return [f_1vf(&a[0]), f_1vf(&a[1]), f_1vf(&a[2]), f_1vf(&a[3])];
                }
            }
        }
        r_5kk = d.get(b"Parent").ok().and_then(|o| o.as_reference().ok());
    }
    [0.0, 0.0, 612.0, 792.0]
}
fn f_298(r_5kr: &Document, r_5ku: ObjectId) -> Dictionary {
    let mut r_5kq = Some(r_5ku);
    while let Some(r_5ks) = r_5kq {
        let d = match r_5kr.get_dictionary(r_5ks) {
            Ok(d) => d,
            Err(_) => break,
        };
        if let Ok(r) = d.get(b"Resources") {
            match r {
                Object::Dictionary(rd) => return rd.clone(),
                Object::Reference(r_5kv) => {
                    if let Ok(rd) = r_5kr.get_dictionary(*r_5kv) {
                        return rd.clone();
                    }
                }
                _ => {}
            }
        }
        r_5kq = d
            .get(b"Parent")
            .ok()
            .and_then(|r_5kt| r_5kt.as_reference().ok());
    }
    Dictionary::new()
}
fn f_w4(r_5kz: [f64; 4], font: &str, signer: &str, when: &str) -> String {
    let (r_5l3, r_5l4) = (r_5kz[2], r_5kz[3]);
    let r_5l2 = (r_5kz[2] - r_5kz[0]).abs().max(200.0);
    let r_5ky = (r_5l2 * 0.04).max(18.0);
    let box_w = (r_5l2 * 0.55).clamp(180.0, 360.0);
    let box_h = 68.0;
    let bx0 = r_5l3 - r_5ky - box_w;
    let r_5kw = r_5l4 - r_5ky;
    let by0 = r_5kw - box_h;
    let r_5l0 = 8.0;
    let tx = bx0 + r_5l0;
    let signer = {
        let r_5kx = f_sa(signer);
        if r_5kx.chars().count() > 42 {
            r_5kx.chars().take(42).collect::<String>() + "..."
        } else {
            r_5kx
        }
    };
    let when = f_sa(when);
    let mut r_5l1 = String::new();
    r_5l1.push_str("\nq\n1.5 w\n1 0 0 RG\n1 0 0 rg\n");
    r_5l1.push_str(&format!("{bx0:.2} {by0:.2} {box_w:.2} {box_h:.2} re\nS\n"));
    r_5l1.push_str("BT\n1 0 0 rg\n");
    r_5l1.push_str(&format!("/{font} 13 Tf\n"));
    r_5l1.push_str(&format!("{tx:.2} {:.2} Td\n", r_5kw - r_5l0 - 12.0));
    r_5l1.push_str(&format!("({}) Tj\n", f_210("CHU KY SO / DA KY SO")));
    r_5l1.push_str(&format!("/{font} 10 Tf\n"));
    r_5l1.push_str("0 -17 Td\n");
    r_5l1.push_str(&format!("({}) Tj\n", f_210(&format!("Nguoi ky: {signer}"))));
    r_5l1.push_str("0 -14 Td\n");
    r_5l1.push_str(&format!("({}) Tj\n", f_210(&format!("Ky ngay: {when}"))));
    r_5l1.push_str("ET\nQ\n");
    r_5l1
}
fn f_qu(pdf: &[u8], signer: &str, when: &str) -> Result<Vec<u8>, String> {
    let mut r_5l7 =
        Document::load_mem(pdf).map_err(|e| format!("nạp PDF (lopdf) lỗi: {e}"))?;
    let (_num, r_5ld) = r_5l7
        .get_pages()
        .into_iter()
        .next()
        .ok_or("PDF không có trang nào để đóng dấu")?;
    let r_5lb = f_28z(&r_5l7, r_5ld);
    let r_5l9 = r_5l7.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica-Bold",
    });
    const C_7A: &str = "DGTStampF";
    let mut r_5le = f_298(&r_5l7, r_5ld);
    let mut r_5l8 = match r_5le.get(b"Font") {
        Ok(Object::Dictionary(d)) => d.clone(),
        Ok(Object::Reference(r_5lg)) => r_5l7.get_dictionary(*r_5lg).cloned().unwrap_or_default(),
        _ => Dictionary::new(),
    };
    r_5l8.set(C_7A, Object::Reference(r_5l9));
    r_5le.set("Font", Object::Dictionary(r_5l8));
    let r_5lf = r_5l7.add_object(Object::Dictionary(r_5le));
    let r_5l5 = f_w4(r_5lb, C_7A, signer, when);
    let r_5lh = r_5l7.add_object(Stream::new(Dictionary::new(), r_5l5.into_bytes()));
    {
        let r_5lc = r_5l7
            .get_object_mut(r_5ld)
            .map_err(|e| format!("đọc trang lỗi: {e}"))?
            .as_dict_mut()
            .map_err(|e| format!("trang không phải dict: {e}"))?;
        r_5lc.set("Resources", Object::Reference(r_5lf));
        let r_5l6 = match r_5lc.get(b"Contents") {
            Ok(Object::Reference(r_5la)) => {
                vec![Object::Reference(*r_5la), Object::Reference(r_5lh)]
            }
            Ok(Object::Array(a)) => {
                let mut a = a.clone();
                a.push(Object::Reference(r_5lh));
                a
            }
            _ => vec![Object::Reference(r_5lh)],
        };
        r_5lc.set("Contents", Object::Array(r_5l6));
    }
    let mut out = Vec::new();
    r_5l7
        .save_to(&mut out)
        .map_err(|e| format!("lưu PDF (lopdf) lỗi: {e}"))?;
    Ok(out)
}
fn f_15v(
    module: &str,
    input: &str,
    output: &str,
    stamp: bool,
    tsa_url: Option<&str>,
) -> Result<(), String> {
    println!("Module : {module}");
    let pdf =
        std::fs::read(input).map_err(|e| format!("đọc PDF '{input}' lỗi: {e}"))?;
    println!("PDF vào: {input} ({} bytes)", pdf.len());
    let pkcs11 = context(module)?;
    let r_5lr = f_29c(&pkcs11)?;
    let idx = slot_index();
    let slot = *r_5lr.get(idx).ok_or_else(|| {
        format!(
            "không có token ở slot {idx} (chỉ có {} slot)",
            r_5lr.len()
        )
    })?;
    let label = pkcs11
        .get_token_info(slot)
        .map(|t| t.label().trim().to_string())
        .unwrap_or_default();
    println!("Token  : slot[{idx}] nhãn = {label:?}");
    let r_5lp = pkcs11
        .open_ro_session(slot)
        .map_err(|e| format!("mở phiên token lỗi: {e}"))?;
    let r_5lo = f_2d8(&r_5lp)?;
    let r_5lq = f_1vc().max(0) as u64;
    let r_5lk = if stamp {
        let out = f_qu(&pdf, &f_2ff(&r_5lo.cert), &f_1d8(r_5lq))?;
        println!(
            "\n── DẤU ĐỎ ──\n  Đã vẽ con dấu đỏ ở đầu trang 1 ({} → {} bytes)",
            pdf.len(), out.len()
        );
        out
    } else {
        pdf.clone()
    };
    let r_5lm = prepare(&r_5lk, &r_5lo.cert, r_5lq)
        .map_err(|e| format!("prepare (PAdES) lỗi: {e}"))?;
    println!("\n── PREPARE ──");
    println!("  PDF sau placeholder : {} bytes", r_5lm.pdf.len());
    println!("  tbs_digest (SHA-256): {}", hex(&r_5lm.tbs_digest));
    let r_5ll = rpassword::prompt_password(r#"
Nhập PIN token (ẩn): "#)
        .map_err(|e| format!("đọc PIN lỗi: {e}"))?;
    r_5lp
        .login(UserType::User, Some(&AuthPin::new(r_5ll.into())))
        .map_err(|_| "PIN không đúng hoặc token bị khoá".to_string())?;
    println!("Đăng nhập token: OK");
    let r_5lj = f_1cy(&r_5lp, &r_5lo.leaf_id)?;
    let r_5li = digestinfo_sha256(&r_5lm.tbs_digest);
    let signature = r_5lp
        .sign(&Mechanism::RsaPkcs, r_5lj, &r_5li)
        .map_err(|e| format!("C_Sign trên token lỗi: {e}"))?;
    println!("\n── C_SIGN (token) ──");
    println!(
        "  chữ ký RSA          : {} bytes",
        signature.len()
    );
    println!(
        "  chữ ký (đầu 16B)    : {}…",
        hex(&signature[..signature.len().min(16)])
    );
    let r_5ls = f_1c9(tsa_url, &signature)?;
    let signed = f_1cn(
        &r_5lm.pdf,
        &r_5lm.signed_attrs_der,
        &signature,
        &r_5lo.cert,
        &r_5lo.intermediates,
        r_5ls.as_deref(),
    )
    .map_err(|e| format!("finalize (lắp CMS + nhúng) lỗi: {e}"))?;
    println!("\n── FINALIZE ──");
    println!("  PDF đã ký           : {} bytes", signed.len());
    let r_5ln = verify(&signed).map_err(|e| format!("verify PDF đã ký lỗi: {e}"))?;
    f_23v(&r_5ln);
    std::fs::write(output, &signed).map_err(|e| format!("ghi PDF '{output}' lỗi: {e}"))?;
    println!("\nĐÃ GHI: {output} ({} bytes)", signed.len());
    if r_5ln.is_valid() {
        let level = if r_5ln.timestamp_present {
            "PAdES-T (có tem thời gian)"
        } else {
            "PAdES-B"
        };
        println!(
            "\n✅ KÝ THẬT THÀNH CÔNG — chữ ký {level} từ token iCA hợp lệ (signature + digest + ràng cert)."
        );
        if r_5ln.timestamp_present {
            println!(
                "   timestamp_trusted=false là bình thường khi chưa nạp trust store root TSA (--tsa lúc verify)."
            );
        }
        println!(
            "   signer_chain_ok=false là bình thường khi chưa nạp trust store gốc iCA (chỉ ở mức cấu trúc)."
        );
        Ok(())
    } else {
        Err(
            "PDF đã ghi nhưng VERIFY KHÔNG hợp lệ — xem các cờ ở trên"
                .into(),
        )
    }
}
fn f_2gd() -> Result<(rsa::pkcs1v15::SigningKey<sha2::Sha256>, Vec<u8>), String> {
    use der::Encode;
    use rsa::pkcs1v15::SigningKey;
    use rsa::RsaPrivateKey;
    use std::str::FromStr;
    use x509_cert::builder::{Builder, CertificateBuilder, Profile};
    use x509_cert::name::Name;
    use x509_cert::serial_number::SerialNumber;
    use x509_cert::spki::SubjectPublicKeyInfoOwned;
    use x509_cert::time::Validity;
    let mut r_5lv = rand::thread_rng();
    let r_5lu =
        RsaPrivateKey::new(&mut r_5lv, 2048).map_err(|e| format!("keygen lỗi: {e}"))?;
    let r_5lw = SigningKey::<sha2::Sha256>::new(r_5lu.clone());
    let spki = SubjectPublicKeyInfoOwned::from_key(r_5lu.to_public_key())
        .map_err(|e| format!("spki lỗi: {e}"))?;
    let cert = CertificateBuilder::new(
        Profile::Root,
        SerialNumber::from(1u32),
        Validity::from_now(core::time::Duration::from_secs(365 * 24 * 3600))
            .map_err(|e| format!("validity lỗi: {e}"))?,
        Name::from_str("CN=DigiText Soft Selftest").map_err(|e| format!("name lỗi: {e}"))?,
        spki,
        &r_5lw,
    )
    .map_err(|e| format!("builder lỗi: {e}"))?
    .build()
    .map_err(|e| format!("build cert lỗi: {e}"))?;
    let r_5lt = cert
        .to_der()
        .map_err(|e| format!("cert to_der lỗi: {e}"))?;
    Ok((r_5lw, r_5lt))
}
fn f_15u(
    input: &str,
    output: Option<&str>,
    stamp: bool,
    tsa_url: Option<&str>,
) -> Result<(), String> {
    use rsa::signature::{SignatureEncoding, Signer};
    let pdf =
        std::fs::read(input).map_err(|e| format!("đọc PDF '{input}' lỗi: {e}"))?;
    println!("PDF vào : {input} ({} bytes)", pdf.len());
    println!(
        "Chế độ  : SELFTEST (khoá phần mềm — KHÔNG token){}{}",
        if stamp { " + DẤU ĐỎ" } else { "" },
        if tsa_url.is_some() { " + PAdES-T" } else { "" }
    );
    let (r_5m2, r_5lx) = f_2gd()?;
    let r_5m1 = f_1vc().max(0) as u64;
    let r_5ly = if stamp {
        let out = f_qu(&pdf, &f_2ff(&r_5lx), &f_1d8(r_5m1))?;
        println!(
            "\n── DẤU ĐỎ ──\n  Đã vẽ con dấu đỏ ở đầu trang 1 ({} → {} bytes)",
            pdf.len(), out.len()
        );
        out
    } else {
        pdf.clone()
    };
    let r_5lz = prepare(&r_5ly, &r_5lx, r_5m1)
        .map_err(|e| format!("prepare (PAdES) lỗi: {e}"))?;
    println!("\n── PREPARE ──");
    println!("  PDF sau placeholder : {} bytes", r_5lz.pdf.len());
    println!("  tbs_digest (SHA-256): {}", hex(&r_5lz.tbs_digest));
    let signature = r_5m2.sign(&r_5lz.signed_attrs_der).to_bytes();
    println!(r#"
── SIGN (khoá phần mềm) ──"#);
    println!(
        "  chữ ký RSA          : {} bytes",
        signature.len()
    );
    let r_5m3 = f_1c9(tsa_url, &signature)?;
    let signed = f_1cn(
        &r_5lz.pdf,
        &r_5lz.signed_attrs_der,
        &signature,
        &r_5lx,
        &[],
        r_5m3.as_deref(),
    )
    .map_err(|e| format!("finalize lỗi: {e}"))?;
    println!("\n── FINALIZE ──");
    println!("  PDF đã ký           : {} bytes", signed.len());
    let r_5m0 = verify(&signed).map_err(|e| format!("verify lỗi: {e}"))?;
    f_23v(&r_5m0);
    if let Some(out) = output {
        std::fs::write(out, &signed).map_err(|e| format!("ghi PDF '{out}' lỗi: {e}"))?;
        println!(
            "\nĐÃ GHI (ký bằng khoá phần mềm, KHÔNG phải token): {out}"
        );
    }
    if r_5m0.is_valid() {
        println!(
            r#"
✅ SELFTEST PASS — PDF này ký + verify được. Lệnh `sign` thật trên đúng file sẽ chạy."#
        );
        Ok(())
    } else {
        Err(
            "SELFTEST KHÔNG hợp lệ — PDF có thể không tương thích luồng ký; xem các cờ trên"
                .into(),
        )
    }
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn f_2ob(prog: &str) -> String {
    format!(
        "Dùng:\n  {prog} sign            <input.pdf> <output.pdf> [--timestamp | --tsa-url <url>]  # ký THẬT bằng token (vô hình)\n  {prog} sign-stamp      <input.pdf> <output.pdf> [--timestamp | --tsa-url <url>]  # ký THẬT + DẤU ĐỎ đầu trang 1\n  {prog} selftest        <input.pdf> [output.pdf]  [--timestamp | --tsa-url <url>]  # ký thử khoá phần mềm (KHÔNG token)\n  {prog} selftest-stamp  <input.pdf> <output.pdf> [--timestamp | --tsa-url <url>]  # ký thử khoá phần mềm + DẤU ĐỎ\n  {prog} verify          <signed.pdf> [--roots <bundle.pem>] [--tsa <tsa-root.pem>]  # xác minh + kiểm tin cậy\n  {prog} make-lt         <signed.pdf> <out-lt.pdf> --chain <bundle.pem> --leaf <leaf.pem> [--tsa-roots <bundle root TSA>]  # nâng lên PAdES-LT (OCSP/CRL)\n  {prog} make-lta        <lt.pdf> <out-lta.pdf> [--timestamp | --tsa-url <url>]  # nâng lên PAdES-LTA (archive-timestamp)\n\n--timestamp    : dùng TSA mặc định (DigiCert RSA — chỉ để THỬ; SẢN XUẤT dùng TSA VN đủ điều kiện).\n  --tsa-url <u>  : PAdES-T với TSA chỉ định (RFC 3161, application/timestamp-query).\n  Không có lệnh + đúng 2 tham số ⇒ hiểu là `sign` (tương thích cũ)."
    )
}
fn f_1d6(r_5m4: &[String], name: &str) -> Option<String> {
    r_5m4
        .iter()
        .position(|a| a == name)
        .and_then(|i| r_5m4.get(i + 1).cloned())
}
fn f_23a(r_5m5: &[String]) -> Vec<String> {
    const C_PJ: &[&str] = &["--tsa-url", "--roots", "--tsa", "--chain", "--leaf"];
    let mut out = Vec::new();
    let mut r_5m6 = false;
    for a in r_5m5 {
        if r_5m6 {
            r_5m6 = false;
            continue;
        }
        if a.starts_with("--") {
            if C_PJ.contains(&a.as_str()) {
                r_5m6 = true;
            }
            continue;
        }
        out.push(a.clone());
    }
    out
}
enum CliError {
    Usage,
    Fatal2(String),
    Run(String),
}
impl From<String> for CliError {
    fn from(e: String) -> Self {
        CliError::Run(e)
    }
}
fn f_1tz() -> Result<String, CliError> {
    f_13r()
        .ok_or_else(|| {
            CliError::Fatal2(
                "Không tìm thấy module PKCS#11. Đặt DIGITEXT_PKCS11_MODULE=<đường-dẫn .so/.dll/.dylib>."
                    .into(),
            )
        })
}
pub fn cli_main(prog: &str, r_5m7: &[String]) -> ExitCode {
    match run(r_5m7) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError::Usage) => {
            eprintln!("{}", f_2ob(prog));
            ExitCode::from(2)
        }
        Err(CliError::Fatal2(m)) => {
            eprintln!("{m}");
            ExitCode::from(2)
        }
        Err(CliError::Run(e)) => {
            eprintln!("\n❌ {e}");
            ExitCode::FAILURE
        }
    }
}
fn run(r_5m8: &[String]) -> Result<(), CliError> {
    let r_5mb = f_23a(r_5m8);
    let r_5m9 = r_5mb.first().map(String::as_str);
    let tsa_url: Option<String> = f_1d6(r_5m8, "--tsa-url").or_else(|| {
        r_5m8
            .iter()
            .any(|a| a == "--timestamp")
            .then(|| C_42.to_string())
    });
    let tsa = tsa_url.as_deref();
    let r_5ma = |a: &str| {
        matches!(
            a,
            "sign"
                | "sign-stamp"
                | "selftest"
                | "selftest-stamp"
                | "verify"
                | "make-lt"
                | "make-lta"
        )
    };
    match r_5m9 {
        Some("sign") if r_5mb.len() == 3 => Ok(f_15v(&f_1tz()?, &r_5mb[1], &r_5mb[2], false, tsa)?),
        Some("sign-stamp") if r_5mb.len() == 3 => {
            Ok(f_15v(&f_1tz()?, &r_5mb[1], &r_5mb[2], true, tsa)?)
        }
        Some("selftest") if r_5mb.len() == 2 => Ok(f_15u(&r_5mb[1], None, false, tsa)?),
        Some("selftest") if r_5mb.len() == 3 => Ok(f_15u(&r_5mb[1], Some(&r_5mb[2]), false, tsa)?),
        Some("selftest-stamp") if r_5mb.len() == 3 => {
            Ok(f_15u(&r_5mb[1], Some(&r_5mb[2]), true, tsa)?)
        }
        Some("selftest-stamp") if r_5mb.len() == 2 => Ok(f_15u(&r_5mb[1], None, true, tsa)?),
        Some("verify") if r_5mb.len() >= 2 => Ok(f_15w(
            &r_5mb[1],
            f_1d6(r_5m8, "--roots").as_deref(),
            f_1d6(r_5m8, "--tsa").as_deref(),
        )?),
        Some("make-lt") if r_5mb.len() >= 3 => {
            match (f_1d6(r_5m8, "--chain"), f_1d6(r_5m8, "--leaf")) {
                (Some(c), Some(l)) => {
                    let r_5mc = f_1d6(r_5m8, "--tsa-roots").or_else(|| f_1d6(r_5m8, "--tsa"));
                    Ok(f_15s(&r_5mb[1], &r_5mb[2], &c, &l, r_5mc.as_deref())?)
                }
                _ => {
                    eprintln!(
                        "make-lt cần: --chain <bundle.pem> --leaf <leaf.pem>"
                    );
                    Err(CliError::Usage)
                }
            }
        }
        Some("make-lta") if r_5mb.len() == 3 => {
            Ok(f_15t(&r_5mb[1], &r_5mb[2], tsa.unwrap_or(C_42))?)
        }
        _ if r_5mb.len() == 2 && r_5m9.map(|c| !r_5ma(c)).unwrap_or(false) => {
            Ok(f_15v(&f_1tz()?, &r_5mb[0], &r_5mb[1], false, tsa)?)
        }
        _ => Err(CliError::Usage),
    }
}
