// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
pub mod chinh_ly;
pub mod dashboard;
pub mod doc_status;
pub mod field;
pub mod job;
pub mod permission;
pub mod review_status;
pub mod role;
pub mod text;
pub mod tiff_naming;
pub use doc_status::{DocStatus, Tgl};
pub use field::{Tgr, VanBanField};
pub use job::{JobKind, JobStatus, Tgm, Tgn};
pub use permission::{allowed_for_base, default_set, effective, effective_set_with, effective_with, f_18s, is_role_code, Permission, PermissionOverrides, Tgo};
pub use review_status::{ReviewStatus, Tgp};
pub use role::{Perm, Role, Tgq};
pub use text::{f_ue, f_x2, C_Y};
