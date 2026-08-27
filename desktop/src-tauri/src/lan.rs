// Copyright (c) 2026 DigiText. All rights reserved.
use crate::archive::commands::apply_folder_tree;
use crate::auth::AuthState;
use crate::data_share::{
    SessionStore, Tix, SESSION_ID_HEADER, SESSION_PROOF_HEADER, SESSION_REQ_HEADER,
};
use crate::db::{self, Db};
use crate::error::AppError;
use crate::lan_session;
use crate::sync::{Thy, Toh};
use digitext_core::{Perm, Permission};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, State};
const SERVICE_TYPE: &str = "_digitext-tree._tcp.local.";
const C_MS: &str = "/tree";
const C_MU: &str = "/tree/session";
const C_C6: usize = 32 * 1024 * 1024;
#[derive(Default)]
pub struct LanState(pub Mutex<Option<Thx>>);
pub struct Thx {
    daemon: ServiceDaemon,
    fullname: String,
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
    discovery: Option<JoinHandle<()>>,
    pub code: String,
    pub port: u16,
    pub project: String,
    pub version: i32,
}
impl Thx {
    fn shutdown(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Ok(st) = self.daemon.unregister(&self.fullname) {
            let _ = st.recv_timeout(Duration::from_millis(500));
        }
        if let Some(r_20l) = self.join.take() {
            let _ = r_20l.join();
        }
        self.discovery.take();
        if let Ok(st) = self.daemon.shutdown() {
            let _ = st.recv_timeout(Duration::from_millis(500));
        }
    }
}
pub(crate) fn f_1dh(n: u32) -> String {
    format!("{:06}", n % 1_000_000)
}
fn f_1u4() -> String {
    f_1dh(rand::random::<u32>())
}
const C_FS: u32 = 5;
const C_FR: Duration = Duration::from_secs(60);
#[derive(Default)]
struct T7o {
    count: u32,
    locked_until: Option<std::time::Instant>,
}
#[derive(Default)]
struct PairThrottle {
    fails: std::collections::HashMap<std::net::IpAddr, T7o>,
}
impl PairThrottle {
    fn allowed(&self, r_20m: std::net::IpAddr, r_20n: std::time::Instant) -> bool {
        match self.fails.get(&r_20m).and_then(|s| s.locked_until) {
            Some(r_20o) => r_20n >= r_20o,
            None => true,
        }
    }
    fn record_failure(&mut self, r_20p: std::net::IpAddr, r_20q: std::time::Instant) {
        if self.fails.len() > 1024 {
            self.fails
                .retain(|_, s| s.locked_until.is_some_and(|r_20r| r_20r > r_20q));
        }
        let s = self.fails.entry(r_20p).or_default();
        s.count += 1;
        if s.count >= C_FS {
            s.locked_until = Some(r_20q + C_FR);
            s.count = 0;
        }
    }
    fn record_success(&mut self, r_20s: std::net::IpAddr) {
        self.fails.remove(&r_20s);
    }
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TreeRoute {
    Session { epk: String, nonce: String },
    Bundle(Tix),
    Legacy,
    NotFound,
}
pub(crate) fn f_2m7(
    r_20t: bool,
    r_20v: &str,
    r_20u: Option<&str>,
    rn: Option<&str>,
    proof: Option<&str>,
) -> TreeRoute {
    if !r_20t {
        return TreeRoute::NotFound;
    }
    let path = r_20v.split('?').next().unwrap_or(r_20v);
    if path == C_MU {
        let epk = crate::data_share::f_25a(r_20v, "epk").filter(|e| lan_session::f_2ow(e));
        let nonce = crate::data_share::f_25a(r_20v, "nonce").filter(|n| lan_session::f_2os(n));
        return match (epk, nonce) {
            (Some(epk), Some(nonce)) => TreeRoute::Session { epk, nonce },
            _ => TreeRoute::NotFound,
        };
    }
    if path != C_MS {
        return TreeRoute::NotFound;
    }
    match (r_20u, rn, proof) {
        (Some(s), Some(r), Some(p)) if !s.is_empty() && !r.is_empty() && !p.is_empty() => {
            TreeRoute::Bundle(Tix {
                sid: s.to_string(),
                rn: r.to_string(),
                proof: p.to_string(),
            })
        }
        _ => TreeRoute::Legacy,
    }
}
fn f_2im(r_20w: &tiny_http::HeaderField) -> String {
    r_20w.as_str().as_str().to_ascii_lowercase()
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Taq {
    pub project: String,
    pub version: i32,
    pub phong: crate::archive::types::PhongIn,
    pub ho_sos: Vec<crate::archive::types::T97>,
    #[serde(default)]
    pub strip_accents: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tas {
    pub publishing: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i32>,
}
fn status_of(lan: &LanState) -> Tas {
    match lan.0.lock().ok().and_then(|r_20x| {
        r_20x.as_ref().map(|p| Tas {
            publishing: true,
            code: Some(p.code.clone()),
            port: Some(p.port),
            project: Some(p.project.clone()),
            version: Some(p.version),
        })
    }) {
        Some(s) => s,
        None => Tas {
            publishing: false,
            code: None,
            port: None,
            project: None,
            version: None,
        },
    }
}
#[tauri::command]
pub fn lan_publish_start(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    lan: State<'_, LanState>,
    args: Taq,
) -> Result<Tas, AppError> {
    auth.require_perm(Permission::TreeAssign)?;
    if args.phong.ma_du_an.trim().is_empty() {
        return Err(AppError::validation(
            "Thiếu thông tin cơ quan (Phông).",
        ));
    }
    f_2hj(&lan);
    let (site_code, r_21b, r_212) = {
        let r_215 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        let site_code =
            db::f_1ew(&r_215, crate::data_share::C_KA)?.filter(|r_210| !r_210.trim().is_empty());
        let r_21c = crate::lan_identity::f_19x(&r_215)?;
        let r_213 = crate::lan_identity::f_157(&r_215, &r_21c)?;
        (site_code, r_21c, r_213)
    };
    let r_20z = serde_json::to_vec(&Toh {
        phong: args.phong,
        ho_sos: args.ho_sos,
        strip_accents: args.strip_accents,
        site_code,
    })
    .map_err(|e| {
        AppError::internal(format!(
            "đóng gói cây thư mục lỗi: {e}"
        ))
    })?;
    let code = f_1u4();
    let r_211 = lan_session::code_key(&code).map_err(|e| {
        AppError::internal(format!(
            "dẫn xuất khoá mã ghép nối lỗi: {e}"
        ))
    })?;
    let r_21a = match crate::data_share::C_MT
        .iter()
        .find_map(|&p| tiny_http::Server::http(("0.0.0.0", p)).ok())
    {
        Some(s) => s,
        None => tiny_http::Server::http("0.0.0.0:0")
            .map_err(|e| AppError::internal(format!("mở listener LAN lỗi: {e}")))?,
    };
    let port = r_21a
        .server_addr()
        .to_ip()
        .map(|r_20y| r_20y.port())
        .ok_or_else(|| {
            AppError::internal("không lấy được cổng listener")
        })?;
    let daemon = ServiceDaemon::new()
        .map_err(|e| AppError::internal(format!("mDNS khởi tạo lỗi: {e}")))?;
    let instance = format!("digitext-{}-{}", sanitize(&args.project), port);
    let r_218 = crate::lan_identity::pubkey_b64(&r_21b);
    let r_217: [(&str, &str); 3] = [
        ("project", args.project.as_str()),
        ("version", &{ args.version.to_string() }),
        ("pk", r_218.as_str()),
    ];
    let info = ServiceInfo::new(
        SERVICE_TYPE,
        &instance,
        &format!("{instance}.local."),
        "",
        port,
        &r_217[..],
    )
    .map_err(|e| AppError::internal(format!("mDNS service info lỗi: {e}")))?
    .enable_addr_auto();
    let fullname = info.get_fullname().to_string();
    daemon
        .register(info)
        .map_err(|e| AppError::internal(format!("mDNS đăng ký lỗi: {e}")))?;
    let stop = Arc::new(AtomicBool::new(false));
    let discovery = crate::data_share::f_2gm(
        &crate::data_share::C_MR,
        port,
        Some(r_218.clone()),
        stop.clone(),
    )
    .map(|(_, join)| join);
    let r_21d = stop.clone();
    let join = match std::thread::Builder::new()
        .name("lan-tree-serve".into())
        .spawn(move || serve_loop(r_21a, r_20z, r_211, r_212, r_21b, r_21d))
    {
        Ok(r_216) => r_216,
        Err(e) => {
            let _ = daemon.unregister(&fullname);
            let _ = daemon.shutdown();
            stop.store(true, Ordering::SeqCst);
            return Err(AppError::internal(format!(
                "tạo luồng phục vụ LAN lỗi: {e}"
            )));
        }
    };
    let r_219 = Thx {
        daemon,
        fullname,
        stop,
        join: Some(join),
        discovery,
        code: code.clone(),
        port,
        project: args.project.clone(),
        version: args.version,
    };
    if let Ok(mut r_214) = lan.0.lock() {
        *r_214 = Some(r_219);
    }
    tracing::info!(project = % args.project, port, "lan_publish_start");
    Ok(status_of(&lan))
}
fn f_1gc(r_21f: &tiny_http::Request, r_21e: &str) -> Option<String> {
    let r_21g = r_21e.to_ascii_lowercase();
    r_21f
        .headers()
        .iter()
        .find(|h| f_2im(&h.field) == r_21g)
        .map(|h| h.value.as_str().to_string())
}
fn serve_loop(
    r_21w: tiny_http::Server,
    r_21j: Vec<u8>,
    r_21k: [u8; 32],
    r_21m: String,
    r_21y: ed25519_dalek::SigningKey,
    stop: Arc<AtomicBool>,
) {
    let mut r_221 = PairThrottle::default();
    let mut r_220 = SessionStore::default();
    while !stop.load(Ordering::SeqCst) {
        match r_21w.recv_timeout(Duration::from_millis(250)) {
            Ok(Some(r_21t)) => {
                let r_21o = r_21t
                    .remote_addr()
                    .map(|r_21h| r_21h.ip())
                    .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));
                let r_21q = Instant::now();
                if !r_221.allowed(r_21o, r_21q) {
                    let _ = r_21t
                        .respond(tiny_http::Response::from_string("").with_status_code(429u16));
                    continue;
                }
                let r_21p = *r_21t.method() == tiny_http::Method::Get;
                let r_222 = r_21t.url().to_string();
                let r_21x = f_1gc(&r_21t, SESSION_ID_HEADER);
                let rn = f_1gc(&r_21t, SESSION_REQ_HEADER);
                let proof = f_1gc(&r_21t, SESSION_PROOF_HEADER);
                match f_2m7(
                    r_21p,
                    &r_222,
                    r_21x.as_deref(),
                    rn.as_deref(),
                    proof.as_deref(),
                ) {
                    TreeRoute::Session { epk, nonce } => {
                        match lan_session::f_29j(&r_21y, &r_21m, &epk, &nonce) {
                            Ok((r_21u, session)) => {
                                r_220.insert(session, r_21q);
                                let body = serde_json::to_string(&r_21u).unwrap_or_default();
                                let r_21n = tiny_http::Header::from_bytes(
                                    &b"Content-Type"[..],
                                    &b"application/json"[..],
                                )
                                .expect("header hợp lệ");
                                let _ = r_21t.respond(
                                    tiny_http::Response::from_string(body).with_header(r_21n),
                                );
                            }
                            Err(_) => {
                                let _ = r_21t.respond(
                                    tiny_http::Response::from_string("").with_status_code(400u16),
                                );
                            }
                        }
                    }
                    TreeRoute::Bundle(r_21i) => {
                        match crate::data_share::f_su(&mut r_220, &r_21i, &r_21k, r_21q) {
                            Ok((r_21s, mut r_21v)) => {
                                r_221.record_success(r_21o);
                                match lan_session::f_2cl(&mut r_21v, &r_21j) {
                                    Ok(body) => {
                                        let r_21l = tiny_http::Header::from_bytes(
                                            &b"Content-Type"[..],
                                            &b"application/octet-stream"[..],
                                        )
                                        .expect("header hợp lệ");
                                        let r_21r = tiny_http::Header::from_bytes(
                                            SESSION_PROOF_HEADER.as_bytes(),
                                            r_21s.as_bytes(),
                                        )
                                        .expect("header hợp lệ");
                                        let _ = r_21t.respond(
                                            tiny_http::Response::from_data(body)
                                                .with_header(r_21l)
                                                .with_header(r_21r),
                                        );
                                    }
                                    Err(_) => {
                                        let _ = r_21t.respond(
                                            tiny_http::Response::from_string("")
                                                .with_status_code(500u16),
                                        );
                                    }
                                }
                            }
                            Err(r_21z) => {
                                if r_21z == 403 {
                                    r_221.record_failure(r_21o, r_21q);
                                }
                                let _ = r_21t.respond(
                                    tiny_http::Response::from_string("").with_status_code(r_21z),
                                );
                            }
                        }
                    }
                    TreeRoute::Legacy => {
                        let _ = r_21t.respond(
                            tiny_http::Response::from_string(crate::data_share::C_AF)
                                .with_status_code(426u16),
                        );
                    }
                    TreeRoute::NotFound => {
                        let _ = r_21t
                            .respond(tiny_http::Response::from_string("").with_status_code(404u16));
                    }
                }
            }
            Ok(None) => {}
            Err(_) => break,
        }
    }
}
fn f_2hj(lan: &LanState) {
    if let Ok(mut r_223) = lan.0.lock() {
        if let Some(mut p) = r_223.take() {
            p.shutdown();
        }
    }
}
#[tauri::command]
pub fn lan_publish_stop(
    auth: State<'_, AuthState>,
    lan: State<'_, LanState>,
) -> Result<Tas, AppError> {
    auth.require_perm(Permission::TreeAssign)?;
    f_2hj(&lan);
    Ok(status_of(&lan))
}
#[tauri::command]
pub fn lan_status(auth: State<'_, AuthState>, lan: State<'_, LanState>) -> Result<Tas, AppError> {
    auth.require(Perm::Read)?;
    Ok(status_of(&lan))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tap {
    pub instance: String,
    pub addr: String,
    pub port: u16,
    pub project: String,
    pub version: i32,
    pub fingerprint: String,
    pub source: &'static str,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tao {
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}
#[tauri::command]
pub async fn lan_discover(auth: State<'_, AuthState>, args: Tao) -> Result<Vec<Tap>, AppError> {
    auth.require(Perm::Operate)?;
    let r_226 = Duration::from_millis(args.timeout_ms.unwrap_or(2500).clamp(500, 8000));
    let r_225 = crate::collect::f_1p4();
    let bcast = tauri::async_runtime::spawn(async move {
        crate::collect::f_v5(
            &crate::data_share::C_MR,
            &r_225,
            Duration::from_millis(1200),
        )
        .await
    });
    let peers = tauri::async_runtime::spawn_blocking(move || f_v7(r_226))
        .await
        .map_err(|e| AppError::internal(format!("luồng quét LAN lỗi: {e}")))??;
    let r_224 = bcast.await.unwrap_or_default();
    Ok(f_1s7(peers, r_224))
}
pub(crate) fn f_1s7(mut peers: Vec<Tap>, r_228: Vec<crate::collect::T1c>) -> Vec<Tap> {
    for r_227 in r_228 {
        let addr = r_227.ip.to_string();
        if peers.iter().any(|p| p.addr == addr && p.port == r_227.port) {
            continue;
        }
        peers.push(Tap {
            instance: format!("bcast-{addr}-{}", r_227.port),
            addr,
            port: r_227.port,
            project: String::new(),
            version: 0,
            fingerprint: r_227
                .extra
                .as_deref()
                .map(lan_session::device_fingerprint)
                .unwrap_or_default(),
            source: "broadcast",
        });
    }
    peers
}
fn f_v7(r_22h: Duration) -> Result<Vec<Tap>, AppError> {
    let daemon = ServiceDaemon::new()
        .map_err(|e| AppError::internal(format!("mDNS khởi tạo lỗi: {e}")))?;
    let r_22e = daemon
        .browse(SERVICE_TYPE)
        .map_err(|e| AppError::internal(format!("mDNS quét lỗi: {e}")))?;
    let r_22a = std::time::Instant::now() + r_22h;
    let mut r_22b: Vec<Tap> = Vec::new();
    while let Some(r_22d) = r_22a.checked_duration_since(std::time::Instant::now()) {
        match r_22e.recv_timeout(r_22d) {
            Ok(ServiceEvent::ServiceResolved(info)) => {
                let addr = info
                    .get_addresses()
                    .iter()
                    .filter(|ip| ip.is_ipv4())
                    .map(|ip| ip.to_string())
                    .min_by_key(|r_229| crate::collect::f_1iu(r_229));
                if let Some(addr) = addr {
                    let r_22c = info.get_properties();
                    let project = r_22c
                        .get_property_val_str("project")
                        .unwrap_or("")
                        .to_string();
                    let version = r_22c
                        .get_property_val_str("version")
                        .and_then(|r_22g| r_22g.parse().ok())
                        .unwrap_or(0);
                    let fingerprint = r_22c
                        .get_property_val_str("pk")
                        .map(lan_session::device_fingerprint)
                        .unwrap_or_default();
                    let instance = info.get_fullname().to_string();
                    if !r_22b.iter().any(|p| p.instance == instance) {
                        r_22b.push(Tap {
                            instance,
                            addr,
                            port: info.get_port(),
                            project,
                            version,
                            fingerprint,
                            source: "mdns",
                        });
                    }
                }
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
    if let Ok(r_22f) = daemon.shutdown() {
        let _ = r_22f.recv_timeout(Duration::from_millis(500));
    }
    Ok(r_22b)
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tar {
    pub addr: String,
    pub port: u16,
    pub code: String,
}
#[tauri::command]
pub async fn lan_pull(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tar,
) -> Result<Thy, AppError> {
    let r_22v = auth.require(Perm::Operate)?;
    if args.addr.trim().is_empty() || args.code.trim().is_empty() {
        return Err(AppError::validation(
            "Thiếu địa chỉ máy hoặc mã ghép nối.",
        ));
    }
    let http = reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(Duration::from_secs(4))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    let r_22k = app.state::<Db>();
    let r_22q = match crate::collect::f_1wf(&http, args.addr.trim(), args.port, C_MU, None, &r_22k)
        .await
    {
        crate::collect::SessionVerdict::Ok(s) => *s,
        crate::collect::SessionVerdict::Mismatch => {
            return Err(
                AppError::new(
                    "lan_identity",
                    "Máy phát khai một khoá danh tính KHÁC khoá đã ghim cho máy đó — nghi mạo danh. Đã dừng, không nhận gì.",
                ),
            );
        }
        crate::collect::SessionVerdict::BadSig => {
            return Err(
                AppError::new(
                    "lan_identity",
                    "Máy phát không ký được phiên (chữ ký sai) — đã dừng.",
                ),
            );
        }
        crate::collect::SessionVerdict::Downgrade | crate::collect::SessionVerdict::OldProtocol => {
            return Err(
                AppError::new(
                    "lan_outdated",
                    "Máy phát đang chạy bản cũ, cần cập nhật — kênh LAN không mã hoá đã bị gỡ và KHÔNG có chế độ tương thích ngược.",
                ),
            );
        }
        crate::collect::SessionVerdict::Unreachable(e) => {
            return Err(AppError::new(
                "lan_unreachable",
                format!("Không kết nối được máy: {e}"),
            ));
        }
    };
    let r_22o = args.code.trim().to_string();
    let r_22j = tauri::async_runtime::spawn_blocking(move || lan_session::code_key(&r_22o))
        .await
        .map_err(|e| {
            AppError::internal(format!(
                "luồng dẫn xuất khoá lỗi: {e}"
            ))
        })?
        .map_err(|e| {
            AppError::internal(format!(
                "dẫn xuất khoá mã ghép nối lỗi: {e}"
            ))
        })?;
    let r_22w = format!("http://{}:{}{}", args.addr, args.port, C_MS);
    let (r_22s, mut r_22m) = crate::collect::f_2dt(
        http.get(&r_22w).timeout(Duration::from_secs(10)),
        &r_22q,
        &r_22j,
    )
    .map_err(|e| AppError::new("lan_error", e))?;
    let r_22t = r_22s.send().await.map_err(|e| {
        AppError::new(
            "lan_unreachable",
            format!("Không kết nối được máy: {e}"),
        )
    })?;
    if r_22t.status() == reqwest::StatusCode::FORBIDDEN {
        return Err(AppError::new(
            "pair_code",
            "Mã ghép nối không đúng.",
        ));
    }
    if r_22t.status().as_u16() == 429 {
        return Err(
            AppError::new(
                "pair_code",
                "Đã nhập sai mã ghép nối quá nhiều lần — chờ một phút rồi thử lại.",
            ),
        );
    }
    if !r_22t.status().is_success() {
        return Err(AppError::new(
            "lan_error",
            format!("Máy phát trả lỗi {}.", r_22t.status()),
        ));
    }
    crate::collect::f_ya(&r_22t, &mut r_22m, &r_22q, &r_22j)
        .map_err(|e| AppError::new("lan_identity", e))?;
    let r_22u = crate::collect::f_25n(r_22t, C_C6)
        .await
        .map_err(|e| AppError::new("lan_error", format!("Đọc gói cây lỗi: {e}")))?;
    let r_22r = lan_session::f_1w6(&mut r_22m, &r_22u).map_err(|e| {
        AppError::new(
            "lan_error",
            format!("Gói cây thư mục hỏng: {e}"),
        )
    })?;
    let r_22p: Toh = serde_json::from_slice(&r_22r).map_err(|e| {
        AppError::new(
            "lan_error",
            format!("Gói cây thư mục không hợp lệ: {e}"),
        )
    })?;
    let r_22l = r_22p
        .site_code
        .as_deref()
        .map(str::trim)
        .filter(|r_22i| !r_22i.is_empty())
        .map(str::to_string);
    crate::data_share::f_265(
        &app,
        r_22l.as_deref(),
        crate::data_share::SiteCodeSource::Lan,
    );
    let r_22n = apply_folder_tree(
        &app,
        &db,
        r_22p.phong,
        r_22p.ho_sos,
        r_22p.strip_accents,
        None,
        &r_22v.user_id,
    )
    .await?;
    tracing::info!(
        from = % args.addr, created = r_22n.created,
        "lan_pull: đã áp cây thư mục"
    );
    Ok(Thy {
        applied: true,
        version: 0,
        created: r_22n.created,
        offered_site_code: r_22l,
        peer_fingerprint: Some(lan_session::device_fingerprint(&r_22q.pubkey)),
        peer_device_id: Some(r_22q.device_id.clone()),
    })
}
fn sanitize(s: &str) -> String {
    s.chars()
        .map(|r_22x| {
            if r_22x.is_ascii_alphanumeric() {
                r_22x
            } else {
                '-'
            }
        })
        .collect()
}
