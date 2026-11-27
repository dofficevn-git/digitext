// Copyright (c) 2026 DigiText. All rights reserved.
// Lịch sử đơn vị hình thành phông / lịch sử phông có phiên bản (0039, 27/11/2026 — đợt 5, Digifile B02):
// GET/POST /cl/phong/{id}/lich-su qua Tauri bc_call. Chỉ trực tuyến.

import { invoke } from "@/api/ipc";

export interface PhongLichSu {
  id: string;
  projectId: string;
  phongId: string;
  phienBan: number;
  lichSuDonVi: string | null;
  lichSuPhong: string | null;
  thoiGianTaiLieu: string | null;
  tongSoTaiLieu: string | null;
  phuongAnPhanLoai: string | null;
  ngonNgu: string | null;
  trangThaiPhong: "mo" | "dong";
  nguon: string | null;
  ghiChu: string | null;
  createdBy: string | null;
  createdAt: string;
  nguoiLap: string | null;
}

export interface PhongLichSuMoi {
  lichSuDonVi?: string | null;
  lichSuPhong?: string | null;
  thoiGianTaiLieu?: string | null;
  tongSoTaiLieu?: string | null;
  phuongAnPhanLoai?: string | null;
  ngonNgu?: string | null;
  trangThaiPhong?: "mo" | "dong";
  nguon?: string | null;
  ghiChu?: string | null;
}

export const phongLichSuList = (phongId: string) => invoke<{ phongId: string; rows: PhongLichSu[] }>("bc_call", { method: "GET", path: `/cl/phong/${phongId}/lich-su`, body: null });
export const phongLichSuThem = (phongId: string, body: PhongLichSuMoi) => invoke<PhongLichSu>("bc_call", { method: "POST", path: `/cl/phong/${phongId}/lich-su`, body });
