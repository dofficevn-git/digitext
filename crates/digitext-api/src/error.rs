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
            "thiếu hoặc sai token xác thực",
        )
    }
    pub fn auth_failed() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "auth_failed",
            "tên đăng nhập hoặc mật khẩu không đúng",
        )
    }
    pub fn revoked(status: StatusCode) -> Self {
        Self::new(
            status,
            "revoked",
            "tài khoản hoặc phiên đã bị thu hồi",
        )
    }
    pub fn device_revoked() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "device_revoked",
            "thiết bị này đã bị thu hồi — liên hệ quản trị dự án",
        )
    }
    pub fn forbidden() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "forbidden",
            "không đủ quyền thực hiện thao tác này",
        )
    }
    pub fn not_found() -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "not_found",
            "không tìm thấy bản ghi",
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
            "lỗi hệ thống",
        )
    }
    pub fn too_many_attempts(retry_after_secs: u64) -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "too_many_attempts",
            format!(
                "sai mật khẩu nhiều lần — thử lại sau {retry_after_secs} giây"
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
            return ApiError::conflict("dữ liệu trùng khoá duy nhất");
        }
        if matches!(r_3w, sqlx::Error::RowNotFound) {
            return ApiError::not_found();
        }
        tracing::error!(error = % r_3w, "lỗi database");
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "lỗi hệ thống",
        )
    }
}
impl From<anyhow::Error> for ApiError {
    fn from(r_3x: anyhow::Error) -> Self {
        tracing::error!(error = % r_3x, "lỗi nội bộ");
        ApiError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "lỗi hệ thống",
        )
    }
}
