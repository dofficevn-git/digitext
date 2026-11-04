// Copyright (c) 2026 DigiText. All rights reserved.
//! Kênh tự cập nhật app desktop — phát tĩnh `latest.json` + bộ cài đã ký từ
//! `DIGITEXT_UPDATES_DIR`. Không cần đăng nhập: app kiểm TRƯỚC khi đăng nhập, và mọi
//! bộ cài đều được xác thực bằng chữ ký minisign (khoá công khai nằm trong app) nên
//! tệp phát ra đây không phải bí mật. Chỉ phục vụ tên tệp phẳng với đuôi cho phép —
//! không có đường dẫn con, không đi ra ngoài thư mục.
//!
//! Máy chủ cơ quan (bản Lưu trữ, 4/11/2026): `latest.json` do dev/release-desktop.ps1 ghi URL tạm, máy chủ viết lại URL
//! bộ cài theo địa chỉ của chính nó (`DIGITEXT_PUBLIC_URL`, không có thì lấy Host + X-Forwarded-Proto); `GET /tai-ve`
//! là trang tải bộ cài cho nhân viên cơ quan.

use crate::app::AppState;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::routing::get;
use axum::Router;
use serde_json::Value;

pub fn router() -> Router<AppState> {
    let r = Router::new().route("/updates/{file}", get(serve)).route("/tai-ve", get(tai_ve));
    if crate::config::edition_luu_tru() {
        r.route("/", get(|| async { Redirect::temporary("/tai-ve") }))
    } else {
        r
    }
}

/// Viết lại URL khi máy chủ có địa chỉ công khai riêng hoặc là bản Lưu trữ; máy chủ thi công giữ nguyên tệp.
fn can_viet_lai() -> bool {
    crate::config::public_url().is_some() || crate::config::edition_luu_tru()
}

fn dia_chi_goc(headers: &HeaderMap) -> Option<String> {
    if let Some(u) = crate::config::public_url() {
        return Some(u.to_string());
    }
    let host = headers.get(header::HOST)?.to_str().ok()?.trim();
    if host.is_empty() || !host.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':' | '[' | ']')) {
        return None;
    }
    let proto = headers
        .get("x-forwarded-proto")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.split(',').next().unwrap_or("").trim().to_ascii_lowercase())
        .filter(|v| v == "http" || v == "https")
        .unwrap_or_else(|| "https".into());
    Some(format!("{proto}://{host}"))
}

