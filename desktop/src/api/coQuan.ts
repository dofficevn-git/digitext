// Copyright (c) 2026 DigiText. All rights reserved.
// Cấu hình cơ quan lưu trữ của dự án (0039, 27/11/2026 — đợt 5, Digifile B01): GET/PUT /projects/{id}/co-quan qua Tauri bc_call.

import { invoke } from "@/api/ipc";

export interface CoQuanRow {
  projectId: string;
  maCoQuan: string | null;
  ten: string | null;
  tenCapTren: string | null;
  diaChi: string | null;
  dienThoai: string | null;
  email: string | null;
  loaiHinh: "luu_tru_co_quan" | "luu_tru_lich_su";
  nguoiKy: string | null;
  chucVuNguoiKy: string | null;
  hanNopNgay: number;
  nhacTruocNgay: number[];
  quyenSuDungNgay: number;
  ghiChu: string | null;
  updatedAt: string;
  updatedBy: string | null;
}

export interface CoQuanHieuDung {
  ten: string;
  maCoQuan: string | null;
  tenCapTren: string | null;
  loaiHinh: string;
  nguoiKy: string | null;
  chucVuNguoiKy: string;
  hanNopNgay: number;
  nhacTruocNgay: number[];
  quyenSuDungNgay: number;
  daKhai: boolean;
}

export interface CoQuanDto {
  projectId: string;
  cauHinh: CoQuanRow | null;
  hieuDung: CoQuanHieuDung;
}

export interface CoQuanMoi {
  maCoQuan?: string | null;
  ten?: string | null;
  tenCapTren?: string | null;
  diaChi?: string | null;
  dienThoai?: string | null;
  email?: string | null;
  loaiHinh?: "luu_tru_co_quan" | "luu_tru_lich_su";
  nguoiKy?: string | null;
  chucVuNguoiKy?: string | null;
  hanNopNgay?: number;
  nhacTruocNgay?: number[];
  quyenSuDungNgay?: number;
  ghiChu?: string | null;
}

export const coQuanGet = (projectId: string) => invoke<CoQuanDto>("bc_call", { method: "GET", path: `/projects/${projectId}/co-quan`, body: null });
export const coQuanSet = (projectId: string, body: CoQuanMoi) => invoke<CoQuanDto>("bc_call", { method: "PUT", path: `/projects/${projectId}/co-quan`, body });

export const LOAI_HINH: [CoQuanMoi["loaiHinh"] & string, string][] = [
  ["luu_tru_co_quan", "Lưu trữ cơ quan (lưu trữ hiện hành)"],
  ["luu_tru_lich_su", "Lưu trữ lịch sử"],
];
