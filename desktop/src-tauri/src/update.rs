// Copyright (c) 2026 DigiText. All rights reserved.
//! Tự cập nhật theo máy chủ đang kết nối (4/11/2026): kiểm `<máy chủ>/updates/latest.json` trước; bản thi công
//! thêm các endpoint trong tauri.conf.json làm đường dự phòng, bản lưu trữ chỉ tin máy chủ của cơ quan. Bộ cài
//! vẫn phải đúng chữ ký minisign theo khoá công khai trong app, máy chủ không đẩy được bản lạ.
use crate::auth::commands::server_url;
use crate::db::Db;
use crate::error::AppError;
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use tauri::ipc::Channel;
use tauri::{AppHandle, State, Url};
use tauri_plugin_updater::{Update, UpdaterExt};

#[derive(Default)]
pub struct UpdateState(Mutex<Option<Update>>);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub notes: Option<String>,
}

#[derive(Clone, Serialize)]
#[serde(tag = "event", content = "data", rename_all_fields = "camelCase")]
pub enum UpdateEvent {
    Started { content_length: Option<u64> },
    Progress { chunk_length: usize },
    Finished,
}

fn dung_duoc(u: &Url) -> bool {
    u.scheme() == "https" || cfg!(debug_assertions)
}

pub fn endpoint_may_chu(may_chu: &str) -> Option<Url> {
    Url::parse(&format!("{}/updates/latest.json", may_chu.trim().trim_end_matches('/')))
        .ok()
        .filter(dung_duoc)
}

fn endpoints(app: &AppHandle, db: &State<'_, Db>) -> Result<Vec<Url>, AppError> {
    let mut v: Vec<Url> = server_url(db).ok().and_then(|s| endpoint_may_chu(&s)).into_iter().collect();
    if !crate::edition::LUU_TRU {
        let cau_hinh = app
            .config()
            .plugins
            .0
            .get("updater")
            .and_then(|u| u.get("endpoints"))
            .and_then(|e| e.as_array())
            .cloned()
            .unwrap_or_default();
        for e in cau_hinh {
            if let Some(u) = e.as_str().and_then(|s| Url::parse(s).ok()).filter(dung_duoc) {
                if !v.contains(&u) {
                    v.push(u);
                }
            }
        }
    }
    if v.is_empty() {
        return Err(AppError::new(
            "server_not_configured",
            "Chưa cấu hình địa chỉ máy chủ (https) nên chưa kiểm tra cập nhật được.",
        ));
    }
    Ok(v)
}

#[tauri::command]
pub async fn update_check(
    app: AppHandle,
    db: State<'_, Db>,
    st: State<'_, UpdateState>,
) -> Result<Option<UpdateInfo>, AppError> {
    let eps = endpoints(&app, &db)?;
    let loi = |e: tauri_plugin_updater::Error| AppError::new("update_check", e.to_string());
    let updater = app
        .updater_builder()
        .endpoints(eps)
        .map_err(loi)?
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(loi)?;
    let ban = updater.check().await.map_err(loi)?;
    let info = ban.as_ref().map(|u| UpdateInfo {
        version: u.version.clone(),
        current_version: u.current_version.clone(),
        notes: u.body.clone(),
    });
    *st.0.lock().map_err(|_| AppError::internal("trạng thái cập nhật bị khoá hỏng"))? = ban;
    Ok(info)
}

#[tauri::command]
pub async fn update_install(st: State<'_, UpdateState>, on_event: Channel<UpdateEvent>) -> Result<(), AppError> {
    let ban = st
        .0
        .lock()
        .map_err(|_| AppError::internal("trạng thái cập nhật bị khoá hỏng"))?
        .take()
        .ok_or_else(|| AppError::validation("Chưa có bản cập nhật nào được tìm thấy, hãy kiểm tra lại."))?;
    let mut da_bat_dau = false;
    ban.download_and_install(
        |chunk_length, content_length| {
            if !da_bat_dau {
                da_bat_dau = true;
                let _ = on_event.send(UpdateEvent::Started { content_length });
            }
            let _ = on_event.send(UpdateEvent::Progress { chunk_length });
        },
        || {
            let _ = on_event.send(UpdateEvent::Finished);
        },
    )
    .await
    .map_err(|e| AppError::new("update_install", e.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::endpoint_may_chu;

    #[test]
    fn endpoint_tu_dia_chi_may_chu() {
        assert_eq!(
            endpoint_may_chu("https://luutru.bacgiang.gov.vn/").unwrap().as_str(),
            "https://luutru.bacgiang.gov.vn/updates/latest.json"
        );
        assert_eq!(
            endpoint_may_chu(" https://10.0.0.5:8443 ").unwrap().as_str(),
            "https://10.0.0.5:8443/updates/latest.json"
        );
    }
}
