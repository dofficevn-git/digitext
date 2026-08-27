// Copyright (c) 2026 DigiText. All rights reserved.
use crate::SignError;
use lopdf::{Document, Object, ObjectId};
use std::collections::BTreeSet;
#[derive(Debug, Clone, Default)]
pub struct ValidationMaterial {
    pub certs: Vec<Vec<u8>>,
    pub crls: Vec<Vec<u8>>,
    pub ocsps: Vec<Vec<u8>>,
}
impl ValidationMaterial {
    pub fn is_empty(&self) -> bool {
        self.certs.is_empty() && self.crls.is_empty() && self.ocsps.is_empty()
    }
}
fn f_20t(r_4nt: impl core::fmt::Display) -> SignError {
    SignError::Pdf(r_4nt.to_string())
}
fn find(hay: &[u8], r_4nu: &[u8]) -> Option<usize> {
    hay.windows(r_4nu.len()).position(|r_4nv| r_4nv == r_4nu)
}
fn f_2aa(r_4nw: &Document) -> Result<ObjectId, SignError> {
    r_4nw
        .trailer
        .get(b"Root")
        .ok()
        .and_then(|r_4nx| r_4nx.as_reference().ok())
        .ok_or_else(|| SignError::Pdf("dGhp4bq/dSAvUm9vdA==".into()))
}
fn f_23s(r_4nz: &Document, pdf: &[u8]) -> Result<usize, SignError> {
    if r_4nz.xref_start != 0 {
        return Ok(r_4nz.xref_start);
    }
    let r_4o0 = pdf
        .windows(9)
        .rposition(|r_4o1| r_4o1 == b"startxref")
        .ok_or_else(|| {
            SignError::Pdf("a2jDtG5nIHRo4bqleSBzdGFydHhyZWYgxJHhu4MgbuG7kWkgL1ByZXY=".into())
        })?;
    let r_4ny = &pdf[r_4o0 + 9..];
    let s: String = r_4ny
        .iter()
        .skip_while(|b| b.is_ascii_whitespace())
        .take_while(|b| b.is_ascii_digit())
        .map(|&b| b as char)
        .collect();
    s.parse::<usize>()
        .map_err(|_| SignError::Pdf("c3RhcnR4cmVmIGtow7RuZyBwaOG6o2kgc+G7kQ==".into()))
}
fn f_25l(pdf: &[u8], r_4o2: &Document, id: ObjectId) -> Result<Vec<u8>, SignError> {
    let off = match r_4o2.reference_table.entries.get(&id.0) {
        Some(lopdf::xref::XrefEntry::Normal { offset, .. }) => *offset as usize,
        _ => {
            return Err(
                SignError::Pdf(
                    "b2JqZWN0IGtow7RuZyBjw7MgbeG7pWMgeHJlZiB0aMaw4budbmcgKG9iamVjdCBzdHJlYW0/KSDigJQga2jDtG5nIG5ow7puZyBEU1MgxJHGsOG7o2M="
                        .into(),
                ),
            );
        }
    };
    if off >= pdf.len() {
        return Err(SignError::Pdf("b2Zmc2V0IHhyZWYgbmdvw6BpIGZpbGU=".into()));
    }
    let r_4o3 = off
        + find(&pdf[off..], b"endobj")
            .ok_or_else(|| SignError::Pdf("a2jDtG5nIHRo4bqleSBlbmRvYmo=".into()))?
        + b"endobj".len();
    Ok(pdf[off..r_4o3].to_vec())
}
fn f_x9(r_4o8: &[u8], dss_id: u32) -> Result<Vec<u8>, SignError> {
    let r_4o6 =
        find(r_4o8, b"obj").ok_or_else(|| SignError::Pdf("Y2F0YWxvZyB0aGnhur91ICdvYmon".into()))?;
    let r_4o5 = r_4o6
        + find(&r_4o8[r_4o6..], b"<<")
            .ok_or_else(|| SignError::Pdf("Y2F0YWxvZyB0aGnhur91ICc8PCc=".into()))?;
    let r_4o4 = r_4o5 + 2;
    if find(&r_4o8[r_4o4..], b"/DSS").is_some() {
        return Err(SignError::Pdf(
            "Y2F0YWxvZyDEkcOjIGPDsyAvRFNTIChQREYgxJHDoyDhu58gbeG7qWMgTFQ/KQ==".into(),
        ));
    }
    let mut r_4o7 = Vec::with_capacity(r_4o8.len() + 25);
    r_4o7.extend_from_slice(&r_4o8[..r_4o4]);
    r_4o7.extend_from_slice(format!("/DSS {dss_id} 0 R ").as_bytes());
    r_4o7.extend_from_slice(&r_4o8[r_4o4..]);
    r_4o7.push(b'\n');
    Ok(r_4o7)
}
fn f_2ht(id: u32, r_4o9: &[u8]) -> Vec<u8> {
    let mut r_4oa = format!("{id} 0 obj\n<< /Length {} >>\nstream\n", r_4o9.len()).into_bytes();
    r_4oa.extend_from_slice(r_4o9);
    r_4oa.extend_from_slice(b"\nendstream\nendobj\n");
    r_4oa
}
fn refs(r_4ob: &[u32]) -> String {
    r_4ob
        .iter()
        .map(|i| format!("{i} 0 R"))
        .collect::<Vec<_>>()
        .join(" ")
}
pub fn embed_dss(r_4ot: &[u8], r_4ov: &ValidationMaterial) -> Result<Vec<u8>, SignError> {
    if r_4ov.is_empty() {
        return Err(SignError::Pdf(
            "a2jDtG5nIGPDsyBtYXRlcmlhbCDEkeG7gyBuaMO6bmcgL0RTUw==".into(),
        ));
    }
    let r_4ok = Document::load_mem(r_4ot).map_err(f_20t)?;
    let r_4of = f_2aa(&r_4ok)?;
    let r_4oh = f_25l(r_4ot, &r_4ok, r_4of)?;
    let r_4os = f_23s(&r_4ok, r_4ot)?;
    let mut r_4op = r_4ok.max_id + 1;
    let mut r_4oc = || {
        let id = r_4op;
        r_4op += 1;
        id
    };
    let r_4oi: Vec<u32> = r_4ov.certs.iter().map(|_| r_4oc()).collect();
    let r_4oj: Vec<u32> = r_4ov.crls.iter().map(|_| r_4oc()).collect();
    let r_4oq: Vec<u32> = r_4ov.ocsps.iter().map(|_| r_4oc()).collect();
    let dss_id = r_4oc();
    let r_4oo = dss_id;
    let r_4od = r_4ot.len();
    let mut buf: Vec<u8> = Vec::new();
    if r_4ot.last() != Some(&b'\n') {
        buf.push(b'\n');
    }
    let mut xref: Vec<(u32, usize)> = Vec::new();
    let r_4om = |buf: &mut Vec<u8>, xref: &mut Vec<(u32, usize)>, id: u32, r_4oe: &[u8]| {
        xref.push((id, r_4od + buf.len()));
        buf.extend_from_slice(r_4oe);
    };
    for (id, der) in r_4oi.iter().zip(&r_4ov.certs) {
        r_4om(&mut buf, &mut xref, *id, &f_2ht(*id, der));
    }
    for (id, der) in r_4oj.iter().zip(&r_4ov.crls) {
        r_4om(&mut buf, &mut xref, *id, &f_2ht(*id, der));
    }
    for (id, der) in r_4oq.iter().zip(&r_4ov.ocsps) {
        r_4om(&mut buf, &mut xref, *id, &f_2ht(*id, der));
    }
    let mut r_4ol = format!("{dss_id} 0 obj\n<< /Type /DSS");
    if !r_4oi.is_empty() {
        r_4ol.push_str(&format!(" /Certs [ {} ]", refs(&r_4oi)));
    }
    if !r_4oj.is_empty() {
        r_4ol.push_str(&format!(" /CRLs [ {} ]", refs(&r_4oj)));
    }
    if !r_4oq.is_empty() {
        r_4ol.push_str(&format!(" /OCSPs [ {} ]", refs(&r_4oq)));
    }
    r_4ol.push_str(" >>\nendobj\n");
    r_4om(&mut buf, &mut xref, dss_id, r_4ol.as_bytes());
    let r_4og = f_x9(&r_4oh, dss_id)?;
    r_4om(&mut buf, &mut xref, r_4of.0, &r_4og);
    let xref_off = r_4od + buf.len();
    buf.extend_from_slice(b"xref\n");
    xref.sort_by_key(|(id, _)| *id);
    let mut i = 0;
    while i < xref.len() {
        let r_4ou = xref[i].0;
        let mut r_4on = i;
        while r_4on + 1 < xref.len() && xref[r_4on + 1].0 == xref[r_4on].0 + 1 {
            r_4on += 1;
        }
        buf.extend_from_slice(
            format!("{} {}\n", r_4ou, r_4on - i + 1)
                .into_bytes()
                .as_slice(),
        );
        for (_, off) in &xref[i..=r_4on] {
            buf.extend_from_slice(format!("{off:010} {:05} n\r\n", 0).into_bytes().as_slice());
        }
        i = r_4on + 1;
    }
    let mut trailer = format!(
        "trailer\n<< /Size {} /Root {} {} R /Prev {}",
        r_4oo + 1,
        r_4of.0,
        r_4of.1,
        r_4os
    );
    if let Some(id_ser) = f_2di(&r_4ok) {
        trailer.push_str(&format!(" /ID {id_ser}"));
    }
    trailer.push_str(" >>\n");
    buf.extend_from_slice(trailer.as_bytes());
    buf.extend_from_slice(format!("startxref\n{xref_off}\n%%EOF").as_bytes());
    let mut r_4or = r_4ot.to_vec();
    r_4or.extend_from_slice(&buf);
    Ok(r_4or)
}
fn f_2di(r_4oy: &Document) -> Option<String> {
    let r_4ow = r_4oy.trailer.get(b"ID").ok()?.as_array().ok()?;
    let mut r_4p0 = Vec::new();
    for r_4oz in r_4ow {
        let r_4ox = match r_4oz {
            Object::String(b, _) => b,
            _ => return None,
        };
        let mut hex = String::with_capacity(r_4ox.len() * 2);
        for b in r_4ox {
            hex.push_str(&format!("{b:02X}"));
        }
        r_4p0.push(format!("<{hex}>"));
    }
    Some(format!("[ {} ]", r_4p0.join(" ")))
}
pub fn f_2p0(r_4p6: &[u8], r_4pb: usize) -> Result<ValidationMaterial, SignError> {
    let r_4p8 = r_4p6
        .windows(5)
        .rposition(|r_4pc| r_4pc == b"%%EOF")
        .ok_or_else(|| SignError::Pdf("dGhp4bq/dSAlJUVPRg==".into()))?;
    if r_4p6[r_4p8 + 5..].iter().any(|b| !b.is_ascii_whitespace()) {
        return Err(SignError::Pdf(
            "Y8OzIGJ5dGUgc2F1ICUlRU9GIGN14buRaSAobmdvw6BpIHbDuW5nIERTUyk=".into(),
        ));
    }
    let r_4pa = Document::load_mem(&r_4p6[..r_4pb]).map_err(f_20t)?;
    let r_4p5 = Document::load_mem(r_4p6).map_err(f_20t)?;
    let r_4p2 = f_2aa(&r_4pa)?;
    if f_2aa(&r_4p5)? != r_4p2 {
        return Err(SignError::Pdf("aW5jcmVtZW50IMSR4buVaSAvUm9vdA==".into()));
    }
    for (id, obj) in &r_4pa.objects {
        let r_4p4 = r_4p5
            .objects
            .get(id)
            .ok_or_else(|| SignError::Pdf("aW5jcmVtZW50IHhvw6Egb2JqZWN0IMSRw6Mga8O9".into()))?;
        if *id == r_4p2 {
            f_1a0(obj, r_4p4)?;
        } else if r_4p4 != obj {
            return Err(SignError::Pdf(
                "aW5jcmVtZW50IHPhu61hIG9iamVjdCDEkcOjIGvDvSAobmdvw6BpIHZp4buHYyB0aMOqbSAvRFNTKQ=="
                    .into(),
            ));
        }
    }
    let r_4p3 = r_4p5
        .catalog()
        .map_err(f_20t)?
        .get(b"DSS")
        .map_err(|_| SignError::Pdf("Y2F0YWxvZyBt4bubaSB0aGnhur91IC9EU1M=".into()))?
        .as_reference()
        .map_err(|_| SignError::Pdf("L0RTUyBraMO0bmcgcGjhuqNpIHRoYW0gY2hp4bq/dQ==".into()))?;
    let r_4p1 = f_10d(&r_4p5, r_4p3)?;
    let r_4p9: BTreeSet<ObjectId> = r_4p5
        .objects
        .keys()
        .filter(|r_4p7| !r_4pa.objects.contains_key(r_4p7))
        .copied()
        .collect();
    if r_4p9 != r_4p1 {
        return Err(
            SignError::Pdf(
                "aW5jcmVtZW50IHRow6ptIG9iamVjdCBraMO0bmcgdGh14buZYyAvRFNTIChuZ2hpIHR14buTbiBu4buZaSBkdW5nIG5nb8OgaSB2w7luZyBrw70p"
                    .into(),
            ),
        );
    }
    f_25q(&r_4p5)
}
fn f_1a0(r_4ph: &Object, r_4pf: &Object) -> Result<(), SignError> {
    let r_4pd = r_4ph.as_dict().map_err(f_20t)?;
    let b = r_4pf.as_dict().map_err(f_20t)?;
    if r_4pd.has(b"DSS") {
        return Err(SignError::Pdf(
            "Y2F0YWxvZyDEkcOjIGvDvSB24buRbiDEkcOjIGPDsyAvRFNT".into(),
        ));
    }
    if !b.has(b"DSS") {
        return Err(SignError::Pdf(
            "Y2F0YWxvZyBt4bubaSBraMO0bmcgdGjDqm0gL0RTUw==".into(),
        ));
    }
    for (r_4pe, r_4pi) in r_4pd.iter() {
        match b.get(r_4pe) {
            Ok(r_4pg) if r_4pg == r_4pi => {}
            _ => {
                return Err(SignError::Pdf(
                    "aW5jcmVtZW50IHPhu61hL3hvw6EgbeG7mXQga2hvw6EgY2F0YWxvZyDEkcOjIGvDvQ==".into(),
                ));
            }
        }
    }
    if b.len() != r_4pd.len() + 1 {
        return Err(SignError::Pdf(
            "aW5jcmVtZW50IHRow6ptIGtob8OhIGNhdGFsb2cga2jDoWMgbmdvw6BpIC9EU1M=".into(),
        ));
    }
    Ok(())
}
fn f_10d(r_4pk: &Document, r_4pm: ObjectId) -> Result<BTreeSet<ObjectId>, SignError> {
    let r_4pl = r_4pk.get_dictionary(r_4pm).map_err(f_20t)?;
    for (r_4po, _) in r_4pl.iter() {
        if !matches!(r_4po.as_slice(), b"Type" | b"Certs" | b"CRLs" | b"OCSPs") {
            return Err(SignError::Pdf(format!(
                "L0RTUyBjaOG7qWEga2hvw6Ega2jDtG5nIGjhu5cgdHLhu6M6IC8={}",
                String::from_utf8_lossy(r_4po)
            )));
        }
    }
    let mut r_4pn = BTreeSet::new();
    r_4pn.insert(r_4pm);
    for r_4pp in [b"Certs".as_slice(), b"CRLs", b"OCSPs"] {
        if let Ok(r_4pj) = r_4pl.get(r_4pp).and_then(|o| o.as_array()) {
            for o in r_4pj {
                let id = o.as_reference().map_err(|_| {
                    SignError::Pdf("beG7pWMgL0RTUyBraMO0bmcgcGjhuqNpIHRoYW0gY2hp4bq/dQ==".into())
                })?;
                r_4pn.insert(id);
            }
        }
    }
    Ok(r_4pn)
}
pub fn f_25q(r_4pr: &Document) -> Result<ValidationMaterial, SignError> {
    let Ok(r_4pt) = r_4pr
        .catalog()
        .map_err(f_20t)?
        .get(b"DSS")
        .and_then(|o| o.as_reference())
    else {
        return Ok(ValidationMaterial::default());
    };
    let r_4ps = r_4pr.get_dictionary(r_4pt).map_err(f_20t)?;
    let r_4pw = |r_4pu: &[u8]| -> Result<Vec<Vec<u8>>, SignError> {
        let mut r_4pv = Vec::new();
        if let Ok(r_4pq) = r_4ps.get(r_4pu).and_then(|o| o.as_array()) {
            for o in r_4pq {
                let id = o.as_reference().map_err(f_20t)?;
                let r_4px = r_4pr
                    .get_object(id)
                    .and_then(|o| o.as_stream())
                    .map_err(|_| {
                        SignError::Pdf("beG7pWMgL0RTUyBraMO0bmcgcGjhuqNpIHN0cmVhbQ==".into())
                    })?;
                r_4pv.push(r_4px.content.clone());
            }
        }
        Ok(r_4pv)
    };
    Ok(ValidationMaterial {
        certs: r_4pw(b"Certs")?,
        crls: r_4pw(b"CRLs")?,
        ocsps: r_4pw(b"OCSPs")?,
    })
}
const C_4G: usize = 16384;
pub struct Thh {
    pub pdf: Vec<u8>,
    pub tbs: Vec<u8>,
    contents_start: usize,
    contents_end: usize,
}
fn f_2ro(r_4pz: &mut [u8], r_4q0: usize, r_4q1: usize, r_4py: [usize; 4]) -> Result<(), SignError> {
    let r_4q2 = r_4q1 - r_4q0 + 1;
    let mut s = format!("[{} {} {} {}", r_4py[0], r_4py[1], r_4py[2], r_4py[3]);
    if s.len() + 1 > r_4q2 {
        return Err(SignError::Pdf(
            "Qnl0ZVJhbmdlIGTDoGkgaMahbiBwbGFjZWhvbGRlcg==".into(),
        ));
    }
    while s.len() + 1 < r_4q2 {
        s.push(' ');
    }
    s.push(']');
    r_4pz[r_4q0..=r_4q1].copy_from_slice(s.as_bytes());
    Ok(())
}
fn f_rz(r_4q9: &[u8], r_4q4: &[u8], id: u32) -> Result<Vec<u8>, SignError> {
    let r_4q5 = find(r_4q9, r_4q4).ok_or_else(|| {
        SignError::Pdf(format!(
            "b2JqZWN0IHRoaeG6v3Ug{}",
            String::from_utf8_lossy(r_4q4)
        ))
    })?;
    let r_4q3 = r_4q5 + r_4q4.len();
    let r_4q7 = find(&r_4q9[r_4q3..], b"[").ok_or_else(|| {
        SignError::Pdf(format!(
            "{}IGtow7RuZyBwaOG6o2kgbeG6o25nIGlubGluZQ==",
            String::from_utf8_lossy(r_4q4)
        ))
    })?;
    if r_4q9[r_4q3..r_4q3 + r_4q7]
        .iter()
        .any(|b| !b.is_ascii_whitespace())
    {
        return Err(SignError::Pdf(format!(
            "{}IGtow7RuZyBwaOG6o2kgbeG6o25nIGlubGluZSAoZ2nDoSB0cuG7iyBsw6AgdGhhbSBjaGnhur91Pyk=",
            String::from_utf8_lossy(r_4q4)
        )));
    }
    let r_4q6 = r_4q3 + r_4q7;
    let r_4qa = r_4q6
        + 1
        + find(&r_4q9[r_4q6 + 1..], b"]")
            .ok_or_else(|| SignError::Pdf("beG6o25nIHRoaeG6v3UgJ10n".into()))?;
    let mut r_4q8 = Vec::with_capacity(r_4q9.len() + 12);
    r_4q8.extend_from_slice(&r_4q9[..r_4qa]);
    r_4q8.extend_from_slice(format!(" {id} 0 R").as_bytes());
    r_4q8.extend_from_slice(&r_4q9[r_4qa..]);
    Ok(r_4q8)
}
fn f_s0(
    r_4qc: &mut Vec<u8>,
    r_4qb: usize,
    xref: &mut [(u32, usize)],
    r_4qd: ObjectId,
    r_4qf: u32,
    r_4qg: usize,
    id_ser: Option<String>,
) {
    let xref_off = r_4qb + r_4qc.len();
    r_4qc.extend_from_slice(b"xref\n");
    xref.sort_by_key(|(id, _)| *id);
    let mut i = 0;
    while i < xref.len() {
        let r_4qh = xref[i].0;
        let mut r_4qe = i;
        while r_4qe + 1 < xref.len() && xref[r_4qe + 1].0 == xref[r_4qe].0 + 1 {
            r_4qe += 1;
        }
        r_4qc.extend_from_slice(format!("{} {}\n", r_4qh, r_4qe - i + 1).as_bytes());
        for (_, off) in &xref[i..=r_4qe] {
            r_4qc.extend_from_slice(format!("{off:010} {:05} n\r\n", 0).as_bytes());
        }
        i = r_4qe + 1;
    }
    let mut trailer = format!(
        "trailer\n<< /Size {} /Root {} {} R /Prev {}",
        r_4qf + 1,
        r_4qd.0,
        r_4qd.1,
        r_4qg
    );
    if let Some(s) = id_ser {
        trailer.push_str(&format!(" /ID {s}"));
    }
    trailer.push_str(" >>\n");
    r_4qc.extend_from_slice(trailer.as_bytes());
    r_4qc.extend_from_slice(format!("startxref\n{xref_off}\n%%EOF").as_bytes());
}
pub fn f_23h(r_4qt: &[u8]) -> Result<Thh, SignError> {
    let r_4qr = Document::load_mem(r_4qt).map_err(f_20t)?;
    let r_4qq = f_2aa(&r_4qr)?;
    let r_4qi = r_4qr
        .catalog()
        .map_err(f_20t)?
        .get(b"AcroForm")
        .map_err(|_| SignError::Pdf("dGhp4bq/dSAvQWNyb0Zvcm0gKFBERiBjaMawYSBrw70/KQ==".into()))?
        .as_reference()
        .map_err(|_| SignError::Pdf("L0Fjcm9Gb3JtIGtow7RuZyBwaOG6o2kgdGhhbSBjaGnhur91".into()))?;
    let r_4qv = *r_4qr
        .get_pages()
        .values()
        .next()
        .ok_or_else(|| SignError::Pdf("UERGIGtow7RuZyBjw7MgdHJhbmc=".into()))?;
    let r_4qk = f_25l(r_4qt, &r_4qr, r_4qi)?;
    let r_4qx = f_25l(r_4qt, &r_4qr, r_4qv)?;
    let r_4qy = f_23s(&r_4qr, r_4qt)?;
    let dts_sig_id = r_4qr.max_id + 1;
    let dts_field_id = r_4qr.max_id + 2;
    let r_4ql = r_4qt.len();
    let mut r_4qp: Vec<u8> = Vec::new();
    if r_4qt.last() != Some(&b'\n') {
        r_4qp.push(b'\n');
    }
    let mut xref: Vec<(u32, usize)> = Vec::new();
    xref.push((dts_sig_id, r_4ql + r_4qp.len()));
    let r_4qs = format!(
        "{dts_sig_id} 0 obj\n<< /Type /DocTimeStamp /Filter /Adobe.PPKLite /SubFilter /ETSI.RFC3161 /ByteRange "
    );
    r_4qp.extend_from_slice(r_4qs.as_bytes());
    let r_4qn = r_4ql + r_4qp.len();
    r_4qp.extend_from_slice(b"[0 9999999999 9999999999 9999999999]");
    let r_4qo = r_4ql + r_4qp.len() - 1;
    r_4qp.extend_from_slice(b" /Contents <");
    let contents_start = r_4ql + r_4qp.len();
    r_4qp.extend_from_slice(&vec![b'0'; C_4G * 2]);
    let contents_end = r_4ql + r_4qp.len();
    r_4qp.extend_from_slice(b"> >>\nendobj\n");
    xref.push((dts_field_id, r_4ql + r_4qp.len()));
    let field = format!(
        "{dts_field_id} 0 obj\n<< /Type /Annot /Subtype /Widget /FT /Sig /T (DigiTextDocTS) /Rect [0 0 0 0] /F 132 /P {} {} R /V {dts_sig_id} 0 R >>\nendobj\n",
        r_4qv.0, r_4qv.1
    );
    r_4qp.extend_from_slice(field.as_bytes());
    let r_4qj = f_rz(&r_4qk, b"/Fields", dts_field_id)?;
    xref.push((r_4qi.0, r_4ql + r_4qp.len()));
    r_4qp.extend_from_slice(&r_4qj);
    r_4qp.push(b'\n');
    let r_4qw = f_rz(&r_4qx, b"/Annots", dts_field_id)?;
    xref.push((r_4qv.0, r_4ql + r_4qp.len()));
    r_4qp.extend_from_slice(&r_4qw);
    r_4qp.push(b'\n');
    f_s0(
        &mut r_4qp,
        r_4ql,
        &mut xref,
        r_4qq,
        dts_field_id,
        r_4qy,
        f_2di(&r_4qr),
    );
    let mut r_4qu = r_4qt.to_vec();
    r_4qu.extend_from_slice(&r_4qp);
    let r_4r1 = r_4qu.len();
    let r_4qm = [0usize, contents_start, contents_end, r_4r1 - contents_end];
    f_2ro(&mut r_4qu, r_4qn, r_4qo, r_4qm)?;
    let r_4qz = r_4qu.get(r_4qm[0]..r_4qm[0] + r_4qm[1]).ok_or_else(|| {
        SignError::Pdf("Qnl0ZVJhbmdlIERvY1RpbWVTdGFtcCBuZ2/DoGkgYmnDqm4gKGJ1Zyk=".into())
    })?;
    let r_4r0 = r_4qu.get(r_4qm[2]..r_4qm[2] + r_4qm[3]).ok_or_else(|| {
        SignError::Pdf("Qnl0ZVJhbmdlIERvY1RpbWVTdGFtcCBuZ2/DoGkgYmnDqm4gKGJ1Zyk=".into())
    })?;
    let mut tbs = Vec::with_capacity(r_4qz.len() + r_4r0.len());
    tbs.extend_from_slice(r_4qz);
    tbs.extend_from_slice(r_4r0);
    Ok(Thh {
        pdf: r_4qu,
        tbs,
        contents_start,
        contents_end,
    })
}
pub fn f_1cp(r_4r3: &Thh, token: &[u8]) -> Result<Vec<u8>, SignError> {
    let hex = crate::pdf::f_1gv(token);
    let r_4r4 = r_4r3.contents_end - r_4r3.contents_start;
    if hex.len() > r_4r4 {
        return Err(SignError::Pdf(format!(
            "token TSA {}IGJ5dGUgdsaw4bujdCDDtCAvQ29udGVudHMgRG9jVGltZVN0YW1wIA=={} byte",
            token.len(),
            r_4r4 / 2
        )));
    }
    let mut r_4r2 = r_4r3.pdf.clone();
    for b in &mut r_4r2[r_4r3.contents_start..r_4r3.contents_end] {
        *b = b'0';
    }
    r_4r2[r_4r3.contents_start..r_4r3.contents_start + hex.len()].copy_from_slice(hex.as_bytes());
    Ok(r_4r2)
}
pub struct T9s {
    pub has_dss: bool,
    pub material: ValidationMaterial,
    pub archive_present: bool,
    pub archive_structural_ok: bool,
    pub archive_timestamp_trusted: bool,
    pub archive_time_unix: Option<i64>,
}
fn f_19v(
    r_4rd: &Document,
    r_4r9: &Document,
    id: ObjectId,
    r_4rb: &[u8],
    r_4r6: ObjectId,
) -> Result<(), SignError> {
    let r_4r5 = r_4rd.get_dictionary(id).map_err(f_20t)?;
    let b = r_4r9.get_dictionary(id).map_err(f_20t)?;
    if r_4r5.len() != b.len() {
        return Err(SignError::Pdf(
            "aW5jcmVtZW50IMSR4buVaSBz4buRIGtob8OhIG9iamVjdCDEkcOjIGvDvQ==".into(),
        ));
    }
    for (r_4ra, r_4re) in r_4r5.iter() {
        if r_4ra.as_slice() == r_4rb {
            continue;
        }
        match b.get(r_4ra) {
            Ok(r_4rc) if r_4rc == r_4re => {}
            _ => {
                return Err(SignError::Pdf(
                    "aW5jcmVtZW50IHPhu61hIGtob8OhIG9iamVjdCDEkcOjIGvDvQ==".into(),
                ));
            }
        }
    }
    let r_4r7 = r_4r5
        .get(r_4rb)
        .and_then(|o| o.as_array())
        .map_err(|_| SignError::Pdf("beG6o25nIGPFqSBraMO0bmcgxJHhu41jIMSRxrDhu6Nj".into()))?;
    let r_4r8 = b
        .get(r_4rb)
        .and_then(|o| o.as_array())
        .map_err(|_| SignError::Pdf("beG6o25nIG3hu5tpIGtow7RuZyDEkeG7jWMgxJHGsOG7o2M=".into()))?;
    if r_4r8.len() != r_4r7.len() + 1 {
        return Err(SignError::Pdf(
            "beG6o25nIGtow7RuZyB0xINuZyDEkcO6bmcgMSBwaOG6p24gdOG7rQ==".into(),
        ));
    }
    for (r_4rf, r_4rg) in r_4r7.iter().zip(r_4r8.iter()) {
        if r_4rf != r_4rg {
            return Err(SignError::Pdf(
                "aW5jcmVtZW50IHPhu61hIHBo4bqnbiB04butIG3huqNuZyBjxak=".into(),
            ));
        }
    }
    if r_4r8.last().and_then(|o| o.as_reference().ok()) != Some(r_4r6) {
        return Err(SignError::Pdf(
            "cGjhuqduIHThu60gdGjDqm0ga2jDtG5nIHBo4bqjaSBmaWVsZCBEb2NUaW1lU3RhbXA=".into(),
        ));
    }
    Ok(())
}
fn f_1xq(r_4rk: &Document, r_4rj: ObjectId) -> Result<ObjectId, SignError> {
    for r_4rl in r_4rk.get_pages().values() {
        if let Ok(r_4ri) = r_4rk.get_dictionary(*r_4rl) {
            if let Ok(r_4rh) = r_4ri.get(b"Annots").and_then(|o| o.as_array()) {
                if r_4rh.iter().any(|o| o.as_reference().ok() == Some(r_4rj)) {
                    return Ok(*r_4rl);
                }
            }
        }
    }
    Err(SignError::Pdf(
        "a2jDtG5nIHRo4bqleSB0cmFuZyBjaOG7qWEgd2lkZ2V0IERvY1RpbWVTdGFtcA==".into(),
    ))
}
fn f_2oz(
    r_4rt: &[u8],
    r_4rs: &Document,
    r_4rq: ObjectId,
    r_4s7: Option<&crate::chain::TrustStore>,
) -> Result<(bool, bool, Option<i64>), SignError> {
    let off = match r_4rs.reference_table.entries.get(&r_4rq.0) {
        Some(lopdf::xref::XrefEntry::Normal { offset, .. }) => *offset as usize,
        _ => {
            return Err(SignError::Pdf(
                "RG9jVGltZVN0YW1wIGtow7RuZyBjw7MgeHJlZiB0aMaw4budbmc=".into(),
            ));
        }
    };
    let r_4rr = off
        + find(&r_4rt[off..], b"endobj")
            .ok_or_else(|| SignError::Pdf("RG9jVGltZVN0YW1wIHRoaeG6v3UgZW5kb2Jq".into()))?;
    let r_4s0 = &r_4rt[off..r_4rr];
    let r_4rp = find(r_4s0, b"/Contents")
        .ok_or_else(|| SignError::Pdf("RG9jVGltZVN0YW1wIHRoaeG6v3UgL0NvbnRlbnRz".into()))?;
    let r_4ry = r_4rp
        + find(&r_4s0[r_4rp..], b"<").ok_or_else(|| {
            SignError::Pdf("RG9jVGltZVN0YW1wIC9Db250ZW50cyBraMO0bmcgcGjhuqNpIGhleA==".into())
        })?;
    let r_4ru = r_4ry
        + 1
        + find(&r_4s0[r_4ry + 1..], b">").ok_or_else(|| {
            SignError::Pdf("RG9jVGltZVN0YW1wIC9Db250ZW50cyB0aGnhur91ICc+Jw==".into())
        })?;
    let r_4rw = off + r_4ry + 1;
    let r_4rv = off + r_4ru;
    let r_4rn = find(r_4s0, b"/ByteRange")
        .ok_or_else(|| SignError::Pdf("RG9jVGltZVN0YW1wIHRoaeG6v3UgL0J5dGVSYW5nZQ==".into()))?;
    let r_4rx = r_4rn
        + find(&r_4s0[r_4rn..], b"[")
            .ok_or_else(|| SignError::Pdf("L0J5dGVSYW5nZSB0aGnhur91ICdbJw==".into()))?;
    let r_4rz = r_4rx
        + 1
        + find(&r_4s0[r_4rx + 1..], b"]")
            .ok_or_else(|| SignError::Pdf("L0J5dGVSYW5nZSB0aGnhur91ICddJw==".into()))?;
    let r_4rm = crate::pdf::f_1yg(r_4rt, off + r_4rx, off + r_4rz)
        .ok_or_else(|| SignError::Pdf("DocTimeStamp /ByteRange sai".into()))?;
    if !crate::pdf::f_wo(r_4rm, r_4rw, r_4rv, r_4rt.len()) {
        return Err(
            SignError::Pdf(
                "RG9jVGltZVN0YW1wIC9CeXRlUmFuZ2Uga2jDtG5nIHBo4bunIHRy4buNbiB0w6BpIGxp4buHdSAobmdoaSBz4butYSBzYXUga2hpIMSRw7NuZyBk4bqldSBhcmNoaXZlKQ=="
                    .into(),
            ),
        );
    }
    let (Some(r_4s1), Some(r_4s2)) = (
        r_4rm[0].checked_add(r_4rm[1]),
        r_4rm[2].checked_add(r_4rm[3]),
    ) else {
        return Err(SignError::Pdf("Qnl0ZVJhbmdlIHRyw6Bu".into()));
    };
    let r_4s3 = r_4rt
        .get(r_4rm[0]..r_4s1)
        .ok_or_else(|| SignError::Pdf("Qnl0ZVJhbmdlIG5nb8OgaSBiacOqbg==".into()))?;
    let r_4s4 = r_4rt
        .get(r_4rm[2]..r_4s2)
        .ok_or_else(|| SignError::Pdf("Qnl0ZVJhbmdlIG5nb8OgaSBiacOqbg==".into()))?;
    let mut r_4ro = Vec::with_capacity(r_4s3.len() + r_4s4.len());
    r_4ro.extend_from_slice(r_4s3);
    r_4ro.extend_from_slice(r_4s4);
    let token = crate::pdf::f_1gu(&r_4rt[r_4rw..r_4rv]).ok_or_else(|| {
        SignError::Pdf("RG9jVGltZVN0YW1wIC9Db250ZW50cyBraMO0bmcgcGjhuqNpIGhleA==".into())
    })?;
    let r_4s8 = crate::tsa::f_2q0(&token, &r_4ro, r_4s7)?;
    if !r_4s8.imprint_ok {
        return Err(
            SignError::Pdf(
                "RG9jVGltZVN0YW1wIGltcHJpbnQga2jDtG5nIGto4bubcCAobuG7mWkgZHVuZyBi4buLIHPhu61hIHNhdSBraGkgxJHDs25nIGThuqV1IGFyY2hpdmUp"
                    .into(),
            ),
        );
    }
    if r_4s8.token_signature_ok == Some(false) {
        return Err(SignError::Pdf(
            "Y2jhu68ga8O9IFRTQSBj4bunYSBEb2NUaW1lU3RhbXAga2jDtG5nIGjhu6NwIGzhu4c=".into(),
        ));
    }
    let r_4s5 = r_4s8.token_signature_ok == Some(true);
    let r_4s6 = r_4s5 && matches!(r_4s8.chain_ok, Some(true));
    Ok((r_4s5, r_4s6, r_4s8.gen_time_unix))
}
pub fn f_2p3(
    r_4sk: &[u8],
    r_4sq: usize,
    r_4su: Option<&crate::chain::TrustStore>,
) -> Result<T9s, SignError> {
    let r_4sm = r_4sk
        .windows(5)
        .rposition(|r_4sv| r_4sv == b"%%EOF")
        .ok_or_else(|| SignError::Pdf("dGhp4bq/dSAlJUVPRg==".into()))?;
    if r_4sk[r_4sm + 5..].iter().any(|b| !b.is_ascii_whitespace()) {
        return Err(SignError::Pdf(
            "Y8OzIGJ5dGUgc2F1ICUlRU9GIGN14buRaQ==".into(),
        ));
    }
    let r_4sp = Document::load_mem(&r_4sk[..r_4sq]).map_err(f_20t)?;
    let r_4si = Document::load_mem(r_4sk).map_err(f_20t)?;
    let r_4sb = f_2aa(&r_4sp)?;
    if f_2aa(&r_4si)? != r_4sb {
        return Err(SignError::Pdf("aW5jcmVtZW50IMSR4buVaSAvUm9vdA==".into()));
    }
    let r_4sn: BTreeSet<ObjectId> = r_4si
        .objects
        .keys()
        .filter(|r_4sl| !r_4sp.objects.contains_key(r_4sl))
        .copied()
        .collect();
    if r_4sp.catalog().map_err(f_20t)?.has(b"DSS") {
        return Err(SignError::Pdf(
            "Y2F0YWxvZyDEkcOjIGvDvSB24buRbiDEkcOjIGPDsyAvRFNT".into(),
        ));
    }
    let r_4sj = r_4si.catalog().map_err(f_20t)?.has(b"DSS");
    let mut r_4sa: BTreeSet<ObjectId> = BTreeSet::new();
    if r_4sj {
        let r_4sd = r_4si
            .catalog()
            .map_err(f_20t)?
            .get(b"DSS")
            .and_then(|o| o.as_reference())
            .map_err(|_| SignError::Pdf("L0RTUyBraMO0bmcgcGjhuqNpIHRoYW0gY2hp4bq/dQ==".into()))?;
        r_4sa.extend(f_10d(&r_4si, r_4sd)?);
    }
    let mut r_4sf = None;
    for id in &r_4sn {
        if let Ok(d) = r_4si.get_dictionary(*id) {
            if d.get(b"Type").and_then(|o| o.as_name()).ok() == Some(b"DocTimeStamp".as_ref()) {
                if r_4sf.is_some() {
                    return Err(SignError::Pdf(
                        "bmhp4buBdSBoxqFuIG3hu5l0IERvY1RpbWVTdGFtcCAoY2jGsGEgaOG7lyB0cuG7oyk="
                            .into(),
                    ));
                }
                r_4sf = Some(*id);
            }
        }
    }
    let mut archive_present = false;
    let mut archive_structural_ok = false;
    let mut archive_timestamp_trusted = false;
    let mut archive_time_unix = None;
    let mut r_4sc: BTreeSet<ObjectId> = BTreeSet::new();
    if let Some(r_4se) = r_4sf {
        archive_present = true;
        r_4sa.insert(r_4se);
        let r_4sh = *r_4sn
            .iter()
            .find(|id| {
                r_4si
                    .get_dictionary(**id)
                    .ok()
                    .and_then(|d| d.get(b"V").ok())
                    .and_then(|o| o.as_reference().ok())
                    == Some(r_4se)
            })
            .ok_or_else(|| {
                SignError::Pdf("RG9jVGltZVN0YW1wIHRoaeG6v3UgZmllbGQgdGhhbSBjaGnhur91".into())
            })?;
        r_4sa.insert(r_4sh);
        let r_4s9 = r_4sp
            .catalog()
            .map_err(f_20t)?
            .get(b"AcroForm")
            .and_then(|o| o.as_reference())
            .map_err(|_| SignError::Pdf("dGhp4bq/dSAvQWNyb0Zvcm0=".into()))?;
        f_19v(&r_4sp, &r_4si, r_4s9, b"Fields", r_4sh)?;
        r_4sc.insert(r_4s9);
        let r_4so = f_1xq(&r_4si, r_4sh)?;
        f_19v(&r_4sp, &r_4si, r_4so, b"Annots", r_4sh)?;
        r_4sc.insert(r_4so);
        let (r_4sr, r_4st, r_4ss) = f_2oz(r_4sk, &r_4si, r_4se, r_4su)?;
        archive_structural_ok = r_4sr;
        archive_timestamp_trusted = r_4st;
        archive_time_unix = r_4ss;
    }
    if r_4sn != r_4sa {
        return Err(SignError::Pdf(
            "aW5jcmVtZW50IHRow6ptIG9iamVjdCBraMO0bmcgdGh14buZYyAvRFNTIGhheSBEb2NUaW1lU3RhbXA="
                .into(),
        ));
    }
    for (id, obj) in &r_4sp.objects {
        let r_4sg = r_4si
            .objects
            .get(id)
            .ok_or_else(|| SignError::Pdf("aW5jcmVtZW50IHhvw6Egb2JqZWN0IMSRw6Mga8O9".into()))?;
        if *id == r_4sb {
            if r_4sj {
                f_1a0(obj, r_4sg)?;
            } else if r_4sg != obj {
                return Err(SignError::Pdf("aW5jcmVtZW50IHPhu61hIGNhdGFsb2c=".into()));
            }
        } else if r_4sc.contains(id) {
        } else if r_4sg != obj {
            return Err(SignError::Pdf(
                "aW5jcmVtZW50IHPhu61hIG9iamVjdCDEkcOjIGvDvSAobmdvw6BpIC9EU1MgKyBEb2NUaW1lU3RhbXAp"
                    .into(),
            ));
        }
    }
    let material = if r_4sj {
        f_25q(&r_4si)?
    } else {
        ValidationMaterial::default()
    };
    Ok(T9s {
        has_dss: r_4sj,
        material,
        archive_present,
        archive_structural_ok,
        archive_timestamp_trusted,
        archive_time_unix,
    })
}
