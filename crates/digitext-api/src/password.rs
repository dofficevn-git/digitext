// Copyright (c) 2026 DigiText. All rights reserved.
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use std::sync::LazyLock;
fn f_1g7() -> Argon2<'static> {
    Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(19456, 2, 1, None).expect("tham số argon2 hợp lệ"),
    )
}
pub fn hash(password: &str) -> anyhow::Result<String> {
    let r_74 = SaltString::generate(&mut OsRng);
    Ok(f_1g7()
        .hash_password(password.as_bytes(), &r_74)
        .map_err(|e| anyhow::anyhow!("băm mật khẩu thất bại: {e}"))?
        .to_string())
}
pub fn verify(phc: &str, password: &str) -> bool {
    let Ok(r_75) = PasswordHash::new(phc) else {
        return false;
    };
    f_1g7().verify_password(password.as_bytes(), &r_75).is_ok()
}
static C_4H: LazyLock<String> =
    LazyLock::new(|| hash("digitext-dummy-password").expect("hash mồi"));
pub fn f_181(password: &str) {
    let _ = verify(&C_4H, password);
}
pub async fn f_1g0(password: &str) -> anyhow::Result<String> {
    let password = password.to_owned();
    tokio::task::spawn_blocking(move || hash(&password))
        .await
        .map_err(|e| anyhow::anyhow!("tác vụ băm mật khẩu lỗi: {e}"))?
}
const C_BK: usize = 8;
static C_8U: LazyLock<tokio::sync::Semaphore> = LazyLock::new(|| tokio::sync::Semaphore::new(C_BK));
pub async fn f_2pn(phc: &str, password: &str) -> bool {
    let Ok(_permit) = C_8U.acquire().await else {
        return false;
    };
    let (phc, password) = (phc.to_owned(), password.to_owned());
    tokio::task::spawn_blocking(move || verify(&phc, &password))
        .await
        .unwrap_or(false)
}
pub async fn f_182(password: &str) {
    let Ok(_permit) = C_8U.acquire().await else {
        return;
    };
    let password = password.to_owned();
    let _ = tokio::task::spawn_blocking(move || f_181(&password)).await;
}
