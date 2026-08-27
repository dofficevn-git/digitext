// Copyright (c) 2026 DigiText. All rights reserved.

export function chuaChonPhong(maDa: string | null | undefined): boolean {
  return (maDa ?? "").trim() === "";
}

export const V4bj =
  "Hãy chọn Phông (cơ quan) ở bộ lọc để xem hồ sơ của cơ quan đó.";

export function bangTrongText(maDa: string | null | undefined): string {
  return chuaChonPhong(maDa) ? V4bj : "Không tìm thấy dữ liệu.";
}
