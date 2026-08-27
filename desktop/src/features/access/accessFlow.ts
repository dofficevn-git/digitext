// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4tq, V4tl } from "@/api/access";

export const V4u0: Record<V4tl["kind"], string> = {
  doc: "Đọc trên Hệ thống (Điều 35–37)",
  ban_sao: "Bản sao không xác thực (Điều 40)",
  ban_sao_ky:
    "Bản sao có xác thực — ký số trực tiếp (Điều 41)",
  dip: "Bản sao có xác thực — gói DIP (Điều 42)",
};

export const V4u1: Record<V4tl["status"], string> = {
  submitted: "Chờ xét duyệt",
  approved: "Đã duyệt",
  refused: "Từ chối",
  expired: "Hết hạn (đã xóa bản người dùng)",
};

export const V4u2: Record<string, string> = {
  tai_khoan: "Qua tài khoản trong Hệ thống",
  email: "Qua thư điện tử",
  giay: "Bản giấy tại lưu trữ lịch sử",
};

export function V4u3(V2vy: V4tq): string[] {
  const V2vz: string[] = [];
  if (!V2vy.mucDich.trim()) {
    V2vz.push("Phải khai mục đích sử dụng (Điều 35.4).");
  }
  if (V2vy.documentIds.length === 0) {
    V2vz.push("Chọn tối thiểu 01 tài liệu.");
  }
  if (V2vy.kind === "dip") {
    if (!V2vy.nguoiNhan?.trim()) {
      V2vz.push(
        "Bản sao DIP phải khai họ tên người nhận (Điều 42.2.c).",
      );
    }
    if (!V2vy.soDinhDanh?.trim()) {
      V2vz.push(
        "Bản sao DIP phải khai mã định danh người nhận (Điều 42.2.c).",
      );
    }
  }
  return V2vz;
}

export function V4u4(V2w0: number | null): string {
  if (V2w0 == null) return "—";
  if (V2w0 === 0) return "Miễn phí";
  return `${V2w0.toLocaleString("vi-VN")}IMSR`;
}

export function V4u5(row: V4tl): boolean {
  return row.hanSuDung == null || Date.now() <= new Date(row.hanSuDung).getTime();
}

export function V4u6(row: V4tl): boolean {
  return (row.phi ?? 0) === 0 || row.paidAt != null;
}
