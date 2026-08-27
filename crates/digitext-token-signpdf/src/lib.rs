// Copyright (c) 2026 DigiText. All rights reserved.
use cryptoki::context::{CInitializeArgs, Pkcs11};
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
        .map_err(|e| format!("a2jDtG5nIG7huqFwIMSRxrDhu6NjIG1vZHVsZSAn{module}': {e}"))?;
    pkcs11
        .initialize(CInitializeArgs::OsThreads)
        .map_err(|e| format!("a2jhu59pIHThuqFvIFBLQ1MjMTEgbOG7l2k6IA=={e}"))?;
    Ok(pkcs11)
}
fn f_29c(pkcs11: &Pkcs11) -> Result<Vec<Slot>, String> {
    let r_5ic = pkcs11
        .get_slots_with_token()
        .map_err(|e| format!("bGnhu4d0IGvDqiBzbG90IGzhu5dpOiA={e}"))?;
    if !r_5ic.is_empty() {
        return Ok(r_5ic);
    }
    let r_5ia = pkcs11
        .get_all_slots()
        .map_err(|e| format!("bGnhu4d0IGvDqiB0b8OgbiBi4buZIHNsb3QgbOG7l2k6IA=={e}"))?;
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
            "KGNow7ogw706IGdldF9zbG90c193aXRoX3Rva2VuKCkgcuG7l25nIOKAlCBkw7luZyBmYWxsYmFjayBjYXJkLXByZXNlbnQsIA=={} slot)",
            present.len()
        );
        return Ok(present);
    }
    if !r_5ia.is_empty() {
        eprintln!(
            "KGNow7ogw706IGtow7RuZyBzbG90IG7DoG8gYsOhbyB0b2tlbi1wcmVzZW50IOKAlCB0aOG7rSB0cuG7sWMgdGnhur9wIA=={} slot)",
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
                "  [{tag}] {}IGJ5dGVzIERFUiAoa2jDtG5nIHBhcnNlIMSRxrDhu6NjOiA={e})",
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
        .map_err(|e| format!("dMOsbSBjZXJ0IGzhu5dpOiA={e}"))?;
    let mut r_5it: Vec<(Vec<u8>, Option<Vec<u8>>)> = Vec::new();
    for r_5iu in r_5is {
        if let Some(der) = attr_bytes(r_5iy, r_5iu, AttributeType::Value) {
            let r_5iv = attr_bytes(r_5iy, r_5iu, AttributeType::Id);
            r_5it.push((der, r_5iv));
        }
    }
    if r_5it.is_empty() {
        return Err("dG9rZW4ga2jDtG5nIGPDsyBjaOG7qW5nIHRoxrAgKGNlcnRpZmljYXRlKQ==".into());
    }
    println!(
        "\n── {}IGNo4bupbmcgdGjGsCB0csOqbiB0b2tlbiDilIDilIA=",
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
        .ok_or("a2jDtG5nIGNo4buNbiDEkcaw4bujYyBjaOG7qW5nIHRoxrAgbmfGsOG7nWkga8O9")?;
    let cert = r_5it[r_5ix.leaf].0.clone();
    let leaf_id = r_5it[r_5ix.leaf].1.clone();
    let intermediates: Vec<Vec<u8>> = r_5ix
        .intermediates
        .iter()
        .map(|&i| r_5it[i].0.clone())
        .collect();
    println!(
        "CuKUgOKUgCDEkMODIENI4buMTiBuZ8aw4budaSBrw70gKGxlYWYgPSBjZXJ0Ww=={}]) ──",
        r_5ix.leaf
    );
    f_23u("leaf", &cert);
    println!(
        "U+G7kSBjZXJ0IHRydW5nIGdpYW4gbmjDum5nIHbDoG8gQ01TOiA={}",
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
        .map_err(|e| format!("dMOsbSBraG/DoSBrw70gbOG7l2k6IA=={e}"))?;
    let r_5j1 = leaf_id
        .as_deref()
        .and_then(|r_5j4| {
            r_5j3.iter().copied().find(|r_5j0| {
                attr_bytes(r_5j2, *r_5j0, AttributeType::Id).as_deref() == Some(r_5j4)
            })
        })
        .or_else(|| r_5j3.first().copied())
        .ok_or("dG9rZW4ga2jDtG5nIGPDsyBraG/DoSBrw70gKENLQV9TSUdOKQ==")?;
    if let Ok(r_5iz) = r_5j2.get_attributes(r_5j1, &[AttributeType::KeyType]) {
        if let Some(Attribute::KeyType(kt)) = r_5iz.into_iter().next() {
            if kt != KeyType::RSA {
                return Err(
                    format!(
                        "a2hvw6Ega2jDtG5nIHBo4bqjaSBSU0EgKA=={kt:?}KSDigJQgY8O0bmcgY+G7pSBuw6B5IGNo4buJIGvDvSBSU0E="
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
        "  signer_chain_ok       : {}IChj4bqnbiB0cnVzdCBzdG9yZSBpQ0EgxJHhu4MgdHJ1ZSk=",
        r.signer_chain_ok
    );
    println!("  timestamp_present     : {}", r.timestamp_present);
    println!(
        "  timestamp_structural_ok: {}IChpbXByaW50ICsgY2jhu68ga8O9IFRTQSBraOG7m3Ap",
        r.timestamp_structural_ok
    );
    println!(
        "  timestamp_trusted     : {}IChj4bqnbiAtLXRzYSA8cm9vdCBUU0E+IMSR4buDIHRydWUp",
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
        "  revocation_ok         : {}IChPQ1NQL0NSTCBuw7NpIEdvb2QgdOG6oWkgbeG7kWMg{:?})",
        r.revocation_ok, r.revocation_reference_unix
    );
    println!(
        "  revocation_trusted    : {}ICht4buRYyB0csOqbiBkbyBUU0EgxJHDs25nLCBraMO0bmcgcGjhuqNpIHNpZ25pbmdUaW1lIHThu7Ega2hhaSk=",
        r.revocation_trusted
    );
    println!(
        "  has_archive_timestamp : {}ICht4bubaSDhu58gbeG7qWMgQ+G6pFUgVFLDmkMp",
        r.has_archive_timestamp
    );
    println!(
        "  archive_ts_trusted    : {}IChj4bqnbiAtLXRzYSA8cm9vdCBUU0E+IMSR4buDIHRydWUp",
        r.archive_timestamp_trusted
    );
    println!("  → is_valid() PAdES-B  : {}", r.is_valid());
    println!("  → is_lt()   PAdES-LT  : {}", r.is_lt());
    println!("  → is_lta()  PAdES-LTA : {}", r.is_lta());
}
fn f_1ow(path: &str) -> Result<TrustStore, String> {
    let pem = std::fs::read_to_string(path)
        .map_err(|e| format!("xJHhu41jIHRydXN0IHN0b3JlICc={path}JyBs4buXaTog{e}"))?;
    TrustStore::from_pem(&pem)
        .map_err(|e| format!("buG6oXAgdHJ1c3Qgc3RvcmUgJw=={path}JyBs4buXaTog{e}"))
}
fn f_15w(input: &str, roots: Option<&str>, tsa: Option<&str>) -> Result<(), String> {
    let pdf =
        std::fs::read(input).map_err(|e| format!("xJHhu41jIFBERiAn{input}JyBs4buXaTog{e}"))?;
    println!("PDF     : {input} ({} bytes)", pdf.len());
    let r_5j6 = roots.map(f_1ow).transpose()?;
    let r_5j7 = tsa.map(f_1ow).transpose()?;
    match &r_5j6 {
        Some(s) => {
            println!(
                "VHJ1c3Qgc3RvcmUgbmfGsOG7nWkga8O9OiA={} cert (anchor)",
                s.len()
            )
        }
        None => {
            println!(
                "VHJ1c3Qgc3RvcmUgbmfGsOG7nWkga8O9OiAoa2jDtG5nKSDihpIgc2lnbmVyX2NoYWluX29rIGNo4buJIOG7nyBt4bupYyBj4bqldSB0csO6Yw=="
            )
        }
    }
    if let Some(s) = &r_5j7 {
        println!("Trust store TSA     : {} cert", s.len());
    }
    let r_5j5 = f_2q1(&pdf, r_5j6.as_ref(), r_5j7.as_ref())
        .map_err(|e| format!("dmVyaWZ5IGzhu5dpOiA={e}"))?;
    f_23v(&r_5j5);
    if r_5j5.is_lt() {
        println!(
            "CuKchSBQQWRFUy1MVCDigJQgY2jhu68ga8O9IGjhu6NwIGzhu4csIG5lbyB04bubaSByb290IHRpbiBj4bqteSwgY8OzIERTUywgdsOgIGtow7RuZyBi4buLIHRodSBo4buTaSBU4bqgSSBN4buQQyBUSOG7nEkgR0lBTiBETyBUU0EgxJDDk05HLg=="
        );
    } else if r_5j5.signer_chain_ok && r_5j5.is_valid() && r_5j5.has_dss {
        println!(
            "CuKXiyBDaOG7ryBrw70gaOG7o3AgbOG7hyArIG5lbyByb290ICsgY8OzIERTUywgbmjGsG5nIENIxq9BIMSR4bunIExUOiBi4bqxbmcgY2jhu6luZyB0aHUgaOG7k2kgcGjhuqNpIMSR4buRaSBjaGnhur91"
        );
        println!(
            "ICB24bubaSBt4buRYyB0aOG7nWkgZ2lhbiBUSU4gxJDGr+G7okMgKHRlbSBUU0EgbmVvIHJvb3Qg4oCUIGPhuqVwIC0tdHNhIDxidW5kbGUgcm9vdCBUU0E+KTsgY2jhu4kgZOG7sWE="
        );
        println!(
            "ICBzaWduaW5nVGltZSBkbyBuZ8aw4budaSBrw70gdOG7sSBraGFpIHRow6wga2jDtG5nIGvhur90IGx14bqtbiDEkcaw4bujYyAoeGVtIHJldm9jYXRpb25fdHJ1c3RlZCku"
        );
    } else if r_5j5.signer_chain_ok && r_5j5.is_valid() {
        println!(
            "CuKchSBDaOG7ryBrw70gaOG7o3AgbOG7hyBWw4AgbmVvIHThu5tpIHJvb3QgdGluIGPhuq15IChzaWduZXJfY2hhaW5fb2s9dHJ1ZSkuIENoxrBhIGPDsyBEU1Mg4oeSIGNoxrBhIExULg=="
        );
    } else if r_5j5.is_valid() {
        println!(
            "CuKXiyBDaOG7ryBrw70gaOG7o3AgbOG7hyAoUEFkRVMtQikgbmjGsG5nIENIxq9BIG5lbyByb290IHRpbiBj4bqteSDigJQgY+G6pXAgLS1yb290cyA8YnVuZGxlIGlDQT4gxJHhu4Mga2nhu4NtLg=="
        );
    } else {
        return Err(
            "Y2jhu68ga8O9IEtIw5RORyBo4bujcCBs4buHIOKAlCB4ZW0gY8OhYyBj4budIOG7nyB0csOqbg==".into(),
        );
    }
    Ok(())
}
fn f_220(path: &str) -> Result<Vec<Vec<u8>>, String> {
    let r_5ja = std::fs::read(path).map_err(|e| format!("xJHhu41jICc={path}JyBs4buXaTog{e}"))?;
    let r_5j8 = pem::parse_many(&r_5ja).map_err(|e| format!("PEM '{path}JyBo4buPbmc6IA=={e}"))?;
    let r_5j9: Vec<Vec<u8>> = r_5j8
        .into_iter()
        .filter(|p| p.tag() == "CERTIFICATE")
        .map(|p| p.into_contents())
        .collect();
    if r_5j9.is_empty() {
        return Err(format!(
            "'{path}JyBraMO0bmcgY8OzIGto4buRaSBDRVJUSUZJQ0FURQ=="
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
        .map_err(|e| format!("xJHhu41jIHBo4bqjbiBo4buTaSA={url}: {e}"))?;
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
        .map_err(|e| format!("xJHhu41jIHBo4bqjbiBo4buTaSA={url}: {e}"))?;
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
        .map_err(|e| format!("ZOG7sW5nIFRpbWVTdGFtcFJlcSBs4buXaTog{e}"))?;
    let r_5jn = f_1ho(r_5jk, tsa_url, "application/timestamp-query", &r_5jm)?;
    tsa::f_2le(&r_5jn).map_err(|e| format!("VFNBIHThu6sgY2jhu5FpL2tow7RuZyB0cuG6oyB0b2tlbjog{e}"))
}
fn f_1c9(tsa_url: Option<&str>, signature: &[u8]) -> Result<Option<Vec<u8>>, String> {
    match tsa_url {
        None => Ok(None),
        Some(url) => {
            println!("CuKUgOKUgCBURU0gVEjhu5xJIEdJQU4gKFBBZEVTLVQpIOKUgOKUgAogIFRTQTog{url}");
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
        std::fs::read(input).map_err(|e| format!("xJHhu41jIFBERiAn{input}JyBs4buXaTog{e}"))?;
    println!("PDF     : {input} ({} bytes)", pdf.len());
    let leaf = f_220(r_5jw)?.into_iter().next().ok_or("bGVhZiBy4buXbmc=")?;
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
        .ok_or("YnVuZGxlIC0tY2hhaW4gdGhp4bq/dSBjZXJ0IHRydW5nIGdpYW4gKENBIGPhuqVwIHBow6F0KQ==")?;
    let ocsp_url = ocsp::f_1vw(&leaf).ok_or("Y2VydCBsZWFmIGtow7RuZyBraGFpIE9DU1AgVVJMIChBSUEp")?;
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
                println!("ICDinJMgT0NTUCBwaOG6o24gaOG7k2k6IA=={} bytes", r_5k1.len());
                r_5jx.push(r_5k1);
            }
            Err(e) => println!("ICDimqAgT0NTUCB0aOG6pXQgYuG6oWk6IA=={e}"),
        },
        Err(e) => println!("ICDimqAgZOG7sW5nIE9DU1AgcmVxdWVzdCBs4buXaTog{e}"),
    }
    if let Some(u) = &r_5jt {
        match f_1hn(&r_5jo, u) {
            Ok(r_5js) => {
                println!("  ✓ CRL: {} bytes", r_5js.len());
                r_5ju.push(r_5js);
            }
            Err(e) => println!("ICDimqAgQ1JMIHRo4bqldCBi4bqhaTog{e}"),
        }
    }
    if r_5jx.is_empty() && r_5ju.is_empty() {
        println!(
            "ICDimqAgS2jDtG5nIGzhuqV5IMSRxrDhu6NjIE9DU1AgbOG6q24gQ1JMIOKGkiBEU1MgY2jhu4kgY8OzIGNlcnQsIHJldm9jYXRpb25fb2sgc+G6vSA9IGZhbHNl"
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
        .map_err(|e| format!("ZW1iZWRfZHNzIChuaMO6bmcgL0RTUykgbOG7l2k6IA=={e}"))?;
    std::fs::write(output, &lt).map_err(|e| format!("ghi '{output}JyBs4buXaTog{e}"))?;
    println!("CsSQw4MgR0hJIFBBZEVTLUxUOiA={output} ({} bytes)", lt.len());
    let store = f_1ow(r_5jr)?;
    let r_5k4 = match r_5k3 {
        Some(r_5jy) => Some(f_1ow(r_5jy)?),
        None => None,
    };
    let r_5jz = f_2q1(&lt, Some(&store), r_5k4.as_ref())
        .map_err(|e| format!("dmVyaWZ5IExUIGzhu5dpOiA={e}"))?;
    f_23v(&r_5jz);
    if r_5jz.is_lt() {
        println!(
            "CuKchSBQQWRFUy1MVCDigJQgY2jhu68ga8O9IGjhu6NwIGzhu4csIG5lbyByb290IHRpbiBj4bqteSwgY8OzIERTUywga2jDtG5nIGLhu4sgdGh1IGjhu5NpLiBU4buxIGNo4bupbmcgbWluaCBkw6BpIGjhuqFuLg=="
        );
        Ok(())
    } else if r_5jz.has_dss && r_5jz.signer_chain_ok && r_5jz.revocation_ok {
        if r_5k3.is_none() {
            println!(
                "CuKaoO+4jyAgQ0jGr0EgS+G6vlQgTFXhuqxOIMSRxrDhu6NjIG3hu6ljIFBBZEVTLUxUOiB0aGnhur91IGJ1bmRsZSByb290IFRTQSBuw6puIG3hu5FjIMSR4buRaSBjaGnhur91IHRodSBo4buTaSByxqFpIHbhu4Egc2lnbmluZ1RpbWUgTkfGr+G7nEkgS8OdIFThu7AgS0hBSSAocmV2b2NhdGlvbl90cnVzdGVkPWZhbHNlKS4="
            );
            println!(
                "ICAgVOG7h3AgxJHDoyBnaGkgxJHDum5nIChEU1MgKyBuZW8gcm9vdCArIGLhurFuZyBjaOG7qW5nIEdvb2QpLiBL4bq/dCBsdeG6rW4gYuG6sW5nOgogICBkaWdpdGV4dCBzaWduIG1ha2UtbHQgPGluPiA8b3V0PiAtLWNoYWluIDxidW5kbGUgQ0E+IC0tbGVhZiA8bGVhZi5wZW0+IC0tdHNhLXJvb3RzIDxidW5kbGUgcm9vdCBUU0E+CiAgIGhv4bq3YzogZGlnaXRleHQgc2lnbiB2ZXJpZnkgPG91dD4gLS1yb290cyA8YnVuZGxlIENBPiAtLXRzYSA8YnVuZGxlIHJvb3QgVFNBPg=="
            );
            return Ok(());
        }
        Err(
            "xJHDoyBuaMO6bmcgRFNTICsgbmVvIHJvb3QgKyBi4bqxbmcgY2jhu6luZyBuw7NpIEdvb2QsIG5oxrBuZyBt4buRYyDEkeG7kWkgY2hp4bq/dSBsw6Agc2lnbmluZ1RpbWUgVOG7sCBLSEFJIChyZXZvY2F0aW9uX3RydXN0ZWQ9ZmFsc2UpIETDmSDEkMODIMSRxrBhIHJvb3QgVFNBOiBQREYgcGjhuqNpIGPDsyB0ZW0gdGjhu51pIGdpYW4gVFNBIG5lbyDEkcaw4bujYyB04bubaSByb290IMSRw7MgKFBBZEVTLVQpIG3hu5tpIMSR4bqhdCBQQWRFUy1MVA=="
                .into(),
        )
    } else if r_5jz.has_dss && r_5jz.signer_chain_ok {
        Err(
            "xJHDoyBuaMO6bmcgRFNTICsgbmVvIHJvb3QgbmjGsG5nIHJldm9jYXRpb25fb2s9ZmFsc2UgKE9DU1AvQ1JMIHRoaeG6v3UsIGtow7RuZyB2ZXJpZnkgxJHGsOG7o2MsIGhv4bq3YyBi4bqxbmcgY2jhu6luZyBjxakgaMahbiB0aOG7nWkgxJFp4buDbSBrw70gcXXDoSBj4butYSBz4buVIG7hu5tpKSDigJQgeGVtIGPhu50="
                .into(),
        )
    } else {
        Err("Y2jGsGEgxJHhuqF0IFBBZEVTLUxUIOKAlCB4ZW0gY8OhYyBj4budIOG7nyB0csOqbg==".into())
    }
}
fn f_15t(input: &str, output: &str, tsa_url: &str) -> Result<(), String> {
    let lt = std::fs::read(input).map_err(|e| format!("xJHhu41jIFBERiAn{input}JyBs4buXaTog{e}"))?;
    println!("PDF LT  : {input} ({} bytes)", lt.len());
    let r_5k6 = f_23h(&lt).map_err(|e| format!("cHJlcGFyZSBEb2NUaW1lU3RhbXAgbOG7l2k6IA=={e}"))?;
    println!(
        "ICBWw7luZyBhcmNoaXZlLVRTIHBo4bunIDog{} bytes",
        r_5k6.tbs.len()
    );
    println!("\n── ARCHIVE-TIMESTAMP (PAdES-LTA) ──\n  TSA: {tsa_url}");
    let token = fetch_timestamp(&f_1hl(), tsa_url, &r_5k6.tbs)?;
    println!("  ✓ token TSA: {} bytes", token.len());
    let lta = f_1cp(&r_5k6, &token)
        .map_err(|e| format!("ZmluYWxpemUgRG9jVGltZVN0YW1wIGzhu5dpOiA={e}"))?;
    std::fs::write(output, &lta).map_err(|e| format!("ghi '{output}JyBs4buXaTog{e}"))?;
    println!("CsSQw4MgR0hJIFBBZEVTLUxUQTog{output} ({} bytes)", lta.len());
    let r_5k7 = verify(&lta).map_err(|e| format!("dmVyaWZ5IExUQSBs4buXaTog{e}"))?;
    f_23v(&r_5k7);
    if r_5k7.has_archive_timestamp {
        println!(
            "CuKchSBQQWRFUy1MVEEg4oCUIGFyY2hpdmUtdGltZXN0YW1wIHBo4bunIHRy4buNbiB0w6BpIGxp4buHdSAoZ+G7k20gL0RTUykgxJHDum5nIFbhu4AgQ+G6pFUgVFLDmkM7IGNo4buvIGvDvSBn4buRYyArIERTUyB24bqrbiBuZ3V5w6puLg=="
        );
        println!(
            "ICAgaXNfbHRhKCk9dHJ1ZSBjw7JuIGPhuqduOiBzaWduZXJfY2hhaW5fb2sgKyByZXZvY2F0aW9uX3RydXN0ZWQgKyBjZXJ0IFRTQSBj4bunYSBhcmNoaXZlLXRpbWVzdGFtcA=="
        );
        println!(
            "ICAgbmVvIHThu5tpIHJvb3QgVFNBIOKAlCBraeG7g20gYuG6sW5nIGB2ZXJpZnkgLS1yb290cyA8YnVuZGxlIENBPiAtLXRzYSA8YnVuZGxlIHJvb3QgVFNBPmAu"
        );
        Ok(())
    } else {
        Err(
            "xJHDoyBuaMO6bmcgRG9jVGltZVN0YW1wIG5oxrBuZyB2ZXJpZnkga2jDtG5nIHRo4bqleSBhcmNoaXZlLXRpbWVzdGFtcCBo4bujcCBs4buHIOKAlCB4ZW0gY8OhYyBj4bud"
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
        Document::load_mem(pdf).map_err(|e| format!("buG6oXAgUERGIChsb3BkZikgbOG7l2k6IA=={e}"))?;
    let (_num, r_5ld) = r_5l7
        .get_pages()
        .into_iter()
        .next()
        .ok_or("UERGIGtow7RuZyBjw7MgdHJhbmcgbsOgbyDEkeG7gyDEkcOzbmcgZOG6pXU=")?;
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
            .map_err(|e| format!("xJHhu41jIHRyYW5nIGzhu5dpOiA={e}"))?
            .as_dict_mut()
            .map_err(|e| format!("dHJhbmcga2jDtG5nIHBo4bqjaSBkaWN0OiA={e}"))?;
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
        .map_err(|e| format!("bMawdSBQREYgKGxvcGRmKSBs4buXaTog{e}"))?;
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
        std::fs::read(input).map_err(|e| format!("xJHhu41jIFBERiAn{input}JyBs4buXaTog{e}"))?;
    println!("UERGIHbDoG86IA=={input} ({} bytes)", pdf.len());
    let pkcs11 = context(module)?;
    let r_5lr = f_29c(&pkcs11)?;
    let idx = slot_index();
    let slot = *r_5lr.get(idx).ok_or_else(|| {
        format!(
            "a2jDtG5nIGPDsyB0b2tlbiDhu58gc2xvdCA={idx}IChjaOG7iSBjw7Mg{} slot)",
            r_5lr.len()
        )
    })?;
    let label = pkcs11
        .get_token_info(slot)
        .map(|t| t.label().trim().to_string())
        .unwrap_or_default();
    println!("Token  : slot[{idx}XSBuaMOjbiA9IA=={label:?}");
    let r_5lp = pkcs11
        .open_ro_session(slot)
        .map_err(|e| format!("beG7nyBwaGnDqm4gdG9rZW4gbOG7l2k6IA=={e}"))?;
    let r_5lo = f_2d8(&r_5lp)?;
    let r_5lq = f_1vc().max(0) as u64;
    let r_5lk = if stamp {
        let out = f_qu(&pdf, &f_2ff(&r_5lo.cert), &f_1d8(r_5lq))?;
        println!(
            "CuKUgOKUgCBE4bqkVSDEkOG7jiDilIDilIAKICDEkMOjIHbhur0gY29uIGThuqV1IMSR4buPIOG7nyDEkeG6p3UgdHJhbmcgMSAo{} → {} bytes)",
            pdf.len(), out.len()
        );
        out
    } else {
        pdf.clone()
    };
    let r_5lm = prepare(&r_5lk, &r_5lo.cert, r_5lq)
        .map_err(|e| format!("cHJlcGFyZSAoUEFkRVMpIGzhu5dpOiA={e}"))?;
    println!("\n── PREPARE ──");
    println!("  PDF sau placeholder : {} bytes", r_5lm.pdf.len());
    println!("  tbs_digest (SHA-256): {}", hex(&r_5lm.tbs_digest));
    let r_5ll = rpassword::prompt_password("Ck5o4bqtcCBQSU4gdG9rZW4gKOG6qW4pOiA=")
        .map_err(|e| format!("xJHhu41jIFBJTiBs4buXaTog{e}"))?;
    r_5lp
        .login(UserType::User, Some(&AuthPin::new(r_5ll)))
        .map_err(|_| "UElOIGtow7RuZyDEkcO6bmcgaG/hurdjIHRva2VuIGLhu4sga2hvw6E=".to_string())?;
    println!("xJDEg25nIG5o4bqtcCB0b2tlbjogT0s=");
    let r_5lj = f_1cy(&r_5lp, &r_5lo.leaf_id)?;
    let r_5li = digestinfo_sha256(&r_5lm.tbs_digest);
    let signature = r_5lp
        .sign(&Mechanism::RsaPkcs, r_5lj, &r_5li)
        .map_err(|e| format!("Q19TaWduIHRyw6puIHRva2VuIGzhu5dpOiA={e}"))?;
    println!("\n── C_SIGN (token) ──");
    println!(
        "ICBjaOG7ryBrw70gUlNBICAgICAgICAgIDog{} bytes",
        signature.len()
    );
    println!(
        "ICBjaOG7ryBrw70gKMSR4bqndSAxNkIpICAgIDog{}…",
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
    .map_err(|e| format!("ZmluYWxpemUgKGzhuq9wIENNUyArIG5ow7puZykgbOG7l2k6IA=={e}"))?;
    println!("\n── FINALIZE ──");
    println!("ICBQREYgxJHDoyBrw70gICAgICAgICAgIDog{} bytes", signed.len());
    let r_5ln = verify(&signed).map_err(|e| format!("dmVyaWZ5IFBERiDEkcOjIGvDvSBs4buXaTog{e}"))?;
    f_23v(&r_5ln);
    std::fs::write(output, &signed).map_err(|e| format!("ghi PDF '{output}JyBs4buXaTog{e}"))?;
    println!("CsSQw4MgR0hJOiA={output} ({} bytes)", signed.len());
    if r_5ln.is_valid() {
        let level = if r_5ln.timestamp_present {
            "UEFkRVMtVCAoY8OzIHRlbSB0aOG7nWkgZ2lhbik="
        } else {
            "PAdES-B"
        };
        println!(
            "CuKchSBLw50gVEjhuqxUIFRIw4BOSCBDw5RORyDigJQgY2jhu68ga8O9IA=={level}IHThu6sgdG9rZW4gaUNBIGjhu6NwIGzhu4cgKHNpZ25hdHVyZSArIGRpZ2VzdCArIHLDoG5nIGNlcnQpLg=="
        );
        if r_5ln.timestamp_present {
            println!(
                "ICAgdGltZXN0YW1wX3RydXN0ZWQ9ZmFsc2UgbMOgIGLDrG5oIHRoxrDhu51uZyBraGkgY2jGsGEgbuG6oXAgdHJ1c3Qgc3RvcmUgcm9vdCBUU0EgKC0tdHNhIGzDumMgdmVyaWZ5KS4="
            );
        }
        println!(
            "ICAgc2lnbmVyX2NoYWluX29rPWZhbHNlIGzDoCBiw6xuaCB0aMaw4budbmcga2hpIGNoxrBhIG7huqFwIHRydXN0IHN0b3JlIGfhu5FjIGlDQSAoY2jhu4kg4bufIG3hu6ljIGPhuqV1IHRyw7pjKS4="
        );
        Ok(())
    } else {
        Err(
            "UERGIMSRw6MgZ2hpIG5oxrBuZyBWRVJJRlkgS0jDlE5HIGjhu6NwIGzhu4cg4oCUIHhlbSBjw6FjIGPhu50g4bufIHRyw6pu"
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
        RsaPrivateKey::new(&mut r_5lv, 2048).map_err(|e| format!("a2V5Z2VuIGzhu5dpOiA={e}"))?;
    let r_5lw = SigningKey::<sha2::Sha256>::new(r_5lu.clone());
    let spki = SubjectPublicKeyInfoOwned::from_key(r_5lu.to_public_key())
        .map_err(|e| format!("c3BraSBs4buXaTog{e}"))?;
    let cert = CertificateBuilder::new(
        Profile::Root,
        SerialNumber::from(1u32),
        Validity::from_now(core::time::Duration::from_secs(365 * 24 * 3600))
            .map_err(|e| format!("dmFsaWRpdHkgbOG7l2k6IA=={e}"))?,
        Name::from_str("CN=DigiText Soft Selftest").map_err(|e| format!("bmFtZSBs4buXaTog{e}"))?,
        spki,
        &r_5lw,
    )
    .map_err(|e| format!("YnVpbGRlciBs4buXaTog{e}"))?
    .build()
    .map_err(|e| format!("YnVpbGQgY2VydCBs4buXaTog{e}"))?;
    let r_5lt = cert
        .to_der()
        .map_err(|e| format!("Y2VydCB0b19kZXIgbOG7l2k6IA=={e}"))?;
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
        std::fs::read(input).map_err(|e| format!("xJHhu41jIFBERiAn{input}JyBs4buXaTog{e}"))?;
    println!("UERGIHbDoG8gOiA={input} ({} bytes)", pdf.len());
    println!(
        "Q2jhur8gxJHhu5kgIDogU0VMRlRFU1QgKGtob8OhIHBo4bqnbiBt4buBbSDigJQgS0jDlE5HIHRva2VuKQ=={}{}",
        if stamp { "ICsgROG6pFUgxJDhu44=" } else { "" },
        if tsa_url.is_some() { " + PAdES-T" } else { "" }
    );
    let (r_5m2, r_5lx) = f_2gd()?;
    let r_5m1 = f_1vc().max(0) as u64;
    let r_5ly = if stamp {
        let out = f_qu(&pdf, &f_2ff(&r_5lx), &f_1d8(r_5m1))?;
        println!(
            "CuKUgOKUgCBE4bqkVSDEkOG7jiDilIDilIAKICDEkMOjIHbhur0gY29uIGThuqV1IMSR4buPIOG7nyDEkeG6p3UgdHJhbmcgMSAo{} → {} bytes)",
            pdf.len(), out.len()
        );
        out
    } else {
        pdf.clone()
    };
    let r_5lz = prepare(&r_5ly, &r_5lx, r_5m1)
        .map_err(|e| format!("cHJlcGFyZSAoUEFkRVMpIGzhu5dpOiA={e}"))?;
    println!("\n── PREPARE ──");
    println!("  PDF sau placeholder : {} bytes", r_5lz.pdf.len());
    println!("  tbs_digest (SHA-256): {}", hex(&r_5lz.tbs_digest));
    let signature = r_5m2.sign(&r_5lz.signed_attrs_der).to_bytes();
    println!("CuKUgOKUgCBTSUdOIChraG/DoSBwaOG6p24gbeG7gW0pIOKUgOKUgA==");
    println!(
        "ICBjaOG7ryBrw70gUlNBICAgICAgICAgIDog{} bytes",
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
    .map_err(|e| format!("ZmluYWxpemUgbOG7l2k6IA=={e}"))?;
    println!("\n── FINALIZE ──");
    println!("ICBQREYgxJHDoyBrw70gICAgICAgICAgIDog{} bytes", signed.len());
    let r_5m0 = verify(&signed).map_err(|e| format!("dmVyaWZ5IGzhu5dpOiA={e}"))?;
    f_23v(&r_5m0);
    if let Some(out) = output {
        std::fs::write(out, &signed).map_err(|e| format!("ghi PDF '{out}JyBs4buXaTog{e}"))?;
        println!(
            "CsSQw4MgR0hJIChrw70gYuG6sW5nIGtob8OhIHBo4bqnbiBt4buBbSwgS0jDlE5HIHBo4bqjaSB0b2tlbik6IA=={out}"
        );
    }
    if r_5m0.is_valid() {
        println!(
            "CuKchSBTRUxGVEVTVCBQQVNTIOKAlCBQREYgbsOgeSBrw70gKyB2ZXJpZnkgxJHGsOG7o2MuIEzhu4duaCBgc2lnbmAgdGjhuq10IHRyw6puIMSRw7puZyBmaWxlIHPhur0gY2jhuqF5Lg=="
        );
        Ok(())
    } else {
        Err(
            "U0VMRlRFU1QgS0jDlE5HIGjhu6NwIGzhu4cg4oCUIFBERiBjw7MgdGjhu4Mga2jDtG5nIHTGsMahbmcgdGjDrWNoIGx14buTbmcga8O9OyB4ZW0gY8OhYyBj4budIHRyw6pu"
                .into(),
        )
    }
}
fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn f_2ob(prog: &str) -> String {
    format!(
        "RMO5bmc6CiAg{prog}IHNpZ24gICAgICAgICAgICA8aW5wdXQucGRmPiA8b3V0cHV0LnBkZj4gWy0tdGltZXN0YW1wIHwgLS10c2EtdXJsIDx1cmw+XSAgIyBrw70gVEjhuqxUIGLhurFuZyB0b2tlbiAodsO0IGjDrG5oKQogIA=={prog}IHNpZ24tc3RhbXAgICAgICA8aW5wdXQucGRmPiA8b3V0cHV0LnBkZj4gWy0tdGltZXN0YW1wIHwgLS10c2EtdXJsIDx1cmw+XSAgIyBrw70gVEjhuqxUICsgROG6pFUgxJDhu44gxJHhuqd1IHRyYW5nIDEKICA={prog}IHNlbGZ0ZXN0ICAgICAgICA8aW5wdXQucGRmPiBbb3V0cHV0LnBkZl0gIFstLXRpbWVzdGFtcCB8IC0tdHNhLXVybCA8dXJsPl0gICMga8O9IHRo4butIGtob8OhIHBo4bqnbiBt4buBbSAoS0jDlE5HIHRva2VuKQogIA=={prog}IHNlbGZ0ZXN0LXN0YW1wICA8aW5wdXQucGRmPiA8b3V0cHV0LnBkZj4gWy0tdGltZXN0YW1wIHwgLS10c2EtdXJsIDx1cmw+XSAgIyBrw70gdGjhu60ga2hvw6EgcGjhuqduIG3hu4FtICsgROG6pFUgxJDhu44KICA={prog}IHZlcmlmeSAgICAgICAgICA8c2lnbmVkLnBkZj4gWy0tcm9vdHMgPGJ1bmRsZS5wZW0+XSBbLS10c2EgPHRzYS1yb290LnBlbT5dICAjIHjDoWMgbWluaCArIGtp4buDbSB0aW4gY+G6rXkKICA={prog}IG1ha2UtbHQgICAgICAgICA8c2lnbmVkLnBkZj4gPG91dC1sdC5wZGY+IC0tY2hhaW4gPGJ1bmRsZS5wZW0+IC0tbGVhZiA8bGVhZi5wZW0+IFstLXRzYS1yb290cyA8YnVuZGxlIHJvb3QgVFNBPl0gICMgbsOibmcgbMOqbiBQQWRFUy1MVCAoT0NTUC9DUkwpCiAg{prog}IG1ha2UtbHRhICAgICAgICA8bHQucGRmPiA8b3V0LWx0YS5wZGY+IFstLXRpbWVzdGFtcCB8IC0tdHNhLXVybCA8dXJsPl0gICMgbsOibmcgbMOqbiBQQWRFUy1MVEEgKGFyY2hpdmUtdGltZXN0YW1wKQoKLS10aW1lc3RhbXAgICAgOiBkw7luZyBUU0EgbeG6t2MgxJHhu4tuaCAoRGlnaUNlcnQgUlNBIOKAlCBjaOG7iSDEkeG7gyBUSOG7rDsgU+G6ok4gWFXhuqRUIGTDuW5nIFRTQSBWTiDEkeG7pyDEkWnhu4F1IGtp4buHbikuCiAgLS10c2EtdXJsIDx1PiAgOiBQQWRFUy1UIHbhu5tpIFRTQSBjaOG7iSDEkeG7i25oIChSRkMgMzE2MSwgYXBwbGljYXRpb24vdGltZXN0YW1wLXF1ZXJ5KS4KICBLaMO0bmcgY8OzIGzhu4duaCArIMSRw7puZyAyIHRoYW0gc+G7kSDih5IgaGnhu4N1IGzDoCBgc2lnbmAgKHTGsMahbmcgdGjDrWNoIGPFqSku"
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
                "S2jDtG5nIHTDrG0gdGjhuqV5IG1vZHVsZSBQS0NTIzExLiDEkOG6t3QgRElHSVRFWFRfUEtDUzExX01PRFVMRT08xJHGsOG7nW5nLWThuqtuIC5zby8uZGxsLy5keWxpYj4u"
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
                        "bWFrZS1sdCBj4bqnbjogLS1jaGFpbiA8YnVuZGxlLnBlbT4gLS1sZWFmIDxsZWFmLnBlbT4="
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
