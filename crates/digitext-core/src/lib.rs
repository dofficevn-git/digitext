// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
pub mod chinh_ly;
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
pub use permission::{effective, f_18s, Permission, PermissionOverrides, Tgo};
pub use review_status::{ReviewStatus, Tgp};
pub use role::{Perm, Role, Tgq};
pub use text::{f_ue, f_x2, C_Y};
