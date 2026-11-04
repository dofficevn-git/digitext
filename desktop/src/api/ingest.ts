// Copyright (c) 2026 DigiText. All rights reserved.
// Nhập gói vào Hệ thống (bản Lưu trữ GĐ3, 4/11/2026) — Tauri src-tauri/src/ingest.rs → máy chủ routes/ingest.rs.

import { Channel } from "@tauri-apps/api/core";
import { invoke } from "@/api/ipc";

export interface GoiTimThay {
  path: string;
  name: string;
  size: number;
  laThuMuc: boolean;
}

export type KetQuaNhap = "nhap" | "da_co" | "tu_choi" | "loi";

export interface KetQuaGoi {
  tenTep: string;
  objid: string;
  ketQua: KetQuaNhap;
  ketLuanKiem: string;
  soLoi: number;
  soCanhBao: number;
  packageId: string | null;
  hoSoId: string | null;
  maHoSo: string;
  tieuDe: string;
  soVanBan: number;
  soChuaKy: number;
  thongBao: string | null;
}

export interface LoNhap {
  id: string;
  nguon: "lenh" | "tai_len";
  ghiChu: string | null;
  nguoiTao: string | null;
  createdAt: string;
  finishedAt: string | null;
  tong: number;
  nhap: number;
  daCo: number;
  tuChoi: number;
  loi: number;
  soVanBan: number;
  soChuaKy: number;
}

export interface GoiTrongLo {
  id: string;
  tenTep: string;
  objid: string | null;
  loaiGoi: string | null;
  sizeBytes: number | null;
  ketQua: KetQuaNhap;
  ketLuanKiem: string | null;
  soLoi: number;
  soCanhBao: number;
  maHoSo: string | null;
  tieuDe: string | null;
  soVanBan: number;
  soChuaKy: number;
  thongBao: string | null;
  chiTiet: { level: "ok" | "warn" | "err"; group: string; message: string }[] | null;
  createdAt: string;
}

export interface TienDoTai {
  daGui: number;
  tong: number;
  buoc: "tai_len" | "nhap";
}

export function ingestDiscover(paths: string[]): Promise<GoiTimThay[]> {
  return invoke<GoiTimThay[]>("ingest_discover", { args: { paths } });
}

export async function ingestBatchCreate(projectId: string, ghiChu?: string): Promise<string> {
  return (await invoke<{ id: string }>("ingest_batch_create", { args: { projectId, ghiChu: ghiChu ?? null } })).id;
}

export function ingestBatches(projectId: string): Promise<LoNhap[]> {
  return invoke<LoNhap[]>("ingest_batches", { args: { projectId } });
}

export function ingestBatchItems(id: string): Promise<GoiTrongLo[]> {
  return invoke<GoiTrongLo[]>("ingest_batch_items", { args: { id } });
}

export function ingestBatchFinish(id: string): Promise<void> {
  return invoke("ingest_batch_finish", { args: { id } });
}

export function ingestUpload(batchId: string, path: string, nhanCaGoiLoi: boolean, onTienDo: (t: TienDoTai) => void): Promise<KetQuaGoi> {
  const ch = new Channel<TienDoTai>();
  ch.onmessage = onTienDo;
  return invoke<KetQuaGoi>("ingest_upload", { args: { batchId, path, nhanCaGoiLoi }, onProgress: ch });
}
