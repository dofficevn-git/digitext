// Copyright (c) 2026 DigiText. All rights reserved.
//! Mô tả các bảng cl_* (chỉnh lý): cột client được ghi, cột chỉ online, thứ tự cha → con, quyền.
//! Mọi thao tác ghi (online lẫn push offline) đi qua cùng một bảng mô tả này.

use digitext_core::Permission;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermKind {
    /// Danh mục: catalog.create / update / delete.
    Catalog,
    /// Hồ sơ, văn bản, tệp: dossier.create / update / delete.
    Dossier,
    /// Bảng chỉ online, một quyền cho mọi thao tác.
    Online(Permission),
}

#[derive(Debug, Clone, Copy)]
pub struct TableSpec {
    pub name: &'static str,
    /// Thứ tự áp op: cha nhỏ hơn con.
    pub rank: u8,
    /// Cột nghiệp vụ client được ghi (offline lẫn online).
    pub cols: &'static [&'static str],
    /// Cột chỉ được ghi qua route online (push đổi giá trị → từ chối).
    pub online_only: &'static [&'static str],
    /// Cột cha + bảng cha (kiểm tra cùng dự án).
    pub parent: Option<(&'static str, &'static str)>,
    /// `project_id` được NULL (dòng seed dùng chung).
    pub shared: bool,
    /// Bảng sửa được offline (push nhận op). Bảng khác chỉ pull.
    pub offline_write: bool,
    pub perm: PermKind,
    /// Có ghi lịch sử (cl_lich_su) không.
    pub history: bool,
}

/// Cột hệ thống mọi bảng sync đều có, do máy chủ quản.
pub const SYSTEM_COLS: &[&str] = &["id", "project_id", "ver", "created_at", "created_by", "updated_at", "updated_by", "deleted_at"];

const DM: &[&str] = &["ma", "ten", "so_tt"];

