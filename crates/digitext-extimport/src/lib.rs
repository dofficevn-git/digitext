// Copyright (c) 2026 DigiText. All rights reserved.
//! Nhập KHO TÀI LIỆU TỪ BÊN NGOÀI rồi đóng gói TT 05/2025.
//!
//! Đầu vào: thư mục PDF đã quét (Phông/Hộp/Hồ sơ) + MỘT tệp Excel metadata phẳng (mỗi dòng
//! một tài liệu, mang luôn thông tin phông và hồ sơ). Ba bước:
//!   1. [`excel::read`]  — đọc Excel, tự nhận cột theo ba khối phông / hồ sơ / tài liệu;
//!   2. [`check::run`]   — đối chiếu thư mục ↔ Excel, ra báo cáo lỗi/cảnh báo theo hồ sơ;
//!   3. [`pack::run`]    — hồ sơ KHÔNG lỗi → gói SIP/AIP bằng chính bộ dựng `digitext-package`.
//! Không cần Postgres/MinIO. Dùng qua `digitext ext-import …` (xem [`cli`]).
#![forbid(unsafe_code)]
pub mod check;
pub mod cli;
pub mod excel;
pub mod pack;
pub mod report;
