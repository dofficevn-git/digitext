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
}
impl Permission {
    pub const ALL: [Permission; 58] = [
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
            | Permission::DisposalView => Perm::Operate,
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
            | Permission::DeviceRevoke => Perm::Admin,
            Permission::ProjectAssignAdmin => Perm::Super,
        }
    }
    pub fn granted_by_default(self, r_1yx: Role) -> bool {
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
