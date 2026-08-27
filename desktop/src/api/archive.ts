// Copyright (c) 2026 DigiText. All rights reserved.

import { message } from "@tauri-apps/plugin-dialog";

import { invoke } from "@/api/ipc";
import type { V3yc } from "@/domain/types";

export interface V408 {
  maDuAn: string;
  tenDuAn: string;
  maKhoi?: string;
  phongSo?: string;
  mucLucSo?: string;
}

export interface V409 {
  hopSo: string;
  hoSoSo: string;
  tieuDe?: string;
  thoiGian?: string;
  soTo?: string;
  thbq?: string;
  ghiChu?: string;
  nhom?: string;
  nam?: string;
  ngayBatDau?: string;
  ngayKetThuc?: string;
}

export interface V40a {
  sourcePath: string;
  dpi?: number;
  paper?: string;
}

export interface V40b {
  storageRoot?: string;

  jpegQuality?: number;

  stripAccents?: boolean;
}

export interface V40c {
  hoSoId: number;
  folderPath: string;
  pdfPath: string;
  pdfSize: number;
  pageCount: number;
}

export interface V40d {
  id: number;
  phongId?: string;
  hopSo: string;
  hoSoSo: string;
  tieuDe: string;
  thoiGian?: string;
  soTo?: string;
  thbq?: string;
  ghiChu?: string;
  nhom?: string;
  nam?: string;
  folderPath?: string;
  pdfPath?: string;
  pdfSize?: number;
  pageCount?: number;
  status: string;
  ngayBatDau?: string;
  ngayKetThuc?: string;
}

export interface V40e {
  hoSo: number;

  dossier: number;
}

export function V40f(V1u: V408, V1v: V409[]): Promise<V40e> {
  return invoke<V40e>("import_ho_so", { args: { phong: V1u, hoSos: V1v } });
}

export interface V40g {
  rootPath: string;
  phongFolder: string;
  created: number;

  syncedDossier?: number;
}

export function V40h(V1w: V408, V1x: V409[], V1y?: V40b): Promise<V40g> {
  return invoke<V40g>("create_folder_tree", {
    args: { phong: V1w, hoSos: V1x, options: V1y },
  });
}

export function V40i(
  V1z: V408,
  V20: V409,
  V21: V40a[],
  V22?: V40b,
  V23?: string,
  V24?: number,
): Promise<V40c> {
  return invoke<V40c>("finalize_ho_so", {
    args: { phong: V1z, hoSo: V20, pages: V21, options: V22, pdfOut: V23, hoSoId: V24 },
  });
}

export interface V40j {
  pdfPath: string;
  pdfSize: number;
  pageCount: number;
}

export function V40k(V25: V408, V26: V409, V27: V40a[], V28?: V40b): Promise<V40j> {
  return invoke<V40j>("merge_scanned_pages_pdf", {
    args: { phong: V25, hoSo: V26, pages: V27, options: V28 },
  });
}

export interface V40l {
  id: number;
  maDuAn: string;
  tenDuAn: string;
  maKhoi?: string;
  phongSo?: string;
  mucLucSo?: string;
  hopSo: string;
  hoSoSo: string;
  tieuDe: string;
  thoiGian?: string;
  thbq?: string;
  nhom?: string;
  nam?: string;
  folderPath?: string;
  pdfPath?: string;
  status: string;
  ngayBatDau?: string;
  ngayKetThuc?: string;
}

export interface V40m {
  batchId: string;
  hoSo: V40l;
  pages: V3yc[];
}

export function V40n(V29: number, V2a?: string): Promise<V40m> {
  return invoke<V40m>("open_ho_so_for_edit", { args: { hoSoId: V29, pdfPath: V2a } });
}

export function V40o(V2b: string): Promise<V40m> {
  return invoke<V40m>("open_pdf_for_edit", { args: { pdfPath: V2b } });
}

export function V40p(V2c: string, from?: number, V2d?: number): Promise<string[]> {
  return invoke<string[]>("render_pdf_pages", { args: { pdfPath: V2c, from, count: V2d } });
}

export interface V40q {
  pdfPath: string;
  pdfSize: number;
  pageCount: number;
}

export function V40r(V2e: string, V2f: V40a[], V2g?: V40b): Promise<V40q> {
  return invoke<V40q>("rebuild_pdf", { args: { pdfOut: V2e, pages: V2f, options: V2g } });
}

export function V40s(V2h: string, V2i: string[]): Promise<V3yc[]> {
  return invoke<V3yc[]>("insert_pages", { args: { batchId: V2h, paths: V2i } });
}

export function listHoSo(V2j?: string): Promise<V40d[]> {
  return invoke<V40d[]>("list_ho_so", { args: { phongId: V2j } });
}

export function V40t(V2k: number): Promise<void> {
  return invoke<void>("ho_so_delete", { args: { id: V2k } });
}

export function V40u(V2l: number, V2m: V409): Promise<void> {
  return invoke<void>("ho_so_update", { args: { id: V2l, hoSo: V2m } });
}

export interface V40v {
  maDuAn: string;
  tenDuAn: string;
  maKhoi?: string;
  phongSo?: string;
  mucLucSo?: string;
}

export function V40w(): Promise<V40v[]> {
  return invoke<V40v[]>("list_phong");
}

export function V40x(path: string): Promise<void> {
  return invoke<void>("open_pdf", { args: { path } });
}

export async function V40y(path: string): Promise<void> {
  try {
    await V40x(path);
  } catch (V2n) {
    await message(V2n instanceof Error ? V2n.message : String(V2n), {
      title: "S2jDtG5nIG3hu58gxJHGsOG7o2MgUERG",
      kind: "error",
    });
  }
}

export function V40z(): Promise<string> {
  return invoke<string>("get_storage_root");
}

export function V410(path: string | null): Promise<void> {
  return invoke<void>("set_storage_root", { args: { path } });
}

export interface V411 {
  path: string;
}

export function V412(path: string): Promise<void> {
  return invoke<void>("delete_path", { args: { path } });
}

export function V413(path: string, V2o: string): Promise<V411> {
  return invoke<V411>("rename_path", { args: { path, newName: V2o } });
}

export function V414(V2p: string, name: string): Promise<V411> {
  return invoke<V411>("create_subfolder", { args: { parentPath: V2p, name } });
}

export function V415(path: string, V2q: string): Promise<V411> {
  return invoke<V411>("move_path", { args: { path, destDir: V2q } });
}

export interface V416 {
  name: string;
  path: string;
  isDir: boolean;
  isPdf: boolean;
  size: number;
}

export function V417(path?: string): Promise<V416[]> {
  return invoke<V416[]>("read_dir", { args: { path } });
}

export interface V418 {
  id: number;
  maDuAn: string;
  tenDuAn: string;
  hopSo: string;
  hoSoSo: string;
  tieuDe: string;
  phongSo?: string;
  mucLucSo?: string;
  ngayBatDau?: string;
  ngayKetThuc?: string;
  nam?: string;
  thoiGian?: string;
}

export function V419(V2r: string): Promise<V418 | null> {
  return invoke<V418 | null>("ho_so_at", { args: { folderPath: V2r } });
}
