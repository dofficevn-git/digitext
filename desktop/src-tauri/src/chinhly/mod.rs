// Copyright (c) 2026 DigiText. All rights reserved.
//! CHỈNH LÝ trên desktop (kế hoạch 29/9/2026): bản sao SQLite của cl_* (`store`), engine đồng bộ
//! offline ↔ máy chủ (`sync`), lệnh Tauri (`commands`).
pub mod commands;
pub mod legacy;
pub mod project;
pub mod store;
pub mod sync;
