// Copyright (c) 2026 DigiText. All rights reserved.
use crate::role::{Perm, Role};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum Permission {
    DossierRead,
    DossierCreate,
    DossierUpdate,
    DossierDelete,
    DossierExport,
    CatalogRead,
    CatalogCreate,
    CatalogUpdate,
    CatalogDelete,
    CatalogImport,
    CatalogExport,
    ScanView,
    ScanExecute,
    EditorView,
    EditorEdit,
    UploadExecute,
    SignView,
    SignExecute,
    DriveView,
    DriveDownload,
    DocumentList,
    DocumentDownload,
    DocumentReview,
    PackageView,
    PackageCreate,
    PackageDownload,
    SubmissionView,
    SubmissionCreate,
    SubmissionReview,
    SubmissionApprove,
    RecordView,
    RecordCreate,
    AccessView,
    AccessRequest,
    AccessReview,
    DisposalView,
    DisposalReview,
    DisposalDecide,
    BkView,
    BkManage,
    TreeView,
    TreeAssign,
    StatsView,
    StatsViewProject,
    UserView,
    UserCreate,
    UserUpdate,
    UserRevoke,
    UserResetPassword,
    UserGrantPermissions,
    ProjectView,
    ProjectCreate,
    ProjectUpdate,
    ProjectAssignAdmin,
    SettingsView,
    SettingsUpdate,
    DeviceView,
    DeviceRevoke,
    /// Làm mục trong gói việc xác nhận trường được giao (CTV).
    WorkExecute,
    /// Tạo / giao / duyệt gói việc xác nhận trường.
    WorkManage,
    /// Chỉnh lý: hệ thống hoá (đánh số hồ sơ, gán hộp/bó/tập, xuất/nhập mục lục tạm).
    DossierSystematize,
    /// Chỉnh lý: quản lý kho / giá lưu trữ và xếp hồ sơ lên giá.
    KhoGiaManage,
    /// Mượn / trả hồ sơ giấy: xem phiếu.
    LoanView,
    /// Mượn / trả hồ sơ giấy: lập phiếu, ghi trả.
    LoanManage,
}
impl Permission {
    pub const ALL: [Permission; 64] = [
        Permission::DossierRead,
        Permission::DossierCreate,
        Permission::DossierUpdate,
        Permission::DossierDelete,
        Permission::DossierExport,
        Permission::CatalogRead,
        Permission::CatalogCreate,
        Permission::CatalogUpdate,
        Permission::CatalogDelete,
        Permission::CatalogImport,
        Permission::CatalogExport,
        Permission::ScanView,
        Permission::ScanExecute,
        Permission::EditorView,
        Permission::EditorEdit,
        Permission::UploadExecute,
        Permission::SignView,
        Permission::SignExecute,
        Permission::DriveView,
        Permission::DriveDownload,
        Permission::DocumentList,
        Permission::DocumentDownload,
        Permission::DocumentReview,
        Permission::PackageView,
        Permission::PackageCreate,
        Permission::PackageDownload,
        Permission::SubmissionView,
        Permission::SubmissionCreate,
        Permission::SubmissionReview,
        Permission::SubmissionApprove,
        Permission::RecordView,
        Permission::RecordCreate,
        Permission::AccessView,
        Permission::AccessRequest,
        Permission::AccessReview,
        Permission::DisposalView,
        Permission::DisposalReview,
        Permission::DisposalDecide,
        Permission::BkView,
        Permission::BkManage,
        Permission::TreeView,
        Permission::TreeAssign,
        Permission::StatsView,
        Permission::StatsViewProject,
        Permission::UserView,
        Permission::UserCreate,
        Permission::UserUpdate,
        Permission::UserRevoke,
        Permission::UserResetPassword,
        Permission::UserGrantPermissions,
        Permission::ProjectView,
        Permission::ProjectCreate,
        Permission::ProjectUpdate,
        Permission::ProjectAssignAdmin,
        Permission::SettingsView,
        Permission::SettingsUpdate,
        Permission::DeviceView,
        Permission::DeviceRevoke,
        Permission::WorkExecute,
        Permission::WorkManage,
        Permission::DossierSystematize,
        Permission::KhoGiaManage,
        Permission::LoanView,
        Permission::LoanManage,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            Permission::DossierRead => "dossier.read",
            Permission::DossierCreate => "dossier.create",
            Permission::DossierUpdate => "dossier.update",
            Permission::DossierDelete => "dossier.delete",
            Permission::DossierExport => "dossier.export",
            Permission::CatalogRead => "catalog.read",
            Permission::CatalogCreate => "catalog.create",
            Permission::CatalogUpdate => "catalog.update",
            Permission::CatalogDelete => "catalog.delete",
            Permission::CatalogImport => "catalog.import",
            Permission::CatalogExport => "catalog.export",
            Permission::ScanView => "scan.view",
            Permission::ScanExecute => "scan.execute",
            Permission::EditorView => "editor.view",
            Permission::EditorEdit => "editor.edit",
            Permission::UploadExecute => "upload.execute",
            Permission::SignView => "sign.view",
            Permission::SignExecute => "sign.execute",
            Permission::DriveView => "drive.view",
            Permission::DriveDownload => "drive.download",
            Permission::DocumentList => "document.list",
            Permission::DocumentDownload => "document.download",
            Permission::DocumentReview => "document.review",
            Permission::PackageView => "package.view",
            Permission::PackageCreate => "package.create",
            Permission::PackageDownload => "package.download",
            Permission::SubmissionView => "submission.view",
            Permission::SubmissionCreate => "submission.create",
            Permission::SubmissionReview => "submission.review",
            Permission::SubmissionApprove => "submission.approve",
            Permission::RecordView => "record.view",
            Permission::RecordCreate => "record.create",
            Permission::AccessView => "access.view",
            Permission::AccessRequest => "access.request",
            Permission::AccessReview => "access.review",
            Permission::DisposalView => "disposal.view",
            Permission::DisposalReview => "disposal.review",
            Permission::DisposalDecide => "disposal.decide",
            Permission::BkView => "bk.view",
            Permission::BkManage => "bk.manage",
            Permission::TreeView => "tree.view",
            Permission::TreeAssign => "tree.assign",
            Permission::StatsView => "stats.view",
            Permission::StatsViewProject => "stats.view_project",
            Permission::UserView => "user.view",
            Permission::UserCreate => "user.create",
            Permission::UserUpdate => "user.update",
            Permission::UserRevoke => "user.revoke",
            Permission::UserResetPassword => "user.reset_password",
            Permission::UserGrantPermissions => "user.grant_permissions",
            Permission::ProjectView => "project.view",
            Permission::ProjectCreate => "project.create",
            Permission::ProjectUpdate => "project.update",
            Permission::ProjectAssignAdmin => "project.assign_admin",
            Permission::SettingsView => "settings.view",
            Permission::SettingsUpdate => "settings.update",
            Permission::DeviceView => "device.view",
            Permission::DeviceRevoke => "device.revoke",
            Permission::WorkExecute => "work.execute",
            Permission::WorkManage => "work.manage",
            Permission::DossierSystematize => "dossier.systematize",
            Permission::KhoGiaManage => "khogia.manage",
            Permission::LoanView => "loan.view",
            Permission::LoanManage => "loan.manage",
        }
    }
    pub fn min_perm(self) -> Perm {
        match self {
            Permission::DossierRead
            | Permission::CatalogRead
            | Permission::DriveView
            | Permission::DriveDownload
            | Permission::TreeView
            | Permission::AccessView
            | Permission::AccessRequest
            | Permission::SettingsView => Perm::Read,
            Permission::DossierCreate
            | Permission::DossierUpdate
            | Permission::DossierExport
            | Permission::CatalogCreate
            | Permission::CatalogUpdate
            | Permission::CatalogImport
            | Permission::CatalogExport
            | Permission::ScanView
            | Permission::ScanExecute
            | Permission::EditorView
            | Permission::EditorEdit
            | Permission::UploadExecute
            | Permission::SignView
            | Permission::SignExecute
            | Permission::StatsView
            | Permission::SettingsUpdate
            | Permission::PackageView
            | Permission::PackageCreate
            | Permission::PackageDownload
            | Permission::SubmissionView
            | Permission::SubmissionCreate
            | Permission::RecordView
            | Permission::RecordCreate
            | Permission::DisposalView
            | Permission::WorkExecute
            | Permission::LoanView
            | Permission::LoanManage => Perm::Operate,
            Permission::SubmissionReview
            | Permission::SubmissionApprove
            | Permission::AccessReview
            | Permission::DisposalReview
            | Permission::DisposalDecide
            | Permission::BkView
            | Permission::BkManage
            | Permission::DossierDelete
            | Permission::CatalogDelete
            | Permission::DocumentList
            | Permission::DocumentDownload
            | Permission::DocumentReview
            | Permission::TreeAssign
            | Permission::StatsViewProject
            | Permission::UserView
            | Permission::UserCreate
            | Permission::UserUpdate
            | Permission::UserRevoke
            | Permission::UserResetPassword
            | Permission::UserGrantPermissions
            | Permission::ProjectView
            | Permission::ProjectCreate
            | Permission::ProjectUpdate
            | Permission::DeviceView
            | Permission::DeviceRevoke
            | Permission::WorkManage
            | Permission::DossierSystematize
            | Permission::KhoGiaManage => Perm::Admin,
            Permission::ProjectAssignAdmin => Perm::Super,
        }
    }
    pub fn granted_by_default(self, r_1yx: Role) -> bool {
        if r_1yx == Role::Manager {
            // Quản lý (15/10/2026): như quản trị nhưng KHÔNG xoá (hồ sơ, danh mục, quyết định tiêu huỷ;
            // xoá dữ liệu dự án chỉ super_admin ở API); được gán quản trị cho dự án.
            if matches!(self, Permission::DossierDelete | Permission::CatalogDelete | Permission::DisposalDecide) {
                return false;
            }
            if self == Permission::ProjectAssignAdmin {
                return true;
            }
        }
        r_1yx.allows(self.min_perm())
    }
}
impl fmt::Display for Permission {
    fn fmt(&self, r_1yy: &mut fmt::Formatter<'_>) -> fmt::Result {
        r_1yy.write_str(self.as_str())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("quyền không hợp lệ: {0:?}")]
pub struct Tgo(pub String);
impl FromStr for Permission {
    type Err = Tgo;
    fn from_str(r_1yz: &str) -> Result<Self, Self::Err> {
        Permission::ALL
            .into_iter()
            .find(|p| p.as_str() == r_1yz)
            .ok_or_else(|| Tgo(r_1yz.to_owned()))
    }
}
impl TryFrom<String> for Permission {
    type Error = Tgo;
    fn try_from(r_1z0: String) -> Result<Self, Self::Error> {
        r_1z0.parse()
    }
}
impl From<Permission> for String {
    fn from(p: Permission) -> String {
        p.as_str().to_owned()
    }
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionOverrides {
    #[serde(default)]
    pub grants: BTreeSet<Permission>,
    #[serde(default)]
    pub denies: BTreeSet<Permission>,
}
impl PermissionOverrides {
    pub fn is_empty(&self) -> bool {
        self.grants.is_empty() && self.denies.is_empty()
    }
    pub fn sanitized(mut self) -> Self {
        self.grants.retain(|p| !self.denies.contains(p));
        self
    }
}
pub fn effective(r_1z3: Role, r_1z1: &PermissionOverrides, r_1z2: Permission) -> bool {
    if r_1z1.denies.contains(&r_1z2) {
        return false;
    }
    r_1z2.granted_by_default(r_1z3) || r_1z1.grants.contains(&r_1z2)
}
pub fn f_18s(r_1z5: Role, r_1z4: &PermissionOverrides) -> BTreeSet<Permission> {
    Permission::ALL
        .into_iter()
        .filter(|p| effective(r_1z5, r_1z4, *p))
        .collect()
}

/// Bộ quyền mặc định của một vai trò gốc (nguồn seed cho bảng `roles` — migration 0022).
pub fn default_set(role: Role) -> BTreeSet<Permission> {
    Permission::ALL.into_iter().filter(|p| p.granted_by_default(role)).collect()
}
/// Vai trò gốc `role` có được mang quyền `p` không — trần của bộ quyền một vai trò (hệ thống hay tuỳ chỉnh):
/// không vượt tầng của vai trò gốc. Ngoại lệ có sẵn: Quản lý được gán quản trị dự án (tầng Super).
pub fn allowed_for_base(role: Role, p: Permission) -> bool {
    role == Role::SuperAdmin || role.allows(p.min_perm()) || (role == Role::Manager && p == Permission::ProjectAssignAdmin)
}
/// Quyền hiệu lực khi vai trò có bộ quyền riêng (`preset`, từ bảng `roles`); None → mặc định theo vai trò gốc.
/// Chặn riêng luôn thắng; rồi bộ quyền vai trò hoặc cấp riêng.
pub fn effective_with(preset: Option<&BTreeSet<Permission>>, role: Role, ov: &PermissionOverrides, p: Permission) -> bool {
    if ov.denies.contains(&p) {
        return false;
    }
    let base = match preset {
        Some(set) => set.contains(&p),
        None => p.granted_by_default(role),
    };
    base || ov.grants.contains(&p)
}
pub fn effective_set_with(preset: Option<&BTreeSet<Permission>>, role: Role, ov: &PermissionOverrides) -> BTreeSet<Permission> {
    Permission::ALL.into_iter().filter(|p| effective_with(preset, role, ov, *p)).collect()
}
/// Mã vai trò hợp lệ (trùng CHECK của bảng roles): chữ thường, số, gạch dưới, 2–40 ký tự, bắt đầu bằng chữ.
pub fn is_role_code(s: &str) -> bool {
    let b = s.as_bytes();
    (2..=40).contains(&b.len()) && b[0].is_ascii_lowercase() && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'_')
}
#[cfg(test)]
mod quan_ly_tests {
    use super::*;
    #[test]
    fn quan_ly_nhu_quan_tri_nhung_khong_xoa() {
        let m = Role::Manager;
        assert!(Permission::ProjectCreate.granted_by_default(m));
        assert!(Permission::UserCreate.granted_by_default(m));
        assert!(Permission::ProjectAssignAdmin.granted_by_default(m));
        assert!(Permission::SignExecute.granted_by_default(m));
        for p in [Permission::DossierDelete, Permission::CatalogDelete, Permission::DisposalDecide] {
            assert!(!p.granted_by_default(m), "{p} phải bị chặn với quản lý");
            assert!(p.granted_by_default(Role::SuperAdmin));
        }
        assert!(!Permission::ProjectAssignAdmin.granted_by_default(Role::Admin));
        assert_eq!("manager".parse::<Role>(), Ok(Role::Manager));
    }
    #[test]
    fn bo_quyen_vai_tro_thay_mac_dinh_va_khong_vuot_tang() {
        let mut preset = default_set(Role::User);
        preset.remove(&Permission::SignExecute);
        preset.insert(Permission::DocumentList);
        let ov = PermissionOverrides::default();
        assert!(!effective_with(Some(&preset), Role::User, &ov, Permission::SignExecute));
        assert!(effective_with(Some(&preset), Role::User, &ov, Permission::DocumentList));
        assert!(effective_with(None, Role::User, &ov, Permission::SignExecute), "không có bộ quyền → mặc định");
        let mut deny = PermissionOverrides::default();
        deny.denies.insert(Permission::DocumentList);
        assert!(!effective_with(Some(&preset), Role::User, &deny, Permission::DocumentList), "chặn riêng thắng");
        assert!(allowed_for_base(Role::User, Permission::ScanExecute));
        assert!(!allowed_for_base(Role::User, Permission::UserCreate));
        assert!(allowed_for_base(Role::Manager, Permission::ProjectAssignAdmin));
        assert!(!allowed_for_base(Role::Admin, Permission::ProjectAssignAdmin));
        assert!(is_role_code("ky_so_vien") && !is_role_code("Ký số") && !is_role_code("a"));
    }
}
