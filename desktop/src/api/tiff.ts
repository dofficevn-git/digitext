// Copyright (c) 2026 DigiText. All rights reserved.
// Tách TIFF (tài liệu Đảng): quét thư mục PDF, xem trước tên, chạy / tạm dừng / dừng. Lõi ở src-tauri/src/tiff.rs.
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke, isTauri } from "@/api/ipc";

export type TiffRule = "v1" | "v2";

/** Mẫu đặt tên — khớp `digitext_core::tiff_naming::TiffNaming`. */
export interface TiffNaming {
  rule: TiffRule;
  separator: string;
  pad: number;
  start: number;
  mirror: boolean;
  docTypeCodes: string[];
}

export const MA_LOAI_VB_DANG = ["ĐU", "NQ", "QĐ", "SNQ", "KHAC", "TL", "BC", "CT", "TB", "HD"];

/** Giống cấu hình mặc định của tool cũ: phân cách "-", STT 3 chữ số, bắt đầu 1, giữ thư mục cha. */
export function tiffPreset(rule: TiffRule): TiffNaming {
  return { rule, separator: "-", pad: 3, start: 1, mirror: true, docTypeCodes: [...MA_LOAI_VB_DANG] };
}

export interface TiffInput {
  sourceDirs: string[];
  fileName: string;
  path?: string;
  docId?: string;
  pages?: number;
}

export interface LocalPdf {
  path: string;
  sourceDirs: string[];
  fileName: string;
}

export function tiffScanLocal(folder: string): Promise<LocalPdf[]> {
  return invoke<LocalPdf[]>("tiff_scan_local", { folder });
}

export interface TiffPreviewFolder {
  sourceDirs: string[];
  outputDirs: string[];
  files: number;
  firstPdf: string;
  firstTiff: string;
  exists: boolean;
}
export interface TiffPreview {
  folderCount: number;
  fileCount: number;
  folders: TiffPreviewFolder[];
  collisions: string[];
}

export function tiffPreview(inputs: TiffInput[], naming: TiffNaming, outDir?: string): Promise<TiffPreview> {
  return invoke<TiffPreview>("tiff_preview", { args: { inputs, naming, outDir: outDir || null } });
}

export interface TiffRunArgs {
  inputs: TiffInput[];
  naming: TiffNaming;
  outDir: string;
  onExist: "skip" | "overwrite";
  forceRgb: boolean;
  uncompressed: boolean;
  concurrency?: number;
}

export interface TiffProgress {
  doneFiles: number;
  totalFiles: number;
  donePages: number;
  file: string;
  status: "ok" | "skipped" | "failed" | "waiting_dest";
  error: string | null;
}

export interface TiffRunResult {
  totalFiles: number;
  ok: number;
  skipped: number;
  failed: number;
  cancelled: number;
  pages: number;
  bytes: number;
  failures: { fileName: string; source: string; error: string }[];
  reportPath: string | null;
}

export function tiffRun(args: TiffRunArgs): Promise<TiffRunResult> {
  return invoke<TiffRunResult>("tiff_run", { args });
}
export function tiffCancel(): Promise<void> {
  return invoke<void>("tiff_cancel");
}
export function tiffPause(paused: boolean): Promise<void> {
  return invoke<void>("tiff_pause", { paused });
}
export async function onTiffProgress(cb: (p: TiffProgress) => void): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  return listen<TiffProgress>("tiff://progress", (e) => cb(e.payload));
}
