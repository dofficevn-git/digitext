// Copyright (c) 2026 DigiText. All rights reserved.
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: &'static str,
    pub message: String,
}
impl ApiError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        ApiError {
            status,
            code,
            message: message.into(),
        }
    }
    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "dGhp4bq/dSBob+G6t2Mgc2FpIHRva2VuIHjDoWMgdGjhu7Fj",
        )
    }
    pub fn auth_failed() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "auth_failed",
            "dMOqbiDEkcSDbmcgbmjhuq1wIGhv4bq3YyBt4bqtdCBraOG6qXUga2jDtG5nIMSRw7puZw==",
        )
    }
    pub fn revoked(status: StatusCode) -> Self {
        Self::new(
            status,
            "revoked",
            "dMOgaSBraG/huqNuIGhv4bq3YyBwaGnDqm4gxJHDoyBi4buLIHRodSBo4buTaQ==",
        )
    }
    pub fn device_revoked() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "device_revoked",
            "dGhp4bq/dCBi4buLIG7DoHkgxJHDoyBi4buLIHRodSBo4buTaSDigJQgbGnDqm4gaOG7hyBxdeG6o24gdHLhu4sgZOG7sSDDoW4=",
        )
    }
    pub fn forbidden() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "forbidden",
            "a2jDtG5nIMSR4bunIHF1eeG7gW4gdGjhu7FjIGhp4buHbiB0aGFvIHTDoWMgbsOgeQ==",
        )
    }
    pub fn not_found() -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "not_found",
            "a2jDtG5nIHTDrG0gdGjhuqV5IGLhuqNuIGdoaQ==",
        )
    }
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, "conflict", message)
    }
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request", message)
    }
    pub fn bad_request_retryable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "bad_request_retryable", message)
    }
    pub fn service_unavailable(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "service_unavailable",
            message,
        )
    }
    pub fn internal() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "bOG7l2kgaOG7hyB0aOG7kW5n",
        )
    }
    pub fn too_many_attempts(retry_after_secs: u64) -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "too_many_attempts",
            format!(
                "c2FpIG3huq10IGto4bqpdSBuaGnhu4F1IGzhuqduIOKAlCB0aOG7rSBs4bqhaSBzYXUg{retry_after_secs}IGdpw6J5"
            ),
        )
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(json!({ "code" : self.code, "message" : self.message })),
        )
            .into_response()
    }
}
impl From<sqlx::Error> for ApiError {
    fn from(r_3w: sqlx::Error) -> Self {
        if digitext_db::f_1jn(&r_3w) {
            return ApiError::conflict("ZOG7ryBsaeG7h3UgdHLDuW5nIGtob8OhIGR1eSBuaOG6pXQ=");
        }
        if matches!(r_3w, sqlx::Error::RowNotFound) {
            return ApiError::not_found();
        }
        tracing::error!(error = % r_3w, "bOG7l2kgZGF0YWJhc2U=");
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "bOG7l2kgaOG7hyB0aOG7kW5n",
        )
    }
}
impl From<anyhow::Error> for ApiError {
    fn from(r_3x: anyhow::Error) -> Self {
        tracing::error!(error = % r_3x, "bOG7l2kgbuG7mWkgYuG7mQ==");
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "bOG7l2kgaOG7hyB0aOG7kW5n",
        )
    }
}
