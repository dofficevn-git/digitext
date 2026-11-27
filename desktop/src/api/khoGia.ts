// Copyright (c) 2026 DigiText. All rights reserved.
// Kho / giá có sức chứa + gán hộp cho hồ sơ nhập gói (0035, 27/11/2026, đợt 3 nhóm 4B): routes/chinh_ly/kho_gia.rs qua cl_remote.

import { clRemote, qs, type ClRow } from "@/api/chinhly";
import { bcDownload } from "@/api/baoCao";

export type GiaTongHop = ClRow & {
  kho_id: string;
  ten_kho: string;
  phong_ban_id: string | null;
  ten_phong_ban: string | null;
  phong_id: string | null;
  ten_phong: string | null;
  ten: string;
  tu_hop: number | null;
  den_hop: number | null;
  vinh_vien: boolean | null;
  suc_chua_hop: number | null;
  canh_bao_pct: number;
  suc_chua: number | null;
  so_hop: number;
  so_ho_so: number;
  pct_day: number | null;
  gan_day: boolean;
  vuot: boolean;
};

export const kgTongHop = (projectId: string, khoId?: string, phongId?: string) =>
  clRemote<{ rows: GiaTongHop[]; ganDay: number; vuot: number }>("GET", `/cl/kho-gia/tong-hop${qs({ projectId, khoId, phongId })}`);

export const kgChuaHop = (projectId: string, phongId: string, limit = 2000) => clRemote<{ rows: ClRow[] }>("GET", `/cl/kho-gia/chua-hop${qs({ projectId, phongId, limit })}`);

export interface GanHopKetQua {
  applied: number;
  hopsCreated: number;
  phongBanDat: number;
  errors: { hoSoId: string; row?: number; message: string }[];
}
export const kgGanHop = (projectId: string, phongBanId: string, rows: { hoSoId: string; hopSo: string }[]) =>
  clRemote<GanHopKetQua>("POST", "/cl/ho-so/gan-hop", { projectId, phongBanId, rows });

export const kgXuatHoSo = (projectId: string, pham_vi: { khoId?: string; giaId?: string }, ten: string) =>
  bcDownload(`/cl/kho-gia/ho-so${qs({ projectId, ...pham_vi, format: "xlsx" })}`, `ho-so-theo-${ten}.xlsx`);
