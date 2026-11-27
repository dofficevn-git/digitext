// Copyright (c) 2026 DigiText. All rights reserved.
// Chuyển đổi tài liệu lưu trữ số ra giấy (0037, 27/11/2026 — Điều 13–16, Phụ lục VI TT05): routes/chuyen_doi.rs qua Tauri
// bc_get / bc_call / bc_download. Dòng sổ snake_case như bảng.

import { invoke } from "@/api/ipc";
import { qs } from "@/api/chinhly";
import { bcDownload } from "@/api/baoCao";

export interface DongChuyenDoi {
  id: string;
  project_id: string;
  document_id: string | null;
  ho_so_id: string | null;
  van_ban_id: string | null;
  phong_id: string | null;
  nam: number;
  so_chuyen_doi: string;
  ma_tai_lieu: string | null;
  ten_tai_lieu: string | null;
  ma_ho_so: string | null;
  so_trang: number | null;
  muc_dich: string | null;
  nguoi_nhan: string | null;
  can_cu: string | null;
  ky_so_ket_luan: string | null;
  ky_so_chi_tiet: string | null;
  nguoi_thuc_hien: string | null;
  nguoi_thuc_hien_ten?: string | null;
  ten_phong?: string | null;
  chuc_vu: string | null;
  dung_luong_bytes: number | null;
  created_at: string;
}

export interface ChuyenDoiBody {
  projectId: string;
  documentId: string;
  mucDich?: string;
  nguoiNhan?: string;
  canCu?: string;
  chucVu?: string;
}

export const cdTao = (b: ChuyenDoiBody) => invoke<DongChuyenDoi>("bc_call", { method: "POST", path: "/chuyen-doi", body: b });
export const cdSo = (projectId: string, from?: string, to?: string, limit = 500) =>
  invoke<{ rows: DongChuyenDoi[]; total: number }>("bc_get", { path: `/chuyen-doi${qs({ projectId, from, to, limit })}` });
export const cdTai = (r: Pick<DongChuyenDoi, "id" | "so_chuyen_doi">) => bcDownload(`/chuyen-doi/${r.id}/download`, `chuyen-doi-${r.so_chuyen_doi}.pdf`);
export const cdXuatSo = (projectId: string, from?: string, to?: string) => bcDownload(`/chuyen-doi${qs({ projectId, from, to, format: "xlsx" })}`, "so-chuyen-doi.xlsx");
