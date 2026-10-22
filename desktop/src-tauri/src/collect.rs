// Copyright (c) 2026 DigiText. All rights reserved.
use crate::auth::admin::f_2r7;
use crate::auth::client::{self, ChunkOutcome, UploadProbe};
use crate::auth::AuthState;
use crate::collect_progress as cprog;
use crate::data_share::{valid_blake3, Td1, C_KA, DEFAULT_SHARE_PORT, SERVICE_TYPE};
use crate::db::{self, Db, T2w, Tds};
use crate::error::AppError;
use crate::forward_progress::CollectControl;
use crate::forward_progress::{
    FileTicket, ForwardControl, ProgressGate, Reporter, SessionGuard, SessionPhase,
};
use crate::lan_session;
use base64::Engine;
use digitext_core::Perm;
use mdns_sd::{ServiceDaemon, ServiceEvent};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tauri::{AppHandle, Manager, State};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
const C_1T: usize = 4 * 1024 * 1024;
const C_BR: usize = 128 * 1024 * 1024;
#[derive(Default)]
pub struct SyncGuards {
    collecting: std::sync::atomic::AtomicBool,
    forwarding: std::sync::atomic::AtomicBool,
}
pub struct Tme<'a>(&'a std::sync::atomic::AtomicBool);
impl Drop for Tme<'_> {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}
impl SyncGuards {
    fn take(r_qr: &std::sync::atomic::AtomicBool) -> Option<Tme<'_>> {
        use std::sync::atomic::Ordering::SeqCst;
        r_qr.compare_exchange(false, true, SeqCst, SeqCst)
            .ok()
            .map(|_| Tme(r_qr))
    }
    pub fn try_collect(&self) -> Option<Tme<'_>> {
        Self::take(&self.collecting)
    }
    pub fn try_forward(&self) -> Option<Tme<'_>> {
        Self::take(&self.forwarding)
    }
}
pub const C_BI: &str = "lan_manual_peers";
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PeerSource {
    Mdns,
    Manual,
    Scan,
    Broadcast,
}
fn f_2gh(source: PeerSource) -> &'static str {
    match source {
        PeerSource::Mdns => "mDNS",
        PeerSource::Manual => "gõ tay",
        PeerSource::Scan => "quét dải mạng",
        PeerSource::Broadcast => "broadcast",
    }
}
#[derive(Debug, Clone)]
struct Peer {
    candidates: Vec<(String, u16)>,
    source: PeerSource,
    device_id: Option<String>,
}
#[derive(Debug, Default)]
struct Discovery {
    peers: Vec<Peer>,
    self_skipped: usize,
    no_ipv4: usize,
}
pub(crate) fn f_1iu(addr: &str) -> u8 {
    match addr.parse::<std::net::Ipv4Addr>() {
        Ok(ip) if ip.is_link_local() => 2,
        Ok(ip) if ip.is_private() => 0,
        Ok(_) => 1,
        Err(_) => 3,
    }
}
type SeenMap = std::collections::BTreeMap<String, std::collections::BTreeSet<(u8, String, u16)>>;
fn f_1sg(r_qv: &mut SeenMap, r_qt: &str, r_qu: Vec<String>, port: u16) {
    let key = f_159(r_qt).unwrap_or(r_qt).to_string();
    let r_qs = r_qv.entry(key).or_default();
    for a in r_qu {
        r_qs.insert((f_1iu(&a), a, port));
    }
}
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct ManualTargets {
    pub peers: Vec<(String, u16)>,
    pub nets: Vec<Tb9>,
}
pub(crate) fn f_1ze(raw: Option<&str>) -> Result<ManualTargets, String> {
    let mut out = ManualTargets::default();
    let Some(raw) = raw else { return Ok(out) };
    for r_qx in raw.split([',', ';', '\n', '\r', ' ', '\t']) {
        let t = r_qx.trim();
        if t.is_empty() {
            continue;
        }
        if let Some((net, prefix)) = t.split_once('/') {
            let ip = net.parse::<std::net::Ipv4Addr>().map_err(|_| {
                format!("dải \"{t}\" không hợp lệ (vd 192.168.5.0/24)")
            })?;
            let prefix = prefix
                .parse::<u8>()
                .ok()
                .filter(|p| *p <= 32)
                .ok_or_else(|| {
                    format!(
                        "dải \"{t}\" có độ dài tiền tố không hợp lệ (0–32)"
                    )
                })?;
            let n = Tb9 { ip, prefix };
            if !out.nets.contains(&n) {
                out.nets.push(n);
            }
            continue;
        }
        let (ip, port) = match t.rsplit_once(':') {
            Some((ip, port)) => (ip, Some(port)),
            None => (t, None),
        };
        if ip.parse::<std::net::Ipv4Addr>().is_err() {
            return Err(
                format!(
                    "mục \"{t}\" phải có dạng IP, IP:cổng hoặc dải CIDR (vd 192.168.1.18, 192.168.1.18:64040 hay 192.168.5.0/24)"
                ),
            );
        }
        let p = match port {
            None => DEFAULT_SHARE_PORT,
            Some(s) => match s.parse::<u16>() {
                Ok(p) if p > 0 => p,
                _ => {
                    return Err(format!(
                        "cổng \"{s}\" không hợp lệ (1–65535)"
                    ));
                }
            },
        };
        if !out.peers.iter().any(|(a, r_qw)| a == ip && *r_qw == p) {
            out.peers.push((ip.to_string(), p));
        }
    }
    Ok(out)
}
fn f_qo(r_qy: &mut Discovery, extra: Vec<(String, u16)>, source: PeerSource) -> usize {
    let mut added = 0;
    for (addr, port) in extra {
        let dup = r_qy.peers.iter().any(|r_qz| {
            r_qz.candidates
                .iter()
                .any(|(a, r_r0)| *a == addr && *r_r0 == port)
        });
        if dup {
            continue;
        }
        r_qy.peers.push(Peer {
            candidates: vec![(addr, port)],
            source,
            device_id: None,
        });
        added += 1;
    }
    added
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Tb9 {
    pub ip: std::net::Ipv4Addr,
    pub prefix: u8,
}
const C_BP: u32 = 1024;
const C_C0: usize = 2048;
const C_JF: usize = 96;
pub(crate) fn f_1p4() -> Vec<Tb9> {
    if_addrs::get_if_addrs()
        .map(|r_r1| {
            r_r1.into_iter()
                .filter_map(|i| match i.addr {
                    if_addrs::IfAddr::V4(r_r2) => Some(Tb9 {
                        ip: r_r2.ip,
                        prefix: r_r2.prefixlen,
                    }),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Tkq {
    pub targets: Vec<std::net::Ipv4Addr>,
    pub ranges: Vec<String>,
    pub clamped: bool,
}
fn f_1u0(ip: std::net::Ipv4Addr, prefix: u8) -> (u32, u32) {
    let r_r4: u32 = if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix as u32)
    };
    let r_r5 = u32::from(ip) & r_r4;
    let r_r3 = (!r_r4).saturating_sub(1);
    (r_r5, r_r3)
}
pub(crate) fn f_2c9(r_r8: &[Tb9]) -> Tkq {
    let r_r6: std::collections::HashSet<std::net::Ipv4Addr> = r_r8.iter().map(|n| n.ip).collect();
    let mut targets = std::collections::BTreeSet::new();
    let mut ranges = std::collections::BTreeSet::new();
    let mut clamped = false;
    for net in r_r8 {
        if net.ip.is_loopback() || net.ip.is_link_local() || net.ip.is_unspecified() {
            continue;
        }
        if net.prefix >= 31 || net.prefix > 32 {
            continue;
        }
        let (_, hosts) = f_1u0(net.ip, net.prefix);
        let prefix = if hosts > C_BP {
            clamped = true;
            24
        } else {
            net.prefix
        };
        let (r_r7, hosts) = f_1u0(net.ip, prefix);
        ranges.insert(format!("{}/{prefix}", std::net::Ipv4Addr::from(r_r7)));
        for h in 1..=hosts {
            let t = std::net::Ipv4Addr::from(r_r7 + h);
            if !r_r6.contains(&t) {
                targets.insert(t);
            }
        }
    }
    let mut targets: Vec<_> = targets.into_iter().collect();
    if targets.len() > C_C0 {
        clamped = true;
        targets.truncate(C_C0);
    }
    Tkq {
        targets,
        ranges: ranges.into_iter().collect(),
        clamped,
    }
}
async fn f_23z(ip: std::net::Ipv4Addr, port: u16, r_r9: Duration) -> bool {
    matches!(
        tokio::time::timeout(r_r9, tokio::net::TcpStream::connect((ip, port))).await,
        Ok(Ok(_))
    )
}
fn f_2c6(r_rb: usize) -> Vec<u16> {
    let r_ra = &crate::data_share::C_K2;
    if r_rb <= 512 {
        r_ra.to_vec()
    } else {
        r_ra[..1].to_vec()
    }
}
async fn f_2c8(
    targets: Vec<std::net::Ipv4Addr>,
    ports: Vec<u16>,
    r_rh: Duration,
) -> Vec<(std::net::Ipv4Addr, u16)> {
    let mut r_re = Vec::new();
    let ports = std::sync::Arc::new(ports);
    for r_rc in targets.chunks(C_JF) {
        let r_rg: Vec<_> = r_rc
            .iter()
            .map(|&ip| {
                let ports = ports.clone();
                tauri::async_runtime::spawn(async move {
                    for &port in ports.iter() {
                        if f_23z(ip, port, r_rh).await {
                            return Some((ip, port));
                        }
                    }
                    None
                })
            })
            .collect();
        for r_rf in r_rg {
            if let Ok(Some(r_rd)) = r_rf.await {
                r_re.push(r_rd);
            }
        }
    }
    r_re
}
pub(crate) fn f_1ys(r_ri: &crate::data_share::T4u, r_rj: &[u8]) -> Option<(u16, Option<String>)> {
    let r_rl = std::str::from_utf8(r_rj).ok()?;
    let r_rk = r_rl.strip_prefix(r_ri.reply)?.trim();
    let (port, extra) = match r_rk.split_once(' ') {
        Some((p, x)) => (p, Some(x.trim().to_string()).filter(|x| !x.is_empty())),
        None => (r_rk, None),
    };
    let port = port.parse::<u16>().ok().filter(|p| *p > 0)?;
    Some((port, extra))
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct T1c {
    pub ip: std::net::Ipv4Addr,
    pub port: u16,
    pub extra: Option<String>,
}
pub(crate) async fn f_v5(
    r_rn: &'static crate::data_share::T4u,
    nets: &[Tb9],
    r_s2: Duration,
) -> Vec<T1c> {
    let r_ry = r_rn.ports;
    let r_rw: std::collections::HashSet<std::net::Ipv4Addr> = nets.iter().map(|n| n.ip).collect();
    let mut r_rq: std::collections::BTreeMap<std::net::Ipv4Addr, T1c> = Default::default();
    let mut r_s1 = Vec::new();
    for net in nets {
        if net.ip.is_loopback() || net.ip.is_unspecified() || net.prefix > 32 {
            continue;
        }
        let Ok(sock) = tokio::net::UdpSocket::bind((net.ip, 0)).await else {
            continue;
        };
        if sock.set_broadcast(true).is_err() {
            continue;
        }
        let (r_rx, r_rt) = f_1u0(net.ip, net.prefix);
        let mut r_rp = vec![std::net::Ipv4Addr::BROADCAST];
        if net.prefix < 31 {
            r_rp.push(std::net::Ipv4Addr::from(r_rx + r_rt + 1));
        }
        let r_s0 = crate::data_share::f_15p(r_rn);
        let mut sent = false;
        for d in r_rp {
            for &port in r_ry {
                if sock.send_to(r_s0.as_slice(), (d, port)).await.is_ok() {
                    sent = true;
                }
            }
        }
        if sent {
            r_s1.push(sock);
        }
    }
    if r_s1.is_empty() {
        return Vec::new();
    }
    let r_rv: Vec<_> = r_s1
        .into_iter()
        .map(|sock| {
            tauri::async_runtime::spawn(async move {
                let mut out: Vec<T1c> = Vec::new();
                let mut r_rm = [0u8; 512];
                let r_ro = tokio::time::Instant::now() + r_s2;
                loop {
                    let r_rz = r_ro.saturating_duration_since(tokio::time::Instant::now());
                    if r_rz.is_zero() {
                        break;
                    }
                    match tokio::time::timeout(r_rz, sock.recv_from(&mut r_rm)).await {
                        Ok(Ok((n, r_rr))) => {
                            if let std::net::IpAddr::V4(ip) = r_rr.ip() {
                                if let Some((port, extra)) = f_1ys(r_rn, &r_rm[..n]) {
                                    out.push(T1c { ip, port, extra });
                                }
                            }
                        }
                        Ok(Err(_)) => continue,
                        Err(_) => break,
                    }
                }
                out
            })
        })
        .collect();
    for r_ru in r_rv {
        for r_rs in r_ru.await.unwrap_or_default() {
            if !r_rw.contains(&r_rs.ip) {
                r_rq.insert(r_rs.ip, r_rs);
            }
        }
    }
    r_rq.into_values().collect()
}
fn discover(r_se: Duration, r_s8: Option<String>) -> Result<Discovery, AppError> {
    let r_s4 = ServiceDaemon::new()
        .map_err(|e| AppError::internal(format!("mDNS khởi tạo lỗi: {e}")))?;
    let r_sb = r_s4
        .browse(SERVICE_TYPE)
        .map_err(|e| AppError::internal(format!("mDNS quét lỗi: {e}")))?;
    let r_s5 = std::time::Instant::now() + r_se;
    let mut r_sc = SeenMap::new();
    let mut self_skipped = std::collections::BTreeSet::<String>::new();
    while let Some(r_sa) = r_s5.checked_duration_since(std::time::Instant::now()) {
        match r_sb.recv_timeout(r_sa) {
            Ok(ServiceEvent::ServiceResolved(r_s6)) => {
                if f_1jj(r_s6.get_fullname(), r_s8.as_deref()) {
                    self_skipped.insert(r_s6.get_fullname().to_string());
                    continue;
                }
                let r_s7: Vec<String> = r_s6
                    .get_addresses()
                    .iter()
                    .filter(|ip| ip.is_ipv4())
                    .map(|ip| ip.to_string())
                    .collect();
                f_1sg(&mut r_sc, r_s6.get_fullname(), r_s7, r_s6.get_port());
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
    if let Ok(r_sd) = r_s4.shutdown() {
        let _ = r_sd.recv_timeout(Duration::from_millis(500));
    }
    let mut out = Discovery {
        self_skipped: self_skipped.len(),
        ..Discovery::default()
    };
    for (device, r_s3) in r_sc {
        if r_s3.is_empty() {
            tracing::warn!(
                device = % device,
                "máy chia sẻ không quảng bá IPv4 nào — bỏ qua"
            );
            out.no_ipv4 += 1;
            continue;
        }
        tracing::info!(
            device = % device, candidates = ? r_s3,
            "thấy máy chia sẻ trong LAN"
        );
        out.peers.push(Peer {
            candidates: r_s3.into_iter().map(|(_, a, r_s9)| (a, r_s9)).collect(),
            source: PeerSource::Mdns,
            device_id: Some(device.clone()),
        });
    }
    Ok(out)
}
async fn hash_file(path: &Path) -> std::io::Result<String> {
    let mut f = tokio::fs::File::open(path).await?;
    let mut r_sg = blake3::Hasher::new();
    let mut r_sf = vec![0u8; 1 << 16];
    loop {
        let n = f.read(&mut r_sf).await?;
        if n == 0 {
            break;
        }
        r_sg.update(&r_sf[..n]);
    }
    Ok(r_sg.finalize().to_hex().to_string())
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T2v {
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectSummary {
    pub peers: usize,
    pub local: usize,
    pub collected: usize,
    pub skipped: usize,
    pub failed: usize,
    pub total: usize,
    pub scanned_lan: bool,
    pub mirrored: usize,
    pub peers_reachable: usize,
    pub manifest_failed: usize,
    pub advertised: usize,
    pub notes: Vec<String>,
    pub foreign: usize,
    pub project_gate_stale: bool,
    pub scanned_ranges: Vec<String>,
    pub scan_clamped: bool,
    pub orphans: usize,
    pub kho_truncated: bool,
    pub cancelled: usize,
}
const C_BS: usize = 5;
fn f_1jj(r_sj: &str, r_si: Option<&str>) -> bool {
    let Some(r_sh) = r_si else {
        return false;
    };
    if r_sh == r_sj {
        return true;
    }
    match (f_159(r_sh), f_159(r_sj)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}
fn f_159(r_sk: &str) -> Option<&str> {
    let r_sl = r_sk.strip_prefix("digitext-data-")?;
    let instance = r_sl.split('.').next()?;
    let (device, port) = instance.rsplit_once('-')?;
    port.parse::<u16>().ok()?;
    (!device.is_empty()).then_some(device)
}
const C_F7: &str = "owned_projects_cache";
#[derive(Debug, Clone)]
pub(crate) struct Tho {
    allowed: std::collections::HashSet<String>,
    pub(crate) stale: bool,
}
impl Tho {
    pub(crate) fn allows(&self, project_id: &str) -> bool {
        self.allowed.contains(project_id.trim())
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.allowed.is_empty()
    }
    pub(crate) fn sole(&self) -> Option<&str> {
        let mut r_sm = self.allowed.iter();
        match (r_sm.next(), r_sm.next()) {
            (Some(r_sn), None) => Some(r_sn.as_str()),
            _ => None,
        }
    }
}
pub(crate) fn f_1zm(r_so: &str) -> std::collections::HashSet<String> {
    serde_json::from_str::<Vec<String>>(r_so)
        .unwrap_or_default()
        .into_iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
async fn f_1wx(db: &State<'_, Db>, r_sp: &State<'_, AuthState>) -> Tho {
    match f_2r7(db, r_sp, Perm::Admin, |r_sq, r_sw| async move {
        client::f_1eo::<Vec<client::Tl4>>(&r_sq, &r_sw, "/projects").await
    })
    .await
    {
        Ok(projects) => {
            let allowed: std::collections::HashSet<String> =
                projects.into_iter().map(|r_su| r_su.id).collect();
            if let Ok(r_ss) = db.0.lock() {
                let json = serde_json::to_string(&allowed.iter().collect::<Vec<_>>())
                    .unwrap_or_else(|_| "[]".into());
                let _ = db::f_2ei(&r_ss, C_F7, Some(&json));
            }
            Tho {
                allowed,
                stale: false,
            }
        }
        Err(e) => {
            let r_sr =
                db.0.lock()
                    .ok()
                    .and_then(|c| db::f_1ew(&c, C_F7).ok().flatten())
                    .map(|r_sv| f_1zm(&r_sv))
                    .unwrap_or_default();
            if !r_sr.is_empty() {
                tracing::warn!(
                    error = % e, n = r_sr.len(),
                    "không lấy được danh sách dự án — dùng cache"
                );
                return Tho {
                    allowed: r_sr,
                    stale: true,
                };
            }
            let r_st: std::collections::HashSet<String> = r_sp
                .snapshot()
                .and_then(|s| s.project_id)
                .into_iter()
                .collect();
            tracing::warn!(
                error = % e, n = r_st.len(),
                "không lấy được danh sách dự án và chưa có cache — chỉ gom dự án của tài khoản admin"
            );
            Tho {
                allowed: r_st,
                stale: true,
            }
        }
    }
}
fn f_26c(r_sz: &mut CollectSummary, addr: &str, r_sy: &Result<Th1, String>) {
    match r_sy {
        Ok(r_sx) => {
            r_sz.peers_reachable += 1;
            r_sz.advertised += r_sx.files.len();
        }
        Err(e) => {
            r_sz.manifest_failed += 1;
            f_24v(r_sz, &format!("{addr}: {e}"));
        }
    }
}
fn f_24v(r_t1: &mut CollectSummary, r_t0: &str) {
    if r_t1.notes.len() < C_BS {
        r_t1.notes.push(r_t0.to_string());
    }
}
pub(crate) fn f_1t9(r_t2: &str, blake3: &str) -> String {
    let r_t3 = Path::new(r_t2);
    let stem = r_t3
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tai-lieu".into());
    let ext = r_t3
        .extension()
        .map(|s| format!(".{}", s.to_string_lossy()))
        .unwrap_or_default();
    let short: String = blake3.chars().take(8).collect();
    format!("{stem}.{short}{ext}")
}
fn f_14v(filename: &str) -> String {
    let r_t4 = Path::new(filename)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tai-lieu.pdf".into());
    crate::storage::sanitize(&r_t4, 120, false)
}
#[tauri::command]
pub fn lan_manual_peers_get(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<Option<String>, AppError> {
    auth.require(Perm::Admin)?;
    let r_t5 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(db::f_1ew(&r_t5, C_BI)?)
}
#[derive(Debug, Deserialize)]
pub struct Tld {
    pub peers: String,
}
#[tauri::command]
pub fn lan_manual_peers_set(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tld,
) -> Result<usize, AppError> {
    auth.require(Perm::Admin)?;
    let r_t7 = f_1ze(Some(&args.peers)).map_err(AppError::validation)?;
    let r_t8 = args.peers.trim();
    let r_t6 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    db::f_2ei(&r_t6, C_BI, if r_t8.is_empty() { None } else { Some(r_t8) })?;
    Ok(r_t7.peers.len() + r_t7.nets.len())
}
#[tauri::command]
pub async fn data_collect(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    guards: State<'_, SyncGuards>,
    control: State<'_, CollectControl>,
    args: T2v,
) -> Result<CollectSummary, AppError> {
    auth.require(Perm::Admin)?;
    let _lease = guards
        .try_collect()
        .ok_or_else(|| {
            AppError::new(
                "collect_busy",
                "Đang có một phiên thu thập chạy dở — chờ nó xong rồi bấm lại.",
            )
        })?;
    let session_id = control.begin();
    let rep = cprog::Reporter::new(app.clone(), session_id, cprog::Kind::Lan);
    rep.session(cprog::SessionPhase::Start, None);
    let r_te = SessionGuard::new(&rep, |s| control.end(s));
    rep.stage(
        cprog::Stage::Local,
        "Đang xem bản quét trên máy này…",
        0,
    );
    let root = crate::archive::paths::f_29d(&app, crate::archive::paths::f_112(&db).as_deref())?;
    let mut r_to = CollectSummary::default();
    f_2ia(&db, &root).await;
    let gate = f_1wx(&db, &auth).await;
    r_to.project_gate_stale = gate.stale;
    if gate.is_empty() {
        f_24v(
            &mut r_to,
            "Chưa xác định được dự án máy này quản lý (không kết nối được máy chủ và chưa có bản lưu) — dừng thu thập để không gom nhầm dữ liệu của admin khác. Hãy kết nối máy chủ một lần rồi thu thập lại.",
        );
        tracing::warn!("cổng lọc dự án rỗng — dừng thu thập");
        return Ok(f_1cz(r_te, r_to));
    }
    let r_tf = f_1i6(&app, &db, &gate, &mut r_to).await?;
    rep.stage(
        cprog::Stage::Kho,
        format!(
            "Đã nạp {} bản quét trên máy này · đang rà cây Kho…",
            r_to.local
        ),
        0,
    );
    f_1i7(&app, &db, &auth, &root, &gate, &r_tf, &mut r_to).await;
    let code = {
        let conn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        db::f_1ew(&conn, C_KA)?.unwrap_or_default()
    };
    if code.trim().is_empty() {
        tracing::info!(
            local = r_to.local,
            "chưa đặt mã site LAN — chỉ nạp bản scan cục bộ, bỏ qua quét LAN"
        );
        return Ok(f_1cz(r_te, r_to));
    }
    if crate::lan_session::f_2ft(&code) == crate::lan_session::CodeStrength::Weak {
        f_24v(
            &mut r_to,
            "⚠️ Mã site LAN hiện tại YẾU (dò được trong vài giây). Vẫn gom bình thường, nhưng hãy bấm “Sinh mã mạnh” rồi phát lại cây thư mục để các máy nhân viên nhận mã mới.",
        );
    }
    let r_tb = {
        let code = code.clone();
        tauri::async_runtime::spawn_blocking(move || crate::lan_session::code_key(&code))
            .await
            .map_err(|e| {
                AppError::internal(format!(
                    "luồng dẫn xuất khoá mã site lỗi: {e}"
                ))
            })?
            .map_err(|e| {
                AppError::internal(format!(
                    "dẫn xuất khoá mã site lỗi: {e}"
                ))
            })?
    };
    let r_tk = app
        .state::<crate::data_share::T4j>()
        .0
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|p| p.fullname().to_string()));
    let r_tq = Duration::from_millis(args.timeout_ms.unwrap_or(4000).clamp(500, 8000));
    let r_ti = {
        let conn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        db::f_1ew(&conn, C_BI)?
    };
    let r_th = match f_1ze(r_ti.as_deref()) {
        Ok(r_tg) => r_tg,
        Err(e) => {
            f_24v(
                &mut r_to,
                &format!(
                    "Danh sách máy user cố định bị lỗi ({e}) — đã bỏ qua; sửa lại trong ô cài đặt."
                ),
            );
            ManualTargets::default()
        }
    };
    rep.stage(
        cprog::Stage::Discover,
        "Đang tìm máy nhân viên trong mạng (mDNS · broadcast · quét dải)…",
        0,
    );
    let nets = f_1p4();
    let r_ta = {
        let nets = nets.clone();
        tauri::async_runtime::spawn(async move {
            f_v5(&crate::data_share::C_3M, &nets, Duration::from_millis(1200)).await
        })
    };
    let plan = {
        let mut r_t9 = nets.clone();
        r_t9.extend(r_th.nets.iter().copied());
        f_2c9(&r_t9)
    };
    r_to.scanned_ranges = plan.ranges.clone();
    let scan_clamped = plan.clamped;
    tracing::info!(
        ranges = ? plan.ranges, targets = plan.targets.len(),
        "kế hoạch quét dải mạng"
    );
    let r_tn = {
        let r_tm = f_2c6(plan.targets.len());
        tauri::async_runtime::spawn(f_2c8(plan.targets, r_tm, Duration::from_millis(400)))
    };
    let mut r_tc = tauri::async_runtime::spawn_blocking(move || discover(r_tq, r_tk))
        .await
        .map_err(|e| AppError::internal(format!("luồng quét LAN lỗi: {e}")))??;
    let added = f_qo(&mut r_tc, r_th.peers, PeerSource::Manual);
    if added > 0 {
        tracing::info!(
            r_th = added,
            "thêm máy user chỉ định tay vào danh sách gom"
        );
    }
    if let Ok(hits) = r_ta.await {
        let found = f_qo(
            &mut r_tc,
            hits.into_iter()
                .map(|h| (h.ip.to_string(), h.port))
                .collect(),
            PeerSource::Broadcast,
        );
        if found > 0 {
            tracing::info!(
                broadcast = found,
                "máy user tự khai qua UDP broadcast"
            );
        }
    }
    if let Ok(hits) = r_tn.await {
        let found = f_qo(
            &mut r_tc,
            hits.into_iter()
                .map(|(ip, p)| (ip.to_string(), p))
                .collect(),
            PeerSource::Scan,
        );
        if found > 0 {
            tracing::info!(
                scan = found,
                "quét dải mạng thấy thêm máy mở cổng chung"
            );
        }
    }
    r_to.peers = r_tc.peers.len();
    r_to.scanned_lan = true;
    r_to.scan_clamped = scan_clamped;
    let (self_skipped, no_ipv4) = (r_tc.self_skipped, r_tc.no_ipv4);
    if no_ipv4 > 0 {
        f_24v(
            &mut r_to,
            &format!(
                "{no_ipv4} máy chỉ quảng bá địa chỉ IPv6 — chưa kéo được (cần IPv4 trong LAN)"
            ),
        );
    }
    if r_tc.peers.is_empty() && self_skipped > 0 {
        f_24v(
            &mut r_to,
            &format!(
                "đã loại {self_skipped} dịch vụ trùng định danh (device_id) với máy này — bình thường nếu là bản ghi cũ của chính máy; nhưng nếu máy nhân viên được cài bằng cách SAO CHÉP dữ liệu từ máy này thì nó đang bị loại nhầm: đăng nhập lại máy nhân viên khi kết nối được máy chủ để cấp định danh riêng"
            ),
        );
    }
    let http = reqwest::Client::builder()
        .no_proxy()
        .connect_timeout(Duration::from_secs(4))
        .build()
        .unwrap_or_else(|_| reqwest::Client::new());
    rep.stage(
        cprog::Stage::Discover,
        if r_to.peers == 0 {
            "Không thấy máy nhân viên nào đang chia sẻ trong mạng."
                .to_string()
        } else {
            format!(
                "Thấy {} máy nhân viên — đang đọc danh sách tài liệu…",
                r_to.peers
            )
        },
        r_to.peers,
    );
    for peer in &r_tc.peers {
        if control.session_cancelled(session_id) {
            break;
        }
        let label = peer
            .candidates
            .first()
            .map(|(a, _)| a.as_str())
            .unwrap_or("?");
        rep.stage(
            cprog::Stage::Peer,
            format!("Đang đọc danh sách của máy {label}…"),
            r_to.peers,
        );
        let r_tj = f_1c6(&http, peer, &r_tb, &db).await;
        f_26c(&mut r_to, label, &r_tj);
        let Th1 {
            addr,
            port,
            session,
            files,
        } = match r_tj {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(
                    addr = % label, error = % e,
                    "lấy manifest máy user lỗi"
                );
                continue;
            }
        };
        for f in files {
            r_to.total += 1;
            let ticket = cprog::FileTicket::new(
                session_id,
                f.blake3.clone(),
                f.filename.clone(),
                f.source_username.clone(),
                addr.clone(),
                Some(f.ho_so_ref.clone()),
                u64::try_from(f.size).unwrap_or(0),
            );
            if control.session_cancelled(session_id) {
                r_to.cancelled += 1;
                continue;
            }
            if control.file_cancelled(session_id, f.blake3.as_str()) {
                rep.file(&ticket.cancelled(0));
                r_to.cancelled += 1;
                continue;
            }
            let Some(r_tl) = f.project_id.clone() else {
                tracing::warn!(
                    file = % f.filename,
                    "bỏ gom: người quét chưa gắn dự án"
                );
                rep.file(
                    &ticket
                        .error(
                            0,
                            "Người quét chưa được gắn dự án — gắn trên máy chủ rồi gom lại.",
                        ),
                );
                r_to.failed += 1;
                continue;
            };
            if !gate.allows(&r_tl) {
                tracing::debug!(
                    file = % f.filename, project = % r_tl,
                    "bỏ gom: dự án không thuộc máy admin này"
                );
                rep.file(
                    &ticket
                        .skipped(
                            "Thuộc dự án máy này không quản — admin khác sẽ gom.",
                        ),
                );
                r_to.foreign += 1;
                continue;
            }
            if !valid_blake3(&f.blake3) || f.size <= 0 {
                tracing::warn!(
                    file = % f.filename,
                    "bỏ gom: blake3 không hợp lệ hoặc file rỗng"
                );
                rep.file(
                    &ticket
                        .error(
                            0,
                            "Máy nhân viên khai mã nội dung/kích thước không hợp lệ.",
                        ),
                );
                r_to.failed += 1;
                continue;
            }
            let r_td = {
                let conn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
                db::f_10g(&conn, &r_tl, &f.blake3)?
            };
            if r_td {
                rep.file(
                    &ticket.skipped("Máy này đã gom bản này từ trước."),
                );
                r_to.skipped += 1;
                continue;
            }
            let plan = match f_233(&db, &root, &f).await {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(
                        file = % f.filename, error = % e,
                        "không dựng được chỗ trong Kho"
                    );
                    rep.file(&ticket.error(0, e.clone()));
                    r_to.failed += 1;
                    f_24v(&mut r_to, &format!("“{}”: {e}", f.filename));
                    continue;
                }
            };
            if !plan.up_to_date {
                rep.file(&ticket.downloading(0));
                let mut r_tp = DownloadWatch::live(
                    &rep,
                    &ticket,
                    &control,
                    u64::try_from(f.size).unwrap_or(0),
                );
                match f_176(
                    &http, &addr, port, &f, &session, &r_tb, &plan.dest, &mut r_tp,
                )
                .await
                {
                    Ok(Downloaded::Complete) => {}
                    Ok(Downloaded::Cancelled { received }) => {
                        rep.file(&ticket.cancelled(received));
                        r_to.cancelled += 1;
                        continue;
                    }
                    Err(e) => {
                        tracing::warn!(
                            file = % f.filename, error = % e,
                            "kéo file từ user lỗi"
                        );
                        rep.file(&ticket.error(r_tp.received(), e));
                        r_to.failed += 1;
                        continue;
                    }
                }
            }
            {
                let conn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
                f_26b(&conn, &f, &r_tl, &plan.dest)?;
            }
            r_to.collected += 1;
            rep.file(&ticket.done());
            if plan.write_catalog {
                match write_catalog(&db, &f, &plan) {
                    Ok(()) => r_to.mirrored += 1,
                    Err(e) => {
                        tracing::warn!(
                            file = % f.filename, error = % e,
                            "ghi mục lục lỗi"
                        )
                    }
                }
            }
        }
    }
    tracing::info!(
        peers = r_to.peers,
        self_skipped,
        no_ipv4,
        peers_reachable = r_to.peers_reachable,
        manifest_failed = r_to.manifest_failed,
        advertised = r_to.advertised,
        local = r_to.local,
        collected = r_to.collected,
        mirrored = r_to.mirrored,
        skipped = r_to.skipped,
        failed = r_to.failed,
        cancelled = r_to.cancelled,
        "data_collect"
    );
    Ok(f_1cz(r_te, r_to))
}
fn f_1cz(r_tr: SessionGuard<'_, cprog::Reporter>, r_ts: CollectSummary) -> CollectSummary {
    r_tr.finish(
        r_ts.total,
        serde_json::to_value(&r_ts).unwrap_or(serde_json::Value::Null),
    );
    r_ts
}
#[tauri::command]
pub fn data_collect_cancel(
    auth: State<'_, AuthState>,
    control: State<'_, CollectControl>,
    args: T2g,
) -> Result<bool, AppError> {
    auth.require(Perm::Admin)?;
    let r_tt = control.cancel(args.session_id, args.key.clone());
    tracing::info!(
        session = args.session_id, file = ? args.key, r_tt, "data_collect_cancel"
    );
    Ok(r_tt)
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T2g {
    pub session_id: u64,
    #[serde(default)]
    pub key: Option<String>,
}
async fn f_2ia(db: &Db, root: &Path) -> usize {
    let inbox = root.join("_inbox");
    let Ok(mut r_tx) = tokio::fs::read_dir(&inbox).await else {
        return 0;
    };
    let mut r_ty = 0usize;
    while let Ok(Some(r_tv)) = r_tx.next_entry().await {
        let path = r_tv.path();
        if path.extension().and_then(|e| e.to_str()) != Some("pdf") {
            continue;
        }
        if !path
            .file_stem()
            .and_then(|s| s.to_str())
            .is_some_and(valid_blake3)
        {
            continue;
        }
        let r_tw = match db.0.lock() {
            Ok(r_tu) => db::f_10h(&r_tu, &path.to_string_lossy()).unwrap_or(true),
            Err(_) => true,
        };
        if !r_tw && tokio::fs::remove_file(&path).await.is_ok() {
            r_ty += 1;
        }
    }
    let _ = tokio::fs::remove_dir(&inbox).await;
    if r_ty > 0 {
        tracing::info!(
            r_ty,
            "đã dọn kho tạm _inbox của bản cũ"
        );
    }
    r_ty
}
#[derive(Debug)]
struct Tm {
    folder: PathBuf,
    dest: PathBuf,
    up_to_date: bool,
    write_catalog: bool,
}
async fn f_233(db: &Db, root: &Path, f: &Td1) -> Result<Tm, String> {
    let phong_ma = f
        .phong_ma
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(
            "thiếu mã phông — máy nhân viên cần cập nhật hoặc gán phông cho hồ sơ",
        )?;
    let r_u3 = f
        .phong_ten
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(
            "thiếu tên phông — máy nhân viên cần cập nhật hoặc gán phông cho hồ sơ",
        )?;
    let (hop_so, ho_so_so) = f
        .ho_so_ref
        .split_once('/')
        .unwrap_or((f.ho_so_ref.as_str(), ""));
    let folder = crate::storage::f_1jt(root, &[r_u3, hop_so, ho_so_so], false);
    tokio::fs::create_dir_all(&folder)
        .await
        .map_err(|e| format!("tạo thư mục Kho lỗi: {e}"))?;
    let r_u2 = f_14v(&f.filename);
    let mut dest = folder.join(&r_u2);
    let r_tz = {
        let conn = db.0.lock().map_err(|_| "DB bận".to_string())?;
        db::f_1hb(&conn, Some(phong_ma), hop_so, ho_so_so).map_err(|e| e.to_string())?
    };
    let mut up_to_date = matches!(
        hash_file(& dest). await, Ok(h) if h.eq_ignore_ascii_case(& f.blake3)
    );
    if !up_to_date && dest.exists() {
        let r_u1 = hash_file(&dest).await.ok();
        let r_u5 = !r_tz
            && match r_u1.as_deref() {
                Some(h) => {
                    let conn = db.0.lock().map_err(|_| "DB bận".to_string())?;
                    db::f_10m(&conn, h)
                        .map_err(|e| e.to_string())?
                        .is_some_and(|r_u4| r_u4 == f.source_user_id)
                }
                None => false,
            };
        if !r_u5 {
            let r_u0 = f_1t9(&r_u2, &f.blake3);
            tracing::warn!(
                file = % f.filename, r_u0 = % r_u0,
                "Kho đã có file khác nội dung cùng tên — ghi sang tên riêng, không đè"
            );
            dest = folder.join(&r_u0);
            up_to_date = matches!(
                hash_file(& dest). await, Ok(h) if h.eq_ignore_ascii_case(& f.blake3)
            );
        }
    }
    Ok(Tm {
        folder,
        dest,
        up_to_date,
        write_catalog: !r_tz,
    })
}
fn write_catalog(db: &Db, f: &Td1, r_ub: &Tm) -> Result<(), String> {
    let phong_ma = f.phong_ma.as_deref().unwrap_or_default();
    let r_ua = f.phong_ten.as_deref().unwrap_or_default();
    let (hop_so, ho_so_so) = f
        .ho_so_ref
        .split_once('/')
        .unwrap_or((f.ho_so_ref.as_str(), ""));
    let r_u7 = r_ub.folder.to_string_lossy().into_owned();
    let r_u6 = r_ub.dest.to_string_lossy().into_owned();
    let r_u9 = db::T96 {
        phong_id: Some(phong_ma.to_string()),
        hop_so: hop_so.to_string(),
        ho_so_so: ho_so_so.to_string(),
        tieu_de: f.tieu_de.clone().unwrap_or_default(),
        thoi_gian: f.thoi_gian.clone(),
        so_to: f.so_to.clone(),
        thbq: None,
        ghi_chu: None,
        nhom: None,
        nam: f.nam.clone(),
        ngay_bat_dau: None,
        ngay_ket_thuc: None,
    };
    let mut r_u8 = db.0.lock().map_err(|_| "DB bận".to_string())?;
    let r_uc = r_u8.transaction().map_err(|e| e.to_string())?;
    db::f_2o6(
        &r_uc,
        phong_ma,
        r_ua,
        None,
        f.phong_so.as_deref(),
        f.muc_luc_so.as_deref(),
    )
    .map_err(|e| e.to_string())?;
    let id = db::f_2nz(&r_uc, &r_u9).map_err(|e| e.to_string())?;
    db::f_2e5(&r_uc, id, &r_u7, &r_u6, f.size, f.page_count.unwrap_or(0))
        .map_err(|e| e.to_string())?;
    r_uc.commit().map_err(|e| e.to_string())?;
    Ok(())
}
fn f_26b(r_ud: &rusqlite::Connection, f: &Td1, r_ue: &str, path: &Path) -> rusqlite::Result<bool> {
    db::f_1id(
        r_ud,
        Tds {
            source_user_id: &f.source_user_id,
            source_username: f.source_username.as_deref(),
            project_id: r_ue,
            phong_ma: f.phong_ma.as_deref(),
            ho_so_ref: Some(&f.ho_so_ref),
            filename: &f.filename,
            page_count: f.page_count,
            size_bytes: f.size,
            blake3: &f.blake3,
            storage_path: &path.to_string_lossy(),
        },
    )
}
async fn f_1i6(
    app: &AppHandle,
    db: &State<'_, Db>,
    gate: &Tho,
    r_uk: &mut CollectSummary,
) -> Result<Vec<String>, AppError> {
    let r_ug = {
        let app = app.clone();
        tauri::async_runtime::spawn_blocking(move || crate::data_share::f_1p5(&app))
            .await
            .map_err(|e| {
                AppError::internal(format!(
                    "luồng liệt kê cục bộ lỗi: {e}"
                ))
            })?
    };
    let files = match r_ug {
        Ok(s) => s.files,
        Err(e) => {
            tracing::warn!(
                error = % e, "liệt kê bản scan cục bộ lỗi"
            );
            return Ok(Vec::new());
        }
    };
    let mut r_ui: Vec<String> = Vec::with_capacity(files.len());
    for (f, r_uj) in files {
        r_uk.total += 1;
        r_ui.push(r_uj.clone());
        let Some(r_uh) = f.project_id.clone() else {
            tracing::warn!(
                file = % f.filename,
                "bỏ nạp cục bộ: hồ sơ chưa gắn dự án"
            );
            r_uk.failed += 1;
            continue;
        };
        if !valid_blake3(&f.blake3) || f.size <= 0 {
            tracing::warn!(
                file = % f.filename,
                "bỏ nạp cục bộ: blake3 không hợp lệ hoặc file rỗng"
            );
            r_uk.failed += 1;
            continue;
        }
        if !gate.allows(&r_uh) {
            tracing::debug!(
                file = % f.filename, project = % r_uh,
                "bỏ nạp cục bộ: ngoài dự án máy admin quản"
            );
            r_uk.foreign += 1;
            continue;
        }
        let r_uf = {
            let conn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
            db::f_10g(&conn, &r_uh, &f.blake3)?
        };
        if r_uf {
            r_uk.skipped += 1;
            continue;
        }
        {
            let conn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
            f_26b(&conn, &f, &r_uh, Path::new(&r_uj))?;
        }
        r_uk.collected += 1;
        r_uk.local += 1;
    }
    Ok(r_ui)
}
/// Trần số PDF lấy từ MỘT lần chọn thư mục (chỉ để chặn chọn nhầm cả ổ). 20.000 cũ quá thấp:
/// 25/9/2026 chọn thư mục 30.729 PDF bị cắt còn 20.000 mà người dùng không để ý cảnh báo.
const C_BQ: usize = 500_000;
fn f_2fv(r_ul: &str) -> bool {
    crate::storage::f_1j2(r_ul)
}
fn f_2qo(root: &Path, max: usize) -> (Vec<PathBuf>, bool) {
    let mut out = Vec::new();
    let mut r_ur = vec![root.to_path_buf()];
    while let Some(dir) = r_ur.pop() {
        let Ok(r_uq) = std::fs::read_dir(&dir) else {
            continue;
        };
        for r_um in r_uq.flatten() {
            let path = r_um.path();
            let Ok(r_un) = r_um.file_type() else { continue };
            let r_up = r_um.file_name().to_string_lossy().into_owned();
            if r_un.is_dir() {
                if !f_2fv(&r_up) {
                    r_ur.push(path);
                }
                continue;
            }
            if !r_un.is_file() {
                continue;
            }
            let r_uo = path
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
            if r_uo {
                if out.len() >= max {
                    return (out, true);
                }
                out.push(path);
            }
        }
    }
    (out, false)
}
fn f_1km(root: &Path, file: &Path) -> Option<String> {
    let r_us = file.parent()?.strip_prefix(root).ok()?;
    let r_ut: Vec<String> = r_us
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    // Cây KHÔNG theo danh mục: <dự án (có dấu PROJECT_MARKER)>/<phông>/<hộp>/<hồ sơ> → mã hồ sơ
    // là "hộp/hồ sơ" (tầng 3/4); phông lấy từ tầng 2 của relPath. Cây theo danh mục:
    // <phông>/<hộp>/<hồ sơ> → "hộp/hồ sơ" (tầng 2/3) như cũ.
    let project_dir = r_ut
        .first()
        .is_some_and(|d| root.join(d).join(PROJECT_MARKER).exists());
    if project_dir && r_ut.len() >= 3 {
        return Some(format!("{}/{}", r_ut[2], r_ut.get(3).map(String::as_str).unwrap_or("")));
    }
    match r_ut.len() {
        0 | 1 => None,
        2 => Some(r_ut[1].clone()),
        _ => Some(format!("{}/{}", r_ut[1], r_ut[2])),
    }
}
fn f_1wl(r_uv: Option<&str>, gate: &Tho) -> Option<String> {
    match r_uv.map(str::trim).filter(|s| !s.is_empty()) {
        Some(r_uu) if gate.allows(r_uu) => Some(r_uu.to_string()),
        _ => gate.sole().map(str::to_string),
    }
}
fn f_2n7(files: Vec<PathBuf>, r_uw: &std::collections::HashSet<String>) -> Vec<PathBuf> {
    files
        .into_iter()
        .filter(|r_ux| !r_uw.contains(r_ux.to_string_lossy().as_ref()))
        .collect()
}
struct Tfl {
    path: PathBuf,
    filename: String,
    blake3: String,
    page_count: Option<i64>,
    size: i64,
}
async fn f_1i7(
    app: &AppHandle,
    db: &State<'_, Db>,
    r_uy: &State<'_, AuthState>,
    root: &Path,
    gate: &Tho,
    r_v0: &[String],
    r_v4: &mut CollectSummary,
) {
    let Some(session) = r_uy.snapshot() else {
        return;
    };
    let Some(r_v2) = f_1wl(session.project_id.as_deref(), gate) else {
        f_24v(
            r_v4,
            "Có thể còn tài liệu rời trong Kho (không thuộc hồ sơ nào) chưa gom được: máy này quản nhiều dự án mà tài khoản đang đăng nhập không gắn dự án cụ thể. Dùng nút “③ Thêm tài liệu từ ổ cứng” để chọn dự án cho chúng.",
        );
        return;
    };
    let mut r_v1: std::collections::HashSet<String> = match db.0.lock() {
        Ok(conn) => db::f_10i(&conn).unwrap_or_default(),
        Err(_) => return,
    };
    r_v1.extend(r_v0.iter().cloned());
    let scan = {
        let (app, root) = (app.clone(), root.to_path_buf());
        tauri::async_runtime::spawn_blocking(move || {
            let db = app.state::<Db>();
            let (files, truncated) = f_2qo(&root, C_BQ);
            let mut out = Vec::new();
            for path in f_2n7(files, &r_v1) {
                let Some((blake3, page_count, size)) = crate::data_share::f_1g2(&db, &path) else {
                    continue;
                };
                let Some(filename) = path.file_name().map(|s| s.to_string_lossy().into_owned())
                else {
                    continue;
                };
                out.push(Tfl {
                    path,
                    filename,
                    blake3,
                    page_count,
                    size,
                });
            }
            (out, truncated)
        })
        .await
    };
    let (r_uz, truncated) = match scan {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!(error = % e, "luồng quét Kho lỗi");
            return;
        }
    };
    r_v4.kho_truncated = truncated;
    for f in r_uz {
        r_v4.total += 1;
        if f.size <= 0 {
            r_v4.failed += 1;
            continue;
        }
        let ho_so_ref = f_1km(root, &f.path);
        let Ok(conn) = db.0.lock() else { return };
        match db::f_10m(&conn, &f.blake3) {
            Ok(Some(_)) => {
                r_v4.skipped += 1;
                continue;
            }
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(error = % e, "tra trùng nội dung lỗi");
                r_v4.failed += 1;
                continue;
            }
        }
        let r_v3 = db::f_1id(
            &conn,
            db::Tds {
                source_user_id: &session.user_id,
                source_username: Some(&session.username),
                project_id: &r_v2,
                phong_ma: None,
                ho_so_ref: ho_so_ref.as_deref(),
                filename: &f.filename,
                page_count: f.page_count,
                size_bytes: f.size,
                blake3: &f.blake3,
                storage_path: &f.path.to_string_lossy(),
            },
        );
        match r_v3 {
            Ok(true) => {
                r_v4.collected += 1;
                r_v4.orphans += 1;
            }
            Ok(false) => r_v4.skipped += 1,
            Err(e) => {
                tracing::warn!(
                    error = % e,
                    "ghi tài liệu rời vào kho gom lỗi"
                );
                r_v4.failed += 1;
            }
        }
    }
    if r_v4.orphans > 0 || truncated {
        tracing::info!(
            orphans = r_v4.orphans,
            truncated,
            "nhặt tài liệu rời trong Kho"
        );
    }
    if truncated {
        f_24v(
            r_v4,
            "Kho có rất nhiều tệp — lượt này mới quét một phần. Bấm “① Thu thập” thêm vài lần nữa để nhặt hết tài liệu rời.",
        );
    }
}
fn f_2rk(r_vi: u64, r_vh: usize, max: u64) -> bool {
    r_vi.saturating_add(r_vh as u64) > max
}
pub(crate) async fn f_25n(mut r_vl: reqwest::Response, max: usize) -> Result<Vec<u8>, String> {
    let mut r_vj: Vec<u8> = Vec::new();
    while let Some(r_vk) = r_vl.chunk().await.map_err(|e| e.to_string())? {
        if f_2rk(r_vj.len() as u64, r_vk.len(), max as u64) {
            return Err(format!(
                "phản hồi vượt {max} byte (nghi peer bất thường)"
            ));
        }
        r_vj.extend_from_slice(&r_vk);
    }
    Ok(r_vj)
}
async fn f_25s<T: serde::de::DeserializeOwned>(
    r_vn: reqwest::Response,
    max: usize,
) -> Result<T, String> {
    let r_vm = f_25n(r_vn, max).await?;
    serde_json::from_slice(&r_vm).map_err(|e| e.to_string())
}
#[derive(Debug)]
pub(crate) enum SessionVerdict {
    Ok(Box<lan_session::Session>),
    Mismatch,
    BadSig,
    Downgrade,
    OldProtocol,
    Unreachable(String),
}
impl SessionVerdict {
    fn note(&self, addr: &str, port: u16) -> Option<String> {
        Some(match self {
            Self::Ok(_) => return None,
            Self::Mismatch => {
                format!(
                        "{addr}:{port}: khoá danh tính KHÁC khoá đã ghim (nghi mạo danh) — BỎ"
                    )
            }
            Self::BadSig => {
                format!(
                    "{addr}:{port}: chữ ký bắt tay không hợp lệ — BỎ"
                )
            }
            Self::Downgrade => {
                format!(
                        "{addr}:{port}: máy từng có danh tính nay không nói được giao thức phiên (nghi hạ cấp để ép về kênh không mã hoá) — BỎ"
                    )
            }
            Self::OldProtocol => {
                format!(
                        "{addr}:{port}: peer đang chạy bản cũ, cần cập nhật (kênh LAN không mã hoá đã bị gỡ — KHÔNG có chế độ tương thích ngược)"
                    )
            }
            Self::Unreachable(e) => format!("{addr}:{port}: {e}"),
        })
    }
}
pub(crate) async fn f_1wf(
    http: &reqwest::Client,
    addr: &str,
    port: u16,
    session_path: &str,
    r_vr: Option<&str>,
    db: &Db,
) -> SessionVerdict {
    let r_vq = lan_session::ClientHello::new();
    let r_vu = format!(
        "http://{addr}:{port}{session_path}?epk={}&nonce={}",
        r_vq.epk(),
        r_vq.nonce()
    );
    let r_vt = match http.get(&r_vu).timeout(Duration::from_secs(8)).send().await {
        Ok(r) => r,
        Err(e) => return SessionVerdict::Unreachable(e.to_string()),
    };
    if !r_vt.status().is_success() {
        let pinned = r_vr.and_then(|did| {
            db.0.lock()
                .ok()
                .and_then(|c| db::f_1et(&c, did).ok().flatten())
        });
        return if pinned.is_some() {
            SessionVerdict::Downgrade
        } else {
            SessionVerdict::OldProtocol
        };
    }
    let r_vo: lan_session::T90 = match f_25s(r_vt, 8192).await {
        Ok(b) => b,
        Err(_) => return SessionVerdict::BadSig,
    };
    if let Some(did) = r_vr {
        if did != r_vo.device_id {
            return SessionVerdict::Mismatch;
        }
    }
    let session = match r_vq.finish(&r_vo) {
        Ok(s) => s,
        Err(_) => return SessionVerdict::BadSig,
    };
    let pinned = {
        let r_vp = match db.0.lock() {
            Ok(c) => c,
            Err(_) => return SessionVerdict::Unreachable("DB bận".into()),
        };
        db::f_22x(&r_vp, &session.device_id, &session.pubkey, Some(addr)).ok()
    };
    match pinned {
        Some(r_vs) if r_vs == session.pubkey => SessionVerdict::Ok(Box::new(session)),
        Some(_) => SessionVerdict::Mismatch,
        None => SessionVerdict::Unreachable("ghim khoá lỗi".into()),
    }
}
pub(crate) fn f_2dt(
    rb: reqwest::RequestBuilder,
    session: &lan_session::Session,
    r_vv: &[u8; 32],
) -> Result<(reqwest::RequestBuilder, lan_session::Opener), String> {
    let r_vy = lan_session::f_1u2();
    let (mut r_vz, r_vw) = session
        .request_keys(&r_vy)
        .map_err(|e| e.to_string())?
        .client();
    let r_vx = r_vz
        .seal(0, true, &lan_session::f_104(r_vv, &session.sid))
        .map_err(|e| e.to_string())?;
    let rb = rb
        .header(crate::data_share::SESSION_ID_HEADER, &session.sid)
        .header(crate::data_share::SESSION_REQ_HEADER, &r_vy)
        .header(
            crate::data_share::SESSION_PROOF_HEADER,
            base64::engine::general_purpose::STANDARD.encode(r_vx),
        );
    Ok((rb, r_vw))
}
pub(crate) fn f_ya(
    r_w5: &reqwest::Response,
    r_w3: &mut lan_session::Opener,
    session: &lan_session::Session,
    r_w0: &[u8; 32],
) -> Result<(), String> {
    let r_w4 = r_w5
        .headers()
        .get(crate::data_share::SESSION_PROOF_HEADER)
        .and_then(|v| v.to_str().ok())
        .ok_or("máy nguồn không gửi bằng chứng biết mã site")?;
    let r_w1 = base64::engine::general_purpose::STANDARD
        .decode(r_w4)
        .map_err(|_| "bằng chứng của máy nguồn hỏng".to_string())?;
    let r_w2 = r_w3
        .open(0, true, &r_w1)
        .map_err(|_| {
            "bằng chứng của máy nguồn không mở được (sai khoá phiên)"
                .to_string()
        })?;
    if !lan_session::f_24d(&r_w2, &lan_session::f_2dp(r_w0, &session.sid)) {
        return Err(
            "máy nguồn không chứng minh được biết mã site — BỎ"
                .into(),
        );
    }
    Ok(())
}
#[derive(Debug)]
pub(crate) struct Th1 {
    pub addr: String,
    pub port: u16,
    pub session: lan_session::Session,
    pub files: Vec<Td1>,
}
fn f_1hm(status: reqwest::StatusCode) -> String {
    match status.as_u16() {
        403 => {
            "mã site không đúng (hoặc phiên đã hết hạn)".into()
        }
        426 => "peer đang chạy bản cũ, cần cập nhật".into(),
        _ => format!("máy user trả lỗi {status}"),
    }
}
async fn f_1c6(
    http: &reqwest::Client,
    peer: &Peer,
    r_w6: &[u8; 32],
    db: &Db,
) -> Result<Th1, String> {
    let mut r_w7: Vec<String> = Vec::new();
    for (addr, port) in &peer.candidates {
        let session = match f_1wf(
            http,
            addr,
            *port,
            crate::data_share::SESSION_PATH,
            peer.device_id.as_deref(),
            db,
        )
        .await
        {
            SessionVerdict::Ok(s) => *s,
            v => {
                if let Some(n) = v.note(addr, *port) {
                    r_w7.push(format!("[{}] {n}", f_2gh(peer.source)));
                }
                continue;
            }
        };
        let r_wd = format!("http://{addr}:{port}{}", crate::data_share::MANIFEST_PATH);
        let (r_wa, mut r_w8) = match f_2dt(
            http.get(&r_wd).timeout(Duration::from_secs(60)),
            &session,
            r_w6,
        ) {
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let r_wb = match r_wa.send().await {
            Ok(r) => r,
            Err(e) => {
                r_w7.push(if peer.candidates.len() > 1 {
                    format!("{addr}:{port}: {e}")
                } else {
                    e.to_string()
                });
                continue;
            }
        };
        if !r_wb.status().is_success() {
            return Err(f_1hm(r_wb.status()));
        }
        f_ya(&r_wb, &mut r_w8, &session, r_w6)?;
        let r_wc = f_25n(r_wb, C_BR).await?;
        let r_w9 = lan_session::f_1w6(&mut r_w8, &r_wc)
            .map_err(|e| format!("manifest không giải mã được: {e}"))?;
        let files: Vec<Td1> = serde_json::from_slice(&r_w9)
            .map_err(|e| format!("manifest không hợp lệ: {e}"))?;
        return Ok(Th1 {
            addr: addr.clone(),
            port: *port,
            session,
            files,
        });
    }
    Err(if r_w7.is_empty() {
        "không có địa chỉ IPv4 nào để kết nối".into()
    } else {
        r_w7.join("; ")
    })
}
#[allow(clippy::too_many_arguments)]
async fn f_176(
    http: &reqwest::Client,
    addr: &str,
    port: u16,
    f: &Td1,
    session: &lan_session::Session,
    r_wf: &[u8; 32],
    dest: &Path,
    r_wg: &mut DownloadWatch<'_>,
) -> Result<Downloaded, String> {
    let part = dest.with_extension("part");
    match f_17f(http, addr, port, f, session, r_wf, &part, r_wg).await {
        Ok(Downloaded::Complete) => {}
        Ok(cancelled @ Downloaded::Cancelled { .. }) => return Ok(cancelled),
        Err(DownloadFail::Interrupted(e)) => return Err(e),
        Err(DownloadFail::Suspect(e)) => {
            let _ = tokio::fs::remove_file(&part).await;
            return Err(e);
        }
    }
    let r_we = hash_file(&part).await.map_err(|e| e.to_string())?;
    if r_we != f.blake3 {
        let _ = tokio::fs::remove_file(&part).await;
        return Err(
            "hash không khớp sau khi tải (dữ liệu hỏng)".into(),
        );
    }
    tokio::fs::rename(&part, dest)
        .await
        .map_err(|e| format!("chốt file lỗi: {e}"))?;
    Ok(Downloaded::Complete)
}
#[derive(Debug, PartialEq, Eq)]
enum Downloaded {
    Complete,
    Cancelled { received: u64 },
}
struct DownloadWatch<'a> {
    live: Option<Tb3<'a>>,
    received: u64,
}
struct Tb3<'a> {
    rep: &'a cprog::Reporter,
    ticket: &'a cprog::FileTicket,
    control: &'a State<'a, CollectControl>,
    gate: ProgressGate,
}
impl<'a> DownloadWatch<'a> {
    fn live(
        rep: &'a cprog::Reporter,
        ticket: &'a cprog::FileTicket,
        control: &'a State<'a, CollectControl>,
        total: u64,
    ) -> Self {
        Self {
            live: Some(Tb3 {
                rep,
                ticket,
                control,
                gate: ProgressGate::new(total),
            }),
            received: 0,
        }
    }
    fn received(&self) -> u64 {
        self.received
    }
    fn tick(&mut self, r_wi: u64) -> bool {
        self.received = r_wi;
        let Some(r_wh) = self.live.as_mut() else {
            return true;
        };
        if r_wh.gate.allow(r_wi, std::time::Instant::now()) {
            r_wh.rep.file(&r_wh.ticket.downloading(r_wi));
        }
        !r_wh
            .control
            .file_cancelled(r_wh.rep.session_id(), r_wh.ticket.key())
    }
}
#[derive(Debug)]
enum DownloadFail {
    Interrupted(String),
    Suspect(String),
}
impl DownloadFail {
    fn message(self) -> String {
        match self {
            Self::Interrupted(e) | Self::Suspect(e) => e,
        }
    }
}
impl From<DownloadFail> for String {
    fn from(e: DownloadFail) -> String {
        e.message()
    }
}
#[allow(clippy::too_many_arguments)]
async fn f_17f(
    http: &reqwest::Client,
    addr: &str,
    port: u16,
    f: &Td1,
    session: &lan_session::Session,
    r_wo: &[u8; 32],
    part: &Path,
    r_x5: &mut DownloadWatch<'_>,
) -> Result<Downloaded, DownloadFail> {
    use DownloadFail::{Interrupted, Suspect};
    let net = |e: String| Interrupted(e);
    if !valid_blake3(&f.blake3) {
        return Err(Suspect("blake3 không hợp lệ".into()));
    }
    let r_x0 = u64::try_from(f.size).map_err(|_| {
        Suspect("kích thước khai báo không hợp lệ".to_string())
    })?;
    let r_wu = lan_session::f_1md(r_x0);
    let r_wn = lan_session::f_zd(r_x0);
    let r_wr = tokio::fs::metadata(part)
        .await
        .map(|r_wv| r_wv.len())
        .unwrap_or(0);
    let mut r_x3 = r_wr / lan_session::C_7I as u64;
    if r_x3 > r_wu {
        r_x3 = 0;
    }
    let r_x4 = format!(
        "http://{addr}:{port}{}?blake3={}",
        crate::data_share::FILE_PATH,
        f.blake3
    );
    let mut rb = http.get(&r_x4).timeout(Duration::from_secs(600));
    if r_x3 > 0 {
        rb = rb.header("Range", format!("bytes={}-", lan_session::f_ze(r_x3)));
    }
    let (rb, mut r_wx) = f_2dt(rb, session, r_wo).map_err(net)?;
    let mut r_x2 = rb.send().await.map_err(|e| net(e.to_string()))?;
    let status = r_x2.status();
    if !status.is_success() {
        return Err(Interrupted(f_1hm(status)));
    }
    f_ya(&r_x2, &mut r_wx, session, r_wo).map_err(Suspect)?;
    let r_wj = if r_x3 > 0 && status == reqwest::StatusCode::PARTIAL_CONTENT {
        r_x3
    } else {
        0
    };
    let r_ws = r_wj * lan_session::C_7I as u64;
    let mut file = tokio::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(part)
        .await
        .map_err(|e| net(e.to_string()))?;
    file.set_len(r_ws).await.map_err(|e| net(e.to_string()))?;
    file.seek(std::io::SeekFrom::Start(r_ws))
        .await
        .map_err(|e| net(e.to_string()))?;
    let r_wm = r_wn - lan_session::f_ze(r_wj);
    let mut r_x1: u64 = 0;
    let mut r_wk: Vec<u8> = Vec::with_capacity(lan_session::C_7H * 2);
    let mut idx = r_wj;
    let mut done = false;
    let mut fail: Option<DownloadFail> = None;
    let mut cancelled = false;
    if r_ws > 0 && !r_x5.tick(r_ws) {
        cancelled = true;
    }
    'stream: loop {
        if cancelled {
            break 'stream;
        }
        let r_wl = match r_x2.chunk().await {
            Ok(Some(c)) => c,
            Ok(None) => break 'stream,
            Err(e) => {
                fail = Some(Interrupted(e.to_string()));
                break 'stream;
            }
        };
        if f_2rk(r_x1, r_wl.len(), r_wm) {
            fail = Some(
                Suspect(
                    "máy user gửi vượt kích thước đã khai báo (nghi dữ liệu bất thường)"
                        .into(),
                ),
            );
            break 'stream;
        }
        r_x1 = r_x1.saturating_add(r_wl.len() as u64);
        r_wk.extend_from_slice(&r_wl);
        while !done {
            let r_ww = lan_session::f_1dn(r_x0, idx);
            if r_wk.len() < r_ww {
                break;
            }
            let r_wq: Vec<u8> = r_wk.drain(..r_ww).collect();
            let r_wt = idx == r_wu;
            let r_wz = match r_wx.open(idx, r_wt, &r_wq) {
                Ok(r_wy) => r_wy,
                Err(e) => {
                    fail = Some(Suspect(format!("khung {idx} hỏng: {e}")));
                    break 'stream;
                }
            };
            if let Err(e) = file.write_all(&r_wz).await {
                fail = Some(Interrupted(e.to_string()));
                break 'stream;
            }
            idx += 1;
            done = r_wt;
        }
        if !r_x5.tick(r_ws + (idx - r_wj) * lan_session::C_7I as u64) {
            cancelled = true;
            break 'stream;
        }
    }
    let r_wp = file.flush().await.and(file.sync_all().await);
    drop(file);
    if let Some(e) = fail {
        return Err(e);
    }
    if let Err(e) = r_wp {
        return Err(Interrupted(e.to_string()));
    }
    if cancelled {
        return Ok(Downloaded::Cancelled {
            received: r_x5.received(),
        });
    }
    if !done {
        return Err(Interrupted(
            "máy user cắt giữa chừng (thiếu khung cuối)".into(),
        ));
    }
    if !r_wk.is_empty() {
        return Err(Suspect(
            "máy user gửi thừa byte sau khung cuối".into(),
        ));
    }
    Ok(Downloaded::Complete)
}
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForwardSummary {
    pub forwarded: usize,
    pub skipped: usize,
    pub failed: usize,
    pub total: usize,
    pub retagged: usize,
    pub rejected: usize,
    pub cancelled: usize,
}
impl ForwardSummary {
    fn record_landed(&mut self, duplicate: bool) {
        if duplicate {
            self.skipped += 1;
        } else {
            self.forwarded += 1;
        }
    }
}
fn f_1je(code: &str) -> bool {
    matches!(
        code,
        "forbidden" | "bad_request" | "not_found" | "validation"
    )
}
async fn open_batch(db: &State<'_, Db>, r_x6: &State<'_, AuthState>) -> Result<String, AppError> {
    let body = serde_json::json!({});
    let r_x8: serde_json::Value = f_2r7(db, r_x6, Perm::Admin, |r_x7, r_x9| {
        let body = body.clone();
        async move { client::f_23d(&r_x7, &r_x9, "/upload-batches", &body).await }
    })
    .await?;
    r_x8.get("id")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .ok_or_else(|| AppError::internal("server không trả id đợt nộp"))
}
async fn complete_batch(
    db: &State<'_, Db>,
    r_xa: &State<'_, AuthState>,
    id: &str,
    r_xc: &ForwardSummary,
) -> Result<(), AppError> {
    let path = format!("/upload-batches/{id}");
    let body = serde_json::json!(
        { "forwarded" : r_xc.forwarded, "skipped" : r_xc.skipped, "failed" : r_xc.failed,
        "rejected" : r_xc.rejected, }
    );
    let _: serde_json::Value = f_2r7(db, r_xa, Perm::Admin, |r_xb, r_xd| {
        let (path, body) = (path.clone(), body.clone());
        async move { client::f_20e(&r_xb, &r_xd, &path, &body).await }
    })
    .await?;
    Ok(())
}
fn f_1tx(a: &str, b: &str) -> std::cmp::Ordering {
    use std::cmp::Ordering;
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    fn f_15e(r_xf: &mut std::iter::Peekable<std::str::Chars<'_>>) -> (String, usize) {
        let mut s = String::new();
        while r_xf.peek().is_some_and(char::is_ascii_digit) {
            s.push(r_xf.next().unwrap());
        }
        let r_xg = s.len();
        (s.trim_start_matches('0').to_string(), r_xg)
    }
    loop {
        match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(ca), Some(cb)) if ca.is_ascii_digit() && cb.is_ascii_digit() => {
                let ((da, ra), (r_xe, r_xh)) = (f_15e(&mut x), f_15e(&mut y));
                let ord = da
                    .len()
                    .cmp(&r_xe.len())
                    .then_with(|| da.cmp(&r_xe))
                    .then_with(|| ra.cmp(&r_xh));
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            (Some(ca), Some(cb)) => {
                x.next();
                y.next();
                let ord = ca
                    .to_lowercase()
                    .cmp(cb.to_lowercase())
                    .then_with(|| ca.cmp(&cb));
                if ord != Ordering::Equal {
                    return ord;
                }
            }
        }
    }
}
pub(crate) fn f_2ge(r_xi: &mut [db::T2w]) {
    r_xi.sort_by(|a, b| {
        a.project_id
            .cmp(&b.project_id)
            .then_with(|| a.phong_ma.cmp(&b.phong_ma))
            .then_with(|| {
                f_1tx(
                    a.ho_so_ref.as_deref().unwrap_or_default(),
                    b.ho_so_ref.as_deref().unwrap_or_default(),
                )
            })
            .then_with(|| f_1tx(&a.filename, &b.filename))
            .then_with(|| a.id.cmp(&b.id))
    });
}
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T8c {
    #[serde(default)]
    pub ids: Option<Vec<i64>>,
}
#[tauri::command]
pub async fn data_forward(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    guards: State<'_, SyncGuards>,
    control: State<'_, ForwardControl>,
    args: Option<T8c>,
) -> Result<ForwardSummary, AppError> {
    auth.require(Perm::Admin)?;
    let r_xp = crate::archive::paths::f_18r(&app, &db).ok();
    let r_xq = r_xp.as_ref().and_then(|r| r.canonicalize().ok());
    let _lease = guards
        .try_forward()
        .ok_or_else(|| {
            AppError::new(
                "forward_busy",
                "Đang có một phiên gửi lên máy chủ chạy dở — chờ nó xong rồi bấm lại.",
            )
        })?;
    let ids = args.and_then(|a| a.ids);
    let mut pending = {
        let conn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        match &ids {
            Some(ids) => {
                for id in ids {
                    db::f_101(&conn, *id)?;
                }
                db::f_10l(&conn, ids)?
            }
            None => db::f_10k(&conn)?,
        }
    };
    f_2ge(&mut pending);
    let mut r_xs = ForwardSummary {
        total: pending.len(),
        ..Default::default()
    };
    let session_id = control.begin();
    let rep = Reporter::new(app.clone(), session_id);
    rep.session(SessionPhase::Start, r_xs.total, None);
    let r_xo = SessionGuard::new(&rep, |s| control.end(s));
    let r_xn: Option<String> = if pending.is_empty() {
        None
    } else {
        match open_batch(&db, &auth).await {
            Ok(id) => Some(id),
            Err(e) => {
                tracing::warn!(
                    error = % e,
                    "không mở được đợt nộp — vẫn gửi, chỉ mất theo dõi"
                );
                None
            }
        }
    };
    let fcx = ForwardCtx {
        db: &db,
        auth: &auth,
        control: &control,
        rep: &rep,
        session_id,
        batch: r_xn.as_deref(),
        roots: [r_xp.as_deref(), r_xq.as_deref()],
    };
    let summary = std::sync::Mutex::new(r_xs);
    {
        use futures_util::StreamExt;
        let (fcx, summary, control) = (&fcx, &summary, &control);
        futures_util::stream::iter(pending.iter())
            .for_each_concurrent(FORWARD_PARALLEL, |doc| async move {
                if control.session_cancelled(session_id) || control.file_cancelled(session_id, &doc.id) {
                    sum(summary).cancelled += 1;
                    return;
                }
                if let Err(e) = forward_one(fcx, doc, summary, false).await {
                    tracing::warn!(error = % e, file = % doc.filename, "gửi lên máy chủ: lỗi");
                    sum(summary).failed += 1;
                }
            })
            .await;
    }
    let r_xs = summary.into_inner().unwrap_or_else(|e| e.into_inner());
    if let Some(id) = &r_xn {
        if let Err(e) = complete_batch(&db, &auth, id, &r_xs).await {
            tracing::warn!(
                error = % e, batch = % id,
                "không chốt được đợt nộp"
            );
        }
    }
    tracing::info!(
        forwarded = r_xs.forwarded,
        skipped = r_xs.skipped,
        failed = r_xs.failed,
        rejected = r_xs.rejected,
        retagged = r_xs.retagged,
        cancelled = r_xs.cancelled,
        batch = r_xn.as_deref().unwrap_or("-"),
        "data_forward"
    );
    r_xo.finish(
        r_xs.total,
        serde_json::to_value(&r_xs).unwrap_or(serde_json::Value::Null),
    );
    Ok(r_xs)
}
/// Bối cảnh chung của một phiên gửi lên máy chủ (dùng cho cả phiên thường lẫn đẩy thẳng).
struct ForwardCtx<'a> {
    db: &'a State<'a, Db>,
    auth: &'a State<'a, AuthState>,
    control: &'a State<'a, ForwardControl>,
    rep: &'a Reporter,
    session_id: u64,
    batch: Option<&'a str>,
    roots: [Option<&'a Path>; 2],
}
/// Gửi một tệp trong sổ Kho lên máy chủ và ghi kết quả vào sổ + tổng kết. `fresh` = tệp vừa
/// băm xong (đẩy thẳng), không cần băm lại để dò thay đổi.
/// Số tệp gửi song song trong một phiên: đường nội bộ/LAN bị giới hạn bởi thời gian chờ mỗi
/// lượt (hỏi máy chủ, ghi đĩa, ghi CSDL) chứ không phải băng thông, nên 3 luồng cho tệp nhỏ
/// nhanh gấp vài lần; mạng chậm thì các luồng tự chia băng thông.
const FORWARD_PARALLEL: usize = 3;
fn sum(m: &std::sync::Mutex<ForwardSummary>) -> std::sync::MutexGuard<'_, ForwardSummary> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}
async fn forward_one(
    cx: &ForwardCtx<'_>,
    doc: &T2w,
    r_xs: &std::sync::Mutex<ForwardSummary>,
    fresh: bool,
) -> Result<(), AppError> {
    let (db, auth, control, rep, session_id) = (cx.db, cx.auth, cx.control, cx.rep, cx.session_id);
    // Cây máy chủ: tính sẵn (đẩy thẳng từ ổ đĩa) hoặc suy từ vị trí trong Kho.
    let r_xr = doc.rel_path.clone().or_else(|| {
        cx.roots
            .into_iter()
            .flatten()
            .find_map(|r| f_26u(r, &doc.storage_path))
    });
    let resync = if fresh { Resync::Unchanged } else { f_29o(db, doc).await? };
    let doc = match resync {
        Resync::Unchanged => doc.clone(),
        Resync::Retagged(d) => {
            sum(r_xs).retagged += 1;
            *d
        }
        Resync::DuplicateRemoved => {
            rep.file(&f_2kx(session_id, doc, r_xr.as_deref()).skipped());
            sum(r_xs).skipped += 1;
            return Ok(());
        }
        Resync::Missing(why) => {
            {
                let conn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
                db::f_1r6(&conn, doc.id, &why)?;
            }
            rep.file(&f_2kx(session_id, doc, r_xr.as_deref()).error(0, why, false));
            sum(r_xs).failed += 1;
            return Ok(());
        }
    };
    let doc = &doc;
    let ticket = f_2kx(session_id, doc, r_xr.as_deref());
    rep.file(&ticket.uploading(0));
    match f_1dj(
        db,
        auth,
        doc,
        cx.batch,
        r_xr.as_deref(),
        &rep,
        &ticket,
        &control,
    )
    .await
    {
        Ok(Landed::Server { duplicate }) => {
            {
                let conn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
                db::f_1r8(&conn, doc.id)?;
            }
            rep.file(&if duplicate {
                ticket.skipped()
            } else {
                ticket.done()
            });
            sum(r_xs).record_landed(duplicate);
        }
        Ok(Landed::Cancelled { sent }) => {
            rep.file(&ticket.cancelled(sent));
            sum(r_xs).cancelled += 1;
        }
        Err(e) => {
            {
                let conn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
                if e.permanent {
                    tracing::warn!(
                        file = % doc.filename, project = % doc.project_id, error = %
                        e.message,
                        "máy chủ từ chối vĩnh viễn — ngừng thử lại"
                    );
                    db::f_1r7(&conn, doc.id, &e.message)?;
                    sum(r_xs).rejected += 1;
                } else {
                    db::f_1r6(&conn, doc.id, &e.message)?;
                    sum(r_xs).failed += 1;
                }
            }
            rep.file(&ticket.error(0, e.message, e.permanent));
        }
    }
    Ok(())
}
fn f_2kx(session_id: u64, doc: &T2w, r_xt: Option<&str>) -> FileTicket {
    FileTicket::new(
        session_id,
        doc.id,
        doc.filename.clone(),
        doc.project_id.clone(),
        doc.ho_so_ref.clone(),
        r_xt.map(str::to_string),
        doc.size_bytes.max(0) as u64,
    )
}
#[tauri::command]
pub fn data_forward_cancel(
    auth: State<'_, AuthState>,
    control: State<'_, ForwardControl>,
    args: T2h,
) -> Result<bool, AppError> {
    auth.require(Perm::Admin)?;
    let r_xu = control.cancel(args.session_id, args.id);
    tracing::info!(
        session = args.session_id, file = ? args.id, r_xu, "data_forward_cancel"
    );
    Ok(r_xu)
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T2h {
    pub session_id: u64,
    #[serde(default)]
    pub id: Option<i64>,
}
#[derive(Debug)]
enum Resync {
    Unchanged,
    Retagged(Box<T2w>),
    DuplicateRemoved,
    Missing(String),
}
async fn f_29o(db: &Db, doc: &T2w) -> Result<Resync, AppError> {
    let r_xv = match hash_file(Path::new(&doc.storage_path)).await {
        Ok(h) => h,
        Err(e) => {
            return Ok(Resync::Missing(if doc.rel_path.is_some() {
                format!(
                    "không đọc được tệp trên ổ nguồn ({}) — cắm lại ổ đĩa rồi bấm Gửi lại: {e}",
                    doc.storage_path
                )
            } else {
                format!("không đọc được file trong Kho: {e}")
            }));
        }
    };
    if r_xv.eq_ignore_ascii_case(&doc.blake3) {
        return Ok(Resync::Unchanged);
    }
    let size = tokio::fs::metadata(Path::new(&doc.storage_path))
        .await
        .map(|r_xx| r_xx.len() as i64)
        .unwrap_or(doc.size_bytes);
    tracing::info!(
        file = % doc.filename, old = % doc.blake3, new = % r_xv,
        "file trong Kho đã đổi sau khi gom — băm lại và đẩy bản mới"
    );
    let r_xw = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    if !db::f_29p(&r_xw, doc.id, &r_xv, size)? {
        db::f_13u(&r_xw, doc.id)?;
        return Ok(Resync::DuplicateRemoved);
    }
    Ok(Resync::Retagged(Box::new(T2w {
        blake3: r_xv,
        size_bytes: size,
        ..doc.clone()
    })))
}
#[derive(Debug)]
struct ForwardFail {
    message: String,
    permanent: bool,
}
impl ForwardFail {
    fn transient(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            permanent: false,
        }
    }
    fn from_app(e: AppError) -> Self {
        Self {
            permanent: f_1je(e.code()),
            message: e.to_string(),
        }
    }
}
fn f_y5(filename: &str, total: u64) -> Result<(), ForwardFail> {
    if total > 0 {
        return Ok(());
    }
    Err(ForwardFail {
        message: format!(
            "Tệp “{filename}” trong Kho rỗng (0 byte) — máy chủ không nhận. Kiểm tra lại bản quét rồi gom lại."
        ),
        permanent: true,
    })
}
/// File ẩn đặt trong thư mục cấp 1 của Kho khi thư mục đó là "thư mục theo dự án máy chủ"
/// (tạo bởi "Thêm tài liệu từ ổ cứng"). Nội dung = id dự án.
pub(crate) const PROJECT_MARKER: &str = ".digitext-project";
fn f_26u(root: &Path, storage_path: &str) -> Option<String> {
    let r_xy = Path::new(storage_path).parent()?;
    let r_xz = r_xy.strip_prefix(root).ok()?;
    let mut r_y0: Vec<String> = r_xz
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    // Cấp 1 là thư mục theo dự án → bỏ, vì cây trên máy chủ đã nhóm theo dự án.
    if r_y0
        .first()
        .map(|top| root.join(top).join(PROJECT_MARKER).is_file())
        .unwrap_or(false)
    {
        r_y0.remove(0);
    }
    if r_y0.is_empty() {
        None
    } else {
        Some(r_y0.join("/"))
    }
}
#[derive(Debug, PartialEq, Eq)]
enum Landed {
    Server { duplicate: bool },
    Cancelled { sent: u64 },
}
#[allow(clippy::too_many_arguments)]
async fn f_1dj(
    db: &State<'_, Db>,
    r_y2: &State<'_, AuthState>,
    doc: &T2w,
    r_y3: Option<&str>,
    r_yb: Option<&str>,
    rep: &Reporter,
    ticket: &FileTicket,
    control: &State<'_, ForwardControl>,
) -> Result<Landed, ForwardFail> {
    let total = tokio::fs::metadata(&doc.storage_path)
        .await
        .map_err(|e| ForwardFail::transient(format!("kho thiếu file: {e}")))?
        .len();
    f_y5(&doc.filename, total)?;
    let pid = doc.project_id.clone();
    let blake3 = doc.blake3.clone();
    let filename = doc.filename.clone();
    let ho_so_ref = doc.ho_so_ref.clone();
    let source = doc.source_user_id.clone();
    let page_count = doc.page_count;
    let batch = r_y3.map(str::to_string);
    let rel = r_yb.map(str::to_string);
    let r_ya = f_2r7(db, r_y2, Perm::Admin, |base, token| {
        let (r_y8, b, name, rel) = (pid.clone(), blake3.clone(), filename.clone(), rel.clone());
        async move { client::f_2nq(&base, &token, &r_y8, &b, &name, rel.as_deref()).await }
    })
    .await
    .map_err(ForwardFail::from_app)?;
    let mut offset = match r_ya {
        UploadProbe::Complete => return Ok(Landed::Server { duplicate: true }),
        UploadProbe::Offset(r_y6) => r_y6.min(total),
    };
    let mut gate = ProgressGate::new(total);
    if offset > 0 && gate.allow(offset, std::time::Instant::now()) {
        rep.file(&ticket.uploading(offset));
    }
    let mut file = tokio::fs::File::open(&doc.storage_path)
        .await
        .map_err(|e| ForwardFail::transient(e.to_string()))?;
    file.seek(std::io::SeekFrom::Start(offset))
        .await
        .map_err(|e| ForwardFail::transient(e.to_string()))?;
    let mut r_yc = 0u32;
    const C_C5: u32 = 5;
    loop {
        if control.file_cancelled(rep.session_id(), &ticket.id()) {
            return Ok(Landed::Cancelled { sent: offset });
        }
        let r_y9 = offset;
        let mut r_y4 = vec![0u8; C_1T];
        let n = f_25z(&mut file, &mut r_y4)
            .await
            .map_err(|e| ForwardFail::transient(e.to_string()))?;
        r_y4.truncate(n);
        let r_y7 = f_2r7(db, r_y2, Perm::Admin, |base, token| {
            let (pid, filename, ho_so_ref, blake3, source, batch, rel, r_y5) = (
                pid.clone(),
                filename.clone(),
                ho_so_ref.clone(),
                blake3.clone(),
                source.clone(),
                batch.clone(),
                rel.clone(),
                r_y4.clone(),
            );
            async move {
                client::f_2nk(
                    &base,
                    &token,
                    &pid,
                    &filename,
                    ho_so_ref.as_deref(),
                    page_count,
                    &blake3,
                    Some(&source),
                    batch.as_deref(),
                    rel.as_deref(),
                    offset,
                    total,
                    r_y5,
                )
                .await
            }
        })
        .await
        .map_err(ForwardFail::from_app)?;
        match r_y7 {
            ChunkOutcome::Done { duplicate } => return Ok(Landed::Server { duplicate }),
            ChunkOutcome::More(new_off) | ChunkOutcome::Conflict(new_off) => {
                offset = new_off.min(total);
                if gate.allow(offset, std::time::Instant::now()) {
                    rep.file(&ticket.uploading(offset));
                }
                if offset <= r_y9 {
                    r_yc += 1;
                    if r_yc >= C_C5 {
                        return Err(
                            ForwardFail::transient(
                                format!(
                                    "Truyền file không tiến triển (kẹt tại {offset}/{total} byte) — máy chủ/proxy không trả offset đúng; dừng lại."
                                ),
                            ),
                        );
                    }
                } else {
                    r_yc = 0;
                }
                file.seek(std::io::SeekFrom::Start(offset))
                    .await
                    .map_err(|e| ForwardFail::transient(e.to_string()))?;
            }
        }
    }
}
async fn f_25z(file: &mut tokio::fs::File, r_yd: &mut [u8]) -> std::io::Result<usize> {
    let mut r_ye = 0;
    while r_ye < r_yd.len() {
        let n = file.read(&mut r_yd[r_ye..]).await?;
        if n == 0 {
            break;
        }
        r_ye += n;
    }
    Ok(r_ye)
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T2y {
    pub id: i64,
    pub source_username: Option<String>,
    pub project_id: String,
    pub phong_ma: Option<String>,
    pub ho_so_ref: Option<String>,
    pub filename: String,
    pub page_count: Option<i64>,
    pub size_bytes: i64,
    pub forwarded: bool,
    pub forward_tries: i64,
    pub forward_fatal: bool,
    pub collected_at: String,
    pub forward_error: Option<String>,
    pub storage_path: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T2z {
    pub total: i64,
    pub pending: i64,
    pub forwarded: i64,
    pub rejected: i64,
    pub rows: Vec<T2y>,
}
const C_24: i64 = 200;
const C_25: i64 = 200_000;
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T2x {
    pub limit: Option<i64>,
}
#[tauri::command]
pub fn collected_list(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: T2x,
) -> Result<T2z, AppError> {
    auth.require(Perm::Admin)?;
    let limit = args.limit.unwrap_or(C_24).clamp(1, C_25);
    let r_yf = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let (total, pending, forwarded, rejected) = db::f_10e(&r_yf)?;
    let rows = db::f_1na(&r_yf, limit)?
        .into_iter()
        .map(|d| T2y {
            id: d.id,
            source_username: d.source_username,
            project_id: d.project_id,
            phong_ma: d.phong_ma,
            ho_so_ref: d.ho_so_ref,
            filename: d.filename,
            page_count: d.page_count,
            size_bytes: d.size_bytes,
            forwarded: d.forwarded_at.is_some(),
            forward_tries: d.forward_tries,
            forward_fatal: d.forward_fatal,
            collected_at: d.collected_at,
            forward_error: d.forward_error,
            storage_path: d.storage_path,
        })
        .collect::<Vec<_>>();
    Ok(T2z {
        total,
        pending,
        forwarded,
        rejected,
        rows,
    })
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WipeCollectedResult {
    pub rows: usize,
    pub files_removed: usize,
    pub files_missing: usize,
    pub files_failed: usize,
}
/// "Xoá sạch kho gom": xoá MỌI bản đã thu thập trên máy này (bản ghi + tệp trong Kho), kể cả
/// bản đã đẩy lên. Chỉ xoá tệp nằm trong thư mục Kho; thư mục rỗng còn lại được dọn.
/// Không đụng máy chủ. Người dùng đã xác nhận ở giao diện.
#[tauri::command]
pub fn collected_wipe_all(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<WipeCollectedResult, AppError> {
    auth.require(Perm::Admin)?;
    let root = crate::archive::paths::f_29d(&app, crate::archive::paths::f_112(&db).as_deref())?;
    let root_c = root.canonicalize().unwrap_or(root.clone());
    let rows = {
        let conn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        db::collected_all(&conn)?
    };
    let paths: Vec<String> = rows.iter().map(|r| r.storage_path.clone()).collect();
    let out = wipe_collected_files(&root_c, &paths);
    {
        let conn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        db::collected_delete_all(&conn)?;
    }
    tracing::warn!(?out, "collected_wipe_all");
    Ok(out)
}
/// Xoá các tệp kho gom — CHỈ tệp nằm trong `root` (đã canonicalize); tệp ngoài root, tệp đang
/// mở → đếm vào `files_failed`, không đụng. Thư mục rỗng còn lại được dọn tới sát gốc.
pub(crate) fn wipe_collected_files(root_c: &std::path::Path, paths: &[String]) -> WipeCollectedResult {
    let mut out = WipeCollectedResult { rows: paths.len(), files_removed: 0, files_missing: 0, files_failed: 0 };
    let mut dirs: std::collections::BTreeSet<std::path::PathBuf> = std::collections::BTreeSet::new();
    for sp in paths {
        let p = std::path::Path::new(sp);
        match p.canonicalize() {
            Err(_) => out.files_missing += 1,
            Ok(c) if !c.starts_with(root_c) => {
                tracing::warn!(path = % sp, "bản ghi trỏ ra ngoài Kho — không xoá tệp");
                out.files_failed += 1;
            }
            Ok(c) => match std::fs::remove_file(&c) {
                Ok(()) => {
                    out.files_removed += 1;
                    if let Some(d) = c.parent() {
                        dirs.insert(d.to_path_buf());
                    }
                }
                Err(_) => out.files_failed += 1,
            },
        }
    }
    // Dọn thư mục rỗng từ sâu lên nông, dừng ở gốc Kho.
    for d in dirs.into_iter().rev() {
        let mut cur = d;
        while cur != root_c && cur.starts_with(root_c) && std::fs::remove_dir(&cur).is_ok() {
            match cur.parent() {
                Some(p) => cur = p.to_path_buf(),
                None => break,
            }
        }
    }
    out
}

#[cfg(test)]
mod wipe_tests {
    use super::wipe_collected_files;
    #[test]
    fn chi_xoa_tep_trong_kho_va_don_thu_muc_rong() {
        let tmp = std::env::temp_dir().join(format!("digitext-wipe-{}", std::process::id()));
        let root = tmp.join("Kho");
        let outside = tmp.join("NgoaiKho");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(root.join("PhongA").join("Hop1").join("HS1")).unwrap();
        std::fs::create_dir_all(&outside).unwrap();
        let f1 = root.join("PhongA").join("Hop1").join("HS1").join("a.pdf");
        let f2 = root.join("PhongA").join("Hop1").join("HS1").join("b.pdf");
        let giu = root.join("PhongA").join("giu-lai.txt"); // không có trong kho gom → phải còn
        let ngoai = outside.join("x.pdf");
        for f in [&f1, &f2, &giu, &ngoai] {
            std::fs::write(f, b"%PDF").unwrap();
        }
        let root_c = root.canonicalize().unwrap();
        let paths = vec![
            f1.to_string_lossy().to_string(),
            f2.to_string_lossy().to_string(),
            ngoai.to_string_lossy().to_string(),
            root.join("khong-ton-tai.pdf").to_string_lossy().to_string(),
        ];
        let r = wipe_collected_files(&root_c, &paths);
        assert_eq!((r.rows, r.files_removed, r.files_failed, r.files_missing), (4, 2, 1, 1));
        assert!(!f1.exists() && !f2.exists(), "tệp trong kho gom phải bị xoá");
        assert!(ngoai.exists(), "tệp ngoài Kho KHÔNG được xoá");
        assert!(giu.exists(), "tệp không thuộc kho gom phải còn");
        assert!(!root.join("PhongA").join("Hop1").exists(), "thư mục rỗng phải được dọn");
        assert!(root.join("PhongA").exists(), "thư mục còn tệp khác phải giữ");
        assert!(root.exists(), "gốc Kho phải còn");
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
/// "Xoá lỗi, thử lại tất cả": bỏ dấu lỗi của mọi bản chưa lên được (giữ bản trong kho gom).
#[tauri::command]
pub fn collected_clear_errors(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<usize, AppError> {
    auth.require(Perm::Admin)?;
    let conn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let n = db::clear_forward_errors(&conn)?;
    tracing::info!(cleared = n, "collected_clear_errors");
    Ok(n)
}
#[tauri::command]
pub fn collected_retry_rejected(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<usize, AppError> {
    auth.require(Perm::Admin)?;
    let r_yg = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let n = db::f_100(&r_yg)?;
    tracing::info!(cleared = n, "collected_retry_rejected");
    Ok(n)
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tf {
    pub paths: Vec<String>,
    #[serde(default)]
    pub project_id: Option<String>,
    #[serde(default)]
    pub dest_rel: Option<String>,
    #[serde(default)]
    pub phong_ma: Option<String>,
    #[serde(default)]
    pub project_label: Option<String>,
    /// Đẩy thẳng từ ổ đĩa: không chép vào Kho, tệp vừa ghi sổ là gửi lên máy chủ ngay
    /// (phiên gửi chạy song song với phiên ghi sổ).
    #[serde(default)]
    pub direct: bool,
    /// Thư mục đã chọn là CẢ DỰ ÁN (chứa nhiều phông): mỗi thư mục con cấp 1 là một phông,
    /// bỏ tên thư mục đã chọn khỏi cây (22/10/2026). Mặc định false: thư mục đã chọn là một phông.
    #[serde(default)]
    pub sub_as_phong: bool,
}
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tg {
    pub total: usize,
    pub added: usize,
    pub skipped: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub project_id: String,
    pub dest: String,
    pub notes: Vec<String>,
    /// Chế độ đẩy thẳng: kết quả phiên gửi chạy kèm.
    pub direct: bool,
    pub pushed: usize,
    pub push_skipped: usize,
    pub push_failed: usize,
    pub push_rejected: usize,
    pub push_cancelled: usize,
}
const C_44: usize = 3;
fn f_14x(dest_rel: Option<&str>) -> Vec<String> {
    let r_yh: Vec<String> = dest_rel
        .unwrap_or_default()
        .split(['/', '\\'])
        .map(str::trim)
        .filter(|s| !s.is_empty() && *s != "." && *s != "..")
        .take(C_44)
        .map(str::to_string)
        .collect();
    if r_yh.is_empty() {
        vec![crate::data_share::C_P0.to_string()]
    } else {
        r_yh
    }
}
/// Thư mục đích trong Kho. Tệp KHÔNG gắn hồ sơ theo mục lục (`phong_ma` trống) nằm dưới
/// một thư mục theo DỰ ÁN máy chủ (`<mã · tên>/Chưa phân loại` hay `<mã · tên>/<vị trí tự gõ>`)
/// để Kho tài liệu không gộp mọi dự án vào một chỗ. Tệp gắn hồ sơ theo mục lục giữ nguyên
/// `<phông>/<hộp>/<hồ sơ>` vì đường dẫn đó là khoá tra `ho_so.folder_path`.
fn dest_segments(
    dest_rel: Option<&str>,
    phong_ma: Option<&str>,
    project_label: Option<&str>,
    project_id: &str,
) -> Vec<String> {
    let mut r = f_14x(dest_rel);
    if phong_ma.map(str::trim).filter(|s| !s.is_empty()).is_some() {
        return r;
    }
    let label = project_label
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(project_id);
    r.insert(0, label.to_string());
    r
}
/// Chọn một thư mục thì giữ luôn tên thư mục đó trong Kho (như chép thư mục trong Windows):
/// chọn `PHONG SCTBG` → `Chưa phân loại/PHONG SCTBG/HOP 1/…`. Trước đây tên bị bỏ nên các HOP
/// của nhiều phông dồn chung một chỗ (25/9/2026: 4.510 tệp mất tầng phông). Hồ sơ theo mục lục
/// (`phong_ma`) thì KHÔNG thêm: thư mục đích đã là `<phông>/<hộp>/<hồ sơ>`.
fn keep_selected_folder_name(src: &Path, sub: Vec<String>, chua_phan_loai: bool) -> Vec<String> {
    if !chua_phan_loai {
        return sub;
    }
    match src.file_name().map(|n| n.to_string_lossy().into_owned()).filter(|n| !n.is_empty()) {
        Some(name) => std::iter::once(name).chain(sub).collect(),
        None => sub,
    }
}
#[cfg(test)]
mod keep_name_tests {
    use super::keep_selected_folder_name;
    use std::path::Path;
    #[test]
    fn giu_ten_thu_muc_da_chon_khi_chua_phan_loai() {
        let sub = vec!["HOP 1".to_string(), "HO SO 2".to_string()];
        assert_eq!(
            keep_selected_folder_name(Path::new(r"F:\TTLTLS BNI\PHONG SCTBG"), sub.clone(), true),
            vec!["PHONG SCTBG", "HOP 1", "HO SO 2"]
        );
        assert_eq!(keep_selected_folder_name(Path::new(r"F:\TTLTLS BNI\PHONG SCTBG"), sub.clone(), false), sub);
        assert_eq!(keep_selected_folder_name(Path::new(r"F:\"), sub.clone(), true), sub);
    }
}
/// Một thư mục con cấp 1 của thư mục đã chọn: tên và số thư mục con của nó (để đoán
/// đây là cấp Hộp hay cấp Phông).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeekSub {
    pub name: String,
    pub dirs: usize,
}
/// Xem trước một thư mục người dùng chọn ở "Thêm từ ổ đĩa": chỉ đọc hai cấp đầu, không
/// duyệt sâu, để giao diện xác nhận "thư mục này là một phông hay là cả dự án".
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PeekFolder {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub subs: Vec<PeekSub>,
    /// Tệp PDF nằm ngay trong thư mục (không tính thư mục con).
    pub pdfs: usize,
}
fn count_subdirs(dir: &Path) -> usize {
    std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
                .filter(|e| !f_2fv(&e.file_name().to_string_lossy()))
                .count()
        })
        .unwrap_or(0)
}
fn peek_folder(path: &Path) -> PeekFolder {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string_lossy().into_owned());
    let is_dir = path.is_dir();
    let mut subs = Vec::new();
    let mut pdfs = 0usize;
    if is_dir {
        if let Ok(rd) = std::fs::read_dir(path) {
            for e in rd.flatten() {
                let Ok(t) = e.file_type() else { continue };
                let n = e.file_name().to_string_lossy().into_owned();
                if t.is_dir() {
                    if !f_2fv(&n) {
                        subs.push(PeekSub { dirs: count_subdirs(&e.path()), name: n });
                    }
                } else if t.is_file() && n.to_ascii_lowercase().ends_with(".pdf") {
                    pdfs += 1;
                }
            }
        }
        subs.sort_by(|a, b| digitext_core::tiff_naming::natural_cmp(&a.name, &b.name));
    }
    PeekFolder { path: path.to_string_lossy().into_owned(), name, is_dir, subs, pdfs }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeekArgs {
    pub paths: Vec<String>,
}
#[tauri::command]
pub async fn collect_peek_folders(auth: State<'_, AuthState>, args: PeekArgs) -> Result<Vec<PeekFolder>, AppError> {
    auth.require(Perm::Admin)?;
    let paths = args.paths;
    tauri::async_runtime::spawn_blocking(move || paths.iter().map(|p| peek_folder(Path::new(p))).collect::<Vec<_>>())
        .await
        .map_err(|e| AppError::internal(format!("luồng đọc thư mục lỗi ({e})")))
}
fn inside(root: &Path, path: &Path) -> bool {
    let r_yi = |r_yj: &Path| r_yj.canonicalize().unwrap_or_else(|_| r_yj.to_path_buf());
    r_yi(path).strip_prefix(r_yi(root)).is_ok()
}
#[tauri::command]
pub async fn collect_add_files(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    guards: State<'_, SyncGuards>,
    control: State<'_, CollectControl>,
    args: Tf,
) -> Result<Tg, AppError> {
    auth.require(Perm::Admin)?;
    let _lease = guards
        .try_collect()
        .ok_or_else(|| {
            AppError::new(
                "collect_busy",
                "Đang có một phiên thu thập chạy dở — chờ nó xong rồi thêm tài liệu.",
            )
        })?;
    let session = auth
        .snapshot()
        .ok_or_else(|| AppError::new("unauthorized", "Chưa đăng nhập."))?;
    let root = crate::archive::paths::f_29d(&app, crate::archive::paths::f_112(&db).as_deref())?;
    let gate = f_1wx(&db, &auth).await;
    let r_yr = match args.project_id.as_deref().map(str::trim) {
        Some(p) if !p.is_empty() => {
            if !gate.is_empty() && !gate.allows(p) {
                return Err(
                    AppError::validation(
                        "Dự án đã chọn không thuộc quyền quản trị của máy này.",
                    ),
                );
            }
            p.to_string()
        }
        _ => {
            f_1wl(session.project_id.as_deref(), &gate)
                .ok_or_else(|| {
                    AppError::validation(
                        "Máy này quản nhiều dự án — hãy chọn dự án đích cho số tài liệu này.",
                    )
                })?
        }
    };
    let r_yx = dest_segments(
        args.dest_rel.as_deref(),
        args.phong_ma.as_deref(),
        args.project_label.as_deref(),
        &r_yr,
    );
    let r_yw: Vec<&str> = r_yx.iter().map(String::as_str).collect();
    let folder = crate::storage::f_1jt(&root, &r_yw, false);
    tokio::fs::create_dir_all(&folder).await.map_err(|e| {
        AppError::internal(format!(
            "tạo thư mục đích trong Kho lỗi: {e}"
        ))
    })?;
    // Thư mục cấp 1 theo dự án được đánh dấu bằng file ẩn để lúc đẩy lên máy chủ bỏ
    // cấp này khỏi relPath (cây máy chủ tự thêm cấp dự án — không đánh dấu là lặp 2 lần).
    if args.phong_ma.as_deref().map(str::trim).filter(|s| !s.is_empty()).is_none() {
        let marker = crate::storage::f_1jt(&root, &r_yw[..1], false).join(PROJECT_MARKER);
        if let Err(e) = tokio::fs::write(&marker, r_yr.as_bytes()).await {
            tracing::warn!(error = % e, path = % marker.display(), "không ghi được dấu thư mục dự án");
        }
    }
    // Chọn CẢ THƯ MỤC mà không gắn hồ sơ danh mục và không gõ "Vị trí trong Kho": thư mục đã tự
    // mang cây phân loại (PHONG/HOP/HO SO) → đặt thẳng dưới thư mục dự án, KHÔNG chèn tầng
    // "Chưa phân loại" (28/9/2026: 30.729 tệp LTLSBN thừa tầng này). Tệp lẻ vẫn vào "Chưa phân loại".
    let khong_danh_muc = args.phong_ma.as_deref().map(str::trim).filter(|s| !s.is_empty()).is_none();
    let khong_vi_tri = args.dest_rel.as_deref().map(str::trim).filter(|s| !s.is_empty()).is_none();
    let dir_folder = if khong_danh_muc && khong_vi_tri {
        crate::storage::f_1jt(&root, &r_yw[..1], false)
    } else {
        folder.clone()
    };
    let mut r_yy = Tg {
        project_id: r_yr.clone(),
        dest: r_yx.join("/"),
        ..Default::default()
    };
    let session_id = control.begin();
    let rep = cprog::Reporter::new(app.clone(), session_id, cprog::Kind::Disk).direct(args.direct);
    rep.session(cprog::SessionPhase::Start, None);
    let r_ym = SessionGuard::new(&rep, |s| control.end(s));
    rep.stage(
        cprog::Stage::Disk,
        "Đang đọc danh sách tệp đã chọn…",
        0,
    );
    let mut r_yv = 0usize;
    let mut r_yo: Vec<(PathBuf, PathBuf)> = Vec::new();
    let chua_phan_loai = args.phong_ma.as_deref().map(str::trim).filter(|s| !s.is_empty()).is_none();
    for r_ys in &args.paths {
        let src = PathBuf::from(r_ys);
        let Ok(r_yq) = std::fs::metadata(&src) else {
            r_yy.total += 1;
            r_yy.failed += 1;
            f_26s(
                &rep,
                session_id,
                &mut r_yv,
                &src,
                "Không đọc được tệp",
            );
            f_24t(
                &mut r_yy,
                &format!("Không đọc được: {}", f_2ev(&src)),
            );
            continue;
        };
        if r_yq.is_dir() {
            let (files, r_z0) = f_2qo(&src, C_BQ);
            if r_z0 {
                f_24t(
                    &mut r_yy,
                    "Thư mục có quá nhiều tệp — lượt này mới lấy một phần, hãy thêm lại lần nữa.",
                );
            }
            for f in files {
                let sub = f
                    .parent()
                    .and_then(|p| p.strip_prefix(&src).ok())
                    .map(|r_yu| {
                        r_yu.components()
                            .filter_map(|c| match c {
                                std::path::Component::Normal(s) => {
                                    Some(s.to_string_lossy().into_owned())
                                }
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                let sub = if args.sub_as_phong && chua_phan_loai {
                    // Thư mục chọn = dự án: con cấp 1 là phông; tệp nằm ngay dưới gốc → "Chưa phân loại".
                    if sub.is_empty() {
                        vec![crate::data_share::C_P0.to_string()]
                    } else {
                        sub
                    }
                } else {
                    keep_selected_folder_name(&src, sub, chua_phan_loai)
                };
                let r_yt: Vec<&str> = sub.iter().map(String::as_str).collect();
                r_yo.push((f, crate::storage::f_1jt(&dir_folder, &r_yt, false)));
            }
        } else {
            let r_yn = src
                .extension()
                .and_then(|s| s.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
            if !r_yn {
                r_yy.total += 1;
                r_yy.failed += 1;
                f_26s(
                    &rep,
                    session_id,
                    &mut r_yv,
                    &src,
                    "Không phải tệp PDF",
                );
                f_24t(
                    &mut r_yy,
                    &format!("Bỏ qua (không phải PDF): {}", f_2ev(&src)),
                );
                continue;
            }
            r_yo.push((src, folder.clone()));
        }
    }
    rep.stage(
        cprog::Stage::Disk,
        if r_yo.iter().all(|(src, _)| inside(&root, src)) {
            // Tệp chép tay sẵn trong Kho (Quét Kho → Đẩy): chỉ ghi sổ tại chỗ, không chép.
            format!(
                "Đang ghi sổ {} tệp có sẵn trong Kho (không chép){}…",
                r_yo.len(),
                if args.direct { " và gửi lên máy chủ" } else { "" }
            )
        } else if args.direct {
            format!("Đang ghi sổ {} tệp trên ổ đĩa (không chép) và gửi lên máy chủ…", r_yo.len())
        } else {
            format!("Đang thêm {} tệp từ ổ cứng vào Kho…", r_yo.len())
        },
        0,
    );
    let r_yz = Ti {
        project_id: &r_yr,
        phong_ma: f_13d(args.phong_ma.as_deref()),
        root: &root,
    };
    // Đẩy thẳng từ ổ đĩa: phiên gửi lên máy chủ chạy SONG SONG với vòng ghi sổ; mỗi tệp vừa
    // ghi sổ được đưa qua kênh để gửi ngay, không chờ hết danh sách, không băm lại.
    let direct = args.direct;
    let _flease = if direct {
        Some(guards.try_forward().ok_or_else(|| {
            AppError::new(
                "forward_busy",
                "Đang có một phiên gửi lên máy chủ chạy dở — chờ nó xong rồi thêm tài liệu.",
            )
        })?)
    } else {
        None
    };
    let fctrl: State<'_, ForwardControl> = app.state();
    let planned_total = r_yo.len();
    let root_canon = root.canonicalize().ok();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<i64>();
    let tx = direct.then_some(tx);
    let add_fut = async {
    for (src, r_yl) in r_yo {
        r_yy.total += 1;
        let size = std::fs::metadata(&src).map(|r_yp| r_yp.len()).unwrap_or(0);
        let r_yk = match src.parent() {
            Some(p) if inside(&root, &src) => p,
            _ => r_yl.as_path(),
        };
        let ticket =
            cprog::FileTicket::disk(session_id, r_yv, f_2ev(&src), f_1kk(&root, r_yk), size);
        r_yv += 1;
        if control.session_cancelled(session_id) {
            r_yy.cancelled += 1;
            rep.file(&ticket.cancelled(0));
            continue;
        }
        rep.file(&ticket.downloading(0));
        let mut r_z1 = AddWatch::new(&rep, &ticket, &control, size);
        match f_qr(&app, &db, &session, &r_yz, &src, &r_yl, &mut r_z1, direct).await {
            Ok(AddOutcome::Added(id)) => {
                r_yy.added += 1;
                rep.file(&ticket.done());
                if let Some(tx) = &tx {
                    let _ = tx.send(id);
                }
            }
            Ok(AddOutcome::Skipped(id)) => {
                r_yy.skipped += 1;
                rep.file(
                    &ticket
                        .skipped(
                            "Kho đã có bản giống hệt — không thêm lần hai.",
                        ),
                );
                // Đã ghi sổ từ lần trước mà chưa lên máy chủ → vẫn gửi tiếp.
                if let (Some(tx), Some(id)) = (&tx, id) {
                    let _ = tx.send(id);
                }
            }
            Ok(AddOutcome::Cancelled) => {
                r_yy.cancelled += 1;
                rep.file(&ticket.cancelled(r_z1.copied()));
            }
            Err(why) => {
                r_yy.failed += 1;
                rep.file(&ticket.error(r_z1.copied(), why.clone()));
                f_24t(&mut r_yy, &format!("{}: {why}", f_2ev(&src)));
            }
        }
    }
    drop(tx);
    };
    let fwd_fut = async {
        let mut fsum = ForwardSummary::default();
        if !direct {
            return fsum;
        }
        let fsession = fctrl.begin();
        let frep = Reporter::new(app.clone(), fsession);
        frep.session(SessionPhase::Start, planned_total, None);
        let fguard = SessionGuard::new(&frep, |s| fctrl.end(s));
        let batch = match open_batch(&db, &auth).await {
            Ok(id) => Some(id),
            Err(e) => {
                tracing::warn!(error = % e, "không mở được đợt nộp — vẫn gửi, chỉ mất theo dõi");
                None
            }
        };
        let fcx = ForwardCtx {
            db: &db,
            auth: &auth,
            control: &fctrl,
            rep: &frep,
            session_id: fsession,
            batch: batch.as_deref(),
            roots: [Some(root.as_path()), root_canon.as_deref()],
        };
        let fsum = std::sync::Mutex::new(fsum);
        {
            use futures_util::StreamExt;
            let (fcx, fsum, fctrl, control, db) = (&fcx, &fsum, &fctrl, &control, &db);
            futures_util::stream::poll_fn(|cx| rx.poll_recv(cx))
                .for_each_concurrent(FORWARD_PARALLEL, |id| async move {
                    if fctrl.session_cancelled(fsession) || control.session_cancelled(session_id) {
                        // Người dùng dừng: phần còn lại ở nguyên sổ Kho, bấm "Đẩy lên máy chủ" sau.
                        sum(fsum).cancelled += 1;
                        return;
                    }
                    let docs = match db.0.lock() {
                        Ok(conn) => db::f_10l(&conn, &[id]),
                        Err(_) => {
                            sum(fsum).failed += 1;
                            return;
                        }
                    };
                    let Ok(docs) = docs else {
                        sum(fsum).failed += 1;
                        return;
                    };
                    for doc in &docs {
                        if let Err(e) = forward_one(fcx, doc, fsum, true).await {
                            tracing::warn!(error = % e, file = % doc.filename, "đẩy thẳng: lỗi gửi");
                            sum(fsum).failed += 1;
                        }
                    }
                })
                .await;
        }
        let mut fsum = fsum.into_inner().unwrap_or_else(|e| e.into_inner());
        fsum.total = fsum.forwarded + fsum.skipped + fsum.failed + fsum.rejected + fsum.cancelled;
        if let Some(id) = &batch {
            if let Err(e) = complete_batch(&db, &auth, id, &fsum).await {
                tracing::warn!(error = % e, batch = % id, "không chốt được đợt nộp");
            }
        }
        fguard.finish(
            fsum.total,
            serde_json::to_value(&fsum).unwrap_or(serde_json::Value::Null),
        );
        fsum
    };
    let ((), fsum) = tokio::join!(add_fut, fwd_fut);
    r_yy.direct = direct;
    r_yy.pushed = fsum.forwarded;
    r_yy.push_skipped = fsum.skipped;
    r_yy.push_failed = fsum.failed;
    r_yy.push_rejected = fsum.rejected;
    r_yy.push_cancelled = fsum.cancelled;
    tracing::info!(
        total = r_yy.total, added = r_yy.added, skipped = r_yy.skipped, failed = r_yy
        .failed, cancelled = r_yy.cancelled, direct, pushed = r_yy.pushed, push_failed = r_yy
        .push_failed, project = % r_yr, "collect_add_files"
    );
    r_ym.finish(
        r_yy.total,
        serde_json::to_value(&r_yy).unwrap_or(serde_json::Value::Null),
    );
    Ok(r_yy)
}
fn f_26s(rep: &cprog::Reporter, session_id: u64, r_z2: &mut usize, src: &Path, why: &str) {
    let t = cprog::FileTicket::disk(session_id, *r_z2, f_2ev(src), None, 0);
    *r_z2 += 1;
    rep.file(&t.error(0, why));
}
fn f_1kk(root: &Path, dir: &Path) -> Option<String> {
    let r_z3 = dir.strip_prefix(root).ok()?;
    let r_z4: Vec<String> = r_z3
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect();
    (!r_z4.is_empty()).then(|| r_z4.join("/"))
}
fn f_24t(r_z6: &mut Tg, r_z5: &str) {
    if r_z6.notes.len() < C_BS {
        r_z6.notes.push(r_z5.to_string());
    }
}
fn f_2ev(r_z7: &Path) -> String {
    r_z7.file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "(không tên)".into())
}
fn f_13d(r_z8: Option<&str>) -> Option<&str> {
    r_z8.map(str::trim).filter(|s| !s.is_empty())
}
struct Ti<'a> {
    project_id: &'a str,
    phong_ma: Option<&'a str>,
    root: &'a Path,
}
enum AddOutcome {
    /// Đã ghi sổ Kho, kèm id dòng.
    Added(i64),
    /// Đã có sẵn dòng y hệt; kèm id nếu tra được (để đẩy tiếp nếu chưa lên máy chủ).
    Skipped(Option<i64>),
    Cancelled,
}
async fn f_qr(
    app: &AppHandle,
    db: &State<'_, Db>,
    session: &crate::auth::session::Tr,
    r_zd: &Ti<'_>,
    src: &Path,
    r_z9: &Path,
    r_zf: &mut AddWatch<'_>,
    direct: bool,
) -> Result<AddOutcome, String> {
    let size = tokio::fs::metadata(src)
        .await
        .map_err(|e| format!("không đọc được tệp ({e})"))?
        .len() as i64;
    if size <= 0 {
        return Err("tệp rỗng (0 byte) — máy chủ không nhận".into());
    }
    if r_zf.cancelled() {
        return Ok(AddOutcome::Cancelled);
    }
    // Tệp nằm ngoài Kho mà đẩy thẳng, hoặc đã nằm trong Kho: đường dẫn lưu sổ chính là `src`.
    // Có dòng cùng đường dẫn + kích thước → đã ghi sổ lần trước, bỏ qua không cần băm.
    if direct || inside(r_zd.root, src) {
        let conn = db.0.lock().map_err(|_| "DB bận".to_string())?;
        if let Some(id) = db::collected_find_by_path(&conn, r_zd.project_id, &src.to_string_lossy(), size)
            .map_err(|e| e.to_string())?
        {
            return Ok(AddOutcome::Skipped(Some(id)));
        }
    }
    let blake3 = hash_file(src)
        .await
        .map_err(|e| format!("băm tệp lỗi ({e})"))?;
    if r_zf.cancelled() {
        return Ok(AddOutcome::Cancelled);
    }
    let root = r_zd.root;
    // Vị trí tệp "lẽ ra" nằm trong Kho — dùng để tính mã hồ sơ và cây máy chủ, kể cả khi
    // không chép (đẩy thẳng từ ổ đĩa).
    let planned = if inside(root, src) {
        src.to_path_buf()
    } else {
        r_z9.join(f_14v(&src.file_name().unwrap_or_default().to_string_lossy()))
    };
    // Đẩy thẳng: tệp ở nguyên trên ổ nguồn, sổ Kho ghi đường dẫn nguồn.
    let no_copy = direct && !inside(root, src);
    let stored: PathBuf = if no_copy { src.to_path_buf() } else { planned.clone() };
    {
        // Chỉ bỏ qua khi ĐÚNG tệp này đã gom (cùng nội dung + cùng chỗ trong Kho). Tệp khác tên
        // mà trùng từng byte (bản quét lưu nhầm hai số) vẫn là văn bản riêng → vẫn thêm.
        let conn = db.0.lock().map_err(|_| "DB bận".to_string())?;
        if db::collected_same_file(&conn, r_zd.project_id, &blake3, &stored.to_string_lossy())
            .map_err(|e| e.to_string())?
        {
            let id = db::collected_find_id(&conn, r_zd.project_id, &blake3, &stored.to_string_lossy())
                .map_err(|e| e.to_string())?;
            return Ok(AddOutcome::Skipped(id));
        }
    }
    let dest = if inside(root, src) || no_copy {
        src.to_path_buf()
    } else {
        tokio::fs::create_dir_all(r_z9)
            .await
            .map_err(|e| format!("tạo thư mục đích lỗi ({e})"))?;
        let r_za = f_14v(&src.file_name().unwrap_or_default().to_string_lossy());
        let mut dest = r_z9.join(&r_za);
        if matches!(
            hash_file(& dest). await, Ok(h) if ! h.eq_ignore_ascii_case(& blake3)
        ) {
            dest = r_z9.join(f_1t9(&r_za, &blake3));
        }
        match hash_file(&dest).await {
            Ok(h) if h.eq_ignore_ascii_case(&blake3) => {}
            Ok(_) => {
                return Err(
                    format!(
                        "trong Kho đã có tệp khác trùng tên “{}” — đổi tên tệp nguồn rồi thêm lại",
                        dest.file_name().unwrap_or_default().to_string_lossy()
                    ),
                );
            }
            Err(_) => {
                if !f_11j(src, &dest, r_zf).await? {
                    return Ok(AddOutcome::Cancelled);
                }
            }
        }
        dest
    };
    let page_count = {
        let (app, r_zb) = (app.clone(), dest.clone());
        tauri::async_runtime::spawn_blocking(move || {
            let db = app.state::<Db>();
            crate::data_share::f_1g2(&db, &r_zb).and_then(|(_, r_zc, _)| r_zc)
        })
        .await
        .map_err(|e| format!("luồng đọc số trang lỗi ({e})"))?
    };
    let ho_so_ref = f_1km(root, &planned);
    let rel_path = if no_copy { f_26u(root, &planned.to_string_lossy()) } else { None };
    let filename = dest
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "tai-lieu.pdf".into());
    let conn = db.0.lock().map_err(|_| "DB bận".to_string())?;
    let r_ze = db::collected_insert(
        &conn,
        db::Tds {
            source_user_id: &session.user_id,
            source_username: Some(&session.username),
            project_id: r_zd.project_id,
            phong_ma: r_zd.phong_ma,
            ho_so_ref: ho_so_ref.as_deref(),
            filename: &filename,
            page_count,
            size_bytes: size,
            blake3: &blake3,
            storage_path: &dest.to_string_lossy(),
        },
        rel_path.as_deref(),
    )
    .map_err(|e| e.to_string())?;
    Ok(match r_ze {
        Some(id) => AddOutcome::Added(id),
        None => AddOutcome::Skipped(None),
    })
}
const C_2C: usize = 1024 * 1024;
async fn f_11j(src: &Path, dest: &Path, r_zg: &mut AddWatch<'_>) -> Result<bool, String> {
    let r = f_11i(src, dest, r_zg).await;
    if !matches!(r, Ok(true)) {
        let _ = tokio::fs::remove_file(dest).await;
    }
    r
}
async fn f_11i(src: &Path, dest: &Path, r_zj: &mut AddWatch<'_>) -> Result<bool, String> {
    let mut r = tokio::fs::File::open(src)
        .await
        .map_err(|e| format!("mở tệp nguồn lỗi ({e})"))?;
    let mut r_zi = tokio::fs::File::create(dest)
        .await
        .map_err(|e| format!("tạo tệp trong Kho lỗi ({e})"))?;
    let mut r_zh = vec![0u8; C_2C];
    let mut done: u64 = 0;
    loop {
        let n = r
            .read(&mut r_zh)
            .await
            .map_err(|e| format!("đọc tệp nguồn lỗi ({e})"))?;
        if n == 0 {
            break;
        }
        r_zi.write_all(&r_zh[..n])
            .await
            .map_err(|e| format!("chép vào Kho lỗi ({e})"))?;
        done += n as u64;
        if !r_zj.tick(done) {
            return Ok(false);
        }
    }
    r_zi.flush()
        .await
        .map_err(|e| format!("chép vào Kho lỗi ({e})"))?;
    r_zi.sync_all()
        .await
        .map_err(|e| format!("ghi đĩa lỗi ({e})"))?;
    Ok(true)
}
struct AddWatch<'a> {
    rep: &'a cprog::Reporter,
    ticket: &'a cprog::FileTicket,
    control: &'a State<'a, CollectControl>,
    gate: ProgressGate,
    copied: u64,
}
impl<'a> AddWatch<'a> {
    fn new(
        rep: &'a cprog::Reporter,
        ticket: &'a cprog::FileTicket,
        control: &'a State<'a, CollectControl>,
        total: u64,
    ) -> Self {
        Self {
            rep,
            ticket,
            control,
            gate: ProgressGate::new(total),
            copied: 0,
        }
    }
    fn copied(&self) -> u64 {
        self.copied
    }
    fn tick(&mut self, done: u64) -> bool {
        self.copied = done;
        if self.gate.allow(done, std::time::Instant::now()) {
            self.rep.file(&self.ticket.downloading(done));
        }
        !self.cancelled()
    }
    fn cancelled(&self) -> bool {
        self.control
            .file_cancelled(self.rep.session_id(), self.ticket.key())
    }
}
