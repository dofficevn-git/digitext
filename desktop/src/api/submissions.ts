// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";
import type { V4rt } from "@/api/packaging";

export interface V4rx {
  id: string;
  projectId: string;
  /** 0040: cơ quan trong danh mục cơ quan nộp lưu (null = khai tay / đợt cũ). */
  coQuanNopId?: string | null;
  maYeuCau: string;
  orgName: string;
  orgCode: string;
  soLanNop: number;
  fondCode: string;
  cachThuc: string;
  status: string;
  decisionNote: string | null;
  reconcileNote: string | null;
  approvalDoc: string | null;

  hanNop: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface V4ry {
  id: string;
  stt: number;
  maHoSo: string;
  tieuDe: string;
  thoiGianBatDau: string | null;
  thoiGianKetThuc: string | null;
  soTrang: number | null;
  mucDoTiepCan: string | null;
  ghiChu: string | null;
  reviewResult: "pending" | "dat" | "khong_dat";
  reviewNote: string | null;
  /** 0040: hồ sơ có sẵn trong hệ thống (nộp cùng hệ thống); trùng mã trong cùng đợt. */
  hoSoId?: string | null;
  trungTrongDot?: boolean;
}

export interface V4rz extends V4rx {
  summary: string | null;
  receptionNote: string | null;
  approvalNote: string | null;
  items: V4ry[];
  packages: V4rt[];
  duplicateInSystem: string[];
}

export interface V4s0 {
  maHoSo: string;
  tieuDe: string;
  thoiGianBatDau?: string | null;
  thoiGianKetThuc?: string | null;
  soTrang?: number | null;
  mucDoTiepCan?: string | null;
  ghiChu?: string | null;
}

export interface V4s1 {
  projectId?: string | null;
  /** 0040: chọn cơ quan trong danh mục; có thì orgName / orgCode lấy từ đó. */
  coQuanNopId?: string | null;
  orgName: string;
  orgCode: string;
  daNopGiay?: boolean;
  summary?: string | null;
  fondCode: string;
  cachThuc?: "truc_tiep" | "truc_tuyen";
  totalPages?: number | null;

