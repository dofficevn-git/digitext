// Copyright (c) 2026 DigiText. All rights reserved.
// Nhãn, màu và bước của phiếu nộp lưu / danh mục hồ sơ (Điều 19). Căn cứ pháp lý để ở tooltip, không nằm trong nhãn nút.

import type { TrangThaiDanhMuc, TrangThaiPhieu } from "@/api/luutru";
import { str, type ClRow } from "@/api/chinhly";

type Tone = "ok" | "info" | "warn" | "err" | "gray";

export const TT_PHIEU: Record<TrangThaiPhieu, { label: string; tone: Tone; step: number }> = {
  cho_tiep_nhan: { label: "Chờ tiếp nhận", tone: "warn", step: 1 },
  da_tiep_nhan: { label: "Đã tiếp nhận", tone: "ok", step: 2 },
  tra_lai: { label: "Trả lại", tone: "err", step: 2 },
  rut: { label: "Đã rút", tone: "gray", step: 0 },
};

export const BUOC_PHIEU = [
  { key: "nop", label: "Nộp" },
  { key: "tiepnhan", label: "Tiếp nhận" },
  { key: "bienban", label: "Biên bản giao nhận" },
];

export const TT_HO_SO_NOP: Record<string, { label: string; tone: Tone }> = {
  cho_tiep_nhan: { label: "Chờ tiếp nhận", tone: "warn" },
  da_nop: { label: "Đã nộp lưu", tone: "ok" },
  tra_lai: { label: "Bị trả lại", tone: "err" },
};

export const KET_QUA: Record<string, { label: string; tone: Tone }> = {
  cho: { label: "Chưa chấm", tone: "gray" },
  nhan: { label: "Nhận", tone: "ok" },
  tra_lai: { label: "Trả lại", tone: "err" },
};

export const TT_DANH_MUC: Record<TrangThaiDanhMuc, { label: string; tone: Tone; step: number }> = {
  du_thao: { label: "Dự thảo", tone: "gray", step: 0 },
  cho_duyet: { label: "Chờ duyệt", tone: "warn", step: 1 },
  da_duyet: { label: "Đã duyệt", tone: "info", step: 2 },
  ban_hanh: { label: "Đang hiệu lực", tone: "ok", step: 3 },
  het_hieu_luc: { label: "Hết hiệu lực", tone: "gray", step: 3 },
};

export const BUOC_DANH_MUC = [
  { key: "duthao", label: "Dự thảo" },
  { key: "trinh", label: "Trình" },
  { key: "duyet", label: "Duyệt" },
  { key: "banhanh", label: "Ban hành" },
];

/** Hạn nộp: màu và chữ theo số ngày còn lại (âm = quá hạn). */
export function hanNop(conNgay: number | null | undefined): { label: string; tone: Tone } {
  if (conNgay === null || conNgay === undefined) return { label: "Chưa có ngày kết thúc", tone: "gray" };
  if (conNgay < 0) return { label: `Quá hạn ${-conNgay} ngày`, tone: "err" };
  if (conNgay === 0) return { label: "Hạn hôm nay", tone: "warn" };
  if (conNgay <= 15) return { label: `Còn ${conNgay} ngày`, tone: "warn" };
  return { label: `Còn ${conNgay} ngày`, tone: "gray" };
}

export const ngayVn = (iso: string | null | undefined): string => (iso ? new Date(iso).toLocaleDateString("vi-VN") : "—");

/** Hồ sơ đã nộp vào lưu trữ cơ quan (0031) thì mục lục đóng băng; máy chủ cũng từ chối `locked`. */
export const hoSoBiKhoa = (r: ClRow | null | undefined): boolean => !!r && ["cho_tiep_nhan", "da_nop"].includes(str(r, "nop_luu_trang_thai"));
