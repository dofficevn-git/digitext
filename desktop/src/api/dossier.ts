// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export interface V4bl {
  maDa?: string;
  maDv?: string;
  mangCvid?: number;
  thbq?: number;
  soTam: string;
  maHoSo?: string;
  tieuDe?: string;
  ngayBatDau?: string;
  ngayKetThuc?: string;
  soTo?: string;
  trungThua?: boolean;
  tongSoVanBan?: number;
  soLuongTrang?: number;
  cheDoSuDung?: string;
  ngonNgu?: string;
  tinhTrangVatLy?: string;
  ghiChu?: string;
  hopSo?: string;
  boSo?: string;
  tapSo?: string;
  trangThai?: string;
}

export interface V4bm {
  id: number;
  maDa?: string;
  tenDa?: string;
  maDv?: string;
  mangCvid?: number;
  thbq?: number;
  soTam: string;
  maHoSo?: string;
  tieuDe: string;
  ngayBatDau?: string;
  ngayKetThuc?: string;
  soTo?: string;
  trungThua: boolean;
  tongSoVanBan?: number;
  soLuongTrang?: number;
  cheDoSuDung?: string;
  ngonNgu?: string;
  tinhTrangVatLy?: string;
  ghiChu?: string;
  hopSo?: string;
  boSo?: string;
  tapSo?: string;
  trangThai: string;
  soVanBan: number;
  soTaiLieuScan: number;
  scanPdfPath?: string;
  soFileScan: number;
  createdAt: string;
  updatedAt?: string;
  thuTuSapXep?: number;
}

export type V4bn = "thbqAsc" | "thbqDesc" | "timeAsc" | "timeDesc";

export interface V4bo {
  maDa?: string;
  maDv?: string;
  mangCvid?: number;
  thbq?: number;
  search?: string;
}

export interface V4bp {
  thuTu?: number;
  loaiVanBan?: string;
  soKyHieu?: string;
  ngayThang?: string;
  coQuanBanHanh?: string;
  trichYeu?: string;
  toSo?: number;
  soTrang?: number;
  trangSo?: number;
  ngonNgu?: string;
  cheDoSuDung?: string;
  doTinCay?: string;
  butTich?: string;
  tinhTrangVatLy?: string;
  taiLieuSo?: string;
  ghiChu?: string;
  pdfPath?: string;
}

export interface V4bq {
  id: number;
  dossierId: number;
  thuTu: number;
  loaiVanBan?: string;
  soKyHieu?: string;
  ngayThang?: string;
  coQuanBanHanh?: string;
  trichYeu?: string;
  toSo?: number;
  soTrang?: number;
  trangSo?: number;
  ngonNgu?: string;
  cheDoSuDung?: string;
  doTinCay?: string;
  butTich?: string;
  tinhTrangVatLy?: string;
  taiLieuSo?: string;
  ghiChu?: string;
  pdfPath?: string;
  createdAt: string;
  updatedAt?: string;
}

export function V4br(filter?: V4bo): Promise<V4bm[]> {
  return invoke<V4bm[]>("dossier_list", { args: { filter } });
}

export function V4bs(Vvo: number): Promise<V4bm> {
  return invoke<V4bm>("dossier_get", { args: { id: Vvo } });
}

export function V4bt(dossier: V4bl): Promise<number> {
  return invoke<number>("dossier_create", { args: { dossier } });
}

export interface V4bu {
  created: number;

  skippedExisting: number;
}

export function V4bv(maDa?: string): Promise<V4bu> {
  return invoke<V4bu>("dossier_sync_from_kho", { args: { maDa } });
}

export function V4bw(Vvp: number, dossier: V4bl): Promise<void> {
  return invoke<void>("dossier_update", { args: { id: Vvp, dossier } });
}

export function V4bx(Vvq: number): Promise<void> {
  return invoke<void>("dossier_delete", { args: { id: Vvq } });
}

