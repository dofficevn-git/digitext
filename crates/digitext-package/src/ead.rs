// Copyright (c) 2026 DigiText. All rights reserved.
use crate::codes;
use crate::model::{PackageKind, T4o, T8s};
use crate::xml::Xml;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T9a {
    pub file_code: String,
    pub title: String,
    pub maintenance: String,
    pub mode: String,
    #[serde(default)]
    pub language: Vec<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub keyword: Option<String>,
    pub total_doc: u32,
    pub number_of_paper: Option<u32>,
    pub number_of_page: Option<u32>,
    pub physical_format: Option<String>,
    pub infor_sign: Option<String>,
    pub confidence_level: String,
    pub paper_file_code: Option<String>,
    #[serde(default)]
    pub risk_recovery: bool,
    pub risk_recovery_status: Option<String>,
    pub description: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T4y {
    pub doc_id: Option<String>,
    pub doc_code: String,
    pub maintenance: String,
    pub type_name: String,
    pub code_number: Option<String>,
    pub code_notation: Option<String>,
    pub issued_date: Option<String>,
    pub organ_name: Option<String>,
    pub subject: Option<String>,
    #[serde(default)]
    pub language: Vec<String>,
    pub number_of_page: Option<u32>,
    pub infor_sign: Option<String>,
    pub keyword: Option<String>,
    pub mode: String,
    pub confidence_level: Option<String>,
    pub autograph: Option<String>,
    pub physical_format: Option<String>,
    pub process: Option<String>,
    #[serde(default)]
    pub risk_recovery: bool,
    pub risk_recovery_status: Option<String>,
    pub description: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tha {
    pub doc_code: String,
    pub maintenance: String,
    pub type_pic: String,
    pub archives_number: Option<String>,
    pub infor_sign: Option<String>,
    pub event_name: Option<String>,
    pub image_title: Option<String>,
    pub photographer: Option<String>,
    pub photo_place: Option<String>,
    pub photo_time: Option<String>,
    pub colour: Option<String>,
    pub film_size: Option<String>,
    pub doc_attached: Option<String>,
    pub mode: String,
    pub physical_format: Option<String>,
    #[serde(default)]
    pub risk_recovery: bool,
    pub risk_recovery_status: Option<String>,
    pub description: Option<String>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Td7 {
    pub doc_code: String,
    pub maintenance: String,
    pub type_media: String,
    pub archives_number: Option<String>,
    pub infor_sign: Option<String>,
    pub event_name: Option<String>,
    pub movie_title: Option<String>,
    pub recorder: Option<String>,
    pub record_place: Option<String>,
    pub record_date: Option<String>,
    #[serde(default)]
    pub language: Vec<String>,
    pub play_time: Option<String>,
    pub doc_attached: Option<String>,
    pub mode: String,
    pub quality: Option<String>,
    pub physical_format: Option<String>,
    #[serde(default)]
    pub risk_recovery: bool,
    pub risk_recovery_status: Option<String>,
    pub description: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DocEad {
    Doc(T4y),
    Pic(Tha),
    Media(Td7),
}
impl DocEad {
    pub fn class(&self) -> &'static str {
        match self {
            DocEad::Doc(_) => "doc",
            DocEad::Pic(_) => "pic",
            DocEad::Media(_) => "media",
        }
    }
    pub fn premis_object_type(&self) -> &'static str {
        match self {
            DocEad::Doc(_) => "Doc",
            DocEad::Pic(_) => "Pic",
            DocEad::Media(_) => "Media",
        }
    }
}
fn opt(r_3wn: &mut Xml, r_3wm: &str, v: &Option<String>) {
    if let Some(v) = v {
        r_3wn.text_elem(r_3wm, v);
    }
}
fn f_1wi(r_3wp: &mut Xml, r_3wo: &str, v: &Option<u32>) {
    if let Some(v) = v {
        r_3wp.text_elem(r_3wo, &v.to_string());
    }
}
fn bool_flag(r_3wq: bool) -> &'static str {
    if r_3wq {
        "1"
    } else {
        "0"
    }
}
pub fn f_190(r_3ws: &T9a, r_3wr: &str) -> String {
    let mut r_3wu = Xml::new();
    r_3wu.open("simpledc", &[]);
    r_3wu.text_elem(r_3wr, &r_3ws.file_code);
    r_3wu.text_elem("title", &r_3ws.title);
    r_3wu.text_elem("maintenance", &r_3ws.maintenance);
    r_3wu.text_elem("mode", &r_3ws.mode);
    for r_3wt in &r_3ws.language {
        r_3wu.text_elem("language", r_3wt);
    }
    opt(&mut r_3wu, "startDate", &r_3ws.start_date);
    opt(&mut r_3wu, "endDate", &r_3ws.end_date);
    opt(&mut r_3wu, "keyword", &r_3ws.keyword);
    r_3wu.text_elem("totalDoc", &r_3ws.total_doc.to_string());
    f_1wi(&mut r_3wu, "numberOfPaper", &r_3ws.number_of_paper);
    f_1wi(&mut r_3wu, "numberOfPage", &r_3ws.number_of_page);
    opt(&mut r_3wu, "format", &r_3ws.physical_format);
    opt(&mut r_3wu, "inforSign", &r_3ws.infor_sign);
    r_3wu.text_elem("confidenceLevel", &r_3ws.confidence_level);
    opt(&mut r_3wu, "paperFileCode", &r_3ws.paper_file_code);
    r_3wu.text_elem("riskRecovery", bool_flag(r_3ws.risk_recovery));
    opt(
        &mut r_3wu,
        "riskRecoveryStatus",
        &r_3ws.risk_recovery_status,
    );
    opt(&mut r_3wu, "description", &r_3ws.description);
    r_3wu.close("simpledc");
    r_3wu.finish()
}
pub fn f_18z(r_3wv: &T8s, r_3ww: PackageKind) -> String {
    let mut r_3wy = Xml::new();
    r_3wy.open("simpledc", &[]);
    r_3wy.text_elem(r_3ww.hoso_code_tag(), &r_3wv.file_code);
    r_3wy.text_elem("title", &r_3wv.title);
    r_3wy.text_elem("source", &r_3wv.source);
    if r_3ww == PackageKind::SipTailieu {
        if let Some(r_3wx) = r_3wv.total_doc {
            r_3wy.text_elem("totalDoc", &r_3wx.to_string());
        }
    }
    opt(&mut r_3wy, "description", &r_3wv.description);
    r_3wy.close("simpledc");
    r_3wy.finish()
}
pub fn f_18w(r_3wz: &T4o) -> String {
    let mut r_3x0 = Xml::new();
    r_3x0.open("simpledc", &[]);
    r_3x0.text_elem("requestID", &r_3wz.request_id);
    r_3x0.text_elem("requestDate", &r_3wz.request_date);
    r_3x0.text_elem("purpose", &r_3wz.purpose);
    opt(&mut r_3x0, "purposeContent", &r_3wz.purpose_content);
    opt(&mut r_3x0, "feeObjectType", &r_3wz.fee_object_type);
    opt(&mut r_3x0, "researchTopic", &r_3wz.research_topic);
    opt(&mut r_3x0, "description", &r_3wz.description);
    r_3x0.close("simpledc");
    r_3x0.finish()
}
pub fn f_18x(r_3x4: &DocEad, r_3x2: &str) -> String {
    let mut r_3x5 = Xml::new();
    r_3x5.open("simpledc", &[]);
    match r_3x4 {
        DocEad::Doc(r_3x1) => {
            opt(&mut r_3x5, "docId", &r_3x1.doc_id);
            r_3x5.text_elem(r_3x2, &r_3x1.doc_code);
            r_3x5.text_elem("maintenance", &r_3x1.maintenance);
            r_3x5.text_elem("typeName", &r_3x1.type_name);
            opt(&mut r_3x5, "codeNumber", &r_3x1.code_number);
            opt(&mut r_3x5, "codeNotation", &r_3x1.code_notation);
            opt(&mut r_3x5, "issuedDate", &r_3x1.issued_date);
            opt(&mut r_3x5, "organName", &r_3x1.organ_name);
            opt(&mut r_3x5, "subject", &r_3x1.subject);
            for l in &r_3x1.language {
                r_3x5.text_elem("language", l);
            }
            f_1wi(&mut r_3x5, "numberOfPage", &r_3x1.number_of_page);
            opt(&mut r_3x5, "inforSign", &r_3x1.infor_sign);
            opt(&mut r_3x5, "keyword", &r_3x1.keyword);
            r_3x5.text_elem("mode", &r_3x1.mode);
            opt(&mut r_3x5, "confidenceLevel", &r_3x1.confidence_level);
            opt(&mut r_3x5, "autograph", &r_3x1.autograph);
            opt(&mut r_3x5, "format", &r_3x1.physical_format);
            opt(&mut r_3x5, "process", &r_3x1.process);
            r_3x5.text_elem("riskRecovery", bool_flag(r_3x1.risk_recovery));
            opt(
                &mut r_3x5,
                "riskRecoveryStatus",
                &r_3x1.risk_recovery_status,
            );
            opt(&mut r_3x5, "description", &r_3x1.description);
        }
        DocEad::Pic(p) => {
            r_3x5.text_elem(r_3x2, &p.doc_code);
            r_3x5.text_elem("maintenance", &p.maintenance);
            r_3x5.text_elem("typePic", &p.type_pic);
            opt(&mut r_3x5, "archivesNumber", &p.archives_number);
            opt(&mut r_3x5, "inforSign", &p.infor_sign);
            opt(&mut r_3x5, "eventName", &p.event_name);
            opt(&mut r_3x5, "imageTitle", &p.image_title);
            opt(&mut r_3x5, "photographer", &p.photographer);
            opt(&mut r_3x5, "photoPlace", &p.photo_place);
            opt(&mut r_3x5, "photoTime", &p.photo_time);
            opt(&mut r_3x5, "colour", &p.colour);
            opt(&mut r_3x5, "filmSize", &p.film_size);
            opt(&mut r_3x5, "docAttached", &p.doc_attached);
            r_3x5.text_elem("mode", &p.mode);
            opt(&mut r_3x5, "format", &p.physical_format);
            r_3x5.text_elem("riskRecovery", bool_flag(p.risk_recovery));
            opt(&mut r_3x5, "riskRecoveryStatus", &p.risk_recovery_status);
            opt(&mut r_3x5, "description", &p.description);
        }
        DocEad::Media(r_3x3) => {
            r_3x5.text_elem(r_3x2, &r_3x3.doc_code);
            r_3x5.text_elem("maintenance", &r_3x3.maintenance);
            r_3x5.text_elem("typeMedia", &r_3x3.type_media);
            opt(&mut r_3x5, "archivesNumber", &r_3x3.archives_number);
            opt(&mut r_3x5, "inforSign", &r_3x3.infor_sign);
            opt(&mut r_3x5, "eventName", &r_3x3.event_name);
            opt(&mut r_3x5, "movieTitle", &r_3x3.movie_title);
            opt(&mut r_3x5, "recorder", &r_3x3.recorder);
            opt(&mut r_3x5, "recordPlace", &r_3x3.record_place);
            opt(&mut r_3x5, "recordDate", &r_3x3.record_date);
            for l in &r_3x3.language {
                r_3x5.text_elem("language", l);
            }
            opt(&mut r_3x5, "playTime", &r_3x3.play_time);
            opt(&mut r_3x5, "docAttached", &r_3x3.doc_attached);
            r_3x5.text_elem("mode", &r_3x3.mode);
            opt(&mut r_3x5, "quality", &r_3x3.quality);
            opt(&mut r_3x5, "format", &r_3x3.physical_format);
            r_3x5.text_elem("riskRecovery", bool_flag(r_3x3.risk_recovery));
            opt(
                &mut r_3x5,
                "riskRecoveryStatus",
                &r_3x3.risk_recovery_status,
            );
            opt(&mut r_3x5, "description", &r_3x3.description);
        }
    }
    r_3x5.close("simpledc");
    r_3x5.finish()
}
fn f_xz(r_3x6: &mut Vec<String>, field: &str, code: &str, max: u8) {
    if !codes::f_1jd(code, max) {
        r_3x6.push(format!(
            "{field}OiBtw6MgJw=={code}JyBuZ2/DoGkgYuG6o25nIG3DoyAwMS4u{max:02}"
        ));
    }
}
fn f_y8(r_3x8: &mut Vec<String>, field: &str, code: &Option<String>, max: u8) {
    if let Some(r_3x7) = code {
        f_xz(r_3x8, field, r_3x7, max);
    }
}
fn f_y9(r_3x9: &mut Vec<String>, r_3xa: bool, r_3xc: &Option<String>) {
    match r_3xc {
        Some(r_3xb) => f_xz(r_3x9, "riskRecoveryStatus", r_3xb, codes::C_IB),
        None if r_3xa => {
            r_3x9
                .push(
                    "cmlza1JlY292ZXJ5U3RhdHVzOiBi4bqvdCBideG7mWMga2hpIGNo4bq/IMSR4buZIGThu7EgcGjDsm5nIGzDoCAnY8OzJw=="
                        .into(),
                )
        }
        None => {}
    }
}
fn f_y3(r_3xd: &mut Vec<String>, field: &str, code: &str, r_3xe: PackageKind) {
    f_xz(r_3xd, field, code, codes::C_BD);
    if r_3xe.requires_submission_agreement() && code != codes::C_BE {
        r_3xd
            .push(
                format!(
                    "{field}OiBuZ3Xhu5NuIG7hu5lwIGzGsHUgY2jhu4kgbmjhuq1uICcwMTogdsSpbmggdmnhu4VuJyAoxJFhbmcgbMOgICc={code}')"
                ),
            );
    }
}
pub fn f_2p2(r_3xh: &T9a, r_3xi: PackageKind) -> Vec<String> {
    let mut r_3xg = Vec::new();
    if r_3xh.file_code.is_empty() {
        r_3xg.push("YXJjRmlsZUNvZGUvZmlsZUNvZGU6IGtow7RuZyDEkcaw4bujYyBy4buXbmc=".into());
    }
    if r_3xh.title.is_empty() {
        r_3xg.push("dGl0bGU6IGtow7RuZyDEkcaw4bujYyBy4buXbmc=".into());
    }
    f_y3(&mut r_3xg, "maintenance", &r_3xh.maintenance, r_3xi);
    f_xz(&mut r_3xg, "mode", &r_3xh.mode, codes::C_CE);
    if r_3xh.language.is_empty() {
        r_3xg.push("bGFuZ3VhZ2U6IGPhuqduIMOtdCBuaOG6pXQgbeG7mXQgbcOjIG5nw7RuIG5n4buv".into());
    }
    for r_3xj in &r_3xh.language {
        f_xz(&mut r_3xg, "language", r_3xj, codes::C_AD);
    }
    f_xz(
        &mut r_3xg,
        "confidenceLevel",
        &r_3xh.confidence_level,
        codes::C_29,
    );
    f_y8(&mut r_3xg, "format", &r_3xh.physical_format, codes::C_FY);
    f_y9(&mut r_3xg, r_3xh.risk_recovery, &r_3xh.risk_recovery_status);
    let r_3xf = r_3xh.confidence_level == codes::C_2A || r_3xh.confidence_level == codes::C_28;
    if r_3xf {
        if r_3xh.number_of_paper.is_none() {
            r_3xg.push(
                "bnVtYmVyT2ZQYXBlcjogYuG6r3QgYnXhu5ljIHbhu5tpIGjhu5Mgc8ahIHPhu5EgaMOzYQ==".into(),
            );
        }
        if r_3xh.paper_file_code.is_none() {
            r_3xg.push(
                "cGFwZXJGaWxlQ29kZTogYuG6r3QgYnXhu5ljIHbhu5tpIGjhu5Mgc8ahIHPhu5EgaMOzYQ==".into(),
            );
        }
    }
    r_3xg
}
pub fn f_2p1(r_3xl: &T8s, r_3xm: PackageKind) -> Vec<String> {
    let mut r_3xk = Vec::new();
    if r_3xl.file_code.is_empty() {
        r_3xk.push("YXJjRmlsZUNvZGUvZmlsZUNvZGU6IGtow7RuZyDEkcaw4bujYyBy4buXbmc=".into());
    }
    if r_3xl.title.is_empty() {
        r_3xk.push("dGl0bGU6IGtow7RuZyDEkcaw4bujYyBy4buXbmc=".into());
    }
    if !codes::f_1ix(&r_3xl.source) {
        r_3xk
            .push(
                format!(
                    "c291cmNlOiBuaOG6rW4gJzAnICh2xINuIGLhuqNuIMSRaSkgaG/hurdjICcxJyAodsSDbiBi4bqjbiDEkeG6v24pLCDEkWFuZyBsw6AgJw=={}'",
                    r_3xl.source
                ),
            );
    }
    if r_3xm == PackageKind::SipTailieu && r_3xl.total_doc.is_none() {
        r_3xk.push("dG90YWxEb2M6IGLhuq90IGJ14buZYyB24bubaSBTSVBfdGFpbGlldQ==".into());
    }
    r_3xk
}
pub fn f_2ox(r_3xn: &T4o) -> Vec<String> {
    let mut r_3xo = Vec::new();
    if r_3xn.request_id.is_empty() {
        r_3xo.push("cmVxdWVzdElEOiBraMO0bmcgxJHGsOG7o2MgcuG7l25n".into());
    }
    if r_3xn.request_date.is_empty() {
        r_3xo.push("cmVxdWVzdERhdGU6IGtow7RuZyDEkcaw4bujYyBy4buXbmc=".into());
    }
    f_xz(&mut r_3xo, "purpose", &r_3xn.purpose, codes::C_G8);
    f_y8(
        &mut r_3xo,
        "feeObjectType",
        &r_3xn.fee_object_type,
        codes::C_79,
    );
    r_3xo
}
pub fn f_2oy(r_3xu: &DocEad, r_3xs: PackageKind) -> Vec<String> {
    let mut r_3xq = Vec::new();
    match r_3xu {
        DocEad::Doc(r_3xp) => {
            if r_3xp.doc_code.is_empty() {
                r_3xq.push("YXJjRG9jQ29kZS9kb2NDb2RlOiBraMO0bmcgxJHGsOG7o2MgcuG7l25n".into());
            }
            f_y3(&mut r_3xq, "maintenance", &r_3xp.maintenance, r_3xs);
            f_xz(&mut r_3xq, "typeName", &r_3xp.type_name, codes::C_NU);
            f_xz(&mut r_3xq, "mode", &r_3xp.mode, codes::C_CE);
            for l in &r_3xp.language {
                f_xz(&mut r_3xq, "language", l, codes::C_AD);
            }
            f_y8(
                &mut r_3xq,
                "confidenceLevel",
                &r_3xp.confidence_level,
                codes::C_29,
            );
            f_y8(&mut r_3xq, "format", &r_3xp.physical_format, codes::C_FY);
            f_y9(&mut r_3xq, r_3xp.risk_recovery, &r_3xp.risk_recovery_status);
            let r_3xr = matches!(
                r_3xp.confidence_level.as_deref(),
                Some(codes::C_27) | Some(codes::C_28)
            );
            match &r_3xp.process {
                Some(p) if !codes::f_1ix(p) => {
                    r_3xq
                        .push(
                            format!(
                                "cHJvY2Vzczogbmjhuq1uICcwJy8nMScsIMSRYW5nIGzDoCAn{p}'"
                            ),
                        )
                }
                None if r_3xr => {
                    r_3xq
                        .push(
                            "cHJvY2VzczogYuG6r3QgYnXhu5ljIHbhu5tpIHTDoGkgbGnhu4d1IGfhu5FjIMSRaeG7h24gdOG7rS9o4buXbiBo4bujcA=="
                                .into(),
                        )
                }
                _ => {}
            }
        }
        DocEad::Pic(p) => {
            if p.doc_code.is_empty() {
                r_3xq.push("YXJjRG9jQ29kZS9kb2NDb2RlOiBraMO0bmcgxJHGsOG7o2MgcuG7l25n".into());
            }
            f_y3(&mut r_3xq, "maintenance", &p.maintenance, r_3xs);
            f_xz(&mut r_3xq, "typePic", &p.type_pic, codes::C_NV);
            f_xz(&mut r_3xq, "mode", &p.mode, codes::C_CE);
            f_y8(&mut r_3xq, "colour", &p.colour, codes::C_26);
            f_y8(&mut r_3xq, "format", &p.physical_format, codes::C_FY);
            if let Some(a) = &p.doc_attached {
                if !codes::f_1ix(a) {
                    r_3xq.push(format!(
                        "ZG9jQXR0YWNoZWQ6IG5o4bqtbiAnMCcvJzEnLCDEkWFuZyBsw6AgJw=={a}'"
                    ));
                }
            }
            f_y9(&mut r_3xq, p.risk_recovery, &p.risk_recovery_status);
        }
        DocEad::Media(r_3xt) => {
            if r_3xt.doc_code.is_empty() {
                r_3xq.push("YXJjRG9jQ29kZS9kb2NDb2RlOiBraMO0bmcgxJHGsOG7o2MgcuG7l25n".into());
            }
            f_y3(&mut r_3xq, "maintenance", &r_3xt.maintenance, r_3xs);
            f_xz(&mut r_3xq, "typeMedia", &r_3xt.type_media, codes::C_NT);
            f_xz(&mut r_3xq, "mode", &r_3xt.mode, codes::C_CE);
            for l in &r_3xt.language {
                f_xz(&mut r_3xq, "language", l, codes::C_AD);
            }
            f_y8(&mut r_3xq, "format", &r_3xt.physical_format, codes::C_FY);
            if let Some(a) = &r_3xt.doc_attached {
                if !codes::f_1ix(a) {
                    r_3xq.push(format!(
                        "ZG9jQXR0YWNoZWQ6IG5o4bqtbiAnMCcvJzEnLCDEkWFuZyBsw6AgJw=={a}'"
                    ));
                }
            }
            f_y9(&mut r_3xq, r_3xt.risk_recovery, &r_3xt.risk_recovery_status);
        }
    }
    r_3xq
}
