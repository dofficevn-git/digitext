// Copyright (c) 2026 DigiText. All rights reserved.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Th7 {
    pub ma_dv: String,
    pub ma_phong: String,
    #[serde(default)]
    pub ma_don_vi: Option<String>,
    pub ten_don_vi: String,
    #[serde(default)]
    pub so_tt: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Th8 {
    pub ma_dv: String,
    pub ma_phong: String,
    pub ma_don_vi: Option<String>,
    pub ten_don_vi: String,
    pub so_tt: Option<i64>,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tcz {
    pub mang_cvid: i64,
    #[serde(default)]
    pub ma_mang: Option<String>,
    pub ten_mang: String,
    pub ma_dv: String,
    #[serde(default)]
    pub so_tt: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Td0 {
    pub mang_cvid: i64,
    pub ma_mang: Option<String>,
    pub ten_mang: String,
    pub ma_dv: String,
    pub so_tt: Option<i64>,
}
#[derive(Debug, Clone, Deserialize)]
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
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Te3 {
    pub ma_ngon_ngu: String,
    pub ten_ngon_ngu: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Te4 {
    pub ma_ngon_ngu: String,
    pub ten_ngon_ngu: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tb6 {
    pub ma_loai: String,
    pub ten_loai: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tb7 {
    pub ma_loai: String,
    pub ten_loai: String,
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThbqIn {
    pub thbqid: i64,
    pub ten_thbq: String,
    #[serde(default = "default_trang_thai")]
    pub trang_thai: i64,
    #[serde(default)]
    pub gia_tri: i64,
    #[serde(default)]
    pub sort: i64,
}
fn default_trang_thai() -> i64 {
    2
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct To1 {
    pub thbqid: i64,
    pub ten_thbq: String,
    pub trang_thai: i64,
    pub gia_tri: i64,
    pub sort: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Th6 {
    pub phong_ban: Th7,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tcp {
    pub ma_dv: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tcx {
    pub mang: Tcz,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tcy {
    pub mang_cvid: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tnz {
    pub thbq: ThbqIn,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct To0 {
    pub thbqid: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Th5 {
    pub phong: PhongIn,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tco {
    pub ma_du_an: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Te2 {
    pub ngon_ngu: Te3,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tct {
    pub ma_ngon_ngu: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tb5 {
    pub loai_van_ban: Tb6,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tcs {
    pub ma_loai: String,
}