pub const TABLES: &[TableSpec] = &[
    TableSpec { name: "cl_khoi_phong", rank: 10, cols: DM, online_only: &[], parent: None, shared: false, offline_write: true, perm: PermKind::Catalog, history: false },
    TableSpec {
        name: "cl_phong",
        rank: 20,
        cols: &["khoi_id", "ma_du_an", "ten", "phong_so", "muc_luc_so", "logo_key", "ghi_chu", "so_tt"],
        online_only: &[],
        parent: Some(("khoi_id", "cl_khoi_phong")),
        shared: false,
        offline_write: true,
        perm: PermKind::Catalog,
        history: false,
    },
    TableSpec {
        name: "cl_phong_ban",
        rank: 30,
        cols: &["phong_id", "ma_don_vi", "ten", "nhiem_ky", "so_tt"],
        online_only: &[],
        parent: Some(("phong_id", "cl_phong")),
        shared: false,
        offline_write: true,
        perm: PermKind::Catalog,
        history: false,
    },
    TableSpec {
        name: "cl_mang",
        rank: 40,
        cols: &["phong_ban_id", "ma_mang", "ten", "so_tt"],
        online_only: &[],
        parent: Some(("phong_ban_id", "cl_phong_ban")),
        shared: false,
        offline_write: true,
        perm: PermKind::Catalog,
        history: false,
    },
    TableSpec {
        name: "cl_thbq",
        rank: 50,
        cols: &["ma_tt05", "ten", "trang_thai", "gia_tri", "so_tt"],
        online_only: &[],
        parent: None,
        shared: true,
        offline_write: true,
        perm: PermKind::Catalog,
        history: false,
    },
    TableSpec { name: "cl_ngon_ngu", rank: 50, cols: DM, online_only: &[], parent: None, shared: true, offline_write: true, perm: PermKind::Catalog, history: false },
    TableSpec { name: "cl_loai_van_ban", rank: 50, cols: DM, online_only: &[], parent: None, shared: true, offline_write: true, perm: PermKind::Catalog, history: false },
    TableSpec {
        name: "cl_hop",
        rank: 60,
        cols: &["phong_ban_id", "hop_so", "noi_bao_quan", "vi_tri_gia", "vi_tri_ke", "phong_so", "nam", "tu_ho_so", "den_ho_so", "vinh_vien", "ghi_chu"],
        online_only: &[],
        parent: Some(("phong_ban_id", "cl_phong_ban")),
        shared: false,
        offline_write: false,
        perm: PermKind::Online(Permission::DossierSystematize),
        history: false,
    },
    TableSpec {
        name: "cl_bo_tap",
        rank: 60,
        cols: &["phong_ban_id", "loai", "so", "ghi_chu"],
        online_only: &[],
        parent: Some(("phong_ban_id", "cl_phong_ban")),
        shared: false,
        offline_write: false,
        perm: PermKind::Online(Permission::DossierSystematize),
        history: false,
    },
    TableSpec {
        name: "cl_kho",
        rank: 60,
        cols: &["ten", "so_do_key", "ghi_chu", "so_tt"],
        online_only: &[],
        parent: None,
        shared: false,
        offline_write: false,
        perm: PermKind::Online(Permission::KhoGiaManage),
        history: false,
    },
    TableSpec {
        name: "cl_gia",
        rank: 70,
        cols: &["kho_id", "phong_ban_id", "ten", "tu_hop", "den_hop", "ghi_chu", "so_tt"],
        online_only: &[],
        parent: Some(("kho_id", "cl_kho")),
        shared: false,
        offline_write: false,
        perm: PermKind::Online(Permission::KhoGiaManage),
        history: false,
    },
    TableSpec {
        name: "cl_ho_so",
        rank: 80,
        cols: HO_SO_COLS,
        online_only: &["ho_so_so", "thu_tu_sap_xep", "hop_id", "bo_id", "tap_id", "gia_id"],
        parent: Some(("phong_id", "cl_phong")),
        shared: false,
        offline_write: true,
        perm: PermKind::Dossier,
        history: true,
    },
    TableSpec {
        name: "cl_van_ban",
        rank: 90,
        cols: VAN_BAN_COLS,
        online_only: &[],
        parent: Some(("ho_so_id", "cl_ho_so")),
        shared: false,
        offline_write: true,
        perm: PermKind::Dossier,
        history: true,
    },
    TableSpec {
        name: "cl_van_ban_file",
        rank: 100,
        cols: &["van_ban_id", "document_id", "path", "name", "size", "format", "so_trang", "kho_giay", "so_tt"],
        online_only: &[],
        parent: Some(("van_ban_id", "cl_van_ban")),
        shared: false,
        offline_write: true,
        perm: PermKind::Dossier,
        history: false,
    },
    TableSpec {
        name: "cl_muon_tra",
        rank: 110,
        cols: &["ho_so_id", "nguoi_muon", "don_vi", "dien_thoai", "email", "noi_dung", "ngay_muon", "ngay_du_kien_tra", "ngay_thuc_te_tra", "trang_thai", "nguoi_lap_phieu", "ghi_chu"],
        online_only: &[],
        parent: Some(("ho_so_id", "cl_ho_so")),
        shared: false,
        offline_write: false,
        perm: PermKind::Online(Permission::LoanManage),
        history: false,
    },
];

pub const HO_SO_COLS: &[&str] = &[
    "phong_id", "phong_ban_id", "mang_id", "thbq_id",
    "so_tam", "so_ky_hieu_hs", "tieu_de", "ngay_bat_dau", "ngay_ket_thuc", "so_to", "tong_so_vb", "so_luong_trang",
    "che_do_su_dung", "ngon_ngu", "tinh_trang_vat_ly", "trung_thua", "ghi_chu",
    "tu_khoa", "ky_hieu_thong_tin", "muc_do_tin_cay", "ma_ho_so_giay",
    "ct_so_to_bang_chu", "ct_danh_so_tu", "ct_den_so", "ct_so_trung", "ct_so_khuyet", "ct_so_muc_luc_vb",
    "ct_so_muc_luc_vb_bang_chu", "ct_dac_diem_tai_lieu", "ct_dia_danh", "ct_ngay", "ct_thang", "ct_nam",
    "ct_nguoi_lap_hs", "ct_phan_bo_sung",
];

