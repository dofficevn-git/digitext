// Copyright (c) 2026 DigiText. All rights reserved.
use crate::app::AppState;
use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::request::Parts;
use axum::http::HeaderMap;
use std::convert::Infallible;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
const C_1S: &str = "cf-connecting-ip";
pub fn client_ip(headers: &HeaderMap, r_2w: Option<IpAddr>, r_2u: bool) -> Option<IpAddr> {
    if r_2u && r_2w.is_none_or(|r_2v| r_2v.is_loopback()) {
        if let Some(ip) = headers
            .get(C_1S)
            .and_then(|r_2y| r_2y.to_str().ok())
            .map(str::trim)
            .and_then(|r_2x| IpAddr::from_str(r_2x).ok())
        {
            return Some(ip);
        }
    }
    r_2w
}
pub struct T2t(pub Option<IpAddr>);
impl T2t {
    pub fn as_audit(&self) -> Option<String> {
        self.0.map(|ip| ip.to_string())
    }
}
impl FromRequestParts<AppState> for T2t {
    type Rejection = Infallible;
    async fn from_request_parts(r_30: &mut Parts, r_32: &AppState) -> Result<Self, Infallible> {
        let r_31 = r_30
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|r_2z| r_2z.0.ip());
        Ok(T2t(client_ip(&r_30.headers, r_31, r_32.behind_proxy)))
    }
}
