// Copyright (c) 2026 DigiText. All rights reserved.
//! Nhập gói vào Hệ thống (bản Lưu trữ GĐ3, 4/11/2026): tìm gói .zip trong thư mục chọn, tải từng gói lên máy chủ theo khúc
//! 8 MB (đứt mạng tải tiếp từ vị trí máy chủ báo), rồi gọi máy chủ nhập (`routes/ingest.rs`). Máy chủ kiểm gói, dựng mục lục,
//! lưu tài liệu — máy trạm chỉ chuyển tệp.
use crate::auth::admin::f_2r8;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Permission;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::ipc::Channel;
use tauri::State;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

const KHUC: usize = 8 * 1024 * 1024;
const QUYEN: Permission = Permission::SubmissionApprove;

fn pct(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(*b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoiTimThay {
    pub path: String,
    pub name: String,
    pub size: u64,
    /// Thư mục gói đã giải nén — phải nén lại thành .zip trước khi tải lên.
    pub la_thu_muc: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverArgs {
    pub paths: Vec<String>,
}

#[tauri::command]
pub async fn ingest_discover(auth: State<'_, AuthState>, args: DiscoverArgs) -> Result<Vec<GoiTimThay>, AppError> {
    auth.require_perm(QUYEN)?;
    tauri::async_runtime::spawn_blocking(move || {
        digitext_pkgcheck::discover(&args.paths)
            .into_iter()
            .map(|p| GoiTimThay {
                name: p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default(),
                size: std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0),
                la_thu_muc: p.is_dir(),
                path: p.display().to_string(),
            })
            .collect()
    })
    .await
    .map_err(|e| AppError::internal(format!("tìm gói lỗi: {e}")))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCreateArgs {
    pub project_id: String,
    #[serde(default)]
    pub ghi_chu: Option<String>,
}

#[tauri::command]
pub async fn ingest_batch_create(db: State<'_, Db>, auth: State<'_, AuthState>, args: BatchCreateArgs) -> Result<Value, AppError> {
    let body = json!({ "projectId": args.project_id, "ghiChu": args.ghi_chu });
    f_2r8(&db, &auth, QUYEN, move |base, token| {
        let body = body.clone();
        async move { client::f_23d::<Value>(&base, &token, "/ingest/batches", &body).await }
    })
    .await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectArgs {
    pub project_id: String,
}

#[tauri::command]
pub async fn ingest_batches(db: State<'_, Db>, auth: State<'_, AuthState>, args: ProjectArgs) -> Result<Value, AppError> {
    let path = format!("/ingest/batches?projectId={}", pct(&args.project_id));
    f_2r8(&db, &auth, QUYEN, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<Value>(&base, &token, &path).await }
    })
    .await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchArgs {
    pub id: String,
}

#[tauri::command]
pub async fn ingest_batch_items(db: State<'_, Db>, auth: State<'_, AuthState>, args: BatchArgs) -> Result<Value, AppError> {
    let path = format!("/ingest/batches/{}", pct(&args.id));
    f_2r8(&db, &auth, QUYEN, move |base, token| {
        let path = path.clone();
        async move { client::f_1eo::<Value>(&base, &token, &path).await }
    })
    .await
}

#[tauri::command]
pub async fn ingest_batch_finish(db: State<'_, Db>, auth: State<'_, AuthState>, args: BatchArgs) -> Result<Value, AppError> {
    let path = format!("/ingest/batches/{}/finish", pct(&args.id));
    f_2r8(&db, &auth, QUYEN, move |base, token| {
        let path = path.clone();
        async move { client::f_23d::<Value>(&base, &token, &path, &json!({})).await }
    })
    .await
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TienDo {
    pub da_gui: u64,
    pub tong: u64,
    /// "tai_len" khi đang gửi khúc, "nhap" khi máy chủ đang kiểm và nhập.
    pub buoc: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadArgs {
    pub batch_id: String,
    pub path: String,
    #[serde(default)]
    pub nhan_ca_goi_loi: bool,
}

/// Tải một gói .zip lên rồi nhập; trả kết quả của máy chủ (`KetQuaGoi`).
#[tauri::command]
pub async fn ingest_upload(db: State<'_, Db>, auth: State<'_, AuthState>, args: UploadArgs, on_progress: Channel<TienDo>) -> Result<Value, AppError> {
    let p = std::path::PathBuf::from(&args.path);
    if !p.is_file() {
        return Err(AppError::validation("chỉ tải lên được tệp .zip — thư mục gói đã giải nén cần nén lại trước"));
    }
    let ten = p.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let tong = std::fs::metadata(&p)?.len();
    let lo = pct(&args.batch_id);
    let mut f = tokio::fs::File::open(&p).await?;
    let mut offset: u64 = 0;
    let mut lan_lech = 0;
    while offset < tong {
        let n = KHUC.min((tong - offset) as usize);
        let mut buf = vec![0u8; n];
        f.seek(std::io::SeekFrom::Start(offset)).await?;
        f.read_exact(&mut buf).await?;
        let path = format!("/ingest/batches/{lo}/chunk?ten={}&offset={offset}", pct(&ten));
        let (status, v) = f_2r8(&db, &auth, QUYEN, move |base, token| {
            let path = path.clone();
            let buf = buf.clone();
            async move { client::post_bytes(&base, &token, &path, buf).await }
        })
        .await?;
        let moi = v.get("offset").and_then(Value::as_u64).ok_or_else(|| AppError::internal("máy chủ không trả vị trí tải lên"))?;
        if status == 409 {
            lan_lech += 1;
            if lan_lech > 3 || moi > tong {
                return Err(AppError::internal(format!("tải lên lệch vị trí ({moi}/{tong}) — chạy lại để tải tiếp")));
            }
        }
        offset = moi;
        let _ = on_progress.send(TienDo { da_gui: offset, tong, buoc: "tai_len" });
    }
    let _ = on_progress.send(TienDo { da_gui: tong, tong, buoc: "nhap" });
    let path = format!("/ingest/batches/{lo}/nhap?ten={}&nhanCaGoiLoi={}", pct(&ten), args.nhan_ca_goi_loi);
    f_2r8(&db, &auth, QUYEN, move |base, token| {
        let path = path.clone();
        async move { client::post_long::<Value>(&base, &token, &path, &json!({})).await }
    })
    .await
}