  hanNop?: string | null;
  items: V4s0[];
}

export function V4s2(): Promise<V4rx[]> {
  return invoke<V4rx[]>("submissions_list");
}

export function V4s3(V2nw: string): Promise<V4rz> {
  return invoke<V4rz>("submission_get", { args: { id: V2nw } });
}

export function V4s4(body: V4s1): Promise<V4rz> {
  return invoke<V4rz>("submission_register", { args: { body } });
}

export type V4s5 = "confirm" | "refuse" | "reconcile" | "receive" | "complete";

export function V4s6(V2nx: string, V2ny: V4s5, V2nz?: string): Promise<V4rx> {
  return invoke<V4rx>("submission_action", {
    args: { id: V2nx, action: V2ny, note: V2nz ?? null },
  });
}

export function V4s7(
  V2o0: string,
  V2o1: string,
  V2o2: "dat" | "khong_dat",
  V2o3?: string,
): Promise<void> {
  return invoke<void>("submission_review_item", {
    args: { id: V2o0, itemId: V2o1, result: V2o2, note: V2o3 ?? null },
  });
}

export interface V4s8 {
  submission: V4rx;
  convertedAipIds: string[];
}

export function V4s9(V2o4: string, V2o5: string, V2o6?: string): Promise<V4s8> {
  return invoke<V4s8>("submission_approve", {
    args: { id: V2o4, quyetDinhSo: V2o5, note: V2o6 ?? null },
  });
}

export interface V4sa {
  id: string;
  submissionId: string | null;
  kind: string;
  message: string;
  readAt: string | null;
  createdAt: string;
}

export interface V4sb {
  unread: number;
  items: V4sa[];
}

export function V4sc(): Promise<V4sb> {
  return invoke<V4sb>("notifications_list");
}

export function V4sd(V2o7: string): Promise<void> {
  return invoke<void>("notification_read", { args: { id: V2o7 } });
}

export function V4se(): Promise<void> {
  return invoke<void>("notifications_read_all");
}

export interface V4sf extends V4ry {
  sipPackageId: string | null;
  sipZipName: string | null;
}

export interface V4sg {
  submission: V4rx;
  totalRegistered: number;
  totalApproved: number;
  totalReturned: number;
  returnedItems: V4sf[];
  sipPackages: V4rt[];
}

export function V4sh(V2o8: string): Promise<V4sg> {
  return invoke<V4sg>("submission_returned", { args: { id: V2o8 } });
}

export function V4si(V2o9: string, V2oa: string): Promise<string> {
  return invoke<string>("submission_dossier", { args: { id: V2o9, fileName: V2oa } });
}

// ---------------- 0040 (đợt 6): tệp đính kèm, nộp cùng hệ thống, so sánh trùng, tổng quan tiếp nhận — qua bc_call
export interface TepThuNop {
  id: string;
  submissionId: string;
  loai: "van_ban_de_nghi" | "lich_su_phong" | "phuong_an_phan_loai" | "huong_dan_xdgt" | "quyet_dinh" | "khac";
  filename: string;
  sizeBytes: number;
  blake3: string | null;
  ghiChu: string | null;
  uploadedBy: string | null;
  createdAt: string;
}
export const LOAI_TEP_THU_NOP: [TepThuNop["loai"], string][] = [
  ["van_ban_de_nghi", "Văn bản đề nghị nộp lưu"],
  ["lich_su_phong", "Lịch sử đơn vị hình thành phông, lịch sử phông"],
  ["phuong_an_phan_loai", "Phương án phân loại"],
  ["huong_dan_xdgt", "Hướng dẫn xác định giá trị tài liệu"],
  ["quyet_dinh", "Quyết định phê duyệt"],
  ["khac", "Khác"],
];
const call = <T,>(method: string, path: string, body?: unknown) => invoke<T>("bc_call", { method, path, body: body ?? null });
export const subFiles = (id: string) => call<TepThuNop[]>("GET", `/submissions/${id}/files`);
export const subFileThem = (id: string, loai: TepThuNop["loai"], filename: string, contentBase64: string, ghiChu?: string | null) =>
  call<TepThuNop>("POST", `/submissions/${id}/files`, { loai, filename, contentBase64, ghiChu: ghiChu ?? null });
export const subFileXoa = (id: string, fid: string) => call<null>("DELETE", `/submissions/${id}/files/${fid}`);
export const subFileTai = (id: string, fid: string, filename: string) => invoke<string | null>("bc_download", { path: `/submissions/${id}/files/${fid}`, defaultName: filename });
export const subItemsTuHoSo = (id: string, hoSoIds: string[]) => call<V4ry[]>("POST", `/submissions/${id}/items/tu-ho-so`, { hoSoIds });
export interface SoSanhTrung {
  dong: V4ry;
  trongKho: Array<{ id: string; ma_ho_so: string | null; ma_ho_so_tt05: string | null; tieu_de: string; ngay_bat_dau: string | null; ngay_ket_thuc: string | null; so_to: string | null; ten_phong: string | null; ten_thbq: string | null; so_van_ban: number; hop_so: string | null; khop: "ma" | "tieu_de" }>;
  trongDot: V4ry[];
}
export const subSoSanh = (id: string, itemId: string) => call<SoSanhTrung>("GET", `/submissions/${id}/items/${itemId}/so-sanh`);
export interface TongQuanTiepNhan {
  tongGoi: number;
  hoSoDangKy: number;
  hoSoDaDuyet: number;
  hoSoKhongDat: number;
  hoSoTrungDot: number;
  hoSoDaVaoHeThong: number;
  dungLuongGoi: number;
  totalPages: number | null;
  totalBytes: number | null;
  soLanNop: number;
  cachThuc: string;
  createdAt: string;
  confirmedAt: string | null;
  receivedAt: string | null;
  approvedAt: string | null;
  completedAt: string | null;
  hanNop: string | null;
  soTepDinhKem: number;
}
export const subTongQuan = (id: string) => call<TongQuanTiepNhan>("GET", `/submissions/${id}/tong-quan`);
export interface HoSoChon {
  id: string;
  ma_ho_so: string | null;
  ma_ho_so_tt05: string | null;
  tieu_de: string;
  ngay_ket_thuc: string | null;
  so_to: string | null;
  ten_phong: string | null;
}
export const hoSoTim = (projectId: string, q: string) =>
  call<{ rows: HoSoChon[]; total: number }>("GET", `/cl/ho-so?projectId=${encodeURIComponent(projectId)}&q=${encodeURIComponent(q)}&limit=50`);
