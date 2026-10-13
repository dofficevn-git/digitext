// Copyright (c) 2026 DigiText. All rights reserved.

import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { invoke, isTauri } from "@/api/ipc";

export interface V4qr {
  id: string;
  projectId: string;
  filename: string;
  hoSoRef: string | null;
  /** Thư mục phông/hộp/hồ sơ trên máy chủ. */
  relPath?: string | null;
  pageCount: number | null;
  status: string;
}

/** Hình thức ký cho cả lô. */
export interface SignStyle {
  /** Ký có dấu (true) / không dấu (false). */
  withStamp?: boolean;
  /** Ảnh con dấu (base64) — chỉ dùng khi ký có dấu. */
  stampPng?: string | null;
  /** Gắn dấu thời gian (TSA) hay không. */
  timestamp?: boolean;
  /** Tên hiển thị trên khối chữ ký thay tên trong chứng thư (trống = tên trong chứng thư). */
  signerName?: string | null;
}

export interface SignCertInfo {
  subject: string;
  organization: string;
  issuer: string;
  serial: string;
  thumbprint: string;
  notAfter: string;
}

export interface V4qs {
  index: number;
  label: string;
}

export function V4qt(projectId?: string): Promise<V4qr[]> {
  return invoke<V4qr[]>("sign_list_jobs", { args: projectId ? { projectId } : null });
}

/** Vị trí khối chữ ký: góc trên-trái theo tỉ lệ trang (0..1, y từ mép trên). */
export interface SignPlacement {
  lastPage: boolean;
  x: number;
  y: number;
}

/** PDF/A của một tài liệu chờ ký (base64) — để hiện trang chọn vị trí. */
export function signPreviewPdf(jobId: string): Promise<string> {
  return invoke<string>("sign_preview_pdf", { args: { jobId } });
}

export interface SignBatchProgress {
  done: number;
  total: number;
  jobId: string;
  status: "signed" | "rejected" | "failed";
  error: string | null;
}

export interface SignBatchResult {
  total: number;
  signed: number;
  rejected: number;
  failed: number;
  cancelled: number;
  stoppedReason: string | null;
}

export function signBatch(args: {
  jobIds: string[];
  modulePath: string;
  slotIndex: number;
  pin: string;
  placement: SignPlacement | null;
  style?: SignStyle;
}): Promise<SignBatchResult> {
  return invoke<SignBatchResult>("sign_batch", { args });
}

export function signBatchCancel(): Promise<void> {
  return invoke<void>("sign_batch_cancel");
}

export function signBatchPause(paused: boolean): Promise<void> {
  return invoke<void>("sign_batch_pause", { paused });
}

/** Đọc ảnh con dấu trên máy → base64 (≤ 500 KB). */
export function signLoadStamp(path: string): Promise<string> {
  return invoke<string>("sign_load_stamp", { path });
}

/** Mở token bằng PIN, đọc chứng thư sẽ dùng để ký (thumbprint, tên đơn vị…). */
export function signCertInfo(args: { modulePath: string; slotIndex: number; pin: string }): Promise<SignCertInfo> {
  return invoke<SignCertInfo>("sign_cert_info", { args });
}

/** Trả các tài liệu ký lỗi về hàng chờ ký. */
export function signRetry(documentIds: string[]): Promise<{ requeued: number; skipped: number }> {
  return invoke<{ requeued: number; skipped: number }>("sign_retry", { documentIds });
}

/** "Huỷ" lô ký: tài liệu vừa ký trong lô quay về chưa ký (xoá bản đã ký trên máy chủ, đưa về hàng chờ). */
export function signUnsign(documentIds: string[]): Promise<{ reverted: number; skipped: number }> {
  return invoke<{ reverted: number; skipped: number }>("sign_unsign", { documentIds });
}

/** Xoá các bản đã ký đã tải về `outDir` (chỉ tệp .pdf trong đúng thư mục đó). */
export function signRemoveLocal(outDir: string, paths: string[]): Promise<number> {
  return invoke<number>("sign_remove_local", { outDir, paths });
}

export async function onSignBatchProgress(cb: (p: SignBatchProgress) => void): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  return listen<SignBatchProgress>("sign://batch", (e) => cb(e.payload));
}

export function V4qu(): Promise<string[]> {
  return invoke<string[]>("sign_detect_modules");
}

export function V4qv(V2gw: string): Promise<V4qs[]> {
  return invoke<V4qs[]>("sign_list_tokens", { args: { modulePath: V2gw } });
}

export function V4qw(V2gx: {
  jobId: string;
  modulePath: string;
  slotIndex: number;
  pin: string;
  placement?: SignPlacement | null;
}): Promise<string> {
  return invoke<string>("sign_execute", { args: V2gx });
}
