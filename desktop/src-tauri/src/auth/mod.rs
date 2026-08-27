// Copyright (c) 2026 DigiText. All rights reserved.
pub mod admin;
pub mod captcha;
pub mod client;
pub mod commands;
pub mod guard;
pub mod session;
pub mod store;
pub mod verifier;
use session::Tr;
use std::sync::RwLock;
pub struct Td {
    pub session: Tr,
    pub wrap_key: Option<[u8; 32]>,
    pub access_token: Option<String>,
}
#[derive(Default)]
pub struct AuthState(pub RwLock<Option<Td>>, tokio::sync::Mutex<()>);
impl AuthState {
    pub async fn lock_refresh(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.1.lock().await
    }
    pub fn snapshot(&self) -> Option<Tr> {
        self.0
            .read()
            .ok()
            .and_then(|r_l9| r_l9.as_ref().map(|r_l8| r_l8.session.clone()))
    }
    pub fn set(&self, r_la: Option<Td>) {
        if let Ok(mut r_lb) = self.0.write() {
            *r_lb = r_la;
        }
    }
    pub fn set_locked(&self, r_ld: bool) {
        if let Ok(mut r_le) = self.0.write() {
            if let Some(r_lc) = r_le.as_mut() {
                r_lc.session.locked = r_ld;
                if r_ld {
                    r_lc.wrap_key = None;
                    r_lc.access_token = None;
                }
            }
        }
    }
    pub fn wrap_key(&self) -> Option<[u8; 32]> {
        self.0.read().ok().and_then(|r_lf| r_lf.as_ref()?.wrap_key)
    }
    pub fn access_token(&self) -> Option<String> {
        self.0
            .read()
            .ok()
            .and_then(|r_lg| r_lg.as_ref()?.access_token.clone())
    }
}
