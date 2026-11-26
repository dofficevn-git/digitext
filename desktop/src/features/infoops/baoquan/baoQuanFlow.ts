// Copyright (c) 2026 DigiText. All rights reserved.
// Nhãn, màu, bước của bảo quản Điều 29–31 (0032). Căn cứ pháp lý để ở tooltip / nút (i), không nằm trong nhãn.

import type { V3y0 } from "@/domain/types";
import type { TrangThaiKeHoach, TrangThaiLoi, TrangThaiSuCo, MucDoSuCo, TinhTrangLich } from "@/api/baoquan";
import { V4sz } from "@/api/records";

export const TT_KE_HOACH: Record<TrangThaiKeHoach, { label: string; tone: V3y0; step: number }> = {
  du_thao: { label: "Dự thảo", tone: "gray", step: 0 },
  cho_duyet: { label: "Chờ duyệt", tone: "warn", step: 1 },
  da_duyet: { label: "Đã duyệt", tone: "info", step: 2 },
  dang_thuc_hien: { label: "Đang kiểm", tone: "run", step: 3 },
  hoan_thanh: { label: "Hoàn thành", tone: "ok", step: 4 },
  huy: { label: "Đã huỷ", tone: "gray", step: 0 },
};

export const BUOC_KE_HOACH = [
  { key: "duthao", label: "Dự thảo" },
  { key: "trinh", label: "Trình" },
  { key: "duyet", label: "Duyệt" },
  { key: "kiem", label: "Kiểm tra" },
  { key: "ketluan", label: "Kết luận" },
];

export const TT_LOI: Record<TrangThaiLoi, { label: string; tone: V3y0 }> = {
  moi: { label: "Mới", tone: "err" },
  da_khac_phuc: { label: "Đã khắc phục", tone: "warn" },
  kiem_lai_dat: { label: "Kiểm lại đạt", tone: "ok" },
  kiem_lai_loi: { label: "Kiểm lại lỗi", tone: "err" },
  bo_qua: { label: "Bỏ qua", tone: "gray" },
};

export const TT_TEP: Record<string, { label: string; tone: V3y0 }> = {
  cho: { label: "Chờ kiểm", tone: "gray" },
  dat: { label: "Đạt", tone: "ok" },
  loi: { label: "Lỗi", tone: "err" },
  bo_qua: { label: "Bỏ qua", tone: "gray" },
};

export const TT_LICH: Record<TinhTrangLich["tinhTrang"], { label: string; tone: V3y0 }> = {
  dung_han: { label: "Đúng hạn", tone: "ok" },
  tre: { label: "Trễ", tone: "err" },
  chua_co: { label: "Chưa có bản nào", tone: "warn" },
  tat: { label: "Tắt", tone: "gray" },
};

export const LOAI_SAO_LUU: Record<string, string> = {
  gia_tang: "Gia tăng (hằng ngày)",
  day_du: "Đầy đủ (hằng tháng)",
  day_du_3_nam: "Đầy đủ 3 năm (phương tiện riêng)",
};

export const TT_PHUONG_TIEN: Record<string, { label: string; tone: V3y0 }> = {
  dang_dung: { label: "Đang dùng", tone: "ok" },
  da_thay: { label: "Đã thay", tone: "gray" },
  hong: { label: "Hỏng", tone: "err" },
  loai_bo: { label: "Loại bỏ", tone: "gray" },
};

export const LOAI_PHUONG_TIEN = [
  { value: "o_cung", label: "Ổ cứng ngoài" },
  { value: "nas", label: "NAS / máy chủ sao lưu" },
  { value: "bang_tu", label: "Băng từ" },
  { value: "quang", label: "Đĩa quang" },
  { value: "dam_may", label: "Đám mây" },
  { value: "khac", label: "Khác" },
];

export const MUC_DO: Record<MucDoSuCo, { label: string; tone: V3y0 }> = {
  thap: { label: "Thấp", tone: "gray" },
  trung_binh: { label: "Trung bình", tone: "info" },
  cao: { label: "Cao", tone: "warn" },
  nghiem_trong: { label: "Nghiêm trọng", tone: "err" },
};

export const TT_SU_CO: Record<TrangThaiSuCo, { label: string; tone: V3y0 }> = {
  moi: { label: "Mới", tone: "err" },
  dang_xu_ly: { label: "Đang xử lý", tone: "warn" },
  da_phuc_hoi: { label: "Đã phục hồi", tone: "ok" },
  dong: { label: "Đã đóng", tone: "gray" },
};

export const fmtN = (v: number) => v.toLocaleString("vi-VN");
export const fmtMB = (b: number | null | undefined) => (b === null || b === undefined ? "—" : b >= 1_073_741_824 ? `${(b / 1_073_741_824).toFixed(2)} GB` : `${(b / 1_048_576).toFixed(1)} MB`);
export const ngayGio = (iso: string | null | undefined) => (iso ? new Date(iso).toLocaleString("vi-VN", { hour: "2-digit", minute: "2-digit", day: "2-digit", month: "2-digit", year: "numeric" }) : "—");
export const ngay = (iso: string | null | undefined) => (iso ? new Date(iso).toLocaleDateString("vi-VN") : "—");
/** `datetime-local` cần "yyyy-MM-ddTHH:mm" theo giờ máy. */
export const toLocalInput = (iso: string | null | undefined) => {
  const d = iso ? new Date(iso) : new Date();
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}T${p(d.getHours())}:${p(d.getMinutes())}`;
};

/** Sổ biên bản lập xong không trả id → lấy biên bản MỚI NHẤT cùng loại (trong 2 phút) để gắn vào kế hoạch / sự cố. */
export async function ganBienBanMoiNhat(kind: string, projectId: string | null, gan: (recordId: string) => Promise<unknown>): Promise<boolean> {
  const rows = await V4sz({ kinds: [kind], projectId, limit: 5 });
  const moi = rows.filter((r) => Date.now() - new Date(r.createdAt).getTime() < 120_000).sort((a, b) => b.createdAt.localeCompare(a.createdAt))[0];
  if (!moi) return false;
  await gan(moi.id);
  return true;
}
