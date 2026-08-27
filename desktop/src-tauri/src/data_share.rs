// Copyright (c) 2026 DigiText. All rights reserved.
use crate::auth::AuthState;
use crate::db::{self, Db};
use crate::error::AppError;
use crate::lan_session::{self, CodeStrength};
use base64::Engine;
use digitext_core::Perm;
use mdns_sd::{ServiceDaemon, ServiceInfo};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, State};
pub const SERVICE_TYPE: &str = "_digitext-data._tcp.local.";
pub const C_KA: &str = "lan_site_code";
pub const C_K1: &str = "lan_share_port";
pub const DEFAULT_SHARE_PORT: u16 = 45817;
pub const C_K2: [u16; 4] = [
    DEFAULT_SHARE_PORT,
    DEFAULT_SHARE_PORT + 1,
    DEFAULT_SHARE_PORT + 2,
    DEFAULT_SHARE_PORT + 3,
];
#[derive(Debug, Clone, Copy)]
pub struct T4u {
    pub magic: &'static [u8],
    pub reply: &'static str,
    pub min_req: usize,
    pub ports: &'static [u16],
}
pub const C_3M: T4u = T4u {
    magic: b"DIGITEXT-DISCOVER-v1",
    reply: "DIGITEXT-HERE-v1 ",
    min_req: 64,
    ports: &C_K2,
};
pub const C_MR: T4u = T4u {
    magic: b"DIGITEXT-TREE-DISCOVER-v1",
    reply: "DIGITEXT-TREE-HERE-v1 ",
    min_req: 160,
    ports: &C_MT,
};
pub const C_MT: [u16; 4] = [
    DEFAULT_SHARE_PORT + 4,
    DEFAULT_SHARE_PORT + 5,
    DEFAULT_SHARE_PORT + 6,
    DEFAULT_SHARE_PORT + 7,
];
const C_4B: u32 = 32;
pub fn f_15p(r_15u: &T4u) -> Vec<u8> {
    let mut v = r_15u.magic.to_vec();
    v.resize(r_15u.min_req.max(r_15u.magic.len()), 0);
    v
}
pub(crate) fn f_1iz(r_15v: &T4u, r_15w: &[u8]) -> bool {
    r_15w.len() >= r_15v.min_req && r_15w.starts_with(r_15v.magic)
}
pub(crate) fn f_15o(r_15x: &T4u, tcp_port: u16, r_15y: Option<&str>) -> Vec<u8> {
    let mut r_15z = format!("{}{tcp_port}", r_15x.reply);
    if let Some(x) = r_15y.map(str::trim).filter(|x| !x.is_empty()) {
        r_15z.push(' ');
        r_15z.push_str(x);
    }
    r_15z.into_bytes()
}
pub const SESSION_PATH: &str = "/data/session";
pub const MANIFEST_PATH: &str = "/data/v2/manifest";
pub const FILE_PATH: &str = "/data/v2/file";
const C_AG: [&str; 4] = [
    "/data/manifest",
    "/data/file",
    "/data/challenge",
    "/data/authsign",
];
pub const SESSION_ID_HEADER: &str = "X-Session-Id";
pub const SESSION_REQ_HEADER: &str = "X-Session-Req";
pub const SESSION_PROOF_HEADER: &str = "X-Session-Proof";
const C_JW: Duration = Duration::from_secs(30 * 60);
const C_C3: usize = 32;
const C_BW: usize = 100_000;
#[derive(Default)]
pub struct T4j(pub Mutex<Option<Tlm>>);
pub struct Tlm {
    daemon: ServiceDaemon,
    fullname: String,
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
    discovery: Option<JoinHandle<()>>,
    pub port: u16,
}
impl Tlm {
    pub fn fullname(&self) -> &str {
        &self.fullname
    }
    fn shutdown(&mut self) {
        if self.stop.swap(true, Ordering::SeqCst) {
            return;
        }
        if let Ok(st) = self.daemon.unregister(&self.fullname) {
            let _ = st.recv_timeout(Duration::from_millis(500));
        }
        if let Ok(st) = self.daemon.shutdown() {
            let _ = st.recv_timeout(Duration::from_millis(500));
        }
        let _ = std::net::TcpStream::connect(("127.0.0.1", self.port));
        self.join.take();
        self.discovery.take();
    }
}
impl Drop for Tlm {
    fn drop(&mut self) {
        self.shutdown();
    }
}
pub(crate) fn f_25a(r_163: &str, r_161: &str) -> Option<String> {
    r_163.split_once('?')?.1.split('&').find_map(|r_162| {
        let (r_160, v) = r_162.split_once('=')?;
        (r_160 == r_161).then(|| v.to_string())
    })
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Tix {
    pub sid: String,
    pub rn: String,
    pub proof: String,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Route {
    Session { epk: String, nonce: String },
    Manifest(Tix),
    File { blake3: String, auth: Tix },
    Legacy,
}
fn f_2ov(r_164: &str) -> bool {
    r_164.len() == 64 && r_164.bytes().all(|b| b.is_ascii_hexdigit())
}
pub(crate) fn route(
    r_165: bool,
    r_168: &str,
    sid: Option<&str>,
    rn: Option<&str>,
    proof: Option<&str>,
) -> Result<Route, u16> {
    if !r_165 {
        return Err(405);
    }
    let path = r_168.split('?').next().unwrap_or(r_168);
    if C_AG.contains(&path) {
        return Ok(Route::Legacy);
    }
    if path == SESSION_PATH {
        let epk = f_25a(r_168, "epk").filter(|e| lan_session::f_2ow(e));
        let nonce = f_25a(r_168, "nonce").filter(|n| lan_session::f_2os(n));
        return match (epk, nonce) {
            (Some(epk), Some(nonce)) => Ok(Route::Session { epk, nonce }),
            _ => Err(400),
        };
    }
    let auth = match (sid, rn, proof) {
        (Some(r_167), Some(r_166), Some(p))
            if f_2ov(r_167) && lan_session::f_2os(r_166) && !p.trim().is_empty() =>
        {
            Tix {
                sid: r_167.to_string(),
                rn: r_166.to_string(),
                proof: p.to_string(),
            }
        }
        _ => return Err(400),
    };
    match path {
        MANIFEST_PATH => Ok(Route::Manifest(auth)),
        FILE_PATH => match f_25a(r_168, "blake3") {
            Some(h) if valid_blake3(&h) => Ok(Route::File { blake3: h, auth }),
            _ => Err(400),
        },
        _ => Err(404),
    }
}
pub(crate) fn valid_blake3(h: &str) -> bool {
    h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit())
}
struct Tl2 {
    session: lan_session::Session,
    seen_rn: std::collections::HashSet<String>,
    last: Instant,
}
#[derive(Default)]
pub(crate) struct SessionStore {
    map: std::collections::HashMap<String, Tl2>,
}
impl SessionStore {
    pub(crate) fn insert(&mut self, session: lan_session::Session, r_16a: Instant) {
        self.map.retain(|_, s| r_16a.duration_since(s.last) < C_JW);
        while self.map.len() >= C_C3 {
            let Some(r_16b) = self
                .map
                .iter()
                .min_by_key(|(_, s)| s.last)
                .map(|(r_169, _)| r_169.clone())
            else {
                break;
            };
            self.map.remove(&r_16b);
        }
        self.map.insert(
            session.sid.clone(),
            Tl2 {
                session,
                seen_rn: std::collections::HashSet::new(),
                last: r_16a,
            },
        );
    }
    fn claim(&mut self, sid: &str, rn: &str, r_16c: Instant) -> Option<lan_session::Session> {
        let r_16d = self.map.get_mut(sid)?;
        if r_16c.duration_since(r_16d.last) >= C_JW || r_16d.seen_rn.len() >= C_BW {
            self.map.remove(sid);
            return None;
        }
        if !r_16d.seen_rn.insert(rn.to_string()) {
            tracing::warn!(
                "từ chối request LAN phát lại (nonce request đã dùng)"
            );
            return None;
        }
        r_16d.last = r_16c;
        Some(r_16d.session.clone())
    }
}
pub(crate) fn f_su(
    r_16l: &mut SessionStore,
    auth: &Tix,
    r_16e: &[u8; 32],
    r_16i: Instant,
) -> Result<(String, lan_session::Sealer), u16> {
    let session = r_16l.claim(&auth.sid, &auth.rn, r_16i).ok_or(403u16)?;
    let (mut r_16j, mut r_16k) = session.request_keys(&auth.rn).map_err(|_| 500u16)?.server();
    let r_16f = b64().decode(&auth.proof).map_err(|_| 403u16)?;
    let r_16g = r_16j.open(0, true, &r_16f).map_err(|_| 403u16)?;
    if !lan_session::f_24d(&r_16g, &lan_session::f_104(r_16e, &session.sid)) {
        return Err(403);
    }
    let r_16h = r_16k
        .seal(0, true, &lan_session::f_2dp(r_16e, &session.sid))
        .map_err(|_| 500u16)?;
    Ok((b64().encode(r_16h), r_16k))
}
fn b64() -> base64::engine::general_purpose::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}
pub(crate) fn f_2co(header: Option<&str>, r_16p: u64) -> Result<u64, u16> {
    let Some(r_16n) = header.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(0);
    };
    let Some(r_16m) = r_16n.strip_prefix("bytes=") else {
        return Err(416);
    };
    let Some((s, e)) = r_16m.split_once('-') else {
        return Err(416);
    };
    if !e.trim().is_empty() {
        return Err(416);
    }
    let Ok(r_16o) = s.trim().parse::<u64>() else {
        return Err(416);
    };
    if r_16o == 0 {
        return Ok(0);
    }
    if r_16o >= r_16p || r_16o % lan_session::C_7H as u64 != 0 {
        return Err(416);
    }
    Ok(r_16o)
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SiteCodeSource {
    Server,
    Lan,
}
pub(crate) fn f_2ex(r_16r: Option<&str>, r_16s: &str, r_16t: SiteCodeSource) -> bool {
    if r_16s.trim().is_empty() || r_16t != SiteCodeSource::Server {
        return false;
    }
    match r_16r.map(str::trim) {
        Some(r_16q) if !r_16q.is_empty() => r_16q != r_16s.trim(),
        _ => true,
    }
}
pub(crate) fn f_1zw(r_16u: Option<&str>) -> Result<Option<u16>, String> {
    let Some(s) = r_16u.map(str::trim).filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    match s.parse::<u16>() {
        Ok(p) if p >= 1024 => Ok(Some(p)),
        _ => {
            Err(
                format!(
                    "Cổng chia sẻ không hợp lệ — nhập số 1024–65535 (bỏ trống = cổng chung {DEFAULT_SHARE_PORT})."
                ),
            )
        }
    }
}
fn hash_file(path: &str) -> std::io::Result<String> {
    let mut r_16w = File::open(path)?;
    let mut r_16x = blake3::Hasher::new();
    let mut r_16v = vec![0u8; 1 << 16];
    loop {
        let n = r_16w.read(&mut r_16v)?;
        if n == 0 {
            break;
        }
        r_16x.update(&r_16v[..n]);
    }
    Ok(r_16x.finalize().to_hex().to_string())
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Td1 {
    pub blake3: String,
    pub size: i64,
    pub filename: String,
    pub ho_so_ref: String,
    pub page_count: Option<i64>,
    pub source_user_id: String,
    pub source_username: Option<String>,
    pub project_id: Option<String>,
    #[serde(default)]
    pub phong_ma: Option<String>,
    #[serde(default)]
    pub phong_ten: Option<String>,
    #[serde(default)]
    pub phong_so: Option<String>,
    #[serde(default)]
    pub muc_luc_so: Option<String>,
    #[serde(default)]
    pub tieu_de: Option<String>,
    #[serde(default)]
    pub thoi_gian: Option<String>,
    #[serde(default)]
    pub nam: Option<String>,
    #[serde(default)]
    pub so_to: Option<String>,
}
struct Tgw {
    path: String,
    filename: String,
    size: i64,
    mtime: i64,
}
fn f_1a2(r_16z: &std::path::Path) -> Vec<Tgw> {
    crate::dossier::store::f_219(r_16z)
        .into_iter()
        .filter_map(|p| {
            let meta = std::fs::metadata(&p).ok()?;
            let mtime = meta
                .modified()
                .ok()
                .and_then(|r_170| r_170.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|r_16y| r_16y.as_secs() as i64)
                .unwrap_or(0);
            let filename = p.file_name()?.to_string_lossy().into_owned();
            Some(Tgw {
                path: p.to_string_lossy().into_owned(),
                filename,
                size: meta.len() as i64,
                mtime,
            })
        })
        .collect()
}
fn f_1g5(r_171: &Db, r_172: &Tgw) -> Option<(String, Option<i64>)> {
    if let Ok(conn) = r_171.0.lock() {
        if let Ok(Some(hit)) = db::f_20u(&conn, &r_172.path, r_172.size, r_172.mtime) {
            return Some(hit);
        }
    }
    let blake3 = hash_file(&r_172.path).ok()?;
    let page_count = match crate::dossier::store::f_122(std::path::Path::new(&r_172.path)) {
        0 => None,
        n => Some(n),
    };
    if let Ok(conn) = r_171.0.lock() {
        let _ = db::f_20v(
            &conn,
            &r_172.path,
            r_172.size,
            r_172.mtime,
            &blake3,
            page_count,
        );
    }
    Some((blake3, page_count))
}
pub(crate) fn f_1g2(r_174: &Db, path: &std::path::Path) -> Option<(String, Option<i64>, i64)> {
    let meta = std::fs::metadata(path).ok()?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|r_176| r_176.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|r_173| r_173.as_secs() as i64)
        .unwrap_or(0);
    let r_175 = Tgw {
        path: path.to_string_lossy().into_owned(),
        filename: path.file_name()?.to_string_lossy().into_owned(),
        size: meta.len() as i64,
        mtime,
    };
    let (blake3, page_count) = f_1g5(r_174, &r_175)?;
    Some((blake3, page_count, r_175.size))
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T1a {
    pub ho_so_ref: String,
    pub reason: String,
}
#[derive(Debug, Default)]
pub(crate) struct Shareable {
    pub files: Vec<(Td1, String)>,
    pub blocked: Vec<T1a>,
}
fn f_22q(r_177: &std::path::Path) -> Option<String> {
    let name = r_177.parent()?.parent()?.file_name()?.to_string_lossy();
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_string())
}
pub(crate) const C_OZ: &str = "chua-phan-loai";
pub(crate) const C_P0: &str = "Chưa phân loại";
fn f_29b(row: &db::Tln) -> Result<(std::path::PathBuf, String, String), String> {
    let r_178 = crate::dossier::store::f_299(row.folder_path.as_deref(), Some(&row.pdf_path))
        .ok_or_else(|| {
            "không tìm thấy thư mục hồ sơ trên đĩa".to_string()
        })?;
    let phong_ma = row
        .phong_ma
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| C_OZ.to_string());
    let phong_ten = row
        .phong_ten
        .clone()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| f_22q(&r_178))
        .unwrap_or_else(|| C_P0.to_string());
    Ok((r_178, phong_ma, phong_ten))
}
pub(crate) fn f_1p5(r_179: &AppHandle) -> Result<Shareable, String> {
    let r_17b = r_179.state::<Db>();
    let r_17g = {
        let r_17a = r_17b.0.lock().map_err(|_| "DB bận".to_string())?;
        db::f_2er(&r_17a).map_err(|e| e.to_string())?
    };
    let mut r_17f = Shareable::default();
    for row in r_17g {
        let ho_so_ref = format!("{}/{}", row.hop_so, row.ho_so_so);
        let (r_17c, phong_ma, phong_ten) = match f_29b(&row) {
            Ok(v) => v,
            Err(reason) => {
                r_17f.blocked.push(T1a { ho_so_ref, reason });
                continue;
            }
        };
        for r_17d in f_1a2(&r_17c) {
            let Some((blake3, page_count)) = f_1g5(&r_17b, &r_17d) else {
                continue;
            };
            let r_17e = r_17d.path.clone();
            r_17f.files.push((
                Td1 {
                    blake3,
                    size: r_17d.size,
                    filename: r_17d.filename,
                    ho_so_ref: ho_so_ref.clone(),
                    page_count,
                    source_user_id: row.source_user_id.clone(),
                    source_username: row.source_username.clone(),
                    project_id: row.source_project_id.clone(),
                    phong_ma: Some(phong_ma.clone()),
                    phong_ten: Some(phong_ten.clone()),
                    phong_so: row.phong_so.clone(),
                    muc_luc_so: row.muc_luc_so.clone(),
                    tieu_de: Some(row.tieu_de.clone()),
                    thoi_gian: row.thoi_gian.clone(),
                    nam: row.nam.clone(),
                    so_to: row.so_to.clone(),
                },
                r_17e,
            ));
        }
    }
    Ok(r_17f)
}
fn f_vi(r_17h: &AppHandle) -> Result<Vec<Td1>, String> {
    Ok(f_1p5(r_17h)?
        .files
        .into_iter()
        .map(|(r_17i, _)| r_17i)
        .collect())
}
fn f_2et(r_17k: &Db) -> Vec<std::path::PathBuf> {
    let Ok(r_17j) = r_17k.0.lock() else {
        return Vec::new();
    };
    let Ok(r_17m) = db::f_2er(&r_17j) else {
        return Vec::new();
    };
    r_17m
        .into_iter()
        .filter_map(|r_17l| {
            crate::dossier::store::f_299(r_17l.folder_path.as_deref(), Some(&r_17l.pdf_path))
        })
        .collect()
}
pub(crate) fn f_20k(path: &std::path::Path, r_17o: &[std::path::PathBuf]) -> bool {
    match path.parent() {
        Some(r_17p) => r_17o.iter().any(|r_17n| r_17n == r_17p),
        None => false,
    }
}
fn f_1gf(r_17s: &tiny_http::Request, r_17r: &str) -> Option<String> {
    let r_17q = r_17r.to_ascii_lowercase();
    r_17s
        .headers()
        .iter()
        .find(|h| h.field.as_str().as_str().to_ascii_lowercase() == r_17q)
        .map(|h| h.value.as_str().to_string())
}
fn f_2k4(r_17u: u16, r_17t: &str) -> tiny_http::Response<std::io::Cursor<Vec<u8>>> {
    tiny_http::Response::from_string(r_17t).with_status_code(r_17u)
}
pub(crate) const C_AF: &str = "Giao thức LAN cũ đã bị gỡ (nó gửi mã site dạng trần trên mạng không mã hoá). Hãy cập nhật DigiText trên MỌI máy trong site rồi thu thập lại.";
fn f_1jz() -> tiny_http::Header {
    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).expect("header")
}
fn f_uo() -> tiny_http::Header {
    tiny_http::Header::from_bytes(&b"Content-Type"[..], &b"application/octet-stream"[..])
        .expect("header")
}
fn f_24e(r_17v: &str) -> tiny_http::Header {
    tiny_http::Header::from_bytes(SESSION_PROOF_HEADER.as_bytes(), r_17v.as_bytes())
        .expect("header")
}
fn serve_loop(
    r_184: tiny_http::Server,
    r_17w: AppHandle,
    r_17x: [u8; 32],
    r_17y: String,
    r_185: ed25519_dalek::SigningKey,
    stop: Arc<AtomicBool>,
) {
    let mut r_186 = SessionStore::default();
    while !stop.load(Ordering::SeqCst) {
        let r_182 = match r_184.recv_timeout(Duration::from_millis(250)) {
            Ok(Some(r_181)) => r_181,
            Ok(None) => continue,
            Err(_) => break,
        };
        let r_17z = *r_182.method() == tiny_http::Method::Get;
        let r_187 = r_182.url().to_string();
        let sid = f_1gf(&r_182, SESSION_ID_HEADER);
        let rn = f_1gf(&r_182, SESSION_REQ_HEADER);
        let proof = f_1gf(&r_182, SESSION_PROOF_HEADER);
        let r_180 = Instant::now();
        match route(
            r_17z,
            &r_187,
            sid.as_deref(),
            rn.as_deref(),
            proof.as_deref(),
        ) {
            Ok(Route::Legacy) => {
                let _ = r_182.respond(f_2k4(426, C_AF));
            }
            Ok(Route::Session { epk, nonce }) => {
                match lan_session::f_29j(&r_185, &r_17y, &epk, &nonce) {
                    Ok((r_183, session)) => {
                        r_186.insert(session, r_180);
                        let body = serde_json::to_string(&r_183).unwrap_or_default();
                        let _ = r_182
                            .respond(tiny_http::Response::from_string(body).with_header(f_1jz()));
                    }
                    Err(e) => {
                        tracing::warn!(
                            error = % e, "bắt tay phiên LAN lỗi"
                        );
                        let _ = r_182.respond(f_2k4(400, ""));
                    }
                }
            }
            Ok(Route::Manifest(auth)) => match f_su(&mut r_186, &auth, &r_17x, r_180) {
                Ok((proof_b64, mut sealer)) => match f_vi(&r_17w) {
                    Ok(files) => {
                        let json = serde_json::to_string(&files).unwrap_or_else(|_| "[]".into());
                        match lan_session::f_2cl(&mut sealer, json.as_bytes()) {
                            Ok(body) => {
                                let _ = r_182.respond(
                                    tiny_http::Response::from_data(body)
                                        .with_header(f_uo())
                                        .with_header(f_24e(&proof_b64)),
                                );
                            }
                            Err(_) => {
                                let _ = r_182.respond(f_2k4(500, ""));
                            }
                        }
                    }
                    Err(e) => {
                        let _ = r_182.respond(f_2k4(500, &e));
                    }
                },
                Err(status) => {
                    let _ = r_182.respond(f_2k4(status, ""));
                }
            },
            Ok(Route::File { blake3, auth }) => match f_su(&mut r_186, &auth, &r_17x, r_180) {
                Ok((proof_b64, sealer)) => f_2dj(r_182, &r_17w, &blake3, &proof_b64, sealer),
                Err(status) => {
                    let _ = r_182.respond(f_2k4(status, ""));
                }
            },
            Err(status) => {
                let _ = r_182.respond(f_2k4(status, ""));
            }
        }
    }
}
fn f_2dj(
    r_18d: tiny_http::Request,
    r_188: &AppHandle,
    hash: &str,
    r_18b: &str,
    r_18e: lan_session::Sealer,
) {
    let r_18a = r_188.state::<Db>();
    let path = {
        match r_18a
            .0
            .lock()
            .ok()
            .and_then(|r_189| db::f_218(&r_189, hash).ok().flatten())
        {
            Some(p) => p,
            None => {
                let _ = r_18d.respond(f_2k4(404, ""));
                return;
            }
        }
    };
    if !f_20k(std::path::Path::new(&path), &f_2et(&r_18a)) {
        tracing::warn!(
            hash = % hash,
            "từ chối phục vụ: file không nằm trong vùng chia sẻ"
        );
        let _ = r_18d.respond(f_2k4(404, ""));
        return;
    }
    let r_18c = f_1gf(&r_18d, "Range");
    f_2hs(
        r_18d,
        std::path::Path::new(&path),
        r_18c.as_deref(),
        r_18b,
        r_18e,
    );
}
pub(crate) fn f_2hs(
    r_18p: tiny_http::Request,
    path: &std::path::Path,
    r_18n: Option<&str>,
    r_18m: &str,
    r_18r: lan_session::Sealer,
) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            let _ = r_18p.respond(f_2k4(404, ""));
            return;
        }
    };
    let r_18l = file.metadata().map(|r_18k| r_18k.len()).unwrap_or(0);
    let r_18v = lan_session::f_zd(r_18l);
    let r_18i = lan_session::f_1md(r_18l);
    let r_18s = match f_2co(r_18n, r_18v) {
        Ok(r_18q) => r_18q,
        Err(code) => {
            let _ = r_18p.respond(f_2k4(code, ""));
            return;
        }
    };
    let r_18t = r_18s / lan_session::C_7H as u64;
    let mut f = file;
    if f.seek(SeekFrom::Start(r_18t * lan_session::C_7I as u64))
        .is_err()
    {
        let _ = r_18p.respond(f_2k4(500, ""));
        return;
    }
    let r_18o = lan_session::SealingReader::new(f, r_18r, r_18t, r_18i);
    let r_18j = r_18v - r_18s;
    let r_18f = tiny_http::Header::from_bytes(&b"Accept-Ranges"[..], &b"bytes"[..]).expect("h");
    let mut r_18h = vec![f_uo(), r_18f, f_24e(r_18m)];
    let r_18u = if r_18s == 0 {
        200
    } else {
        let r_18g = format!("bytes {}-{}/{}", r_18s, r_18v - 1, r_18v);
        r_18h.push(
            tiny_http::Header::from_bytes(&b"Content-Range"[..], r_18g.as_bytes()).expect("h"),
        );
        206
    };
    let _ = r_18p.respond(tiny_http::Response::new(
        tiny_http::StatusCode(r_18u),
        r_18h,
        r_18o,
        Some(r_18j as usize),
        None,
    ));
}
fn sanitize(r_18x: &str) -> String {
    r_18x
        .chars()
        .map(|r_18w| {
            if r_18w.is_ascii_alphanumeric() {
                r_18w
            } else {
                '-'
            }
        })
        .collect()
}
fn f_2hk(r_18z: &T4j) {
    if let Ok(mut r_18y) = r_18z.0.lock() {
        if let Some(mut p) = r_18y.take() {
            p.shutdown();
        }
    }
}
pub fn f_2hl(r_190: &AppHandle) {
    let r_191 = r_190.state::<T4j>();
    f_2hk(&r_191);
}
fn f_uq(r_193: Option<u16>, r_192: &[u16]) -> Result<tiny_http::Server, AppError> {
    if let Some(p) = r_193 {
        return tiny_http::Server::http(("0.0.0.0", p))
            .map_err(|e| {
                AppError::new(
                    "share_port",
                    format!(
                        "Không mở được cổng chia sẻ cố định {p} ({e}) — cổng đang bị chương trình khác chiếm? Sửa hoặc bỏ trống cổng trong màn “Đồng bộ dữ liệu”."
                    ),
                )
            });
    }
    for &p in r_192 {
        if let Ok(r_194) = tiny_http::Server::http(("0.0.0.0", p)) {
            return Ok(r_194);
        }
    }
    tracing::warn!(
        ? r_192,
        "cả thang cổng chia sẻ đều bận — dùng cổng ngẫu nhiên (máy admin không quét thấy, vẫn tìm được qua mDNS/broadcast/nhập tay)"
    );
    tiny_http::Server::http("0.0.0.0:0")
        .map_err(|e| AppError::internal(format!("mở listener chia sẻ lỗi: {e}")))
}
pub(crate) fn f_2gm(
    r_196: &'static T4u,
    tcp_port: u16,
    r_197: Option<String>,
    stop: Arc<AtomicBool>,
) -> Option<(u16, JoinHandle<()>)> {
    let r_19a = r_196
        .ports
        .iter()
        .find_map(|&p| std::net::UdpSocket::bind(("0.0.0.0", p)).ok())?;
    r_19a
        .set_read_timeout(Some(Duration::from_millis(250)))
        .ok()?;
    let r_19b = r_19a.local_addr().ok()?.port();
    tracing::info!(r_19b, tcp_port, "nghe gói hỏi UDP broadcast");
    std::thread::Builder::new()
        .name("lan-discovery-reply".into())
        .spawn(move || {
            let reply = f_15o(r_196, tcp_port, r_197.as_deref());
            let mut r_195 = [0u8; 512];
            let mut r_19c = Instant::now();
            let mut r_199 = 0u32;
            while !stop.load(Ordering::SeqCst) {
                let Ok((n, r_198)) = r_19a.recv_from(&mut r_195) else {
                    continue;
                };
                if !f_1iz(r_196, &r_195[..n]) {
                    continue;
                }
                if r_19c.elapsed() >= Duration::from_secs(1) {
                    r_19c = Instant::now();
                    r_199 = 0;
                }
                if r_199 >= C_4B {
                    continue;
                }
                r_199 += 1;
                let _ = r_19a.send_to(&reply, r_198);
            }
        })
        .ok()
        .map(|join| (r_19b, join))
}
fn f_1iw(r_19d: &AppHandle) -> bool {
    r_19d
        .try_state::<AuthState>()
        .and_then(|a| a.snapshot())
        .is_some_and(|r_19e| r_19e.role.allows(Perm::Admin))
}
pub fn f_2h9(r_19g: &AppHandle) -> Result<bool, AppError> {
    let r_19f = f_1iw(r_19g);
    let (code, device, pinned, signing_key) = {
        let r_19j = r_19g.state::<Db>();
        let conn = r_19j.0.lock().map_err(|_| AppError::DbPoisoned)?;
        let mut code = db::f_1ew(&conn, C_KA)?.unwrap_or_default();
        if code.trim().is_empty() && r_19f {
            code = lan_session::f_1e9();
            db::f_2ei(&conn, C_KA, Some(&code))?;
            tracing::info!(
                "máy quản trị chưa có mã site — đã tự sinh mã ngẫu nhiên 150 bit"
            );
        }
        let signing_key = crate::lan_identity::f_19x(&conn)?;
        let device = crate::lan_identity::f_157(&conn, &signing_key)?;
        let pinned = match f_1zw(db::f_1ew(&conn, C_K1)?.as_deref()) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(
                    error = % e,
                    "giá trị cổng chia sẻ cố định hỏng — dùng cổng ngẫu nhiên"
                );
                None
            }
        };
        (code, device, pinned, signing_key)
    };
    let r_19q = r_19g.state::<T4j>();
    f_2hk(&r_19q);
    if code.trim().is_empty() {
        return Ok(false);
    }
    if lan_session::f_2ft(&code) == CodeStrength::Weak {
        tracing::warn!(
            "mã site LAN hiện tại YẾU (dò được) — vẫn chạy để không làm gián đoạn site, nhưng hãy bấm “Sinh mã mạnh” ở màn Đồng bộ dữ liệu rồi phát lại cây thư mục"
        );
    }
    let r_19i = lan_session::code_key(&code).map_err(|e| {
        AppError::internal(format!(
            "dẫn xuất khoá mã site lỗi: {e}"
        ))
    })?;
    if let Ok(conn) = r_19g.state::<Db>().0.lock() {
        match db::f_24i(&conn) {
            Ok(n) if n > 0 => {
                tracing::info!(removed = n, "dọn cache hash PDF mồ côi")
            }
            Err(e) => {
                tracing::warn!(
                    error = % e, "dọn cache hash PDF lỗi (bỏ qua)"
                )
            }
            _ => {}
        }
    }
    let r_19p = f_uq(pinned, &C_K2)?;
    let port = r_19p
        .server_addr()
        .to_ip()
        .map(|a| a.port())
        .ok_or_else(|| {
            AppError::internal("không lấy được cổng listener")
        })?;
    let daemon = ServiceDaemon::new()
        .map_err(|e| AppError::internal(format!("mDNS khởi tạo lỗi: {e}")))?;
    let instance = format!("digitext-data-{}-{}", sanitize(&device), port);
    let r_19o = crate::lan_identity::pubkey_b64(&signing_key);
    let r_19n: [(&str, &str); 2] = [("v", "1"), ("pk", r_19o.as_str())];
    let info = ServiceInfo::new(
        SERVICE_TYPE,
        &instance,
        &format!("{instance}.local."),
        "",
        port,
        &r_19n[..],
    )
    .map_err(|e| AppError::internal(format!("mDNS service info lỗi: {e}")))?
    .enable_addr_auto();
    let fullname = info.get_fullname().to_string();
    daemon
        .register(info)
        .map_err(|e| AppError::internal(format!("mDNS đăng ký lỗi: {e}")))?;
    let stop = Arc::new(AtomicBool::new(false));
    let discovery = f_2gm(&C_3M, port, None, stop.clone()).map(|(_, join)| join);
    let r_19r = stop.clone();
    let r_19h = r_19g.clone();
    let r_19k = device.clone();
    let join = match std::thread::Builder::new()
        .name("lan-data-serve".into())
        .spawn(move || serve_loop(r_19p, r_19h, r_19i, r_19k, signing_key, r_19r))
    {
        Ok(r_19m) => r_19m,
        Err(e) => {
            let _ = daemon.unregister(&fullname);
            let _ = daemon.shutdown();
            stop.store(true, Ordering::SeqCst);
            return Err(AppError::internal(format!(
                "tạo luồng chia sẻ lỗi: {e}"
            )));
        }
    };
    if let Ok(mut r_19l) = r_19q.0.lock() {
        *r_19l = Some(Tlm {
            daemon,
            fullname,
            stop,
            join: Some(join),
            discovery,
            port,
        });
    }
    tracing::info!(
        port,
        "data_share: bắt đầu chia sẻ qua LAN"
    );
    Ok(true)
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tlo {
    pub sharing: bool,
    pub port: Option<u16>,
    pub file_count: usize,
    pub has_site_code: bool,
    pub site_code_weak: bool,
    pub device_fingerprint: String,
    pub blocked: Vec<T1a>,
}
#[tauri::command]
pub fn data_share_status(
    app: AppHandle,
    auth: State<'_, AuthState>,
    share: State<'_, T4j>,
) -> Result<Tlo, AppError> {
    auth.require(Perm::Operate)?;
    let port = share
        .0
        .lock()
        .ok()
        .and_then(|r_19x| r_19x.as_ref().map(|p| p.port));
    let (rows, strength, r_19v) = {
        let r_19t = app.state::<Db>();
        let r_19s = r_19t.0.lock().map_err(|_| AppError::DbPoisoned)?;
        let rows = db::f_2er(&r_19s)?;
        let strength = lan_session::f_2ft(&db::f_1ew(&r_19s, C_KA)?.unwrap_or_default());
        let r_19y = crate::lan_identity::f_19x(&r_19s)?;
        let r_19w = lan_session::device_fingerprint(&crate::lan_identity::pubkey_b64(&r_19y));
        (rows, strength, r_19w)
    };
    let mut file_count = 0usize;
    let mut blocked = Vec::new();
    for row in &rows {
        match f_29b(row) {
            Ok((r_19u, _, _)) => {
                file_count += crate::dossier::store::f_219(&r_19u).len();
            }
            Err(reason) => blocked.push(T1a {
                ho_so_ref: format!("{}/{}", row.hop_so, row.ho_so_so),
                reason,
            }),
        }
    }
    Ok(Tlo {
        sharing: port.is_some(),
        port,
        file_count,
        has_site_code: strength != CodeStrength::Empty,
        site_code_weak: strength == CodeStrength::Weak,
        device_fingerprint: r_19v,
        blocked,
    })
}
#[tauri::command]
pub fn data_share_restart(app: AppHandle, auth: State<'_, AuthState>) -> Result<bool, AppError> {
    auth.require(Perm::Operate)?;
    f_2h9(&app)
}
pub(crate) fn f_265(r_19z: &AppHandle, code: Option<&str>, r_1a5: SiteCodeSource) {
    let Some(code) = code.map(str::trim).filter(|r_1a0| !r_1a0.is_empty()) else {
        return;
    };
    let r_1a3 = r_19z.state::<Db>();
    let r_1a1 = match r_1a3.0.lock() {
        Ok(r_1a4) => {
            let r_1a2 = db::f_1ew(&r_1a4, C_KA).ok().flatten();
            if f_2ex(r_1a2.as_deref(), code, r_1a5) {
                let _ = db::f_2ei(&r_1a4, C_KA, Some(code));
                true
            } else {
                if r_1a5 == SiteCodeSource::Lan {
                    tracing::info!(
                        "mã site đến qua LAN KHÔNG tự ghi — chờ người vận hành xác nhận vân tay máy phát (chặn peer mDNS giả chiếm kênh chia sẻ)"
                    );
                } else if matches!(
                    r_1a2.as_deref().map(str::trim), Some(r_1a0) if ! r_1a0.is_empty() &&
                    r_1a0 != code
                ) {
                    tracing::warn!(
                        ?r_1a5,
                        "bỏ qua mã site đến kèm cây thư mục"
                    );
                }
                false
            }
        }
        Err(_) => false,
    };
    if r_1a1 {
        let _ = f_2h9(r_19z);
        tracing::info!(
            ? r_1a5,
            "nhận mã site LAN kèm cây thư mục → (bật lại) chia sẻ"
        );
    }
}
#[tauri::command]
pub fn lan_site_code_get(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<Option<String>, AppError> {
    auth.require(Perm::Operate)?;
    let r_1a6 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(db::f_1ew(&r_1a6, C_KA)?)
}
#[tauri::command]
pub fn lan_share_port_get(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<Option<u16>, AppError> {
    auth.require(Perm::Operate)?;
    let r_1a7 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(f_1zw(db::f_1ew(&r_1a7, C_K1)?.as_deref()).unwrap_or(None))
}
#[derive(Debug, Deserialize)]
pub struct Tli {
    pub port: String,
}
#[tauri::command]
pub fn lan_share_port_set(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tli,
) -> Result<bool, AppError> {
    auth.require(Perm::Operate)?;
    let r_1a9 = f_1zw(Some(&args.port)).map_err(AppError::validation)?;
    {
        let r_1a8 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        db::f_2ei(&r_1a8, C_K1, r_1a9.map(|p| p.to_string()).as_deref())?;
    }
    f_2h9(&app)
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tlj {
    pub code: String,
    #[serde(default)]
    pub allow_weak: bool,
}
pub(crate) fn f_2qt() -> String {
    "Mã site này quá yếu (dò được trong vài giây trên mạng nội bộ). Bấm “Sinh mã mạnh” để lấy mã ngẫu nhiên 30 ký tự, rồi phát lại cây thư mục để các máy nhân viên tự nhận mã mới."
        .to_string()
}
pub(crate) fn f_2fu(r_1aa: &str, allow_weak: bool) -> Result<Option<String>, String> {
    let r_1ab = r_1aa.trim();
    match lan_session::f_2ft(r_1ab) {
        CodeStrength::Empty => Ok(None),
        CodeStrength::Strong => Ok(lan_session::f_x4(r_1ab)),
        CodeStrength::Weak if allow_weak => Ok(Some(r_1ab.to_string())),
        CodeStrength::Weak => Err(f_2qt()),
    }
}
fn f_2hp(r_1ac: &AppHandle, r_1ae: &Db, code: Option<&str>) -> Result<bool, AppError> {
    {
        let r_1ad = r_1ae.0.lock().map_err(|_| AppError::DbPoisoned)?;
        db::f_2ei(&r_1ad, C_KA, code)?;
    }
    f_2h9(r_1ac)
}
#[tauri::command]
pub fn lan_site_code_set(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tlj,
) -> Result<bool, AppError> {
    auth.require(Perm::Operate)?;
    let r_1af = f_2fu(&args.code, args.allow_weak).map_err(AppError::validation)?;
    if args.allow_weak && matches!(& r_1af, Some(c) if lan_session::f_2ft(c) == CodeStrength::Weak)
    {
        tracing::warn!(
            "nhận mã site YẾU theo xác nhận của người vận hành — cần xoay vòng"
        );
    }
    f_2hp(&app, &db, r_1af.as_deref())
}
#[tauri::command]
pub fn lan_site_code_generate(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<String, AppError> {
    auth.require(Perm::Operate)?;
    let code = lan_session::f_1e9();
    f_2hp(&app, &db, Some(&code))?;
    Ok(code)
}
