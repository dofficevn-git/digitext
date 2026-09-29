// Copyright (c) 2026 DigiText. All rights reserved.
use crate::auth::admin::f_2r7;
use crate::auth::client;
use crate::auth::AuthState;
use crate::db::Db;
use crate::error::AppError;
use digitext_core::Perm;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, State};
pub const C_6L: &str = "data://changed";
pub const C_CA: Duration = Duration::from_millis(750);
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Topic {
    Session,
    Users,
    Devices,
    Projects,
    Catalog,
    Hoso,
    Dossier,
    Khogia,
    Scan,
    Collected,
    Lan,
    Tree,
    Docs,
    Ocr,
    OcrFields,
    Submissions,
    Notifications,
    Records,
    Access,
    Disposal,
    Info,
    Packages,
    Sign,
    Stats,
    Ops,
    Settings,
    UploadBatches,
    /// Chỉnh lý (cl_*): mirror đổi sau ghi cục bộ / đồng bộ.
    ChinhLy,
    /// Trạng thái đồng bộ chỉnh lý (outbox, xung đột).
    ClSync,
}
impl Topic {
    #[allow(dead_code)]
    pub const ALL: [Topic; 29] = [
        Topic::Session,
        Topic::Users,
        Topic::Devices,
        Topic::Projects,
        Topic::Catalog,
        Topic::Hoso,
        Topic::Dossier,
        Topic::Khogia,
        Topic::Scan,
        Topic::Collected,
        Topic::Lan,
        Topic::Tree,
        Topic::Docs,
        Topic::Ocr,
        Topic::OcrFields,
        Topic::Submissions,
        Topic::Notifications,
        Topic::Records,
        Topic::Access,
        Topic::Disposal,
        Topic::Info,
        Topic::Packages,
        Topic::Sign,
        Topic::Stats,
        Topic::Ops,
        Topic::Settings,
        Topic::UploadBatches,
        Topic::ChinhLy,
        Topic::ClSync,
    ];
}
#[derive(Debug, Clone, Serialize)]
struct T4i {
    topics: Vec<Topic>,
}
#[derive(Debug, Default)]
pub struct Throttle {
    last: HashMap<Topic, Instant>,
}
impl Throttle {
    pub fn allow(&mut self, r_28v: Topic, r_28t: Instant, r_28s: Duration) -> bool {
        match self.last.get(&r_28v) {
            Some(r_28u) if r_28t.duration_since(*r_28u) < r_28s => false,
            _ => {
                self.last.insert(r_28v, r_28t);
                true
            }
        }
    }
    pub fn forget(&mut self, r_28w: Topic) {
        self.last.remove(&r_28w);
    }
}
fn f_2kr() -> &'static Mutex<Throttle> {
    static T: OnceLock<Mutex<Throttle>> = OnceLock::new();
    T.get_or_init(|| Mutex::new(Throttle::default()))
}
pub fn touch(r_28y: &AppHandle, topics: &[Topic]) {
    let r_28z = Instant::now();
    let r_28x: Vec<Topic> = match f_2kr().lock() {
        Ok(mut r_290) => topics
            .iter()
            .copied()
            .filter(|r_291| r_290.allow(*r_291, r_28z, C_CA))
            .collect(),
        Err(_) => topics.to_vec(),
    };
    emit(r_28y, r_28x);
}
pub fn f_2li(r_292: &AppHandle, topics: &[Topic]) {
    if let Ok(mut r_293) = f_2kr().lock() {
        for r_294 in topics {
            r_293.forget(*r_294);
        }
    }
    emit(r_292, topics.to_vec());
}
fn emit(r_295: &AppHandle, topics: Vec<Topic>) {
    if topics.is_empty() {
        return;
    }
    let _ = r_295.emit(C_6L, T4i { topics });
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Changes {
    pub seq: i64,
    pub topics: Vec<String>,
    pub full: bool,
}
#[tauri::command]
pub async fn changes_poll(
    db: State<'_, Db>,
    auth: State<'_, AuthState>,
    since: Option<i64>,
) -> Result<Changes, AppError> {
    f_2r7(&db, &auth, Perm::Read, move |r_296, r_298| async move {
        let r_297 = match since {
            Some(s) => format!("/changes?since={s}"),
            None => "/changes".to_string(),
        };
        client::f_1eo::<Changes>(&r_296, &r_298, &r_297).await
    })
    .await
}
