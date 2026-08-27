// Copyright (c) 2026 DigiText. All rights reserved.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T54 {
    #[serde(default)]
    pub ma_da: Option<String>,
    #[serde(default)]
    pub ma_dv: Option<String>,
    #[serde(default)]
    pub mang_cvid: Option<i64>,
    #[serde(default)]
    pub thbq: Option<i64>,
    pub so_tam: String,
    #[serde(default)]
    pub ma_ho_so: Option<String>,
    #[serde(default)]
    pub tieu_de: String,
    #[serde(default)]
    pub ngay_bat_dau: Option<String>,
    #[serde(default)]
    pub ngay_ket_thuc: Option<String>,
    #[serde(default)]
    pub so_to: Option<String>,
    #[serde(default)]
    pub trung_thua: bool,
    #[serde(default)]
    pub tong_so_van_ban: Option<i64>,
    #[serde(default)]
    pub so_luong_trang: Option<i64>,
    #[serde(default)]
    pub che_do_su_dung: Option<String>,
    #[serde(default)]
    pub ngon_ngu: Option<String>,
    #[serde(default)]
    pub tinh_trang_vat_ly: Option<String>,
    #[serde(default)]
    pub ghi_chu: Option<String>,
    #[serde(default)]
    pub hop_so: Option<String>,
    #[serde(default)]
    pub bo_so: Option<String>,
    #[serde(default)]
    pub tap_so: Option<String>,
    #[serde(default)]
    pub trang_thai: Option<String>,
}
impl T54 {
    pub fn trang_thai_or_default(&self) -> String {
        match self.trang_thai.as_deref().map(str::trim) {
            Some(r_1ul) if !r_1ul.is_empty() => r_1ul.to_string(),
            _ => "temp".to_string(),
        }
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T55 {
    pub id: i64,
    pub ma_da: Option<String>,
    pub ten_da: Option<String>,
    pub ma_dv: Option<String>,
    pub mang_cvid: Option<i64>,
    pub thbq: Option<i64>,
    pub so_tam: String,
    pub ma_ho_so: Option<String>,
    pub tieu_de: String,
    pub ngay_bat_dau: Option<String>,
    pub ngay_ket_thuc: Option<String>,
    pub so_to: Option<String>,
    pub trung_thua: bool,
    pub tong_so_van_ban: Option<i64>,
    pub so_luong_trang: Option<i64>,
    pub che_do_su_dung: Option<String>,
    pub ngon_ngu: Option<String>,
    pub tinh_trang_vat_ly: Option<String>,
    pub ghi_chu: Option<String>,
    pub hop_so: Option<String>,
    pub bo_so: Option<String>,
    pub tap_so: Option<String>,
    pub trang_thai: String,
    pub so_van_ban: i64,
    pub so_tai_lieu_scan: i64,
    pub scan_pdf_path: Option<String>,
    pub so_file_scan: i64,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub thu_tu_sap_xep: Option<i64>,
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DossierFilter {
    #[serde(default)]
    pub ma_da: Option<String>,
    #[serde(default)]
    pub ma_dv: Option<String>,
    #[serde(default)]
    pub mang_cvid: Option<i64>,
    #[serde(default)]
    pub thbq: Option<i64>,
    #[serde(default)]
    pub search: Option<String>,
}
impl DossierFilter {
    fn nonempty(r_1um: &Option<String>) -> Option<&str> {
        r_1um
            .as_deref()
            .map(str::trim)
            .filter(|r_1un| !r_1un.is_empty())
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
}
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VanBanIn {
    #[serde(default)]
    pub thu_tu: i64,
    #[serde(default)]
    pub loai_van_ban: Option<String>,
    #[serde(default)]
    pub so_ky_hieu: Option<String>,
    #[serde(default)]
    pub ngay_thang: Option<String>,
    #[serde(default)]
    pub co_quan_ban_hanh: Option<String>,
    #[serde(default)]
    pub trich_yeu: Option<String>,
    #[serde(default)]
    pub to_so: Option<i64>,
    #[serde(default)]
    pub so_trang: Option<i64>,
    #[serde(default)]
    pub trang_so: Option<i64>,
    #[serde(default)]
    pub ngon_ngu: Option<String>,
    #[serde(default)]
    pub che_do_su_dung: Option<String>,
    #[serde(default)]
    pub do_tin_cay: Option<String>,
    #[serde(default)]
    pub but_tich: Option<String>,
    #[serde(default)]
    pub tinh_trang_vat_ly: Option<String>,
    #[serde(default)]
    pub tai_lieu_so: Option<String>,
    #[serde(default)]
    pub ghi_chu: Option<String>,
    #[serde(default)]
    pub pdf_path: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tpq {
    pub id: i64,
    pub dossier_id: i64,
    pub thu_tu: i64,
    pub loai_van_ban: Option<String>,
    pub so_ky_hieu: Option<String>,
    pub ngay_thang: Option<String>,
    pub co_quan_ban_hanh: Option<String>,
    pub trich_yeu: Option<String>,
    pub to_so: Option<i64>,
    pub so_trang: Option<i64>,
    pub trang_so: Option<i64>,
    pub ngon_ngu: Option<String>,
    pub che_do_su_dung: Option<String>,
    pub do_tin_cay: Option<String>,
    pub but_tich: Option<String>,
    pub tinh_trang_vat_ly: Option<String>,
    pub tai_lieu_so: Option<String>,
    pub ghi_chu: Option<String>,
    pub pdf_path: Option<String>,
    pub created_at: String,
    pub updated_at: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tay {
    #[serde(default)]
    pub filter: Option<DossierFilter>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T53 {
    pub id: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tmd {
    #[serde(default)]
    pub ma_da: Option<String>,
}
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tmf {
    pub created: usize,
    pub skipped_existing: usize,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T38 {
    pub dossier: T54,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tp5 {
    pub id: i64,
    pub dossier: T54,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tb2 {
    pub dossier_id: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T3j {
    pub dossier_id: i64,
    pub van_ban: VanBanIn,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tp7 {
    pub id: i64,
    pub van_ban: VanBanIn,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tpo {
    pub id: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tmg {
    #[serde(default)]
    pub ma_da: Option<String>,
    #[serde(default)]
    pub scheme: Option<String>,
    #[serde(default)]
    pub per_box: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tlc {
    pub id: i64,
    #[serde(default)]
    pub hop_so: Option<String>,
    #[serde(default)]
    pub bo_so: Option<String>,
    #[serde(default)]
    pub tap_so: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T1h {
    #[serde(default)]
    pub ma_da: Option<String>,
    pub rows: Vec<T54>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tfk {
    pub url: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tle {
    #[serde(default)]
    pub url: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T91 {
    #[serde(default)]
    pub entity: Option<String>,
    #[serde(default)]
    pub ma_da: Option<String>,
    #[serde(default)]
    pub tu_ngay: Option<String>,
    #[serde(default)]
    pub den_ngay: Option<String>,
    #[serde(default)]
    pub search: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tpp {
    pub dossier_id: i64,
    pub van_ban: VanBanIn,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T1i {
    pub items: Vec<Tpp>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct T9r {
    pub van_ban_id: i64,
    pub dossier_id: i64,
    pub ho_so: String,
    pub thu_tu: i64,
    pub so_ky_hieu: Option<String>,
    pub trich_yeu: Option<String>,
    pub ngay_thang: Option<String>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tkl {
    pub file_name: String,
    pub file_path: String,
    pub dpi: i64,
    pub is_pass: bool,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tkp {
    pub file_name: String,
    pub file_path: String,
    pub page_count: i64,
    pub loai_van_ban: Option<String>,
    pub so_ky_hieu: Option<String>,
    pub ngay_thang: Option<String>,
    pub tac_gia: Option<String>,
    pub trich_yeu: Option<String>,
    pub tinh_trang_vat_ly: Option<String>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct T9q {
    #[serde(default)]
    pub filter: Option<DossierFilter>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tkm {
    pub ids: Vec<i64>,
    #[serde(default)]
    pub min_dpi: Option<i64>,
}
