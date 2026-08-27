// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4u9, V4u8 } from "@/api/disposal";

export const V4ul: Record<V4u8["lyDo"], string> = {
  trung_lap: "Trùng lặp (Điều 48.2)",
  het_thoi_han: "Hết thời hạn lưu trữ (Điều 48.3)",
};

export const V4um: Record<V4u8["status"], string> = {
  xem_xet: "Xem xét hủy",
  giu_lai: "Giữ lại (đã loại khỏi Danh mục)",
  da_huy: "Đã hủy",
};

export const V4un: Record<V4u9["status"], string> = {
  hoi_dong: "Hội đồng xét hủy (Điều 49.2)",
  tham_dinh: "Chờ thẩm định (Điều 49.4)",
  quyet_dinh: "Đã có Quyết định hủy (Điều 49.6)",
  da_huy: "Đã hủy trong Hệ thống (Điều 49.7)",
};

export const V4uo: Record<string, string> = {
  phat_hien_trung_lap: "Hệ thống phát hiện trùng lặp",
  het_thoi_han_luu_tru: "Hệ thống ghi nhận hết thời hạn lưu trữ",
  giu_lai: "Giữ lại — loại khỏi Danh mục",
  lap_dot_xet_huy: "Lập đợt xét hủy",
  hoi_dong_tong_hop: "Hội đồng tổng hợp kết quả",
  y_kien_tham_dinh: "Ý kiến thẩm định",
  quyet_dinh_huy: "Quyết định hủy",
  bat_dau_huy: "Bắt đầu thao tác hủy",
  da_huy_tai_lieu: "Đã hủy tài liệu (xóa khỏi kho)",
};

export function V4up(
  V2ym: V4u9,
  V2yn: boolean,
  V2yo: boolean,
): { action: "council" | "appraise" | "decide" | "execute"; label: string }[] {
  const V2yp: { action: "council" | "appraise" | "decide" | "execute"; label: string }[] = [];
  if (V2ym.status === "hoi_dong" && V2yn) {
    V2yp.push({
      action: "council",
      label: "Chốt kết quả Hội đồng (49.2.c)",
    });
  }
  if (V2ym.status === "tham_dinh") {
    if (V2yn && !V2ym.thamDinhNote) {
      V2yp.push({
        action: "appraise",
        label: "Nhập ý kiến thẩm định (49.4)",
      });
    }
    if (V2yo) {
      V2yp.push({ action: "decide", label: "Quyết định hủy (49.6)" });
    }
  }
  if (V2ym.status === "quyet_dinh" && V2yo) {
    V2yp.push({
      action: "execute",
      label: "Thao tác hủy trong Hệ thống (49.7)",
    });
  }
  return V2yp;
}

export function V4uq(V2yq: V4u9): boolean {
  return !V2yq.thamDinhNote;
}
