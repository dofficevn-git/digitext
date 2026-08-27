// Copyright (c) 2026 DigiText. All rights reserved.
use crate::db;
use base64::Engine;
use ed25519_dalek::SigningKey;
use rusqlite::Connection;
const C_45: &str = "lan_device_seed";
fn b64() -> base64::engine::general_purpose::GeneralPurpose {
    base64::engine::general_purpose::STANDARD
}
pub fn f_19x(r_23z: &Connection) -> rusqlite::Result<SigningKey> {
    if let Some(seed_b64) = db::f_1ew(r_23z, C_45)? {
        if let Some(sk) = f_13l(&seed_b64) {
            return Ok(sk);
        }
        tracing::warn!(
            "lan_device_seed hỏng — sinh lại khoá danh tính máy (peer sẽ ghim lại)"
        );
    }
    let sk = SigningKey::generate(&mut rand::rngs::OsRng);
    let seed_b64 = b64().encode(sk.to_bytes());
    db::f_2ei(r_23z, C_45, Some(&seed_b64))?;
    Ok(sk)
}
fn f_13l(r_242: &str) -> Option<SigningKey> {
    let r_241 = b64().decode(r_242).ok()?;
    let r_240: [u8; 32] = r_241.try_into().ok()?;
    Some(SigningKey::from_bytes(&r_240))
}
pub fn pubkey_b64(r_243: &SigningKey) -> String {
    b64().encode(r_243.verifying_key().to_bytes())
}
pub fn f_157(r_244: &Connection, r_246: &SigningKey) -> rusqlite::Result<String> {
    let r_245 = db::f_1ew(r_244, crate::auth::store::C_JX)?
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    Ok(r_245.unwrap_or_else(|| {
        format!(
            "local-{}",
            crate::lan_session::device_fingerprint(&pubkey_b64(r_246))
        )
    }))
}
