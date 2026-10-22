// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use crate::error::ApiError;
use crate::extract::Ts;
use crate::jwt::C_EL;
use crate::password;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::Engine;
use chrono::{DateTime, Duration, Utc};
use digitext_core::{Permission, PermissionOverrides, Role};
use digitext_db::users::User;
use digitext_db::{audit, devices, permissions, projects, tokens, users};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use uuid::Uuid;
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/auth/validate", get(validate))
        .route("/auth/pubkey", get(pubkey))
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBrief {
    pub id: Uuid,
    pub code: String,
    pub name: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    /// Vai trò động (bảng roles); None = vai trò gốc.
    pub role_code: Option<String>,
    pub role_name: String,
    pub project: Option<ProjectBrief>,
    pub grants: Vec<Permission>,
    pub denies: Vec<Permission>,
    /// Quyền hiệu lực đã tính ở máy chủ (bộ quyền vai trò ∪ cấp riêng − chặn riêng).
    pub permissions: Vec<Permission>,
}
impl UserProfile {
    async fn load(
        r_ab: &AppState,
        user: &User,
        r_a8: &PermissionOverrides,
    ) -> Result<Self, ApiError> {
        let project = match user.project_id {
            Some(r_aa) => projects::get(&r_ab.pool, r_aa)
                .await?
                .map(|r_a9| ProjectBrief {
                    id: r_a9.id,
                    code: r_a9.code,
                    name: r_a9.name,
                }),
            None => None,
        };
        let code = user.role_code.clone().unwrap_or_else(|| user.role.as_str().to_owned());
        let role_name = match digitext_db::roles::get(&r_ab.pool, &code).await {
            Ok(Some(r)) => r.name,
            _ => user.role.display_name().to_owned(),
        };
        let preset = crate::routes::roles::preset_cached(&r_ab.pool, user).await?;
        Ok(UserProfile {
            id: user.id,
            username: user.username.clone(),
            display_name: user.display_name.clone(),
            role: user.role,
            role_code: user.role_code.clone(),
            role_name,
            project,
            grants: r_a8.grants.iter().copied().collect(),
            denies: r_a8.denies.iter().copied().collect(),
            permissions: digitext_core::effective_set_with(preset.as_ref(), user.role, r_a8).into_iter().collect(),
        })
    }
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub offline_lease: String,
    pub offline_expiry: DateTime<Utc>,
    pub device_id: Option<Uuid>,
    pub user: UserProfile,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub device_id: Option<Uuid>,
    #[serde(default)]
    pub device_name: Option<String>,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub app_version: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tii {
    pub refresh_token: String,
}
fn f_1g3(r_ac: &str) -> String {
    let digest = Sha256::digest(r_ac.as_bytes());
    format!("{digest:x}")
}
fn f_1u6() -> String {
    let mut r_ad = [0u8; 32];
    rand::RngCore::fill_bytes(&mut rand::rngs::OsRng, &mut r_ad);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r_ad)
}
async fn f_1jp(
    r_ai: &AppState,
    user: &User,
    device_id: Option<Uuid>,
    r_ah: Option<Uuid>,
) -> Result<SessionResponse, ApiError> {
    let r_ae = Utc::now();
    let r_af = permissions::f_1o5(&r_ai.pool, user.id).await?;
    let preset = crate::routes::roles::preset_cached(&r_ai.pool, user).await?;
    let effective = digitext_core::effective_set_with(preset.as_ref(), user.role, &r_af);
    let access_token = r_ai.keys.sign_access(user, device_id, r_ae)?;
    let (offline_lease, offline_expiry) =
        r_ai.keys.sign_offline_lease(user, &r_af, &effective, device_id, r_ae)?;
    let r_ag = f_1u6();
    tokens::insert(
        &r_ai.pool,
        user.id,
        device_id,
        &f_1g3(&r_ag),
        r_ae + Duration::days(C_EL),
        r_ah,
    )
    .await?;
    Ok(SessionResponse {
        access_token,
        refresh_token: r_ag,
        offline_lease,
        offline_expiry,
        device_id,
        user: UserProfile::load(r_ai, user, &r_af).await?,
    })
}
async fn login(
    State(r_ap): State<AppState>,
    r_aj: crate::client_ip::T2t,
    Json(r_an): Json<LoginRequest>,
) -> Result<Json<SessionResponse>, ApiError> {
    let ip = r_aj.0;
    let r_al = ip.filter(|i| !i.is_loopback()).map(|i| i.to_string());
    if let Err(retry_after) = r_ap.limiter.check(&r_an.username) {
        return Err(ApiError::too_many_attempts(retry_after));
    }
    if let Some(ip) = ip {
        if let Err(retry_after) = r_ap.limiter.check_ip(ip) {
            return Err(ApiError::too_many_attempts(retry_after));
        }
    }
    let Some((user, r_am)) = users::f_1ej(&r_ap.pool, &r_an.username).await? else {
        password::f_182(&r_an.password).await;
        r_ap.limiter.record_failure(&r_an.username);
        if let Some(ip) = ip {
            r_ap.limiter.record_failure_ip(ip);
        }
        f_sr(&r_ap, None, &r_an.username, r_al.clone()).await;
        return Err(ApiError::auth_failed());
    };
    if !password::f_2pn(&r_am, &r_an.password).await {
        r_ap.limiter.record_failure(&r_an.username);
        if let Some(ip) = ip {
            r_ap.limiter.record_failure_ip(ip);
        }
        f_sr(&r_ap, Some(user.id), &r_an.username, r_al.clone()).await;
        return Err(ApiError::auth_failed());
    }
    r_ap.limiter.record_success(&r_an.username);
    if let Some(ip) = ip {
        r_ap.limiter.record_success_ip(ip);
    }
    if !user.is_active {
        return Err(ApiError::revoked(StatusCode::FORBIDDEN));
    }
    let device_id = if crate::routes::devices::f_153(&r_ap.pool, &user, r_an.device_id).await? {
        r_an.device_id
    } else {
        None
    };
    let r_ak = devices::register(
        &r_ap.pool,
        devices::Til {
            id: device_id,
            name: r_an.device_name.as_deref().unwrap_or("máy trạm"),
            platform: r_an.platform.as_deref().unwrap_or("unknown"),
            app_version: r_an.app_version.as_deref(),
            project_id: user.project_id,
            registered_by: Some(user.id),
        },
    )
    .await?;
    if r_ak.is_revoked() {
        return Err(ApiError::device_revoked());
    }
    let r_ao = f_1jp(&r_ap, &user, Some(r_ak.id), None).await?;
    audit::append(
        &r_ap.pool,
        audit::Tp {
            actor_id: Some(user.id),
            device_id: Some(r_ak.id),
            action: "login",
            target: Some(format!("user:{}", user.id)),
            detail: None,
            ip: r_al,
        },
    )
    .await?;
    Ok(Json(r_ao))
}
/// Châm chước cho refresh token VỪA xoay bị gửi lại: client hết thời gian chờ (máy chủ chậm —
/// 25/9/2026 ổ cứng quá tải, COMMIT 25 s) nên gửi lại đúng token cũ. Không có khoảng này thì bị
/// coi là trộm token → thu hồi cả phiên → 4.275 tệp đang đẩy lên lỗi «Chưa đăng nhập».
/// Chỉ áp cho lý do `rotated` (không phải logout / thu hồi thiết bị / reuse).
const REFRESH_REUSE_GRACE_SECS: i64 = 120;
fn rotated_recently(t: &tokens::Tij, now: chrono::DateTime<Utc>) -> bool {
    t.revoked_reason.as_deref() == Some("rotated")
        && t.revoked_at.is_some_and(|at| now - at <= chrono::Duration::seconds(REFRESH_REUSE_GRACE_SECS))
}
async fn refresh(
    State(r_ay): State<AppState>,
    r_aq: crate::client_ip::T2t,
    Json(r_av): Json<Tii>,
) -> Result<Json<SessionResponse>, ApiError> {
    let r_at = r_aq.as_audit();
    let r_as = f_1g3(&r_av.refresh_token);
    let Some(r_aw) = tokens::find_by_hash(&r_ay.pool, &r_as).await? else {
        return Err(ApiError::auth_failed());
    };
    let r_au = Utc::now();
    // 19/10/2026: token đã đánh dấu `rotated` mà KHÔNG có token kế tiếp = máy chủ nghẽn sau bước thu hồi,
    // chưa cấp được token mới → client chưa từng nhận token nào khác, gửi lại token cũ là đúng, không phải
    // trộm. Thiếu nhánh này: 6 phút sau gửi lại → thu hồi cả phiên → 6.009 tệp đang đẩy lỗi «Chưa đăng nhập».
    let grace = rotated_recently(&r_aw, r_au)
        || (r_aw.revoked_reason.as_deref() == Some("rotated") && !tokens::has_child(&r_ay.pool, r_aw.id).await?);
    if grace {
        tracing::warn!(token = % r_aw.id, "refresh token vừa xoay bị gửi lại trong thời gian châm chước — cấp phiên mới, không thu hồi");
    }
    if r_aw.revoked_at.is_some() && !grace {
        tokens::f_2a1(&r_ay.pool, r_aw.user_id, r_aw.device_id, "refresh_reuse").await?;
        audit::append(
            &r_ay.pool,
            audit::Tp {
                actor_id: Some(r_aw.user_id),
                device_id: r_aw.device_id,
                action: "refresh_reuse",
                target: Some(format!("refresh_token:{}", r_aw.id)),
                detail: None,
                ip: r_at.clone(),
            },
        )
        .await?;
        return Err(ApiError::revoked(StatusCode::UNAUTHORIZED));
    }
    if r_aw.expires_at <= r_au {
        return Err(ApiError::auth_failed());
    }
    let user = users::f_1eg(&r_ay.pool, r_aw.user_id)
        .await?
        .ok_or_else(|| ApiError::revoked(StatusCode::UNAUTHORIZED))?;
    if !user.is_active {
        tokens::f_29z(&r_ay.pool, user.id, "user_inactive").await?;
        return Err(ApiError::revoked(StatusCode::FORBIDDEN));
    }
    if let Some(device_id) = r_aw.device_id {
        if let Some(r_ar) = devices::get(&r_ay.pool, device_id).await? {
            if r_ar.is_revoked() {
                tokens::f_29y(&r_ay.pool, device_id, "device_revoked").await?;
                return Err(ApiError::device_revoked());
            }
        }
    }
    if !grace
        && !tokens::revoke(&r_ay.pool, r_aw.id, "rotated").await?
        && !tokens::find_by_hash(&r_ay.pool, &r_as)
            .await?
            .is_some_and(|t| rotated_recently(&t, Utc::now()))
    {
        tokens::f_2a1(&r_ay.pool, r_aw.user_id, r_aw.device_id, "refresh_reuse").await?;
        audit::append(
            &r_ay.pool,
            audit::Tp {
                actor_id: Some(r_aw.user_id),
                device_id: r_aw.device_id,
                action: "refresh_reuse",
                target: Some(format!("refresh_token:{}", r_aw.id)),
                detail: None,
                ip: r_at.clone(),
            },
        )
        .await?;
        return Err(ApiError::revoked(StatusCode::UNAUTHORIZED));
    }
    if let Some(device_id) = r_aw.device_id {
        devices::touch(&r_ay.pool, device_id).await?;
    }
    let r_ax = f_1jp(&r_ay, &user, r_aw.device_id, Some(r_aw.id)).await?;
    audit::append(
        &r_ay.pool,
        audit::Tp {
            actor_id: Some(user.id),
            device_id: r_aw.device_id,
            action: "refresh",
            target: Some(format!("user:{}", user.id)),
            detail: None,
            ip: r_at,
        },
    )
    .await?;
    Ok(Json(r_ax))
}
async fn logout(
    State(r_b1): State<AppState>,
    auth: Ts,
    Json(r_az): Json<Tii>,
) -> Result<StatusCode, ApiError> {
    if let Some(r_b0) = tokens::find_by_hash(&r_b1.pool, &f_1g3(&r_az.refresh_token)).await? {
        if r_b0.user_id == auth.user.id {
            tokens::revoke(&r_b1.pool, r_b0.id, "logout").await?;
        }
    }
    audit::append(
        &r_b1.pool,
        audit::Tp {
            actor_id: Some(auth.user.id),
            device_id: auth.claims.device_id,
            action: "logout",
            target: Some(format!("user:{}", auth.user.id)),
            detail: None,
            ip: auth.ip(),
        },
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}
async fn validate(
    State(r_b3): State<AppState>,
    auth: Ts,
) -> Result<Json<serde_json::Value>, ApiError> {
    let r_b2 = UserProfile::load(&r_b3, &auth.user, &auth.overrides).await?;
    Ok(Json(json!({ "valid" : true, "user" : r_b2 })))
}
async fn pubkey(State(r_b4): State<AppState>) -> Json<serde_json::Value> {
    Json(json!({ "alg" : "EdDSA", "publicKeyPem" : r_b4.keys.public_pem }))
}
async fn f_sr(r_b6: &AppState, r_b7: Option<Uuid>, username: &str, r_b5: Option<String>) {
    let _ = audit::append(
        &r_b6.pool,
        audit::Tp {
            actor_id: r_b7,
            device_id: None,
            action: "login_failed",
            target: r_b7.map(|id| format!("user:{id}")),
            detail: Some(json!({ "username" : username })),
            ip: r_b5,
        },
    )
    .await;
}
