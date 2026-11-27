// Copyright (c) 2026 DigiText. All rights reserved.
// Bảng mã TT05 còn thiếu (0040, đợt 6, Digifile B03): chế độ sử dụng, mức độ tin cậy, tình trạng vật lý, tình trạng dự phòng.
// Chỉ trực tuyến; màn Danh mục › Mã TT05 và các ô chọn trong form hồ sơ / văn bản (có hằng số dự phòng khi ngoại tuyến).

import { invoke } from "@/api/ipc";

export type NhomMa = "che_do_su_dung" | "muc_do_tin_cay" | "tinh_trang_vat_ly" | "tinh_trang_du_phong";
export const NHOM_MA: [NhomMa, string][] = [
  ["che_do_su_dung", "Chế độ sử dụng"],
  ["muc_do_tin_cay", "Mức độ tin cậy"],
  ["tinh_trang_vat_ly", "Tình trạng vật lý"],
  ["tinh_trang_du_phong", "Tình trạng dự phòng"],
];

export interface MaTt05 {
  id: string;
  projectId: string | null;
  nhom: NhomMa;
  ma: string;
  ten: string;
  soTt: number | null;
  ghiChu: string | null;
}

export interface MaTt05Moi {
  nhom: NhomMa;
  ma: string;
  ten: string;
  soTt?: number | null;
  ghiChu?: string | null;
}

const call = <T,>(method: string, path: string, body?: unknown) => invoke<T>("bc_call", { method, path, body: body ?? null });

export const maTt05List = (projectId: string, nhom?: NhomMa) =>
  call<{ nhom: NhomMa[]; rows: MaTt05[] }>("GET", `/cl/ma-tt05?projectId=${encodeURIComponent(projectId)}${nhom ? `&nhom=${nhom}` : ""}`);
export const maTt05Them = (projectId: string, m: MaTt05Moi, dungChung = false) => call<MaTt05>("POST", "/cl/ma-tt05", { projectId, dungChung, ...m });
export const maTt05Sua = (id: string, m: MaTt05Moi) => call<MaTt05>("PUT", `/cl/ma-tt05/${id}`, m);
export const maTt05Xoa = (id: string) => call<null>("DELETE", `/cl/ma-tt05/${id}`);

/** Bộ chọn [mã, tên] cho một nhóm, có hằng số dự phòng khi không tải được. */
export function tuyChon(rows: MaTt05[] | null, nhom: NhomMa, duPhong: [string, string][]): [string, string][] {
  const xs = (rows ?? []).filter((r) => r.nhom === nhom).map((r) => [r.ma, r.ten] as [string, string]);
  return xs.length > 0 ? xs : duPhong;
}
