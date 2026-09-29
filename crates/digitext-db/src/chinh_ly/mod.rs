// Copyright (c) 2026 DigiText. All rights reserved.
//! CHỈNH LÝ tài liệu giấy (migration 0012, kế hoạch 29/9/2026 — port nghiệp vụ Digitext.2025).
//! Máy chủ là nguồn sự thật; desktop giữ bản sao SQLite, sửa offline rồi đồng bộ qua `sync`.
//!
//! - `spec`: mô tả 15 bảng cl_* (cột ghi được, cột chỉ online, cha → con, quyền).
//! - `rows`: đọc / ghi dòng dạng JSON theo spec.
//! - `validate`: chuẩn hoá + kiểm tra dữ liệu (ngày, số, mã TT05, bắt buộc).
//! - `merge`: gộp 3 chiều.
//! - `sync`: `apply_op` (một luồng ghi cho online lẫn push), `pull`, `prune`.
//! - `lich_su`: lịch sử sửa từng trường (máy chủ sinh).
//! - `ho_so`: truy vấn hồ sơ / văn bản qua view `cl_ho_so_v`.
//! - `danh_muc`: đọc gộp danh mục.

pub mod danh_muc;
pub mod ho_so;
pub mod lich_su;
pub mod merge;
pub mod rows;
pub mod spec;
pub mod sync;
#[cfg(test)]
mod tests;
pub mod validate;

pub use merge::Row;
pub use sync::{apply_op, OpCtx, OpKind, OpResult, SyncOp};
