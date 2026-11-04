// Copyright (c) 2026 DigiText. All rights reserved.
//! Bản phần mềm, chốt lúc build bằng biến `VITE_EDITION` (cùng biến Vite đọc cho giao diện):
//! rỗng = bản thi công, `luu_tru` = bản Lưu trữ cài tại máy chủ chủ đầu tư (docs/KE-HOACH-BAN-LUU-TRU.md).
pub const LUU_TRU: bool = cfg!(luu_tru);
