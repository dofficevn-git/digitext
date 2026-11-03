// Copyright (c) 2026 DigiText. All rights reserved.
use crate::config;
use crate::db::T96;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhongIn {
    pub ma_du_an: String,
    pub ten_du_an: String,
    #[serde(default)]
    pub ma_khoi: Option<String>,
    #[serde(default)]
    pub phong_so: Option<String>,
    #[serde(default)]
    pub muc_luc_so: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T97 {
    pub hop_so: String,
    pub ho_so_so: String,
    #[serde(default)]
    pub tieu_de: String,
    #[serde(default)]
    pub thoi_gian: Option<String>,
    #[serde(default)]
    pub so_to: Option<String>,
    #[serde(default)]
    pub thbq: Option<String>,
    #[serde(default)]
    pub ghi_chu: Option<String>,
    #[serde(default)]
    pub nhom: Option<String>,
    #[serde(default)]
    pub nam: Option<String>,
    #[serde(default)]
    pub ngay_bat_dau: Option<String>,
    #[serde(default)]
    pub ngay_ket_thuc: Option<String>,
}
impl T97 {
    pub fn to_input(&self, phong_id: &str) -> T96 {
        T96 {
            phong_id: Some(phong_id.to_string()),
            hop_so: self.hop_so.clone(),
            ho_so_so: self.ho_so_so.clone(),
            tieu_de: self.tieu_de.clone(),
            thoi_gian: self.thoi_gian.clone(),
            so_to: self.so_to.clone(),
            thbq: self.thbq.clone(),
            ghi_chu: self.ghi_chu.clone(),
            nhom: self.nhom.clone(),
            nam: self.nam.clone(),
            ngay_bat_dau: self.ngay_bat_dau.clone(),
            ngay_ket_thuc: self.ngay_ket_thuc.clone(),
        }
    }
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tgg {
    pub source_path: String,
    #[serde(default)]
    pub dpi: u32,
    #[serde(default)]
    pub paper: Option<String>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T86 {
    #[serde(default)]
    pub storage_root: Option<String>,
    #[serde(default = "default_quality")]
    pub jpeg_quality: u8,
    #[serde(default)]
    pub strip_accents: bool,
}
fn default_quality() -> u8 {
    config::C_3W
}
impl Default for T86 {
    fn default() -> Self {
        Self {
            storage_root: None,
            jpeg_quality: default_quality(),
            strip_accents: false,
        }
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T85 {
    pub phong: PhongIn,
    pub ho_so: T97,
    pub pages: Vec<Tgg>,
    #[serde(default)]
    pub options: T86,
    #[serde(default)]
    pub pdf_out: Option<String>,
    #[serde(default)]
    pub ho_so_id: Option<i64>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tfh {
    pub ho_so_id: i64,
    #[serde(default)]
    pub pdf_path: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T6s {
    pub batch_id: String,
    pub ho_so: crate::db::T94,
    pub pages: Vec<crate::scanner::Tkt>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T9w {
    pub batch_id: String,
    pub paths: Vec<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tfj {
    pub pdf_path: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tiq {
    pub pdf_path: String,
    #[serde(default)]
    pub from: Option<u32>,
    #[serde(default)]
    pub count: Option<u32>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tif {
    pub pdf_out: String,
    pub pages: Vec<Tgg>,
    #[serde(default)]
    pub options: T86,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RebuildResult {
    pub pdf_path: String,
    pub pdf_size: i64,
    pub page_count: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T87 {
    pub ho_so_id: i64,
    pub folder_path: String,
    pub pdf_path: String,
    pub pdf_size: i64,
    pub page_count: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Td8 {
    pub phong: PhongIn,
    pub ho_so: T97,
    pub pages: Vec<Tgg>,
    #[serde(default)]
    pub options: T86,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tdb {
    pub pdf_path: String,
    pub pdf_size: i64,
    pub page_count: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T9o {
    pub phong: PhongIn,
    pub ho_sos: Vec<T97>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tau {
    #[serde(default)]
    pub phong_id: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T95 {
    pub id: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T99 {
    pub id: i64,
    pub ho_so: T97,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tfi {
    pub path: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tkk {
    pub data: Vec<u8>,
    #[serde(default)]
    pub ext: Option<String>,
    #[serde(default)]
    pub default_name: Option<String>,
    #[serde(default)]
    pub open: bool,
    /// Có `dir` → ghi thẳng `dir/default_name` (tạo thư mục nếu thiếu), không mở hộp thoại (bộ hồ sơ nghiệm thu, 3/11/2026).
    #[serde(default)]
    pub dir: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T8a {
    pub phong: PhongIn,
    pub ho_sos: Vec<T97>,
    #[serde(default)]
    pub options: T86,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T8b {
    pub root_path: String,
    pub phong_folder: String,
    pub created: usize,
    pub synced_dossier: usize,
}
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T9p {
    pub ho_so: usize,
    pub dossier: usize,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tlk {
    #[serde(default)]
    pub path: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tgu {
    pub path: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tgv {
    pub path: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tip {
    pub path: String,
    pub new_name: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T39 {
    pub parent_path: String,
    pub name: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tdk {
    pub path: String,
    pub dest_dir: String,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T4q {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub is_pdf: bool,
    pub size: u64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tie {
    #[serde(default)]
    pub path: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T88 {
    pub folder_path: String,
}
