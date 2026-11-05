// Copyright (c) 2026 DigiText. All rights reserved.
// Tra cứu tài liệu lưu trữ (bản Lưu trữ GĐ4, 5/11/2026) — Tauri src-tauri/src/tra_cuu.rs → máy chủ routes/tra_cuu.rs.

import { invoke } from "@/api/ipc";

export interface DongTraCuu {
  vanBanId: string;
  hoSoId: string;
  phongId: string;
  maPhong: string;
  tenPhong: string;
  maHoSo: string;
  objid: string | null;
  tieuDeHoSo: string;
  thbq: string | null;
  soThuTu: number;
  loaiVb: string | null;
  soKyHieu: string | null;
  ngayThang: string | null;
  coQuan: string | null;
  trichYeu: string | null;
  soTrang: number | null;
  cheDo: string | null;
  hanChe: boolean;
  documentId: string | null;
  kySo: "hop_le" | "khong_hop_le" | "chua_ky" | null;
  daHuy: boolean;
  khop: "muc_luc" | "noi_dung" | null;
  trichDoan: string | null;
  trang: number | null;
}

export interface KetQuaTraCuu {
  tong: number;
  dong: DongTraCuu[];
}

export interface TraCuuQuery {
  projectId: string;
  q?: string;
  noiDung?: boolean;
  phongId?: string;
  hoSoId?: string;
  thbqId?: string;
  loaiVb?: string;
  namTu?: number;
  namDen?: number;
  cheDo?: "cong_khai" | "han_che";
  daHuy?: boolean;
  limit?: number;
  offset?: number;
}

export interface MucChon {
  id: string;
  ten: string;
  so: number;
}

export interface BoLoc {
  phong: MucChon[];
  thbq: MucChon[];
  loaiVb: { ma: string; ten: string; so: number }[];
  namMin: number | null;
  namMax: number | null;
  tongVanBan: number;
  tongHoSo: number;
  docThang: boolean;
  canBo: boolean;
}

export interface DongNhatKy {
  id: number;
  at: string;
  action: "khai_thac.xem" | "khai_thac.doc" | "khai_thac.tai" | "khai_thac.dip" | string;
  nguoi: string | null;
  tenDangNhap: string | null;
  documentId: string | null;
  detail: Record<string, unknown> | null;
  ip: string | null;
}

export function traCuuTim(query: TraCuuQuery): Promise<KetQuaTraCuu> {
  return invoke<KetQuaTraCuu>("tra_cuu_tim", { args: { query } });
}

export function traCuuBoLoc(projectId: string): Promise<BoLoc> {
  return invoke<BoLoc>("tra_cuu_bo_loc", { args: { projectId } });
}

export function traCuuNhatKy(projectId: string, limit = 500): Promise<DongNhatKy[]> {
  return invoke<DongNhatKy[]>("tra_cuu_nhat_ky", { args: { projectId, limit } });
}

/** Bản đọc có watermark (base64 PDF). */
export function traCuuDoc(documentId: string): Promise<string> {
  return invoke<string>("tra_cuu_doc", { args: { documentId } });
}
