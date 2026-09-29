// Copyright (c) 2026 DigiText. All rights reserved.
// Nhãn và luồng bước của tiêu huỷ tài liệu hết giá trị. Căn cứ pháp lý để riêng ở LEGAL, không nằm trong nhãn.

import type { V4u9, V4u8 } from "@/api/disposal";

export const V4ul: Record<V4u8["lyDo"], string> = {
  trung_lap: "Trùng lặp",
  het_thoi_han: "Hết thời hạn lưu trữ",
};

export const V4um: Record<V4u8["status"], string> = {
  xem_xet: "Xem xét huỷ",
  giu_lai: "Giữ lại",
  da_huy: "Đã huỷ",
};

export const V4un: Record<V4u9["status"], string> = {
  hoi_dong: "Hội đồng xét huỷ",
  tham_dinh: "Chờ thẩm định",
  quyet_dinh: "Đã có quyết định huỷ",
  da_huy: "Đã huỷ trong hệ thống",
};

/** Thứ tự bước của một đợt, dùng cho Stepper. */
export const BUOC_DOT: { key: V4u9["status"]; label: string }[] = [
  { key: "hoi_dong", label: "Hội đồng" },
  { key: "tham_dinh", label: "Thẩm định" },
  { key: "quyet_dinh", label: "Quyết định" },
  { key: "da_huy", label: "Đã huỷ" },
];

export const LEGAL: Record<string, string> = {
  trung_lap: "Điều 48.2",
  het_thoi_han: "Điều 48.3",
  danh_muc: "Điều 48.4",
  ghi_chu: "Điều 48.5",
  lap_dot: "Điều 49.1",
  hoi_dong: "Điều 49.2",
  tham_dinh: "Điều 49.4",
  quyet_dinh: "Điều 49.6",
  huy: "Điều 49.7",
  tu_quyet: "Điều 49.8",
};

export const V4uo: Record<string, string> = {
  phat_hien_trung_lap: "Hệ thống phát hiện trùng lặp",
  het_thoi_han_luu_tru: "Hệ thống ghi nhận hết thời hạn lưu trữ",
  giu_lai: "Giữ lại, loại khỏi danh mục",
  lap_dot_xet_huy: "Lập đợt xét huỷ",
  hoi_dong_tong_hop: "Hội đồng tổng hợp kết quả",
  y_kien_tham_dinh: "Ý kiến thẩm định",
  quyet_dinh_huy: "Quyết định huỷ",
  bat_dau_huy: "Bắt đầu thao tác huỷ",
  da_huy_tai_lieu: "Đã huỷ tài liệu, xoá khỏi kho",
};

export type DotAction = "council" | "appraise" | "decide" | "execute";

export function V4up(b: V4u9, canReview: boolean, canDecide: boolean): { action: DotAction; label: string; legal: string }[] {
  const out: { action: DotAction; label: string; legal: string }[] = [];
  if (b.status === "hoi_dong" && canReview) out.push({ action: "council", label: "Chốt kết quả Hội đồng", legal: "Điều 49.2.c" });
  if (b.status === "tham_dinh") {
    if (canReview && !b.thamDinhNote) out.push({ action: "appraise", label: "Nhập ý kiến thẩm định", legal: "Điều 49.4" });
    if (canDecide) out.push({ action: "decide", label: "Ra quyết định huỷ", legal: "Điều 49.6" });
  }
  if (b.status === "quyet_dinh" && canDecide) out.push({ action: "execute", label: "Huỷ trong hệ thống", legal: "Điều 49.7" });
  return out;
}

export function V4uq(b: V4u9): boolean {
  return !b.thamDinhNote;
}
