// Copyright (c) 2026 DigiText. All rights reserved.
use crate::error::AppError;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use chrono::{DateTime, Utc};
use digitext_core::{f_18s, Permission, PermissionOverrides, Role};
use jsonwebtoken::{Algorithm as JwtAlg, DecodingKey, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
const C_DM: usize = 12;
fn f_s9() -> Argon2<'static> {
    Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19456, 3, 1, Some(32)).expect("tham số argon2 hợp lệ"),
    )
}
pub struct T4l {
    pub verifier_hex: String,
    pub wrap_key: [u8; 32],
}
pub fn f_1u7() -> [u8; 16] {
    let mut salt = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    salt
}
pub fn derive(r_mq: &str, salt: &[u8]) -> Result<T4l, AppError> {
    let mut r_mp = [0u8; 32];
    f_s9()
        .hash_password_into(r_mq.as_bytes(), salt, &mut r_mp)
        .map_err(|e| AppError::internal(format!("argon2 thất bại: {e}")))?;
    let mut r_mn = Sha256::new();
    r_mn.update(b"digitext-verify");
    r_mn.update(r_mp);
    let verifier_hex = format!("{:x}", r_mn.finalize());
    let r_mo = hkdf::Hkdf::<Sha256>::new(None, &r_mp);
    let mut wrap_key = [0u8; 32];
    r_mo.expand(b"digitext-wrap", &mut wrap_key)
        .map_err(|e| AppError::internal(format!("hkdf thất bại: {e}")))?;
    Ok(T4l {
        verifier_hex,
        wrap_key,
    })
}
pub fn f_2pm(r_mr: &T4l, r_ms: &str) -> bool {
    r_mr.verifier_hex.as_bytes().ct_eq(r_ms.as_bytes()).into()
}
pub fn f_2cn(wrap_key: &[u8; 32], r_mx: &[u8]) -> Result<Vec<u8>, AppError> {
    let r_mt = Aes256Gcm::new(wrap_key.into());
    let mut r_mv = [0u8; C_DM];
    rand::rngs::OsRng.fill_bytes(&mut r_mv);
    let r_mu = r_mt
        .encrypt(Nonce::from_slice(&r_mv), r_mx)
        .map_err(|_| AppError::internal("mã hoá secrets thất bại"))?;
    let mut r_mw = Vec::with_capacity(C_DM + r_mu.len());
    r_mw.extend_from_slice(&r_mv);
    r_mw.extend_from_slice(&r_mu);
    Ok(r_mw)
}
pub fn f_1we(wrap_key: &[u8; 32], r_n0: &[u8]) -> Result<Vec<u8>, AppError> {
    if r_n0.len() < C_DM {
        return Err(AppError::new(
            "auth_cache_corrupt",
            "cache tài khoản hỏng",
        ));
    }
    let (r_mz, r_my) = r_n0.split_at(C_DM);
    Aes256Gcm::new(wrap_key.into())
        .decrypt(Nonce::from_slice(r_mz), r_my)
        .map_err(|_| {
            AppError::new(
                "auth_cache_corrupt",
                "không giải mã được cache tài khoản",
            )
        })
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tkz {
    pub refresh_token: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tat {
    pub sub: String,
    pub username: String,
    pub role: Role,
    pub project_id: Option<String>,
    pub device_id: Option<String>,
    pub tv: i32,
    pub purpose: String,
    pub iat: i64,
    pub exp: i64,
    #[serde(default)]
    pub offline_expiry: Option<i64>,
    #[serde(default)]
    pub perm_grants: Vec<String>,
    #[serde(default)]
    pub perm_denies: Vec<String>,
}
pub fn f_2pv(r_n5: &str, r_n6: &str, r_n3: &str, r_n2: Option<&str>) -> Result<Tat, AppError> {
    let key = DecodingKey::from_ed_pem(r_n6.as_bytes()).map_err(|_| {
        AppError::new(
            "auth_cache_corrupt",
            "public key server không hợp lệ",
        )
    })?;
    let r_n1 = jsonwebtoken::decode::<Tat>(r_n5, &key, &Validation::new(JwtAlg::EdDSA))
        .map_err(|_| {
            AppError::new(
                "offline_expired",
                "giấy phép offline không hợp lệ hoặc đã hết hạn — cần đăng nhập online lại",
            )
        })?
        .claims;
    if r_n1.purpose != "offline_lease" || r_n1.sub != r_n3 {
        return Err(AppError::new(
            "auth_cache_corrupt",
            "giấy phép offline không khớp tài khoản",
        ));
    }
    if let Some(r_n4) = r_n1.device_id.as_deref() {
        if r_n2 != Some(r_n4) {
            return Err(
                AppError::new(
                    "auth_cache_corrupt",
                    "giấy phép offline không khớp thiết bị này — đăng nhập online lại",
                ),
            );
        }
    }
    Ok(r_n1)
}
pub fn f_18o(r_n8: DateTime<Utc>, r_n7: Option<DateTime<Utc>>) -> DateTime<Utc> {
    match r_n7 {
        Some(r_n9) if r_n9 > r_n8 => r_n9,
        _ => r_n8,
    }
}
pub fn f_1vz(
    offline_expiry: DateTime<Utc>,
    r_nb: DateTime<Utc>,
    r_na: Option<DateTime<Utc>>,
) -> bool {
    offline_expiry > f_18o(r_nb, r_na)
}
pub fn f_1mr(r_nd: &Tat, r_nc: Role) -> bool {
    r_nd.role == r_nc
}
pub fn f_18p(r_ne: &Tat) -> Vec<Permission> {
    let mut r_nh = PermissionOverrides::default();
    for r_ng in &r_ne.perm_grants {
        if let Ok(p) = r_ng.parse::<Permission>() {
            r_nh.grants.insert(p);
        }
    }
    for r_nf in &r_ne.perm_denies {
        if let Ok(p) = r_nf.parse::<Permission>() {
            r_nh.denies.insert(p);
        }
    }
    f_18s(r_ne.role, &r_nh.sanitized()).into_iter().collect()
}
