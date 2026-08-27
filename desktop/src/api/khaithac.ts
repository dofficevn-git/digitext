// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export interface V4ld {
  maDa?: string;
  maDv?: string;
  mangCvid?: number;

  thbqs?: number[];
  maKho?: number;
  maGia?: number;
  hetThbq?: boolean;
  giuLai?: boolean;
  vinhVien?: boolean;
  trungThua?: boolean;
  search?: string;
  page?: number;
  pageSize?: number;
}

export interface V4le {
  hoSoId: number;
  maDa?: string;
  tenDa?: string;
  maDv?: string;
  mangCvid?: number;
  maHoSo?: string;
  hopTaiLieuSo?: string;
  soBo?: string;
  soTap?: string;
  hososo: string;
  tieuDeHs: string;
  thbq?: number;
  thoiHanBaoQuanStr?: string;
  soTo?: string;
  thoiGianBatDau?: string;
  thoiGianKetThuc?: string;
  trungThua: boolean;
  soLuongTrang?: number;
  ghiChu?: string;
  soVanBan: number;
  tenKho?: string;
  tenGia?: string;

  status: string;
}

export interface V4lf {
  total: number;
  rows: V4le[];
}

export interface V4lg {
  dossierId: number;
  hoSo: string;
  hopSo?: string;
  thuTu: number;
  loaiVanBan?: string;
  soKyHieu?: string;
  ngayThang?: string;
  coQuanBanHanh?: string;
  trichYeu?: string;
  butTich?: string;
  toSo?: number;
  soTrang?: number;
  trangSo?: number;
  ghiChu?: string;
}

export interface V4lh {
  maKho: number;
  tenKho: string;
}

export interface V4li {
  maGia: number;
  tenGia: string;
  maKho: number;
}

export function V4lj(filter: V4ld): Promise<V4lf> {
  return invoke<V4lf>("khai_thac_search", { args: { filter } });
}

export function V4lk(filter: V4ld): Promise<V4le[]> {
  return invoke<V4le[]>("khai_thac_rows", { args: { filter } });
}

export function V4ll(filter: V4ld): Promise<V4lg[]> {
  return invoke<V4lg[]>("khai_thac_van_ban", { args: { filter } });
}

export function V4lm(): Promise<V4lh[]> {
  return invoke<V4lh[]>("kho_list");
}

export function V4ln(V1uk?: number): Promise<V4li[]> {
  return invoke<V4li[]>("gia_list", { args: { maKho: V1uk } });
}

export interface V4lo {
  maKho: number;
  tenKho: string;

  soDoKho?: string;
  ghiChu?: string;
  soTt?: number;
  soGia: number;
}

export interface V4lp {
  maGia: number;
  tenGia: string;
  maKho: number;
  tenKho?: string;
  maDa?: string;
  tenDuAn?: string;
  maDv?: string;
  tenDonVi?: string;
  tuHop?: number;
  denHop?: number;
  ghiChu?: string;
  soTt?: number;
  soHoSo: number;
}

export interface V4lq {
  maKho?: number;
  tenKho: string;
  soDoKho?: string;
  ghiChu?: string;
  soTt?: number;
}

export interface V4lr {
  maGia?: number;
  tenGia: string;
  maKho: number;
  maDa?: string;
  maDv?: string;
  tuHop?: number;
  denHop?: number;
  ghiChu?: string;
  soTt?: number;
}

export function V4ls(): Promise<V4lo[]> {
  return invoke<V4lo[]>("kho_list_detail");
}

export function V4lt(kho: V4lq): Promise<number> {
  return invoke<number>("kho_upsert", { args: { kho } });
}

export function V4lu(V1ul: number): Promise<void> {
  return invoke<void>("kho_delete", { args: { maKho: V1ul } });
}

export function V4lv(V1um?: number): Promise<V4lp[]> {
  return invoke<V4lp[]>("gia_list_detail", { args: { maKho: V1um } });
}

export function V4lw(V1un: V4lr): Promise<number> {
  return invoke<number>("gia_upsert", { args: { gia: V1un } });
}

export function V4lx(V1uo: number): Promise<void> {
  return invoke<void>("gia_delete", { args: { maGia: V1uo } });
}

export function V4ly(V1up: number[], V1uq?: number): Promise<number> {
  return invoke<number>("dossier_assign_gia", { args: { maGia: V1uq, dossierIds: V1up } });
}

export function V4lz(V1ur: number): Promise<number> {
  return invoke<number>("dossier_assign_gia_by_range", { args: { maGia: V1ur } });
}
