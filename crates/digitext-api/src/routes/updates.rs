// Copyright (c) 2026 DigiText. All rights reserved.
//! Kênh tự cập nhật app desktop — phát tĩnh `latest.json` + bộ cài đã ký từ
//! `DIGITEXT_UPDATES_DIR`. Không cần đăng nhập: app kiểm TRƯỚC khi đăng nhập, và mọi
//! bộ cài đều được xác thực bằng chữ ký minisign (khoá công khai nằm trong app) nên
//! tệp phát ra đây không phải bí mật. Chỉ phục vụ tên tệp phẳng với đuôi cho phép —
//! không có đường dẫn con, không đi ra ngoài thư mục.

use crate::app::AppState;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new().route("/updates/{file}", get(serve))
}

const ALLOWED_EXT: &[(&str, &str)] = &[
    (".json", "application/json; charset=utf-8"),
    (".exe", "application/vnd.microsoft.portable-executable"),
    (".msi", "application/x-msi"),
    (".sig", "text/plain; charset=utf-8"),
    (".zip", "application/zip"),
    (".tar.gz", "application/gzip"),
    (".AppImage", "application/octet-stream"),
    (".dmg", "application/octet-stream"),
];

fn safe_name(file: &str) -> Option<(&str, &'static str)> {
    if file.is_empty()
        || file.len() > 200
        || file.contains(['/', '\\'])
        || file.starts_with('.')
        || file.contains("..")
        || !file.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
    {
        return None;
    }
    ALLOWED_EXT
        .iter()
        .find(|(ext, _)| file.ends_with(ext))
        .map(|(_, ct)| (file, *ct))
}

async fn serve(State(state): State<AppState>, Path(file): Path<String>) -> Response {
    let Some((name, ct)) = safe_name(&file) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let path = state.updates_dir.join(name);
    let Ok(f) = tokio::fs::File::open(&path).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let len = f.metadata().await.ok().map(|m| m.len());
    let mut resp = Body::from_stream(tokio_util::io::ReaderStream::new(f)).into_response();
    let h = resp.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static(ct));
    // latest.json phải luôn mới — không cho proxy (Cloudflare) cache.
    h.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache, no-store, must-revalidate"));
    if let Some(n) = len {
        h.insert(header::CONTENT_LENGTH, HeaderValue::from(n));
    }
    resp
}

#[cfg(test)]
mod tests {
    use super::safe_name;

    #[test]
    fn chi_nhan_ten_phang_duoi_hop_le() {
        assert!(safe_name("latest.json").is_some());
        assert!(safe_name("DigiText_0.16.0_x64-setup.exe").is_some());
        assert!(safe_name("DigiText_0.16.0_x64-setup.exe.sig").is_some());
        assert!(safe_name("../.env").is_none());
        assert!(safe_name("a/b.json").is_none());
        assert!(safe_name("a\\b.json").is_none());
        assert!(safe_name(".hidden.json").is_none());
        assert!(safe_name("x.pem").is_none());
        assert!(safe_name("").is_none());
    }
}
