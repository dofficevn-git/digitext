// Copyright (c) 2026 DigiText. All rights reserved.
use digitext_core::{Permission, Role};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use std::time::Duration;
#[derive(Debug)]
pub enum ClientError {
    Unreachable(String),
    Api {
        status: u16,
        code: String,
        message: String,
    },
}
impl ClientError {
    pub fn is_revoked(&self) -> bool {
        matches!(
            self, ClientError::Api { code, .. } if code == "revoked" || code ==
            "device_revoked"
        )
    }
    pub fn is_unauthorized(&self) -> bool {
        matches!(self, ClientError::Api { status: 401, .. })
    }
    pub fn code(&self) -> &str {
        match self {
            ClientError::Unreachable(_) => "server_unreachable",
            ClientError::Api { code, .. } => code,
        }
    }
}
const UPLOAD_TIMEOUT: Duration = Duration::from_secs(120);
fn http() -> &'static reqwest::Client {
    static C_1V: OnceLock<reqwest::Client> = OnceLock::new();
    C_1V.get_or_init(|| {
        reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .build()
            .expect("reqwest client")
    })
}
fn api(r_eo: &str, path: &str) -> String {
    format!("{}/api/v1{path}", r_eo.trim_end_matches('/'))
}
pub(crate) fn f_1gh(r_ep: &str) -> [String; 2] {
    let b = r_ep.trim_end_matches('/');
    [format!("{b}/healthz"), api(b, "/auth/pubkey")]
}
pub async fn f_22y(r_eq: &str) -> bool {
    for r_er in f_1gh(r_eq) {
        if let Ok(r) = http()
            .get(r_er)
            .timeout(Duration::from_secs(8))
            .send()
            .await
        {
            if r.status().is_success() {
                return true;
            }
        }
    }
    false
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectBrief {
    pub id: String,
    pub code: String,
    pub name: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    #[serde(default)]
    pub project: Option<ProjectBrief>,
    #[serde(default)]
    pub grants: Vec<Permission>,
    #[serde(default)]
    pub denies: Vec<Permission>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    #[allow(dead_code)]
    pub access_token: String,
    pub refresh_token: String,
    pub offline_lease: String,
    pub offline_expiry: String,
    pub device_id: Option<String>,
    pub user: UserProfile,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest<'a> {
    pub username: &'a str,
    pub password: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<&'a str>,
    pub device_name: &'a str,
    pub platform: &'a str,
    pub app_version: &'a str,
}
#[derive(Deserialize)]
struct T6x {
    code: Option<String>,
    message: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Thu {
    public_key_pem: String,
}
async fn parse<T: serde::de::DeserializeOwned>(
    resp: Result<reqwest::Response, reqwest::Error>,
) -> Result<T, ClientError> {
    let resp = resp.map_err(|e| ClientError::Unreachable(e.to_string()))?;
    let status = resp.status();
    if status.is_success() {
        return resp.json::<T>().await.map_err(|e| {
            ClientError::Unreachable(format!(
                "phản hồi server không hợp lệ: {e}"
            ))
        });
    }
    let r_es = resp.json::<T6x>().await.unwrap_or(T6x {
        code: None,
        message: None,
    });
    Err(ClientError::Api {
        status: status.as_u16(),
        code: r_es.code.unwrap_or_else(|| "error".into()),
        message: r_es
            .message
            .unwrap_or_else(|| format!("máy chủ trả lỗi HTTP {status}")),
    })
}
pub async fn login(r_et: &str, r_eu: &LoginRequest<'_>) -> Result<SessionResponse, ClientError> {
    parse(
        http()
            .post(api(r_et, "/auth/login"))
            .json(r_eu)
            .send()
            .await,
    )
    .await
}
pub async fn refresh(r_ev: &str, refresh_token: &str) -> Result<SessionResponse, ClientError> {
    parse(
        http()
            .post(api(r_ev, "/auth/refresh"))
            .json(&serde_json::json!({ "refreshToken" : refresh_token }))
            .send()
            .await,
    )
    .await
}
pub async fn f_1c8(r_ew: &str) -> Result<String, ClientError> {
    let r_ex: Thu = parse(http().get(api(r_ew, "/auth/pubkey")).send().await).await?;
    Ok(r_ex.public_key_pem)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tl6 {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub display_name: String,
    pub role: Role,
    pub project_id: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tl7 {
    pub effective: Vec<Permission>,
    pub grants: Vec<Permission>,
    pub denies: Vec<Permission>,
    pub role_defaults: Vec<Permission>,
    pub grantable: Vec<Permission>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tl4 {
    pub id: String,
    pub code: String,
    pub name: String,
    pub admin_ids: Vec<String>,
    pub is_active: bool,
    pub created_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tp9 {
    pub id: String,
    pub forwarded_by: Option<String>,
    pub forwarded_by_name: Option<String>,
    pub note: Option<String>,
    pub status: String,
    pub forwarded: i64,
    pub skipped: i64,
    pub failed: i64,
    pub rejected: i64,
    pub received: i64,
    pub project_count: i64,
    pub created_at: String,
    pub completed_at: Option<String>,
}
async fn f_1yv(resp: Result<reqwest::Response, reqwest::Error>) -> Result<(), ClientError> {
    let resp = resp.map_err(|e| ClientError::Unreachable(e.to_string()))?;
    let status = resp.status();
    if status.is_success() {
        return Ok(());
    }
    let r_ey = resp.json::<T6x>().await.unwrap_or(T6x {
        code: None,
        message: None,
    });
    Err(ClientError::Api {
        status: status.as_u16(),
        code: r_ey.code.unwrap_or_else(|| "error".into()),
        message: r_ey
            .message
            .unwrap_or_else(|| format!("máy chủ trả lỗi HTTP {status}")),
    })
}
pub async fn f_1eo<T: serde::de::DeserializeOwned>(
    r_ez: &str,
    r_f0: &str,
    path: &str,
) -> Result<T, ClientError> {
    // Danh sách lớn (30.000+ tài liệu) qua đường hầm có thể quá 15 s mặc định.
    parse(http().get(api(r_ez, path)).timeout(Duration::from_secs(60)).bearer_auth(r_f0).send().await).await
}
pub async fn f_23d<T: serde::de::DeserializeOwned>(
    r_f1: &str,
    r_f3: &str,
    path: &str,
    r_f2: &serde_json::Value,
) -> Result<T, ClientError> {
    parse(
        http()
            .post(api(r_f1, path))
            .bearer_auth(r_f3)
            .json(r_f2)
            .send()
            .await,
    )
    .await
}
/// POST cho các lệnh "toàn bộ" (OCR / PDF 2 lớp / bóc tách / ghép lại): máy chủ có thể mất
/// hàng chục giây với vài chục nghìn tài liệu — không dùng hạn 15 s mặc định.
pub const LONG_POST_TIMEOUT: Duration = Duration::from_secs(600);
pub async fn post_long<T: serde::de::DeserializeOwned>(
    base: &str,
    token: &str,
    path: &str,
    body: &serde_json::Value,
) -> Result<T, ClientError> {
    parse(
        http()
            .post(api(base, path))
            .timeout(LONG_POST_TIMEOUT)
            .bearer_auth(token)
            .json(body)
            .send()
            .await,
    )
    .await
}
pub async fn f_20e<T: serde::de::DeserializeOwned>(
    r_f4: &str,
    r_f6: &str,
    path: &str,
    r_f5: &serde_json::Value,
) -> Result<T, ClientError> {
    parse(
        http()
            .patch(api(r_f4, path))
            .bearer_auth(r_f6)
            .json(r_f5)
            .send()
            .await,
    )
    .await
}
fn f_29h(r_f7: &reqwest::Response, name: &str) -> Option<u64> {
    r_f7.headers().get(name)?.to_str().ok()?.parse().ok()
}
async fn f_rt(r_f9: reqwest::Response) -> ClientError {
    let status = r_f9.status().as_u16();
    let r_f8 = r_f9.json::<T6x>().await.unwrap_or(T6x {
        code: None,
        message: None,
    });
    ClientError::Api {
        status,
        code: r_f8.code.unwrap_or_else(|| "error".into()),
        message: r_f8
            .message
            .unwrap_or_else(|| format!("máy chủ trả lỗi HTTP {status}")),
    }
}
#[derive(Debug)]
pub enum UploadProbe {
    Complete,
    Offset(u64),
}
pub async fn f_2nq(
    r_fa: &str,
    r_fc: &str,
    project_id: &str,
    blake3: &str,
    filename: &str,
    rel_path: Option<&str>,
) -> Result<UploadProbe, ClientError> {
    // Gửi tên + đường dẫn: "đã có" phải là đúng tệp này, không phải tệp khác cùng nội dung.
    let mut q: Vec<(&str, &str)> = vec![("projectId", project_id), ("blake3", blake3), ("filename", filename)];
    if let Some(r) = rel_path {
        q.push(("relPath", r));
    }
    let r_fb = http()
        .head(api(r_fa, "/documents/upload"))
        .timeout(UPLOAD_TIMEOUT)
        .bearer_auth(r_fc)
        .query(&q)
        .send()
        .await
        .map_err(|e| ClientError::Unreachable(e.to_string()))?;
    if !r_fb.status().is_success() {
        return Err(f_rt(r_fb).await);
    }
    if r_fb.headers().get("upload-complete").is_some() {
        return Ok(UploadProbe::Complete);
    }
    Ok(UploadProbe::Offset(
        f_29h(&r_fb, "upload-offset").unwrap_or(0),
    ))
}
#[derive(Debug, PartialEq, Eq)]
pub enum ChunkOutcome {
    More(u64),
    Conflict(u64),
    Done { duplicate: bool },
}
#[allow(clippy::too_many_arguments)]
pub async fn f_2nk(
    r_fd: &str,
    r_fn: &str,
    project_id: &str,
    filename: &str,
    r_fg: Option<&str>,
    r_fi: Option<i64>,
    blake3: &str,
    r_fh: Option<&str>,
    r_fe: Option<&str>,
    r_fl: Option<&str>,
    offset: u64,
    r_fo: u64,
    r_ff: Vec<u8>,
) -> Result<ChunkOutcome, ClientError> {
    let mut r_fk: Vec<(&str, String)> = vec![
        ("projectId", project_id.to_string()),
        ("filename", filename.to_string()),
        ("blake3", blake3.to_string()),
    ];
    if let Some(r) = r_fg {
        r_fk.push(("hoSoRef", r.to_string()));
    }
    if let Some(r_fj) = r_fi {
        r_fk.push(("pageCount", r_fj.to_string()));
    }
    if let Some(r_fp) = r_fh {
        r_fk.push(("onBehalfOf", r_fp.to_string()));
    }
    if let Some(b) = r_fe {
        r_fk.push(("batchId", b.to_string()));
    }
    if let Some(r) = r_fl {
        r_fk.push(("relPath", r.to_string()));
    }
    let r_fm = http()
        .patch(api(r_fd, "/documents/upload"))
        // Khúc cuối còn PUT kho object + ghi CSDL: máy chủ ổ quay bận có thể quá 15 s mặc định
        // (25/9/2026) → báo nhầm "Không kết nối được máy chủ". Tải lên được chờ lâu hơn.
        .timeout(UPLOAD_TIMEOUT)
        .bearer_auth(r_fn)
        .query(&r_fk)
        .header("Upload-Offset", offset.to_string())
        .header("Upload-Length", r_fo.to_string())
        .header("content-type", "application/offset+octet-stream")
        .body(r_ff)
        .send()
        .await
        .map_err(|e| ClientError::Unreachable(e.to_string()))?;
    let status = r_fm.status();
    if status.as_u16() == 201 || r_fm.headers().get("upload-complete").is_some() {
        return Ok(ChunkOutcome::Done {
            duplicate: r_fm.headers().get("upload-duplicate").is_some(),
        });
    }
    if status.as_u16() == 409 {
        return Ok(ChunkOutcome::Conflict(
            f_29h(&r_fm, "upload-offset").unwrap_or(0),
        ));
    }
    if status.is_success() {
        return Ok(ChunkOutcome::More(
            f_29h(&r_fm, "upload-offset").unwrap_or(offset),
        ));
    }
    Err(f_rt(r_fm).await)
}
pub async fn f_1ei(r_fq: &str, r_ft: &str, path: &str) -> Result<Vec<u8>, ClientError> {
    let r_fs = http()
        .get(api(r_fq, path))
        .bearer_auth(r_ft)
        .timeout(C_4D)
        .send()
        .await
        .map_err(|e| ClientError::Unreachable(e.to_string()))?;
    let status = r_fs.status();
    if !status.is_success() {
        let r_fr = r_fs.json::<T6x>().await.unwrap_or(T6x {
            code: None,
            message: None,
        });
        return Err(ClientError::Api {
            status: status.as_u16(),
            code: r_fr.code.unwrap_or_else(|| "error".into()),
            message: r_fr
                .message
                .unwrap_or_else(|| format!("máy chủ trả lỗi HTTP {status}")),
        });
    }
    r_fs.bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| ClientError::Unreachable(format!("tải file lỗi: {e}")))
}
const C_4D: Duration = Duration::from_secs(30 * 60);
pub async fn f_1ez(
    r_fu: &str,
    r_fz: &str,
    path: &str,
    r_fx: &std::path::Path,
    r_fw: &(dyn Fn(u64, Option<u64>) + Sync),
) -> Result<u64, ClientError> {
    use tokio::io::AsyncWriteExt;
    let mut r_fy = http()
        .get(api(r_fu, path))
        .bearer_auth(r_fz)
        .timeout(C_4D)
        .send()
        .await
        .map_err(|e| ClientError::Unreachable(e.to_string()))?;
    if !r_fy.status().is_success() {
        return Err(f_rt(r_fy).await);
    }
    let r_g0 = r_fy.content_length();
    let mut file = tokio::fs::File::create(r_fx).await.map_err(|e| {
        ClientError::Unreachable(format!(
            "không tạo được file tạm: {e}"
        ))
    })?;
    let mut received: u64 = 0;
    r_fw(0, r_g0);
    while let Some(r_fv) = r_fy
        .chunk()
        .await
        .map_err(|e| ClientError::Unreachable(format!("tải file lỗi: {e}")))?
    {
        file.write_all(&r_fv)
            .await
            .map_err(|e| ClientError::Unreachable(format!("ghi file lỗi: {e}")))?;
        received += r_fv.len() as u64;
        r_fw(received, r_g0);
    }
    file.flush()
        .await
        .map_err(|e| ClientError::Unreachable(format!("ghi file lỗi: {e}")))?;
    drop(file);
    match r_g0 {
        Some(t) if received != t => {
            Err(
                ClientError::Unreachable(
                    format!(
                        "tải thiếu: nhận {received}/{t} byte (đứt giữa chừng)"
                    ),
                ),
            )
        }
        _ => Ok(received),
    }
}
pub async fn f_24x<T: serde::de::DeserializeOwned>(
    r_g1: &str,
    r_g3: &str,
    path: &str,
    r_g2: &serde_json::Value,
) -> Result<T, ClientError> {
    parse(
        http()
            .put(api(r_g1, path))
            .bearer_auth(r_g3)
            .json(r_g2)
            .send()
            .await,
    )
    .await
}
pub async fn f_13z<T: serde::de::DeserializeOwned>(
    r_g4: &str,
    r_g5: &str,
    path: &str,
) -> Result<T, ClientError> {
    parse(
        http()
            .delete(api(r_g4, path))
            .bearer_auth(r_g5)
            .send()
            .await,
    )
    .await
}
pub async fn f_23c(
    r_g6: &str,
    r_g8: &str,
    path: &str,
    r_g7: &serde_json::Value,
) -> Result<(), ClientError> {
    f_1yv(
        http()
            .post(api(r_g6, path))
            .bearer_auth(r_g8)
            .json(r_g7)
            .send()
            .await,
    )
    .await
}
#[derive(Debug, serde::Deserialize)]
pub struct Tpd {
    pub id: String,
    pub duplicate: bool,
}
#[allow(clippy::too_many_arguments)]
pub async fn f_2nn(
    r_g9: &str,
    r_gf: &str,
    project_id: &str,
    filename: &str,
    r_gb: Option<&str>,
    r_gc: Option<i64>,
    blake3: &str,
    r_ga: Vec<u8>,
) -> Result<Tpd, ClientError> {
    let mut r_ge: Vec<(&str, String)> = vec![
        ("projectId", project_id.to_string()),
        ("filename", filename.to_string()),
        ("blake3", blake3.to_string()),
    ];
    if let Some(r) = r_gb {
        r_ge.push(("hoSoRef", r.to_string()));
    }
    if let Some(r_gd) = r_gc {
        r_ge.push(("pageCount", r_gd.to_string()));
    }
    parse(
        http()
            .post(api(r_g9, "/documents"))
            .bearer_auth(r_gf)
            .query(&r_ge)
            .header("content-type", "application/pdf")
            .body(r_ga)
            .send()
            .await,
    )
    .await
}
