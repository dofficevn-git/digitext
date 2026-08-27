// Copyright (c) 2026 DigiText. All rights reserved.

import { V4dr } from "@/features/dossier/logic";
import type { V4nu, V4nv, V4nw, V4nt } from "@/api/report";

export interface V4o1<T> {
  head: string;
  value: (r: T) => string | number;
  num?: boolean;
}

function V4o2(ten?: string, ma?: string): string {
  return ten || ma || "(Chưa gán phông)";
}

export const V4o3: V4o1<V4nt>[] = [
  { head: "Phông", value: (r) => V4o2(r.tenDuAn, r.maDuAn) },
  { head: "Số hồ sơ", value: (r) => r.soHoSo, num: true },
  { head: "Số PDF", value: (r) => r.soPdf, num: true },
  { head: "A0", value: (r) => r.a0, num: true },
  { head: "A1", value: (r) => r.a1, num: true },
  { head: "A2", value: (r) => r.a2, num: true },
  { head: "A3", value: (r) => r.a3, num: true },
  { head: "A4", value: (r) => r.a4, num: true },
  { head: "A5", value: (r) => r.a5, num: true },
  { head: "S2jDoWM=", value: (r) => r.khac, num: true },
  { head: "Tổng trang", value: (r) => r.tongTrang, num: true },
  { head: "Quy đổi A4", value: (r) => Math.round(r.quyDoi * 10) / 10, num: true },
];

export const V4o4: V4o1<V4nu>[] = [
  {
    head: "Phòng ban",
    value: (r) => r.tenDonVi || r.maDv || "(Chưa gán phòng ban)",
  },
  { head: "Hộp vĩnh viễn", value: (r) => r.hopVinhVien, num: true },
  { head: "Hồ sơ vĩnh viễn", value: (r) => r.hoSoVinhVien, num: true },
  { head: "Hộp có thời hạn", value: (r) => r.hopCoThoiHan, num: true },
  { head: "Hồ sơ có thời hạn", value: (r) => r.hoSoCoThoiHan, num: true },
  { head: "Bó hồ sơ loại", value: (r) => r.boLoai, num: true },
  { head: "Tập hồ sơ loại", value: (r) => r.tapLoai, num: true },
];

export const V4o5: V4o1<V4nv>[] = [
  { head: "Nhân viên", value: (r) => r.ten },
  { head: "Số hồ sơ", value: (r) => r.soHoSo, num: true },
  { head: "Số văn bản", value: (r) => r.soVanBan, num: true },
  { head: "Số trang quét", value: (r) => r.soTrangQuet, num: true },
  { head: "Thao tác thêm", value: (r) => r.thaoTacThem, num: true },
  { head: "Thao tác sửa", value: (r) => r.thaoTacSua, num: true },
];

export const V4o6: V4o1<V4nw>[] = [
  { head: "Phông", value: (r) => V4o2(r.tenDuAn, r.maDuAn) },
  { head: "Thời gian khởi tạo", value: (r) => V4dr(r.thoiGianKhoiTao) },
  { head: "Hộp vĩnh viễn", value: (r) => r.hopVinhVien, num: true },
  { head: "Hồ sơ vĩnh viễn", value: (r) => r.hoSoVinhVien, num: true },
  { head: "Hộp có thời hạn", value: (r) => r.hopCoThoiHan, num: true },
  { head: "Hồ sơ có thời hạn", value: (r) => r.hoSoCoThoiHan, num: true },
  { head: "Bó hồ sơ loại", value: (r) => r.boLoai, num: true },
  { head: "Tập hồ sơ loại", value: (r) => r.tapLoai, num: true },
];

export function V4o7<T>(cols: V4o1<T>[], V21s: T[]): Record<string, unknown>[] {
  return V21s.map((r) => {
    const o: Record<string, unknown> = {};
    for (const c of cols) o[c.head] = c.value(r);
    return o;
  });
}