/// Đổi `platforms.*.url` thành `<gốc>/updates/<tên tệp>`; chỉ giữ tên tệp an toàn (cùng luật `safe_name`).
fn viet_lai_latest(mut v: Value, goc: &str) -> Value {
    if let Some(ps) = v.get_mut("platforms").and_then(Value::as_object_mut) {
        for p in ps.values_mut() {
            let ten = p
                .get("url")
                .and_then(Value::as_str)
                .and_then(|u| u.rsplit('/').next())
                .filter(|t| safe_name(t).is_some())
                .map(str::to_string);
            if let (Some(ten), Some(o)) = (ten, p.as_object_mut()) {
                o.insert("url".into(), Value::String(format!("{goc}/updates/{ten}")));
            }
        }
    }
    v
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

async fn serve(State(state): State<AppState>, Path(file): Path<String>, headers: HeaderMap) -> Response {
    let Some((name, ct)) = safe_name(&file) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let path = state.updates_dir.join(name);
    if name == "latest.json" && can_viet_lai() {
        if let (Some(goc), Ok(raw)) = (dia_chi_goc(&headers), tokio::fs::read(&path).await) {
            if let Ok(v) = serde_json::from_slice::<Value>(&raw) {
                let mut resp = axum::Json(viet_lai_latest(v, &goc)).into_response();
                resp.headers_mut()
                    .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache, no-store, must-revalidate"));
                return resp;
            }
        }
    }
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

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

/// Trang tải bộ cài: đọc `latest.json`, lấy bộ cài Windows, nút tải trỏ đường dẫn tương đối `/updates/<tệp>`.
async fn tai_ve(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let latest = tokio::fs::read(state.updates_dir.join("latest.json"))
        .await
        .ok()
        .and_then(|raw| serde_json::from_slice::<Value>(&raw).ok());
    let ten_app = if crate::config::edition_luu_tru() { "DigiText Lưu trữ" } else { "DigiText" };
    let goc = dia_chi_goc(&headers).unwrap_or_default();
    let than = match latest.as_ref().and_then(|v| {
        let tep = v.pointer("/platforms/windows-x86_64/url")?.as_str()?.rsplit('/').next()?;
        safe_name(tep)?;
        Some((v, tep.to_string()))
    }) {
        Some((v, tep)) => {
            let ver = html_escape(v.get("version").and_then(Value::as_str).unwrap_or("?"));
            let ngay = v.get("pub_date").and_then(Value::as_str).and_then(|d| d.get(..10)).map(|d| {
                let p: Vec<&str> = d.split('-').collect();
                if p.len() == 3 { format!("{}/{}/{}", p[2], p[1], p[0]) } else { d.to_string() }
            });
            let ghi_chu = v.get("notes").and_then(Value::as_str).map(str::trim).filter(|n| !n.is_empty());
            format!(
                r#"<p class="ver">Phiên bản {ver}{ngay}</p>
<a class="btn" href="/updates/{tep}" download>Tải bộ cài cho Windows</a>
{ghi_chu}
<h2>Cài đặt</h2>
<ol>
<li>Chạy tệp vừa tải, làm theo hướng dẫn (không cần quyền quản trị máy).</li>
<li>Mở <b>{app}</b>, bấm <b>Máy chủ</b> ở cuối màn đăng nhập, nhập địa chỉ <code>{goc}</code>.</li>
<li>Đăng nhập bằng tài khoản cán bộ lưu trữ cấp. Các bản sau tự cập nhật từ máy chủ này.</li>
</ol>"#,
                ver = ver,
                ngay = ngay.map(|d| format!(" · phát hành {}", html_escape(&d))).unwrap_or_default(),
                tep = html_escape(&tep),
                ghi_chu = ghi_chu.map(|n| format!("<p class=\"notes\">{}</p>", html_escape(n))).unwrap_or_default(),
                app = ten_app,
                goc = html_escape(&goc),
            )
        }
        None => "<p class=\"ver\">Máy chủ chưa có bộ cài. Quản trị viên chép gói phát hành vào thư mục updates của máy chủ.</p>".to_string(),
    };
    let html = format!(
        r#"<!doctype html>
<html lang="vi"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<title>Tải {app}</title>
<style>
:root{{color-scheme:light dark;--bg:#f5f5f7;--card:#fff;--fg:#1f1f24;--sub:#5c5c66;--pri:#5a58a6;--pri-fg:#fff;--line:#e3e3e8}}
@media (prefers-color-scheme:dark){{:root{{--bg:#18181b;--card:#232327;--fg:#ececf1;--sub:#a1a1aa;--pri:#8b89d6;--pri-fg:#111;--line:#34343a}}}}
body{{margin:0;background:var(--bg);color:var(--fg);font:15px/1.55 "Segoe UI",system-ui,sans-serif}}
main{{max-width:560px;margin:8vh auto;padding:0 16px}}
.card{{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:28px}}
h1{{margin:0 0 4px;font-size:22px}} h2{{font-size:15px;margin:24px 0 6px}}
.ver{{color:var(--sub);margin:0 0 20px}} .notes{{white-space:pre-line;color:var(--sub)}}
.btn{{display:inline-block;background:var(--pri);color:var(--pri-fg);text-decoration:none;padding:10px 18px;border-radius:8px;font-weight:600}}
.btn:focus-visible{{outline:2px solid var(--fg);outline-offset:2px}}
ol{{padding-left:20px;margin:0}} code{{background:var(--bg);padding:1px 6px;border-radius:4px}}
</style></head>
<body><main><div class="card"><h1>{app}</h1>
{than}
</div></main></body></html>"#,
        app = ten_app,
        than = than,
    );
    let mut resp = Html(html).into_response();
    resp.headers_mut().insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    resp
}

#[cfg(test)]
mod tests {
    use super::{safe_name, viet_lai_latest};

    #[test]
    fn viet_lai_url_bo_cai_theo_may_chu() {
        let v = serde_json::json!({
            "version": "0.25.0",
            "platforms": { "windows-x86_64": { "signature": "x", "url": "https://may-chu-luu-tru/updates/DigiText-LuuTru_0.25.0_x64-setup.exe" } }
        });
        let v = viet_lai_latest(v, "https://luutru.co-quan.vn");
        assert_eq!(
            v.pointer("/platforms/windows-x86_64/url").and_then(|u| u.as_str()),
            Some("https://luutru.co-quan.vn/updates/DigiText-LuuTru_0.25.0_x64-setup.exe")
        );
        let xau = serde_json::json!({ "platforms": { "w": { "url": "https://x/updates/../.env" } } });
        assert_eq!(viet_lai_latest(xau, "https://y").pointer("/platforms/w/url").and_then(|u| u.as_str()), Some("https://x/updates/../.env"));
    }

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