pub const VAN_BAN_COLS: &[&str] = &[
    "ho_so_id", "so_thu_tu", "loai_vb", "so_van_ban", "so_ky_hieu", "ngay_thang", "co_quan_ban_hanh", "co_quan_chu_quan",
    "nguoi_ky", "trich_yeu", "to_so", "so_trang", "trang_so", "ngon_ngu", "che_do_su_dung", "do_tin_cay", "but_tich",
    "tinh_trang_vat_ly", "tai_lieu_so", "tu_khoa", "ky_hieu_thong_tin", "ghi_chu",
];

/// Tên bảng sync (kể cả lịch sử) theo thứ tự cha → con, dùng cho pull.
pub const PULL_TABLES: &[&str] = &[
    "cl_khoi_phong", "cl_phong", "cl_phong_ban", "cl_mang", "cl_thbq", "cl_ngon_ngu", "cl_loai_van_ban",
    "cl_hop", "cl_bo_tap", "cl_kho", "cl_gia", "cl_ho_so", "cl_van_ban", "cl_van_ban_file", "cl_muon_tra", "cl_lich_su",
];

pub fn spec(name: &str) -> Option<&'static TableSpec> {
    TABLES.iter().find(|t| t.name == name)
}

impl TableSpec {
    /// Mọi cột client có thể gửi (nghiệp vụ + chỉ online).
    pub fn writable(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.cols.iter().copied().chain(self.online_only.iter().copied())
    }
    pub fn is_writable(&self, col: &str) -> bool {
        self.writable().any(|c| c == col)
    }
    pub fn is_online_only(&self, col: &str) -> bool {
        self.online_only.contains(&col)
    }
    /// Cột ghi vào INSERT / UPDATE: hệ thống (trừ ver / *_at) + nghiệp vụ.
    pub fn insert_cols(&self) -> Vec<&'static str> {
        let mut v = vec!["id", "project_id", "created_by", "updated_by", "deleted_at"];
        v.extend(self.writable());
        v
    }
    pub fn update_cols(&self) -> Vec<&'static str> {
        let mut v = vec!["updated_by", "deleted_at"];
        v.extend(self.writable());
        v
    }
    pub fn perm_for(&self, op: WriteOp) -> Permission {
        match (self.perm, op) {
            (PermKind::Catalog, WriteOp::Insert) => Permission::CatalogCreate,
            (PermKind::Catalog, WriteOp::Update) => Permission::CatalogUpdate,
            (PermKind::Catalog, WriteOp::Delete) => Permission::CatalogDelete,
            (PermKind::Dossier, WriteOp::Insert) => Permission::DossierCreate,
            (PermKind::Dossier, WriteOp::Update) => Permission::DossierUpdate,
            (PermKind::Dossier, WriteOp::Delete) => Permission::DossierDelete,
            (PermKind::Online(p), _) => p,
        }
    }
    /// Tên bảng con (xoá mềm lan xuống).
    pub fn children(&self) -> Vec<&'static TableSpec> {
        TABLES.iter().filter(|t| matches!(t.parent, Some((_, p)) if p == self.name)).collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteOp {
    Insert,
    Update,
    Delete,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bang_va_cot_hop_le() {
        assert_eq!(TABLES.len(), 15);
        for t in TABLES {
            assert!(t.name.starts_with("cl_"));
            for c in t.writable() {
                assert!(!SYSTEM_COLS.contains(&c), "{} có cột hệ thống {c}", t.name);
            }
            if let Some((col, parent)) = t.parent {
                assert!(t.is_writable(col));
                assert!(spec(parent).is_some(), "{} thiếu bảng cha {parent}", t.name);
                assert!(spec(parent).unwrap().rank < t.rank);
            }
        }
        assert!(spec("cl_ho_so").unwrap().is_online_only("ho_so_so"));
        assert!(!spec("cl_ho_so").unwrap().is_online_only("tieu_de"));
        assert_eq!(spec("cl_ho_so").unwrap().children().iter().map(|c| c.name).collect::<Vec<_>>(), vec!["cl_van_ban", "cl_muon_tra"]);
        assert_eq!(spec("cl_van_ban").unwrap().perm_for(WriteOp::Delete), Permission::DossierDelete);
        assert_eq!(spec("cl_kho").unwrap().perm_for(WriteOp::Update), Permission::KhoGiaManage);
        for n in PULL_TABLES {
            assert!(*n == "cl_lich_su" || spec(n).is_some());
        }
    }
}
