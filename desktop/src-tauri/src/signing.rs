// Copyright (c) 2026 DigiText. All rights reserved.
use futures_util::{FutureExt as _, StreamExt as _};
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use base64::Engine;
use digitext_core::Permission;
use serde::{Deserialize, Serialize};
use tauri::State;
const SHA256_DIGESTINFO_PREFIX: [u8; 19] = [
    0x30, 0x31, 0x30, 0x0d, 0x06, 0x09, 0x60, 0x86, 0x48, 0x01, 0x65, 0x03, 0x04, 0x02, 0x01, 0x05,
    0x00, 0x04, 0x20,
];
pub(crate) fn digestinfo_sha256(digest: &[u8]) -> Vec<u8> {
    let mut r_2oc = Vec::with_capacity(SHA256_DIGESTINFO_PREFIX.len() + digest.len());
    r_2oc.extend_from_slice(&SHA256_DIGESTINFO_PREFIX);
    r_2oc.extend_from_slice(digest);
    r_2oc
}
fn b64(r_2od: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(r_2od)
}
fn unb64(r_2oe: &str) -> Result<Vec<u8>, String> {
    base64::engine::general_purpose::STANDARD
        .decode(r_2oe)
        .map_err(|e| format!("base64 không hợp lệ: {e}"))
}
fn f_ug(r_2of: &[u8], r_2og: &[Vec<u8>]) -> serde_json::Value {
    begin_body(r_2of, r_2og, None, &SignStyle::default())
}
/// Hình thức ký cho cả lô: có/không dấu, ảnh con dấu (base64), có/không dấu thời gian (TSA).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignStyle {
    #[serde(default)]
    pub with_stamp: Option<bool>,
    #[serde(default)]
    pub stamp_png: Option<String>,
    #[serde(default)]
    pub timestamp: Option<bool>,
    /// Tên hiển thị trên khối chữ ký thay tên trong chứng thư (trống = tên trong chứng thư).
    #[serde(default)]
    pub signer_name: Option<String>,
}
/// Vị trí khối chữ ký chọn trên tệp đầu tiên, áp cho cả lô: góc trên-trái theo TỈ LỆ trang
/// (0..1, `y` từ mép trên); `last_page` = đặt ở trang cuối.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Placement {
    #[serde(default)]
    pub last_page: bool,
    pub x: f32,
    pub y: f32,
}
fn begin_body(
    cert: &[u8],
    inters: &[Vec<u8>],
    placement: Option<&Placement>,
    style: &SignStyle,
) -> serde_json::Value {
    let mut v = serde_json::json!({
        "certificate": b64(cert),
        "intermediates": inters.iter().map(|d| b64(d)).collect::<Vec<_>>(),
    });
    if let Some(p) = placement {
        v["placement"] = serde_json::json!({ "lastPage": p.last_page, "x": p.x, "y": p.y });
    }
    if let Some(w) = style.with_stamp {
        v["withStamp"] = serde_json::json!(w);
    }
    if let Some(png) = style.stamp_png.as_deref().filter(|s| !s.is_empty()) {
        v["stampPng"] = serde_json::json!(png);
    }
    if let Some(t) = style.signer_name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        v["signerName"] = serde_json::json!(t);
    }
    v
}
fn f_10o(session_id: &str, signature: &[u8]) -> serde_json::Value {
    complete_body(session_id, signature, &SignStyle::default())
}
fn complete_body(session_id: &str, signature: &[u8], style: &SignStyle) -> serde_json::Value {
    let mut v = serde_json::json!({ "sessionId" : session_id, "signature" : b64(signature) });
    if let Some(t) = style.timestamp {
        v["timestamp"] = serde_json::json!(t);
    }
    v
}
pub(crate) trait To9: Send {
    fn certificate(&self) -> &[u8];
    fn intermediates(&self) -> &[Vec<u8>];
    fn sign_digest(&self, sha256_digest: &[u8]) -> Result<Vec<u8>, String>;
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignJob {
    pub id: String,
    pub project_id: String,
    pub filename: String,
    #[serde(default)]
    pub ho_so_ref: Option<String>,
    #[serde(default)]
    pub rel_path: Option<String>,
    #[serde(default)]
    pub page_count: Option<i32>,
    pub status: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BeginResp {
    session_id: String,
    tbs_digest: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompleteResp {
    status: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Toa {
    pub index: usize,
    pub label: String,
}
pub(crate) const MODULE_ENV: &str = "DIGITEXT_PKCS11_MODULE";
pub(crate) fn f_1tj(r_2ol: Option<String>) -> Vec<String> {
    #[cfg(target_os = "windows")]
    const BUILTIN: &[&str] = &[
        r"C:\Windows\System32\vtcaPKCS11.dll",
        r"C:\Windows\System32\ica_csp11_v1.dll",
        r"C:\Windows\System32\eps2003csp11.dll",
        r"C:\Windows\System32\eTPKCS11.dll",
        r"C:\Windows\System32\opensc-pkcs11.dll",
        // Bit4id (TokenME EVO / Universal Middleware) — 13/10/2026 máy trạm cắm TokenME EVO v2 không dò được.
        r"C:\Windows\System32\bit4xpki.dll",
        r"C:\Windows\System32\bit4ipki.dll",
    ];
    #[cfg(target_os = "macos")]
    const BUILTIN: &[&str] = &[
        "/usr/local/lib/ica_csp11_v1.dylib",
        "/usr/local/lib/libvtcaPKCS11.dylib",
        "/usr/local/lib/libeps2003csp11.dylib",
        "/usr/local/lib/libeTPkcs11.dylib",
        "/Library/OpenSC/lib/opensc-pkcs11.so",
        "/opt/homebrew/lib/opensc-pkcs11.so",
        "/usr/local/lib/opensc-pkcs11.so",
    ];
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    const BUILTIN: &[&str] = &[
        "/usr/lib/vtca/libvtcaPKCS11.so",
        "/usr/lib/libica_csp11.so",
        "/usr/lib/libeps2003csp11.so",
        "/usr/lib/x86_64-linux-gnu/opensc-pkcs11.so",
        "/usr/lib64/opensc-pkcs11.so",
        "/usr/lib/opensc-pkcs11.so",
    ];
    let mut r_2om: Vec<String> = r_2ol
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .into_iter()
        .collect();
    r_2om.extend(BUILTIN.iter().map(|s| (*s).to_string()));
    r_2om
}
pub(crate) fn f_1ae(r_2oo: &[String], r_2op: impl Fn(&str) -> bool) -> Vec<String> {
    let mut r_2oq: Vec<String> = Vec::new();
    for r_2on in r_2oo {
        if r_2op(r_2on) && !r_2oq.contains(r_2on) {
            r_2oq.push(r_2on.clone());
        }
    }
    r_2oq
}
pub(crate) fn f_290(
    requested: &str,
    r_2os: &[String],
    r_2or: impl Fn(&str) -> bool,
) -> Result<String, String> {
    let requested = requested.trim();
    if requested.is_empty() {
        return r_2os
            .first()
            .cloned()
            .ok_or_else(|| {
                format!(
                    "không thấy thư viện PKCS#11 nào trên máy — cài phần mềm token của CA (iCA SmartToken, Viettel-CA, Bit4id TokenME…) rồi thử lại, hoặc nhập đường dẫn module (hoặc đặt biến môi trường {MODULE_ENV})"
                )
            });
    }
    if !r_2or(requested) {
        let goi_y = if r_2os.is_empty() {
            "không dò được module nào khác trên máy".to_string()
        } else {
            format!(
                "module dò được trên máy: {}",
                r_2os.join(", ")
            )
        };
        return Err(format!(
            "không thấy thư viện PKCS#11 tại '{requested}' — {goi_y}"
        ));
    }
    Ok(requested.to_string())
}
fn f_14z() -> Vec<String> {
    let mut r_2ot = f_1tj(std::env::var(MODULE_ENV).ok());
    r_2ot.extend(quet_module_pkcs11());
    f_1ae(&r_2ot, |p| std::path::Path::new(p).exists())
}

/// Tên tệp giống thư viện PKCS#11 của token (mỗi CA một tên: ica_csp11_v1, vtcaPKCS11, bit4xpki, eps2003csp11…).
fn giong_pkcs11(name: &str) -> bool {
    let n = name.to_ascii_lowercase();
    let Some(stem) = n.strip_suffix(".dll") else { return false };
    ["pkcs11", "pkcs#11", "csp11", "p11", "xpki", "ipki", "cryptoki"].iter().any(|k| stem.contains(k))
        && !["msvcp", "vcruntime", "api-ms-", "ucrt"].iter().any(|k| stem.starts_with(k))
        // p11-kit (kho chứng chỉ của Git/MSYS) không phải token; *_s.dll là thư viện phụ của iCA.
        && !stem.contains("p11-kit")
        && !stem.ends_with("_s")
}

/// Quét thư viện PKCS#11 do phần mềm token của CA cài: System32 (64 bit) và Program Files (sâu tối đa 4 cấp).
/// Không quét Program Files (x86) — thư viện 32 bit không nạp được vào DigiText 64 bit.
fn quet_module_pkcs11() -> Vec<String> {
    #[cfg(target_os = "windows")]
    {
        let mut out = Vec::new();
        if let Ok(rd) = std::fs::read_dir(r"C:\Windows\System32") {
            for e in rd.flatten() {
                if giong_pkcs11(&e.file_name().to_string_lossy()) {
                    out.push(e.path().to_string_lossy().into_owned());
                }
            }
        }
        let pf = std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".into());
        let mut st = vec![(std::path::PathBuf::from(pf), 0usize)];
        while let Some((d, depth)) = st.pop() {
            let Ok(rd) = std::fs::read_dir(&d) else { continue };
            for e in rd.flatten() {
                let p = e.path();
                if p.is_dir() {
                    let ten = e.file_name().to_string_lossy().to_ascii_lowercase();
                    if depth < 4 && !matches!(ten.as_str(), "git" | "windowsapps" | "common files" | "windows defender" | "nodejs" | "docker") {
                        st.push((p, depth + 1));
                    }
                } else if giong_pkcs11(&e.file_name().to_string_lossy()) {
                    out.push(p.to_string_lossy().into_owned());
                }
            }
        }
        out.sort();
        out
    }
    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobsArgs {
    #[serde(default)]
    pub project_id: Option<String>,
}
#[tauri::command]
pub async fn sign_list_jobs(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Option<JobsArgs>,
) -> Result<Vec<SignJob>, AppError> {
    // Chọn dự án → lấy cả hàng đợi của dự án đó (tới 5.000) để ký hàng loạt.
    let path = match args.and_then(|a| a.project_id).filter(|p| !p.trim().is_empty()) {
        Some(p) => format!("/sign/jobs?projectId={}&limit=200000", p.trim()),
        None => "/sign/jobs".to_string(),
    };
    f_2r8(
            &db,
            &auth,
            Permission::SignView,
            |r_2ou, token| {
                let path = path.clone();
                async move { client::f_1eo::<Vec<SignJob>>(&r_2ou, &token, &path).await }
            },
        )
        .await
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewArgs {
    pub job_id: String,
}
/// Bản PDF/A (base64) của một tài liệu chờ ký — để hiện trang và chọn vị trí chữ ký.
#[tauri::command]
pub async fn sign_preview_pdf(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: PreviewArgs,
) -> Result<String, AppError> {
    let path = format!("/documents/{}/download?variant=pdfa", args.job_id.trim());
    let bytes = f_2r8(&db, &auth, Permission::SignView, |base, token| {
        let path = path.clone();
        async move { client::f_1ei(&base, &token, &path).await }
    })
    .await?;
    Ok(b64(&bytes))
}
#[tauri::command]
pub async fn sign_detect_modules(auth: State<'_, AuthState>) -> Result<Vec<String>, AppError> {
    auth.require_perm(Permission::SignExecute)?;
    Ok(f_14z())
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Taz {
    #[serde(default)]
    pub module_path: String,
}
#[tauri::command]
pub async fn sign_list_tokens(auth: State<'_, AuthState>, args: Taz) -> Result<Vec<Toa>, AppError> {
    auth.require_perm(Permission::SignExecute)?;
    let module = f_290(&args.module_path, &f_14z(), |p| {
        std::path::Path::new(p).exists()
    })
    .map_err(AppError::validation)?;
    tauri::async_runtime::spawn_blocking(move || token::f_1oh(&module))
        .await
        .map_err(|e| AppError::internal(format!("luồng token lỗi: {e}")))?
        .map_err(AppError::validation)
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tlp {
    pub job_id: String,
    #[serde(default)]
    pub module_path: String,
    pub slot_index: usize,
    pub pin: String,
    #[serde(default)]
    pub placement: Option<Placement>,
}
#[tauri::command]
pub async fn sign_execute(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tlp,
) -> Result<String, AppError> {
    auth.require_perm(Permission::SignExecute)?;
    let Tlp {
        job_id,
        module_path,
        slot_index,
        pin,
        placement,
    } = args;
    let module_path = f_290(&module_path, &f_14z(), |p| std::path::Path::new(p).exists())
        .map_err(AppError::validation)?;
    let r_2ow = tauri::async_runtime::spawn_blocking(move || {
        token::Thb::open(&module_path, slot_index, &pin)
    })
    .await
    .map_err(|e| AppError::internal(format!("luồng token lỗi: {e}")))?
    .map_err(AppError::validation)?;
    let cert = r_2ow.certificate().to_vec();
    let inters = r_2ow.intermediates().to_vec();
    let begin: BeginResp = f_2r8(&db, &auth, Permission::SignExecute, |base, token| {
        let body = begin_body(&cert, &inters, placement.as_ref(), &SignStyle::default());
        let path = format!("/sign/{job_id}/begin");
        async move { client::f_23d(&base, &token, &path, &body).await }
    })
    .await?;
    let r_2ox = unb64(&begin.tbs_digest).map_err(AppError::validation)?;
    let signature = r_2ow.sign_digest(&r_2ox).map_err(AppError::validation)?;
    drop(r_2ow);
    let r_2ov: CompleteResp = f_2r8(&db, &auth, Permission::SignExecute, |base, token| {
        let body = f_10o(&begin.session_id, &signature);
        let path = format!("/sign/{job_id}/complete");
        async move { client::f_23d(&base, &token, &path, &body).await }
    })
    .await?;
    Ok(r_2ov.status)
}
static BATCH_CANCEL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Tạm dừng lô: ký xong tài liệu đang dở thì chờ, GIỮ phiên token (không phải nhập lại PIN).
static BATCH_PAUSE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
const EV_BATCH: &str = "sign://batch";
/// Dừng liền N tài liệu lỗi thì dừng cả lô (token rút ra / mạng đứt / máy chủ từ chối hàng loạt).
const BATCH_MAX_CONSECUTIVE_FAIL: usize = 5;
/// Số tài liệu chạy chồng nhau trong lô (13/10/2026): máy chủ chuẩn bị / lưu song song, token vẫn ký lần lượt.
const BATCH_PARALLEL: usize = 3;
/// Đường hầm Cloudflare rớt vài giây (502 / mất kết nối, 13/10/2026) → thử lại sau 2, 5, 10 giây trước khi báo lỗi.
// 14/10/2026: ổ HDD của kho khựng / mạng máy trạm chập chờn tới vài phút → chờ tổng ~6 phút rồi mới báo lỗi.
const THU_LAI_GIAY: [u64; 10] = [2, 5, 10, 20, 30, 60, 60, 60, 60, 60];
/// Sự kiện "đang chờ máy chủ / mạng" để màn Ký số ghi nhật ký trong lúc thử lại (không tính là lỗi).
const EV_WAIT: &str = "sign://wait";
/// Lỗi đường truyền tạm thời → `Ok(Err(..))` (thử lại được); lỗi thật của máy chủ giữ nguyên `Err`.
fn tach_loi_tam<T>(r: Result<T, client::ClientError>) -> Result<Result<T, String>, client::ClientError> {
    match r {
        Ok(v) => Ok(Ok(v)),
        Err(e @ client::ClientError::Unreachable(_)) => Ok(Err(crate::auth::admin::map_err(e).to_string())),
        Err(e @ client::ClientError::Api { status: 502..=504 | 520..=527, .. }) => Ok(Err(crate::auth::admin::map_err(e).to_string())),
        Err(e) => Err(e),
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchArgs {
    pub job_ids: Vec<String>,
    #[serde(default)]
    pub module_path: String,
    pub slot_index: usize,
    pub pin: String,
    #[serde(default)]
    pub placement: Option<Placement>,
    #[serde(default)]
    pub style: SignStyle,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchProgress {
    pub done: usize,
    pub total: usize,
    pub job_id: String,
    /// "signed" | "rejected" | "failed"
    pub status: String,
    pub error: Option<String>,
}
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResult {
    pub total: usize,
    pub signed: usize,
    pub rejected: usize,
    pub failed: usize,
    pub cancelled: usize,
    pub stopped_reason: Option<String>,
}
/// Ký hàng loạt: mở token + đăng nhập PIN MỘT lần, ký lần lượt từng tài liệu với cùng vị trí.
#[tauri::command]
pub async fn sign_batch(
    app: tauri::AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: BatchArgs,
) -> Result<BatchResult, AppError> {
    use tauri::Emitter;
    auth.require_perm(Permission::SignExecute)?;
    BATCH_CANCEL.store(false, std::sync::atomic::Ordering::SeqCst);
    BATCH_PAUSE.store(false, std::sync::atomic::Ordering::SeqCst);
    let BatchArgs { job_ids, module_path, slot_index, pin, placement, style } = args;
    let module_path = f_290(&module_path, &f_14z(), |p| std::path::Path::new(p).exists())
        .map_err(AppError::validation)?;
    tracing::info!(jobs = job_ids.len(), ?placement, "sign_batch: mở token");
    let tok = tauri::async_runtime::spawn_blocking(move || token::Thb::open(&module_path, slot_index, &pin))
        .await
        .map_err(|e| AppError::internal(format!("luồng token lỗi: {e}")))?
        .map_err(|e| {
            tracing::warn!(error = % e, "sign_batch: mở token / PIN lỗi");
            AppError::validation(e)
        })?;
    let cert = tok.certificate().to_vec();
    let inters = tok.intermediates().to_vec();
    // Phiên token (cryptoki Session) không Sync: khoá trong Mutex, C_Sign chạy ở luồng chặn — các tài liệu khác
    // vẫn chuẩn bị / lưu trên máy chủ trong lúc token ký.
    let tok = std::sync::Arc::new(std::sync::Mutex::new(tok));
    let (db, auth, cert, inters, placement, style) = (&db, &auth, &cert, &inters, &placement, &style);
    let app_ref = &app;
    let ky_mot = |job_id: String| {
        let tok = tok.clone();
        let id = job_id.clone();
        async move {
            let job_id = id;
            let mut lan = 0usize;
            let b: BeginResp = loop {
                if lan > 0 {
                    // Lần trước có thể đã mở phiên trên máy chủ dù mất phản hồi → trả phiên đó về chờ ký rồi mới begin lại.
                    let body = serde_json::json!({ "documentIds": [job_id] });
                    let _ = f_2r8(db, auth, Permission::SignExecute, |base, token| {
                        let body = body.clone();
                        async move { client::f_23d::<serde_json::Value>(&base, &token, "/sign/retry", &body).await }
                    })
                    .await;
                }
                let r = f_2r8(db, auth, Permission::SignExecute, |base, token| {
                    let body = begin_body(cert, inters, placement.as_ref(), style);
                    let path = format!("/sign/{job_id}/begin");
                    async move { tach_loi_tam(client::post_sign(&base, &token, &path, &body).await) }
                })
                .await
                .map_err(|e| e.to_string())?;
                match r {
                    Ok(b) => break b,
                    Err(e) if lan < THU_LAI_GIAY.len() => {
                        tracing::warn!(job = %job_id, error = %e, lan, "sign_batch: begin lỗi đường truyền — thử lại");
                        let _ = app_ref.emit(EV_WAIT, format!("Máy chủ/mạng chưa phản hồi ({e}) — thử lại sau {} giây", THU_LAI_GIAY[lan]));
                        tokio::time::sleep(std::time::Duration::from_secs(THU_LAI_GIAY[lan])).await;
                        lan += 1;
                    }
                    Err(e) => return Err(e),
                }
            };
            let digest = unb64(&b.tbs_digest)?;
            let signature = tauri::async_runtime::spawn_blocking(move || {
                tok.lock().map_err(|_| "phiên token bị hỏng".to_string())?.sign_digest(&digest)
            })
            .await
            .map_err(|e| format!("luồng token lỗi: {e}"))??;
            let session_id = b.session_id;
            let mut lan = 0usize;
            loop {
                let r = f_2r8(db, auth, Permission::SignExecute, |base, token| {
                    let body = complete_body(&session_id, &signature, style);
                    let path = format!("/sign/{job_id}/complete");
                    async move { tach_loi_tam(client::post_sign::<CompleteResp>(&base, &token, &path, &body).await) }
                })
                .await;
                match r {
                    Ok(Ok(c)) => return Ok(c.status),
                    Ok(Err(e)) if lan < THU_LAI_GIAY.len() => {
                        tracing::warn!(job = %job_id, error = %e, lan, "sign_batch: complete lỗi đường truyền — thử lại");
                        let _ = app_ref.emit(EV_WAIT, format!("Máy chủ/mạng chưa phản hồi ({e}) — thử lại sau {} giây", THU_LAI_GIAY[lan]));
                        tokio::time::sleep(std::time::Duration::from_secs(THU_LAI_GIAY[lan])).await;
                        lan += 1;
                    }
                    Ok(Err(e)) => return Err(e),
                    Err(e) if lan > 0 && e.to_string().contains("không còn mở") => {
                        return Err("đã gửi chữ ký nhưng mất kết nối lúc nhận kết quả; máy chủ đã xử lý phiên này \
                                    (thường là đã ký) — kiểm tra lại tệp"
                            .into())
                    }
                    Err(e) => return Err(e.to_string()),
                }
            }
        }
        .map(move |r: Result<String, String>| (job_id, r))
    };
    let total = job_ids.len();
    let mut out = BatchResult { total, ..Default::default() };
    let mut consecutive_fail = 0usize;
    let mut next = 0usize;
    let mut done = 0usize;
    let mut stop: Option<String> = None;
    let mut inflight = futures_util::stream::FuturesUnordered::new();
    loop {
        while stop.is_none() && next < total && inflight.len() < BATCH_PARALLEL {
            if BATCH_CANCEL.load(std::sync::atomic::Ordering::SeqCst) {
                stop = Some("người dùng dừng".into());
                break;
            }
            if BATCH_PAUSE.load(std::sync::atomic::Ordering::SeqCst) {
                break;
            }
            inflight.push(ky_mot(job_ids[next].clone()));
            next += 1;
        }
        if inflight.is_empty() {
            if stop.is_some() || next >= total {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            continue;
        }
        let Some((job_id, result)) = inflight.next().await else { continue };
        done += 1;
        let (status, error) = match result {
            Ok(s) if s == "signed" => {
                out.signed += 1;
                consecutive_fail = 0;
                ("signed".to_string(), None)
            }
            Ok(s) => {
                out.rejected += 1;
                consecutive_fail += 1;
                ("rejected".to_string(), Some(format!("máy chủ từ chối chữ ký (trạng thái: {s})")))
            }
            Err(e) => {
                out.failed += 1;
                consecutive_fail += 1;
                ("failed".to_string(), Some(e))
            }
        };
        if let Some(e) = &error {
            tracing::warn!(job = % job_id, error = % e, "sign_batch: tài liệu lỗi");
        }
        let _ = app.emit(EV_BATCH, BatchProgress { done, total, job_id, status, error });
        if consecutive_fail >= BATCH_MAX_CONSECUTIVE_FAIL && stop.is_none() {
            stop = Some(format!(
                "{BATCH_MAX_CONSECUTIVE_FAIL} tài liệu liền nhau lỗi — dừng để kiểm tra token/mạng/máy chủ"
            ));
        }
    }
    if stop.is_some() {
        out.cancelled = total - next;
        out.stopped_reason = stop;
    }
    drop(inflight);
    drop(tok);
    tracing::info!(?out, "sign_batch");
    Ok(out)
}
#[tauri::command]
pub async fn sign_batch_cancel(auth: State<'_, AuthState>) -> Result<(), AppError> {
    auth.require_perm(Permission::SignExecute)?;
    BATCH_CANCEL.store(true, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}
#[tauri::command]
pub async fn sign_batch_pause(auth: State<'_, AuthState>, paused: bool) -> Result<(), AppError> {
    auth.require_perm(Permission::SignExecute)?;
    BATCH_PAUSE.store(paused, std::sync::atomic::Ordering::SeqCst);
    Ok(())
}
/// Đọc ảnh con dấu trên máy (PNG/JPEG ≤ 500 KB) → base64 để xem trước và gửi kèm khi ký.
#[tauri::command]
pub async fn sign_load_stamp(auth: State<'_, AuthState>, path: String) -> Result<String, AppError> {
    auth.require_perm(Permission::SignExecute)?;
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|e| AppError::validation(format!("không đọc được ảnh con dấu: {e}")))?;
    if bytes.len() > 500 * 1024 {
        return Err(AppError::validation(
            "ảnh con dấu quá lớn (tối đa 500 KB) — thu nhỏ ảnh rồi chọn lại",
        ));
    }
    image::load_from_memory(&bytes)
        .map_err(|e| AppError::validation(format!("tệp không phải ảnh PNG/JPEG hợp lệ: {e}")))?;
    Ok(b64(&bytes))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertInfo {
    pub subject: String,
    pub organization: String,
    pub issuer: String,
    pub serial: String,
    /// SHA-1 của chứng thư (hex in hoa) — "Thumbprint" như Windows hiển thị.
    pub thumbprint: String,
    pub not_after: String,
}
fn rdn_value(name: &x509_cert::name::Name, oid: der::asn1::ObjectIdentifier) -> Option<String> {
    name.0
        .iter()
        .flat_map(|rdn| rdn.0.iter())
        .find(|a| a.oid == oid)
        .and_then(|a| {
            let v = &a.value;
            der::asn1::Utf8StringRef::try_from(v)
                .map(|s| s.as_str().to_string())
                .or_else(|_| der::asn1::PrintableStringRef::try_from(v).map(|s| s.as_str().to_string()))
                .ok()
        })
}
pub(crate) fn cert_info(der_bytes: &[u8]) -> Result<CertInfo, String> {
    use sha1::Digest as _;
    let cert = Certificate::from_der(der_bytes).map_err(|e| format!("chứng thư không đọc được: {e}"))?;
    let tbs = &cert.tbs_certificate;
    const CN: der::asn1::ObjectIdentifier = der::asn1::ObjectIdentifier::new_unwrap("2.5.4.3");
    const O: der::asn1::ObjectIdentifier = der::asn1::ObjectIdentifier::new_unwrap("2.5.4.10");
    let thumbprint: String = sha1::Sha1::digest(der_bytes).iter().map(|b| format!("{b:02X}")).collect();
    Ok(CertInfo {
        subject: rdn_value(&tbs.subject, CN).unwrap_or_else(|| tbs.subject.to_string()),
        organization: rdn_value(&tbs.subject, O).unwrap_or_default(),
        issuer: rdn_value(&tbs.issuer, CN).unwrap_or_else(|| tbs.issuer.to_string()),
        serial: tbs.serial_number.as_bytes().iter().map(|b| format!("{b:02X}")).collect(),
        thumbprint,
        not_after: tbs.validity.not_after.to_string(),
    })
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertArgs {
    #[serde(default)]
    pub module_path: String,
    pub slot_index: usize,
    pub pin: String,
}
/// Mở token bằng PIN và đọc chứng thư sẽ dùng để ký (kiểm tra PIN trước khi chạy lô).
#[tauri::command]
pub async fn sign_cert_info(auth: State<'_, AuthState>, args: CertArgs) -> Result<CertInfo, AppError> {
    auth.require_perm(Permission::SignExecute)?;
    let module_path = f_290(&args.module_path, &f_14z(), |p| std::path::Path::new(p).exists())
        .map_err(AppError::validation)?;
    let (slot, pin) = (args.slot_index, args.pin);
    let cert = tauri::async_runtime::spawn_blocking(move || {
        token::Thb::open(&module_path, slot, &pin).map(|t| t.certificate().to_vec())
    })
    .await
    .map_err(|e| AppError::internal(format!("luồng token lỗi: {e}")))?
    .map_err(AppError::validation)?;
    cert_info(&cert).map_err(AppError::validation)
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryResp {
    pub requeued: usize,
    pub skipped: usize,
}
/// "Ký lại các file lỗi": trả các tài liệu ký lỗi về hàng chờ ký trên máy chủ.
#[tauri::command]
pub async fn sign_retry(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    document_ids: Vec<String>,
) -> Result<RetryResp, AppError> {
    auth.require_perm(Permission::SignExecute)?;
    let body = serde_json::json!({ "documentIds": document_ids });
    f_2r8(&db, &auth, Permission::SignExecute, |base, token| {
        let body = body.clone();
        async move { client::f_23d(&base, &token, "/sign/retry", &body).await }
    })
    .await
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsignResp {
    pub reverted: usize,
    pub skipped: usize,
}
/// Xoá các bản đã ký đã tải về thư mục lưu trong lô bị huỷ. Chỉ xoá tệp .pdf nằm TRONG `out_dir` (thư mục
/// người dùng chọn ở màn Ký số); trả số tệp đã xoá.
#[tauri::command]
pub async fn sign_remove_local(auth: State<'_, AuthState>, out_dir: String, paths: Vec<String>) -> Result<usize, AppError> {
    auth.require_perm(Permission::SignExecute)?;
    let root = std::fs::canonicalize(out_dir.trim()).map_err(|e| AppError::validation(format!("thư mục lưu không hợp lệ: {e}")))?;
    let mut n = 0;
    for p in paths {
        let Ok(f) = std::fs::canonicalize(&p) else { continue };
        let pdf = f.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf"));
        if pdf && f.starts_with(&root) && f.is_file() && std::fs::remove_file(&f).is_ok() {
            n += 1;
        }
    }
    Ok(n)
}
/// "Huỷ" lô ký: các tài liệu vừa ký trong lô quay về chưa ký (máy chủ xoá bản đã ký, đưa về hàng chờ).
#[tauri::command]
pub async fn sign_unsign(db: State<'_, Db>, auth: State<'_, AuthState>, document_ids: Vec<String>) -> Result<UnsignResp, AppError> {
    auth.require_perm(Permission::SignExecute)?;
    let body = serde_json::json!({ "documentIds": document_ids });
    f_2r8(&db, &auth, Permission::SignExecute, |base, token| {
        let body = body.clone();
        async move { client::post_long(&base, &token, "/sign/unsign", &body).await }
    })
    .await
}

use der::{oid::AssociatedOid, Decode, Encode};
use x509_cert::ext::pkix::{BasicConstraints, KeyUsage};
use x509_cert::Certificate;
#[derive(Clone, Debug)]
pub(crate) struct CertMeta {
    pub idx: usize,
    pub subject_der: Vec<u8>,
    pub issuer_der: Vec<u8>,
    pub not_before: i64,
    pub not_after: i64,
    pub can_sign: bool,
    pub is_ca: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CertSelection {
    pub leaf: usize,
    pub intermediates: Vec<usize>,
}
fn cert_extension(cert: &Certificate, r_2oz: der::asn1::ObjectIdentifier) -> Option<Vec<u8>> {
    let r_2oy = cert.tbs_certificate.extensions.as_ref()?;
    r_2oy
        .iter()
        .find(|e| e.extn_id == r_2oz)
        .map(|e| e.extn_value.as_bytes().to_vec())
}
fn key_usage_allows_signing(cert: &Certificate) -> bool {
    match cert_extension(cert, KeyUsage::OID) {
        None => true,
        Some(r_2p0) => r_2p0.get(3).copied().unwrap_or(0) & 0b1100_0000 != 0,
    }
}
fn basic_constraints_ca(cert: &Certificate) -> bool {
    match cert_extension(cert, BasicConstraints::OID) {
        None => false,
        Some(r_2p1) => r_2p1.windows(3).any(|r_2p2| r_2p2 == [0x01, 0x01, 0xFF]),
    }
}
pub(crate) fn parse_cert_meta(idx: usize, r_2p3: &[u8]) -> Option<CertMeta> {
    let cert = Certificate::from_der(r_2p3).ok()?;
    let r_2p4 = &cert.tbs_certificate;
    Some(CertMeta {
        idx,
        subject_der: r_2p4.subject.to_der().ok()?,
        issuer_der: r_2p4.issuer.to_der().ok()?,
        not_before: r_2p4.validity.not_before.to_unix_duration().as_secs() as i64,
        not_after: r_2p4.validity.not_after.to_unix_duration().as_secs() as i64,
        can_sign: key_usage_allows_signing(&cert),
        is_ca: basic_constraints_ca(&cert),
    })
}
pub(crate) fn select_signer(r_2p6: &[CertMeta], r_2p7: i64) -> Option<CertSelection> {
    if r_2p6.is_empty() {
        return None;
    }
    let r_2pa = |m: &CertMeta| (r_2p7 >= m.not_before && r_2p7 <= m.not_after, m.not_before);
    let r_2p5 = |r_2p9: &dyn Fn(&CertMeta) -> bool| {
        r_2p6
            .iter()
            .filter(|m| r_2p9(m))
            .max_by(|a, b| r_2pa(a).cmp(&r_2pa(b)))
    };
    let leaf = r_2p5(&|m| !m.is_ca && m.can_sign)
        .or_else(|| r_2p5(&|m| !m.is_ca))
        .or_else(|| r_2p5(&|_| true))?;
    let mut intermediates = Vec::new();
    let mut r_2pb = vec![leaf.idx];
    let mut r_2pc = leaf.issuer_der.clone();
    while let Some(r_2p8) = r_2p6
        .iter()
        .find(|m| m.is_ca && m.subject_der == r_2pc && !r_2pb.contains(&m.idx))
    {
        intermediates.push(r_2p8.idx);
        r_2pb.push(r_2p8.idx);
        if r_2p8.subject_der == r_2p8.issuer_der {
            break;
        }
        r_2pc = r_2p8.issuer_der.clone();
    }
    Some(CertSelection {
        leaf: leaf.idx,
        intermediates,
    })
}
mod token {
    use super::{To9, Toa};
    use cryptoki::context::{CInitializeArgs, CInitializeFlags, Pkcs11};
    use cryptoki::mechanism::Mechanism;
    use cryptoki::object::{Attribute, AttributeType, ObjectClass, ObjectHandle};
    use cryptoki::session::{Session, UserType};
    use cryptoki::types::AuthPin;
    fn context(module_path: &str) -> Result<Pkcs11, String> {
        let pkcs11 = Pkcs11::new(module_path).map_err(|e| {
            format!("không nạp được module PKCS#11 '{module_path}': {e}")
        })?;
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
    pub fn f_1oh(module_path: &str) -> Result<Vec<Toa>, String> {
        let pkcs11 = context(module_path)?;
        let r_2pd = pkcs11
            .get_slots_with_token()
            .map_err(|e| format!("liệt kê slot lỗi: {e}"))?;
        Ok(r_2pd
            .iter()
            .enumerate()
            .map(|(index, slot)| {
                let label = pkcs11
                    .get_token_info(*slot)
                    .map(|r_2pe| r_2pe.label().trim().to_string())
                    .unwrap_or_default();
                Toa { index, label }
            })
            .collect())
    }
    fn attr_bytes(session: &Session, r_2pg: ObjectHandle, r_2ph: AttributeType) -> Option<Vec<u8>> {
        let r_2pf = session.get_attributes(r_2pg, &[r_2ph]).ok()?;
        r_2pf.into_iter().find_map(|a| match a {
            Attribute::Value(v) if r_2ph == AttributeType::Value => Some(v),
            Attribute::Id(v) if r_2ph == AttributeType::Id => Some(v),
            _ => None,
        })
    }
    pub struct Thb {
        _pkcs11: Pkcs11,
        session: Session,
        key: ObjectHandle,
        cert: Vec<u8>,
        intermediates: Vec<Vec<u8>>,
    }
    impl Thb {
        pub fn open(module_path: &str, slot_index: usize, pin: &str) -> Result<Self, String> {
            let pkcs11 = context(module_path)?;
            let r_2pr = pkcs11
                .get_slots_with_token()
                .map_err(|e| format!("liệt kê slot lỗi: {e}"))?;
            let slot = *r_2pr
                .get(slot_index)
                .ok_or_else(|| format!("không có token ở slot {slot_index}"))?;
            let session = pkcs11
                .open_rw_session(slot)
                .map_err(|e| format!("mở phiên token lỗi: {e}"))?;
            session
                .login(UserType::User, Some(&AuthPin::new(pin.into())))
                .map_err(|_| {
                    "PIN không đúng hoặc token bị khoá".to_string()
                })?;
            let r_2pi = session
                .find_objects(&[Attribute::Class(ObjectClass::CERTIFICATE)])
                .map_err(|e| format!("tìm cert lỗi: {e}"))?;
            let mut r_2pj: Vec<(Vec<u8>, Option<Vec<u8>>)> = Vec::new();
            for r_2pk in r_2pi {
                if let Some(der) = attr_bytes(&session, r_2pk, AttributeType::Value) {
                    let id = attr_bytes(&session, r_2pk, AttributeType::Id);
                    r_2pj.push((der, id));
                }
            }
            if r_2pj.is_empty() {
                return Err("token không có chứng thư (certificate)".into());
            }
            let r_2pn: Vec<super::CertMeta> = r_2pj
                .iter()
                .enumerate()
                .filter_map(|(i, (der, _))| super::parse_cert_meta(i, der))
                .collect();
            let r_2po = chrono::Utc::now().timestamp();
            let r_2pp = super::select_signer(&r_2pn, r_2po).ok_or(
                "không chọn được chứng thư người ký trên token",
            )?;
            let cert = r_2pj[r_2pp.leaf].0.clone();
            let r_2pm = r_2pj[r_2pp.leaf].1.clone();
            let intermediates: Vec<Vec<u8>> = r_2pp
                .intermediates
                .iter()
                .map(|&i| r_2pj[i].0.clone())
                .collect();
            let r_2pq = session
                .find_objects(&[
                    Attribute::Class(ObjectClass::PRIVATE_KEY),
                    Attribute::Sign(true),
                ])
                .map_err(|e| format!("tìm khoá ký lỗi: {e}"))?;
            let key = r_2pm
                .as_deref()
                .and_then(|r_2ps| {
                    r_2pq.iter().copied().find(|r_2pl| {
                        attr_bytes(&session, *r_2pl, AttributeType::Id).as_deref() == Some(r_2ps)
                    })
                })
                .or_else(|| r_2pq.first().copied())
                .ok_or("token không có khoá ký (CKA_SIGN)")?;
            Ok(Self {
                _pkcs11: pkcs11,
                session,
                key,
                cert,
                intermediates,
            })
        }
    }
    impl To9 for Thb {
        fn certificate(&self) -> &[u8] {
            &self.cert
        }
        fn intermediates(&self) -> &[Vec<u8>] {
            &self.intermediates
        }
        fn sign_digest(&self, r_2pu: &[u8]) -> Result<Vec<u8>, String> {
            let r_2pt = super::digestinfo_sha256(r_2pu);
            self.session
                .sign(&Mechanism::RsaPkcs, self.key, &r_2pt)
                .map_err(|e| format!("C_Sign trên token lỗi: {e}"))
        }
    }
}

#[cfg(test)]
mod quet_pkcs11_tests {
    use super::giong_pkcs11;
    #[test]
    fn nhan_dung_thu_vien_token() {
        for ok in ["ica_csp11_v1.dll", "vtcaPKCS11.dll", "bit4xpki.dll", "bit4ipki.dll", "eps2003csp11.dll", "eTPKCS11.dll", "opensc-pkcs11.dll"] {
            assert!(giong_pkcs11(ok), "{ok}");
        }
        for no in ["ica_csp11_v1_s.dll", "libp11-kit-0.dll", "p11-kit-trust.dll", "msvcp110.dll", "kernel32.dll", "bit4xpki.exe"] {
            assert!(!giong_pkcs11(no), "{no}");
        }
    }
}
