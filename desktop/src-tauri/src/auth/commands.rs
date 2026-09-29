// Copyright (c) 2026 DigiText. All rights reserved.
use super::captcha::{CaptchaStore, T2j};
use super::client::{self, ClientError};
use super::session::{SessionMode, Tr};
use super::store::{self, Tb, Tc, C_JX, C_JZ, C_K0};
use super::verifier::{self, Tkz};
use super::{AuthState, Td};
use crate::db::{self, Db};
use crate::error::AppError;
use crate::scanner::Tkx;
use chrono::Utc;
use digitext_core::{f_18s, Permission, PermissionOverrides};
use serde::Deserialize;
use tauri::{AppHandle, State};
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tbc {
    pub username: String,
    pub password: String,
    pub captcha_id: String,
    pub captcha_answer: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tp2 {
    pub password: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tlh {
    pub url: Option<String>,
}
fn device_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .ok()
        .filter(|r_hn| !r_hn.trim().is_empty())
        .unwrap_or_else(|| "máy trạm".to_string())
}
fn platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}
fn f_2ds(r_ho: &Tb, r_hp: SessionMode, locked: bool, r_hq: Vec<Permission>) -> Tr {
    Tr {
        user_id: r_ho.user_id.clone(),
        username: r_ho.username.clone(),
        display_name: r_ho.display_name.clone(),
        role: r_ho.role,
        project_id: r_ho.project_id.clone(),
        project_code: r_ho.project_code.clone(),
        project_name: r_ho.project_name.clone(),
        mode: r_hp,
        offline_expiry: r_ho.offline_expiry.clone(),
        locked,
        permissions: r_hq,
    }
}
fn f_18m(r_hs: &client::UserProfile) -> Vec<Permission> {
    let r_hr = PermissionOverrides {
        grants: r_hs.grants.iter().copied().collect(),
        denies: r_hs.denies.iter().copied().collect(),
    };
    f_18s(r_hs.role, &r_hr.sanitized()).into_iter().collect()
}
async fn f_14l(password: String, r_ht: Vec<u8>) -> Result<verifier::T4l, AppError> {
    tauri::async_runtime::spawn_blocking(move || verifier::derive(&password, &r_ht))
        .await
        .map_err(|e| AppError::internal(format!("luồng derive lỗi: {e}")))?
}
#[tauri::command]
pub fn auth_captcha_new(captcha: State<'_, CaptchaStore>) -> T2j {
    captcha.issue()
}
fn f_28f(r_hv: &State<'_, CaptchaStore>, r_hu: &Tbc) -> Result<(), AppError> {
    if r_hv.verify(&r_hu.captcha_id, &r_hu.captcha_answer) {
        return Ok(());
    }
    Err(AppError::new(
        "captcha_invalid",
        "Mã xác nhận không đúng — nhập lại mã trong ảnh.",
    ))
}
#[tauri::command]
pub async fn auth_login(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    captcha: State<'_, CaptchaStore>,
    args: Tbc,
) -> Result<Tr, AppError> {
    f_28f(&captcha, &args)?;
    let r_hw = f_1p8(&db, &auth, args).await?;
    f_2ha(&app, &r_hw);
    Ok(r_hw)
}
#[tauri::command]
pub async fn auth_switch_user(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    scanner: State<'_, Tkx>,
    captcha: State<'_, CaptchaStore>,
    args: Tbc,
) -> Result<Tr, AppError> {
    if scanner.has_active_batches() {
        return Err(
            AppError::new(
                "busy",
                "Đang quét dở một lô — kết thúc lô quét trước khi đổi người dùng.",
            ),
        );
    }
    f_28f(&captcha, &args)?;
    let r_hx = f_1p8(&db, &auth, args).await?;
    f_2ha(&app, &r_hx);
    Ok(r_hx)
}
fn f_2ha(r_hy: &AppHandle, r_hz: &Tr) {
    if r_hz.role.allows(digitext_core::Perm::Operate) && crate::data_share::f_2h9(r_hy).is_err() {
        tracing::warn!(
            "không bật được chia sẻ dữ liệu LAN sau đăng nhập"
        );
    }
}
async fn f_1p8(
    r_i4: &State<'_, Db>,
    r_i3: &State<'_, AuthState>,
    r_i2: Tbc,
) -> Result<Tr, AppError> {
    let username = r_i2.username.trim().to_owned();
    if username.is_empty() || r_i2.password.is_empty() {
        return Err(AppError::validation(
            "Nhập tên đăng nhập và mật khẩu.",
        ));
    }
    let r_i7 = Utc::now();
    let (server_url, r_i5, r_i8) = {
        let guard = r_i4.0.lock().map_err(|_| AppError::DbPoisoned)?;
        store::f_rg(&guard, r_i7)?;
        (
            db::f_1ew(&guard, C_K0)?.filter(|r_ic| !r_ic.trim().is_empty()),
            db::f_1ew(&guard, C_JX)?,
            db::f_1ew(&guard, C_JZ)?,
        )
    };
    if let Some(url) = server_url.as_deref() {
        let r_i1 = env!("CARGO_PKG_VERSION");
        let r_i6 = device_name();
        let r_ia = client::LoginRequest {
            username: &username,
            password: &r_i2.password,
            device_id: r_i5.as_deref(),
            device_name: &r_i6,
            platform: platform(),
            app_version: r_i1,
        };
        match client::login(url, &r_ia).await {
            Ok(r_ib) => {
                return f_10p(r_i4, r_i3, url, r_ib, &r_i2.password, r_i8).await;
            }
            Err(e @ ClientError::Api { .. }) if e.is_revoked() => {
                let guard = r_i4.0.lock().map_err(|_| AppError::DbPoisoned)?;
                if let Some(r_i0) = store::f_1ea(&guard, &username)? {
                    store::f_13t(&guard, &r_i0.user_id)?;
                }
                let ClientError::Api { code, message, .. } = e else {
                    unreachable!()
                };
                let code: &'static str = if code == "device_revoked" {
                    "device_revoked"
                } else {
                    "revoked"
                };
                return Err(AppError::new(code, message));
            }
            Err(ClientError::Api { status, .. }) if status >= 500 => {
                tracing::info!(
                    status,
                    "máy chủ lỗi 5xx — thử đăng nhập offline"
                );
            }
            Err(ClientError::Api {
                status,
                code,
                message,
            }) => {
                return Err(match code.as_str() {
                    "auth_failed" => AppError::new("auth_failed", message),
                    "too_many_attempts" => AppError::new("too_many_attempts", message),
                    _ => AppError::new(
                        "error",
                        format!("Máy chủ trả lỗi {status}: {message}"),
                    ),
                });
            }
            Err(ClientError::Unreachable(r_i9)) => {
                tracing::info!(
                    % r_i9,
                    "server không tới được — thử đăng nhập offline"
                );
            }
        }
    }
    f_1vy(r_i4, r_i3, &username, r_i2.password, server_url.is_some()).await
}
async fn f_10p(
    r_ig: &State<'_, Db>,
    r_ie: &State<'_, AuthState>,
    url: &str,
    r_ip: client::SessionResponse,
    password: &str,
    r_in: Option<String>,
) -> Result<Tr, AppError> {
    let mut r_io = match r_in {
        Some(p) => p,
        None => client::f_1c8(url).await.map_err(|e| {
            AppError::new(
                "server_unreachable",
                format!(
                    "Không lấy được khoá server: {}",
                    e.code()
                ),
            )
        })?,
    };
    let r_ih = r_ip.device_id.as_deref();
    let r_if = match verifier::f_2pv(&r_ip.offline_lease, &r_io, &r_ip.user.id, r_ih) {
        Ok(c) => c,
        Err(_) => {
            let r_ij = client::f_1c8(url).await.map_err(|_| {
                AppError::new(
                    "server_mismatch",
                    "Không xác minh được chữ ký máy chủ.",
                )
            })?;
            let c = verifier::f_2pv(&r_ip.offline_lease, &r_ij, &r_ip.user.id, r_ih)
                .map_err(|_| {
                    AppError::new(
                        "server_mismatch",
                        "Chữ ký máy chủ không hợp lệ — kiểm tra lại địa chỉ server.",
                    )
                })?;
            r_io = r_ij;
            c
        }
    };
    if !verifier::f_1mr(&r_if, r_ip.user.role) {
        return Err(AppError::new(
            "server_mismatch",
            "Máy chủ trả vai trò không khớp giấy phép đã ký.",
        ));
    }
    let r_iq = verifier::f_1u7();
    let r_il = f_14l(password.to_owned(), r_iq.to_vec()).await?;
    let r_is = serde_json::to_vec(&Tkz {
        refresh_token: r_ip.refresh_token.clone(),
    })
    .map_err(|e| AppError::internal(format!("đóng gói secrets lỗi: {e}")))?;
    let r_ir = verifier::f_2cn(&r_il.wrap_key, &r_is)?;
    let r_im = Utc::now();
    let r_id = Tb {
        user_id: r_ip.user.id.clone(),
        username: r_ip.user.username.clone(),
        display_name: r_ip.user.display_name.clone(),
        role: r_ip.user.role,
        project_id: r_ip.user.project.as_ref().map(|p| p.id.clone()),
        project_code: r_ip.user.project.as_ref().map(|p| p.code.clone()),
        project_name: r_ip.user.project.as_ref().map(|p| p.name.clone()),
        salt: r_iq.to_vec(),
        verifier: r_il.verifier_hex.clone(),
        secrets: r_ir,
        lease_jwt: r_ip.offline_lease.clone(),
        offline_expiry: r_ip.offline_expiry.clone(),
        last_online_at: r_im.to_rfc3339(),
    };
    {
        let r_ik = r_ig.0.lock().map_err(|_| AppError::DbPoisoned)?;
        store::f_2nx(&r_ik, &r_id)?;
        store::f_2dx(&r_ik, &r_id.user_id, false)?;
        db::f_2ei(&r_ik, C_JZ, Some(&r_io))?;
        if let Some(r_ii) = r_ip.device_id.as_deref() {
            db::f_2ei(&r_ik, C_JX, Some(r_ii))?;
        }
        store::f_rg(&r_ik, r_im)?;
    }
    let r_it = f_2ds(&r_id, SessionMode::Online, false, f_18m(&r_ip.user));
    r_ie.set(Some(Td {
        session: r_it.clone(),
        wrap_key: Some(r_il.wrap_key),
        access_token: Some(r_ip.access_token.clone()),
    }));
    tracing::info!(
        user = % r_it.username, "đăng nhập online thành công"
    );
    Ok(r_it)
}
async fn f_1vy(
    r_iw: &State<'_, Db>,
    r_iu: &State<'_, AuthState>,
    username: &str,
    password: String,
    r_j2: bool,
) -> Result<Tr, AppError> {
    let (account, pubkey, r_ix, r_j0) = {
        let guard = r_iw.0.lock().map_err(|_| AppError::DbPoisoned)?;
        (
            store::f_1ea(&guard, username)?,
            db::f_1ew(&guard, C_JZ)?,
            db::f_1ew(&guard, C_JX)?,
            store::max_seen_time(&guard)?,
        )
    };
    let Some(account) = account else {
        return Err(if r_j2 {
            AppError::new(
                    "server_unreachable",
                    "Không kết nối được máy chủ và tài khoản này chưa từng đăng nhập trên máy — cần mạng cho lần đầu.",
                )
        } else {
            AppError::new(
                    "server_not_configured",
                    "Chưa cấu hình địa chỉ máy chủ — vào biểu tượng ⚙ ở màn đăng nhập.",
                )
        });
    };
    let r_iz = f_14l(password, account.salt.clone()).await?;
    if !verifier::f_2pm(&r_iz, &account.verifier) {
        return Err(AppError::new(
            "auth_failed",
            "tên đăng nhập hoặc mật khẩu không đúng",
        ));
    }
    let pubkey = pubkey.ok_or_else(|| {
        AppError::new(
            "auth_cache_corrupt",
            "Thiếu khoá server đã pin — đăng nhập online lại.",
        )
    })?;
    let r_iv = verifier::f_2pv(
        &account.lease_jwt,
        &pubkey,
        &account.user_id,
        r_ix.as_deref(),
    )?;
    if !verifier::f_1mr(&r_iv, account.role) {
        return Err(
            AppError::new(
                "auth_cache_corrupt",
                "Phiên không khớp giấy phép đã ký — đăng nhập online lại.",
            ),
        );
    }
    let r_j1 = Utc::now();
    let r_iy = chrono::DateTime::parse_from_rfc3339(&account.offline_expiry)
        .map_err(|_| {
            AppError::new(
                "auth_cache_corrupt",
                "Hạn offline trong cache không hợp lệ.",
            )
        })?
        .with_timezone(&Utc);
    if !verifier::f_1vz(r_iy, r_j1, r_j0) {
        return Err(
            AppError::new(
                "offline_expired",
                "Quá 30 ngày chưa xác thực online — kết nối mạng và đăng nhập lại.",
            ),
        );
    }
    {
        let guard = r_iw.0.lock().map_err(|_| AppError::DbPoisoned)?;
        store::f_2dx(&guard, &account.user_id, false)?;
        store::f_rg(&guard, r_j1)?;
    }
    let r_j3 = f_2ds(
        &account,
        SessionMode::Offline,
        false,
        verifier::f_18p(&r_iv),
    );
    r_iu.set(Some(Td {
        session: r_j3.clone(),
        wrap_key: Some(r_iz.wrap_key),
        access_token: None,
    }));
    tracing::info!(
        user = % r_j3.username, "đăng nhập offline thành công"
    );
    Ok(r_j3)
}
#[tauri::command]
pub fn auth_lock(db: State<'_, Db>, auth: State<'_, AuthState>) -> Result<(), AppError> {
    let r_j4 = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    store::f_2eh(&r_j4, true)?;
    auth.set_locked(true);
    Ok(())
}
#[tauri::command]
pub async fn auth_unlock(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    args: Tp2,
) -> Result<Tr, AppError> {
    let r_jf = match auth.snapshot() {
        Some(r_jc) => r_jc.user_id,
        None => {
            let guard = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
            store::f_1ec(&guard)?
                .map(|(r_je, _)| r_je)
                .ok_or_else(|| AppError::new("unauthorized", "Chưa đăng nhập."))?
        }
    };
    let (r_j5, pubkey, r_j7, r_ja) = {
        let guard = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        (
            store::f_1eb(&guard, &r_jf)?.ok_or_else(|| {
                AppError::new(
                    "unauthorized",
                    "Tài khoản không còn trên máy.",
                )
            })?,
            db::f_1ew(&guard, C_JZ)?,
            db::f_1ew(&guard, C_JX)?,
            store::max_seen_time(&guard)?,
        )
    };
    let r_j9 = f_14l(args.password, r_j5.salt.clone()).await?;
    if !verifier::f_2pm(&r_j9, &r_j5.verifier) {
        return Err(AppError::new(
            "auth_failed",
            "Mật khẩu không đúng.",
        ));
    }
    let pubkey = pubkey.ok_or_else(|| {
        AppError::new(
            "auth_cache_corrupt",
            "Thiếu khoá server đã pin — đăng nhập online lại.",
        )
    })?;
    let r_j6 = verifier::f_2pv(&r_j5.lease_jwt, &pubkey, &r_j5.user_id, r_j7.as_deref())?;
    if !verifier::f_1mr(&r_j6, r_j5.role) {
        return Err(
            AppError::new(
                "auth_cache_corrupt",
                "Phiên không khớp giấy phép đã ký — đăng nhập online lại.",
            ),
        );
    }
    let r_jb = Utc::now();
    let r_j8 = chrono::DateTime::parse_from_rfc3339(&r_j5.offline_expiry)
        .map_err(|_| {
            AppError::new(
                "auth_cache_corrupt",
                "Hạn offline trong cache không hợp lệ.",
            )
        })?
        .with_timezone(&Utc);
    if !verifier::f_1vz(r_j8, r_jb, r_ja) {
        return Err(
            AppError::new(
                "offline_expired",
                "Quá 30 ngày chưa xác thực online — kết nối mạng và đăng nhập lại.",
            ),
        );
    }
    {
        let guard = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
        store::f_2eh(&guard, false)?;
        store::f_rg(&guard, r_jb)?;
    }
    let r_jd = f_2ds(&r_j5, SessionMode::Offline, false, verifier::f_18p(&r_j6));
    auth.set(Some(Td {
        session: r_jd.clone(),
        wrap_key: Some(r_j9.wrap_key),
        access_token: None,
    }));
    f_2ha(&app, &r_jd);
    Ok(r_jd)
}
#[tauri::command]
pub fn auth_logout(
    app: AppHandle,
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<(), AppError> {
    crate::data_share::f_2hl(&app);
    let r_jg = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    store::f_zz(&r_jg)?;
    auth.set(None);
    Ok(())
}
#[tauri::command]
pub fn auth_current(db: State<'_, Db>, auth: State<'_, AuthState>) -> Result<Option<Tr>, AppError> {
    if let Some(r_jj) = auth.snapshot() {
        return Ok(Some(r_jj));
    }
    let r_ji = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    store::f_rg(&r_ji, Utc::now())?;
    let Some((r_jl, _)) = store::f_1ec(&r_ji)? else {
        return Ok(None);
    };
    let Some(r_jh) = store::f_1eb(&r_ji, &r_jl)? else {
        store::f_zz(&r_ji)?;
        return Ok(None);
    };
    store::f_2eh(&r_ji, true)?;
    let r_jk = f_2ds(&r_jh, SessionMode::Offline, true, Vec::new());
    auth.set(Some(Td {
        session: r_jk.clone(),
        wrap_key: None,
        access_token: None,
    }));
    Ok(Some(r_jk))
}
#[tauri::command]
pub fn auth_list_accounts(db: State<'_, Db>) -> Result<Vec<Tc>, AppError> {
    let r_jm = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(store::f_1n4(&r_jm)?)
}
#[tauri::command]
pub fn auth_get_server_url(db: State<'_, Db>) -> Result<Option<String>, AppError> {
    let r_jn = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    Ok(db::f_1ew(&r_jn, C_K0)?)
}
fn f_1j5(r_jp: &str) -> bool {
    let r_jo = r_jp
        .strip_prefix('[')
        .and_then(|r_jr| r_jr.strip_suffix(']'))
        .unwrap_or(r_jp);
    r_jo.eq_ignore_ascii_case("localhost")
        || r_jo
            .parse::<std::net::IpAddr>()
            .map(|r_jq| r_jq.is_loopback())
            .unwrap_or(false)
}
pub(crate) fn f_yb(url: &str) -> Result<(), &'static str> {
    let r_js = reqwest::Url::parse(url)
        .map_err(|_| "Địa chỉ máy chủ không hợp lệ.")?;
    match r_js.scheme() {
        "https" => Ok(()),
        "http" => {
            if r_js.host_str().map(f_1j5).unwrap_or(false) {
                Ok(())
            } else {
                Err(
                    "Chỉ dùng http:// cho localhost; máy chủ thật phải là https:// (nếu không, token và tài liệu bị lộ trên mạng).",
                )
            }
        }
        _ => {
            Err(
                "Địa chỉ máy chủ phải bắt đầu bằng http:// hoặc https://.",
            )
        }
    }
}
#[tauri::command]
pub fn auth_set_server_url(db: State<'_, Db>, args: Tlh) -> Result<(), AppError> {
    let r_jw = args
        .url
        .as_deref()
        .map(str::trim)
        .filter(|r_jv| !r_jv.is_empty());
    if let Some(url) = r_jw {
        f_yb(url).map_err(AppError::validation)?;
    }
    let r_jt = db.0.lock().map_err(|_| AppError::DbPoisoned)?;
    let r_ju = db::f_1ew(&r_jt, C_K0)?;
    db::f_2ei(&r_jt, C_K0, r_jw)?;
    if r_ju.as_deref() != r_jw {
        db::f_2ei(&r_jt, C_JZ, None)?;
        db::f_2ei(&r_jt, C_JX, None)?;
    }
    Ok(())
}
#[tauri::command]
pub async fn auth_validate_online(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
) -> Result<Tr, AppError> {
    let session = auth.require(digitext_core::Perm::Read)?;
    let r_jx = auth.wrap_key().ok_or_else(|| {
        AppError::new(
            "locked",
            "Mở khoá bằng mật khẩu trước.",
        )
    })?;
    let url = server_url(&db)?;
    let _guard = auth.lock_refresh().await;
    let (session, _access) = f_26h(&db, &auth, r_jx, &url, &session.user_id).await?;
    Ok(session)
}
#[tauri::command]
pub async fn server_ping(db: State<'_, Db>) -> Result<bool, AppError> {
    let Ok(url) = server_url(&db) else {
        return Ok(false);
    };
    Ok(client::f_22y(&url).await)
}
pub(crate) fn server_url(r_jy: &State<'_, Db>) -> Result<String, AppError> {
    let r_jz = r_jy.0.lock().map_err(|_| AppError::DbPoisoned)?;
    db::f_1ew(&r_jz, C_K0)?
        .filter(|r_k0| !r_k0.trim().is_empty())
        .ok_or_else(|| {
            AppError::new(
                "server_not_configured",
                "Chưa cấu hình địa chỉ máy chủ.",
            )
        })
}
pub(super) async fn f_26h(
    r_k2: &State<'_, Db>,
    r_k1: &State<'_, AuthState>,
    r_k9: [u8; 32],
    url: &str,
    r_k8: &str,
) -> Result<(Tr, String), AppError> {
    let account = {
        let guard = r_k2.0.lock().map_err(|_| AppError::DbPoisoned)?;
        store::f_1eb(&guard, r_k8)?.ok_or_else(|| {
            AppError::new(
                "unauthorized",
                "Tài khoản không còn trên máy.",
            )
        })?
    };
    let r_k6 = verifier::f_1we(&r_k9, &account.secrets)?;
    let secrets: Tkz = serde_json::from_slice(&r_k6)
        .map_err(|_| AppError::new("auth_cache_corrupt", "Cache secrets hỏng."))?;
    match client::refresh(url, &secrets.refresh_token).await {
        Ok(r_k4) => {
            let r_k5 = verifier::f_2cn(
                &r_k9,
                &serde_json::to_vec(
                        &Tkz {
                            refresh_token: r_k4.refresh_token.clone(),
                        },
                    )
                    .map_err(|e| AppError::internal(
                        format!("đóng gói secrets lỗi: {e}"),
                    ))?,
            )?;
            let r_k3 = Utc::now();
            let mut account = account;
            account.lease_jwt = r_k4.offline_lease.clone();
            account.offline_expiry = r_k4.offline_expiry.clone();
            account.last_online_at = r_k3.to_rfc3339();
            account.secrets = r_k5;
            account.display_name = r_k4.user.display_name.clone();
            account.role = r_k4.user.role;
            account.project_id = r_k4.user.project.as_ref().map(|p| p.id.clone());
            account.project_code = r_k4.user.project.as_ref().map(|p| p.code.clone());
            account.project_name = r_k4.user.project.as_ref().map(|p| p.name.clone());
            {
                let guard = r_k2.0.lock().map_err(|_| AppError::DbPoisoned)?;
                store::f_2nx(&guard, &account)?;
                store::f_rg(&guard, r_k3)?;
            }
            let r_k7 = f_2ds(&account, SessionMode::Online, false, f_18m(&r_k4.user));
            r_k1.set(
                Some(Td {
                    session: r_k7.clone(),
                    wrap_key: Some(r_k9),
                    access_token: Some(r_k4.access_token.clone()),
                }),
            );
            Ok((r_k7, r_k4.access_token))
        }
        Err(e @ ClientError::Api { .. }) if e.is_revoked() => {
            let guard = r_k2.0.lock().map_err(|_| AppError::DbPoisoned)?;
            store::f_13t(&guard, r_k8)?;
            store::f_zz(&guard)?;
            drop(guard);
            r_k1.set(None);
            Err(
                AppError::new(
                    "revoked",
                    "Tài khoản đã bị thu hồi — liên hệ quản trị dự án.",
                ),
            )
        }
        Err(ClientError::Api { .. }) => {
            Err(
                AppError::new(
                    "relogin_required",
                    "Phiên online đã hết hạn — đăng nhập lại khi có mạng (offline vẫn dùng được tới hạn).",
                ),
            )
        }
        Err(ClientError::Unreachable(_)) => {
            Err(
                AppError::new(
                    "server_unreachable",
                    "Không kết nối được máy chủ.",
                ),
            )
        }
    }
}