export function V4by(Vvr: number): Promise<V4bq[]> {
  return invoke<V4bq[]>("van_ban_list", { args: { dossierId: Vvr } });
}

export function V4bz(Vvs: number, Vvt: V4bp): Promise<number> {
  return invoke<number>("van_ban_create", { args: { dossierId: Vvs, vanBan: Vvt } });
}

export function V4c0(Vvu: number, Vvv: V4bp): Promise<void> {
  return invoke<void>("van_ban_update", { args: { id: Vvu, vanBan: Vvv } });
}

export function V4c1(Vvw: number): Promise<void> {
  return invoke<void>("van_ban_delete", { args: { id: Vvw } });
}

export function V4c2(maDa: string | undefined, Vvx: V4bn, Vvy = 0): Promise<number> {
  return invoke<number>("dossier_systematize", { args: { maDa, scheme: Vvx, perBox: Vvy } });
}

export function V4c3(Vvz: number, Vw0?: string, Vw1?: string, Vw2?: string): Promise<void> {
  return invoke<void>("dossier_set_boxing", {
    args: { id: Vvz, hopSo: Vw0, boSo: Vw1, tapSo: Vw2 },
  });
}

export function V4c4(maDa: string | undefined, Vw3: V4bl[]): Promise<number> {
  return invoke<number>("dossier_bulk_upsert", { args: { maDa, rows: Vw3 } });
}

export interface V4c5 {
  vanBanId: number;
  dossierId: number;
  hoSo: string;
  thuTu: number;
  soKyHieu?: string;
  trichYeu?: string;
  ngayThang?: string;
}

export interface V4c6 {
  fileName: string;
  filePath: string;
  dpi: number;
  isPass: boolean;
}

export function V4c7(filter?: V4bo): Promise<V4c5[]> {
  return invoke<V4c5[]>("dossier_incomplete_van_ban", { args: { filter } });
}

export function V4c8(filter?: V4bo): Promise<number> {
  return invoke<number>("dossier_delete_incomplete_van_ban", { args: { filter } });
}

export function V4c9(Vw4: number[], Vw5?: number): Promise<V4c6[]> {
  return invoke<V4c6[]>("dossier_scan_files", { args: { ids: Vw4, minDpi: Vw5 } });
}

export interface V4ca {
  fileName: string;
  filePath: string;
  pageCount: number;
  loaiVanBan?: string | null;
  soKyHieu?: string | null;
  ngayThang?: string | null;
  tacGia?: string | null;
  trichYeu?: string | null;
  tinhTrangVatLy?: string | null;
}

export function V4cb(Vw6: number): Promise<V4ca[]> {
  return invoke<V4ca[]>("dossier_scan_pdf_files", { args: { id: Vw6 } });
}

export function V4cc(): Promise<string> {
  return invoke<string>("get_ocr_url");
}

export function V4cd(url: string | null): Promise<void> {
  return invoke<void>("set_ocr_url", { args: { url } });
}

export function V4ce(url: string): Promise<void> {
  return invoke<void>("open_url", { args: { url } });
}

export interface V4cf {
  dossierId: number;
  vanBan: V4bp;
}

export function V4cg(Vw7: V4cf[]): Promise<number> {
  return invoke<number>("van_ban_bulk_insert", { args: { items: Vw7 } });
}

export interface V4ch {
  id: number;
  entity: string;
  entityId: number;
  dossierId?: number;
  maDa?: string;
  action: string;
  summary?: string;
  content?: string;
  nguoi?: string;
  createdAt: string;
}

export interface V4ci {
  entity?: string;
  maDa?: string;
  tuNgay?: string;
  denNgay?: string;
  search?: string;
}

export function V4cj(filter?: V4ci): Promise<V4ch[]> {
  return invoke<V4ch[]>("dossier_history", {
    args: { ...(filter ?? {}) },
  });
}
