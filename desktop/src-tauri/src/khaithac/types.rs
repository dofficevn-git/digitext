// Copyright (c) 2026 DigiText. All rights reserved.
use serde::{Deserialize, Serialize};
fn default_page() -> i64 {
    1
}
fn default_page_size() -> i64 {
    50
}
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KhaiThacFilter {
    #[serde(default)]
    pub ma_da: Option<String>,
    #[serde(default)]
    pub ma_dv: Option<String>,
    #[serde(default)]
    pub mang_cvid: Option<i64>,
    #[serde(default)]
    pub thbqs: Vec<i64>,
    #[serde(default)]
    pub ma_kho: Option<i64>,
    #[serde(default)]
    pub ma_gia: Option<i64>,
    #[serde(default)]
    pub het_thbq: bool,
    #[serde(default)]
    pub giu_lai: bool,
    #[serde(default)]
    pub vinh_vien: bool,
    #[serde(default)]
    pub trung_thua: bool,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}
impl Default for KhaiThacFilter {
    fn default() -> Self {
        Self {
            ma_da: None,
            ma_dv: None,
            mang_cvid: None,
            thbqs: Vec::new(),
            ma_kho: None,
            ma_gia: None,
            het_thbq: false,
            giu_lai: false,
            vinh_vien: false,
            trung_thua: false,
            search: None,
            page: default_page(),
            page_size: default_page_size(),
        }
    }
}
impl KhaiThacFilter {
    fn nonempty(r_20i: &Option<String>) -> Option<&str> {
        r_20i
            .as_deref()
            .map(str::trim)
            .filter(|r_20j| !r_20j.is_empty())
    }
    pub fn ma_da_opt(&self) -> Option<&str> {
        Self::nonempty(&self.ma_da)
    }
    pub fn ma_dv_opt(&self) -> Option<&str> {
        Self::nonempty(&self.ma_dv)
    }
    pub fn search_opt(&self) -> Option<&str> {
        Self::nonempty(&self.search)
    }
    pub fn thbq_ids(&self) -> Vec<i64> {
        self.thbqs
            .iter()
            .copied()
            .filter(|&r_20k| r_20k >= 0)
            .collect()
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ta5 {
    pub ho_so_id: i64,
    pub ma_da: Option<String>,
    pub ten_da: Option<String>,
    pub ma_dv: Option<String>,
    pub mang_cvid: Option<i64>,
    pub ma_ho_so: Option<String>,
    pub hop_tai_lieu_so: Option<String>,
    pub so_bo: Option<String>,
    pub so_tap: Option<String>,
    pub hososo: String,
    pub tieu_de_hs: String,
    pub thbq: Option<i64>,
    pub thoi_han_bao_quan_str: Option<String>,
    pub so_to: Option<String>,
    pub thoi_gian_bat_dau: Option<String>,
    pub thoi_gian_ket_thuc: Option<String>,
    pub trung_thua: bool,
    pub so_luong_trang: Option<i64>,
    pub ghi_chu: Option<String>,
    pub so_van_ban: i64,
    pub ten_kho: Option<String>,
    pub ten_gia: Option<String>,
    pub status: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ta4 {
    pub total: i64,
    pub rows: Vec<Ta5>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ta6 {
    pub dossier_id: i64,
    pub ho_so: String,
    pub hop_so: Option<String>,
    pub thu_tu: i64,
    pub loai_van_ban: Option<String>,
    pub so_ky_hieu: Option<String>,
    pub ngay_thang: Option<String>,
    pub co_quan_ban_hanh: Option<String>,
    pub trich_yeu: Option<String>,
    pub but_tich: Option<String>,
    pub to_so: Option<i64>,
    pub so_trang: Option<i64>,
    pub trang_so: Option<i64>,
    pub ghi_chu: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ta9 {
    pub ma_kho: i64,
    pub ten_kho: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T8o {
    pub ma_gia: i64,
    pub ten_gia: String,
    pub ma_kho: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ta7 {
    pub ma_kho: i64,
    pub ten_kho: String,
    pub so_do_kho: Option<String>,
    pub ghi_chu: Option<String>,
    pub so_tt: Option<i64>,
    pub so_gia: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T8l {
    pub ma_gia: i64,
    pub ten_gia: String,
    pub ma_kho: i64,
    pub ten_kho: Option<String>,
    pub ma_da: Option<String>,
    pub ten_du_an: Option<String>,
    pub ma_dv: Option<String>,
    pub ten_don_vi: Option<String>,
    pub tu_hop: Option<i64>,
    pub den_hop: Option<i64>,
    pub ghi_chu: Option<String>,
    pub so_tt: Option<i64>,
    pub so_ho_so: i64,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ta8 {
    #[serde(default)]
    pub ma_kho: Option<i64>,
    pub ten_kho: String,
    #[serde(default)]
    pub so_do_kho: Option<String>,
    #[serde(default)]
    pub ghi_chu: Option<String>,
    #[serde(default)]
    pub so_tt: Option<i64>,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T8m {
    #[serde(default)]
    pub ma_gia: Option<i64>,
    pub ten_gia: String,
    pub ma_kho: i64,
    #[serde(default)]
    pub ma_da: Option<String>,
    #[serde(default)]
    pub ma_dv: Option<String>,
    #[serde(default)]
    pub tu_hop: Option<i64>,
    #[serde(default)]
    pub den_hop: Option<i64>,
    #[serde(default)]
    pub ghi_chu: Option<String>,
    #[serde(default)]
    pub so_tt: Option<i64>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tky {
    #[serde(default)]
    pub filter: Option<KhaiThacFilter>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T8n {
    #[serde(default)]
    pub ma_kho: Option<i64>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Taa {
    pub kho: Ta8,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tcr {
    pub ma_kho: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T8p {
    pub gia: T8m,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tcq {
    pub ma_gia: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tn {
    #[serde(default)]
    pub ma_gia: Option<i64>,
    pub dossier_ids: Vec<i64>,
}
