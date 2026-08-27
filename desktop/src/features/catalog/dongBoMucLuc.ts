// Copyright (c) 2026 DigiText. All rights reserved.

export const V4bh =
  "Lập hồ sơ điện tử · Hệ thống hoá hồ sơ · Tạo lập CSDL · Khai thác dữ liệu";

export function V4bi(n: number | undefined): string {
  if (n == null) return "";
  if (n <= 0) {
    return `Mục lục chỉnh lý đã có đủ các hồ sơ này — không thêm bản nào và không ghi đè bản đã sửa tay.`;
  }
  return `Đã tự động đồng bộ ${n.toLocaleString("vi-VN")} hồ sơ sang mục lục chỉnh lý — xem ở: ${V4bh}.`;
}
