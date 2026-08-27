// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4uu, V4us } from "@/api/infoops";
import type { V41v } from "@/components/Icon";

export const V4v6: Record<V4us["hinhThuc"], string> = {
  danh_muc:
    "Danh mục tài liệu (Điều 44.1 — tối đa 4 trường tin)",
  trich_xuat:
    "Trích xuất nguyên văn (Điều 44.2 — kèm địa chỉ lưu trữ)",
  tong_hop:
    "Thông tin tổng hợp (Điều 44.3 — báo cáo tự động)",
};

export const V4v7: Record<V4us["status"], string> = {
  submitted: "Chờ xét duyệt (SLA 6 ngày làm việc — 46.5)",
  approved: "Đã duyệt",
  refused: "Từ chối",
  fulfilled: "Đã trả kết quả",
};

export const V4v8: Record<string, string> = {
  gia_tang: "Gia tăng hằng ngày (30.1.a)",
  day_du: "Đầy đủ hằng tháng (30.1.a)",
  day_du_3_nam: "Đầy đủ 03 năm (30.1.b)",
};

export function V4v9(V30c: V4uu): string[] {
  const V30d: string[] = [];
  if (!V30c.chuDe.trim()) {
    V30d.push("Phải nêu chủ đề cụ thể (Điều 44.1.a).");
  }
  if (!V30c.mucDich.trim()) {
    V30d.push("Phải khai mục đích sử dụng (Điều 45.3).");
  }
  return V30d;
}

export interface V4va {
  action: "approve" | "refuse" | "pay" | "generate" | "fulfill";
  icon: V41v;
  label: string;
}

export function V4vb(row: V4us): V4va[] {
  const V30e: V4va[] = [];
  if (row.status === "submitted") {
    V30e.push({
      action: "approve",
      icon: "duyet",
      label: "Duyệt (phí tự tính — 46.1.c)",
    });
    V30e.push({ action: "refuse", icon: "tuChoi", label: "Từ chối" });
  }
  if (row.status === "approved") {
    if ((row.phi ?? 0) > 0 && !row.paidAt) {
      V30e.push({
        action: "pay",
        icon: "thanhToan",
        label: "Xác nhận thanh toán (46.4)",
      });
    }
    V30e.push({
      action: "generate",
      icon: row.coKetQua ? "lamMoi" : "ketXuat",
      label: row.coKetQua
        ? "Kết xuất lại (47.1)"
        : "Kết xuất từ Hệ thống (47.1)",
    });
    if (row.coKetQua) {
      V30e.push({
        action: "fulfill",
        icon: "guiKetQua",
        label: "Phê duyệt + trả kết quả (47.3/47.4)",
      });
    }
  }
  return V30e;
}
