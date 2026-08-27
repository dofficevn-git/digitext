// Copyright (c) 2026 DigiText. All rights reserved.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tlt {
    pub ma_du_an: Option<String>,
    pub ten_du_an: Option<String>,
    pub so_ho_so: i64,
    pub so_pdf: i64,
    pub a0: i64,
    pub a1: i64,
    pub a2: i64,
    pub a3: i64,
    pub a4: i64,
    pub a5: i64,
    pub khac: i64,
    pub tong_trang: i64,
    pub quy_doi: f64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T2r {
    pub ma_dv: Option<String>,
    pub ten_don_vi: Option<String>,
    pub hop_vinh_vien: i64,
    pub ho_so_vinh_vien: i64,
    pub hop_co_thoi_han: i64,
    pub ho_so_co_thoi_han: i64,
    pub bo_loai: i64,
    pub tap_loai: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Te5 {
    pub user_id: String,
    pub ten: String,
    pub so_ho_so: i64,
    pub so_van_ban: i64,
    pub so_trang_quet: i64,
    pub thao_tac_them: i64,
    pub thao_tac_sua: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Th4 {
    pub ma_du_an: Option<String>,
    pub ten_du_an: Option<String>,
    pub thoi_gian_khoi_tao: Option<String>,
    pub hop_vinh_vien: i64,
    pub ho_so_vinh_vien: i64,
    pub hop_co_thoi_han: i64,
    pub ho_so_co_thoi_han: i64,
    pub bo_loai: i64,
    pub tap_loai: i64,
}
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T2q {
    #[serde(default)]
    pub ma_da: Option<String>,
}
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Th3 {
    #[serde(default)]
    pub tu_ngay: Option<String>,
    #[serde(default)]
    pub den_ngay: Option<String>,
}
