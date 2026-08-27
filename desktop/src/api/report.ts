// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export interface V4nt {
  maDuAn?: string;
  tenDuAn?: string;
  soHoSo: number;
  soPdf: number;
  a0: number;
  a1: number;
  a2: number;
  a3: number;
  a4: number;
  a5: number;
  khac: number;
  tongTrang: number;
  quyDoi: number;
}

export interface V4nu {
  maDv?: string;
  tenDonVi?: string;
  hopVinhVien: number;
  hoSoVinhVien: number;
  hopCoThoiHan: number;
  hoSoCoThoiHan: number;
  boLoai: number;
  tapLoai: number;
}

export interface V4nv {
  userId: string;
  ten: string;
  soHoSo: number;
  soVanBan: number;
  soTrangQuet: number;
  thaoTacThem: number;
  thaoTacSua: number;
}

export interface V4nw {
  maDuAn?: string;
  tenDuAn?: string;
  thoiGianKhoiTao?: string;
  hopVinhVien: number;
  hoSoVinhVien: number;
  hopCoThoiHan: number;
  hoSoCoThoiHan: number;
  boLoai: number;
  tapLoai: number;
}

export function V4nx(): Promise<V4nt[]> {
  return invoke<V4nt[]>("report_so_hoa");
}

export function V4ny(maDa?: string): Promise<V4nu[]> {
  return invoke<V4nu[]>("report_chinh_ly", { args: { maDa } });
}

export function V4nz(V21q?: string, V21r?: string): Promise<V4nw[]> {
  return invoke<V4nw[]>("report_phan_mem", { args: { tuNgay: V21q, denNgay: V21r } });
}

export function V4o0(): Promise<V4nv[]> {
  return invoke<V4nv[]>("report_nhan_vien");
}
