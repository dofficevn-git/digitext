// Copyright (c) 2026 DigiText. All rights reserved.

import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { invoke, isTauri } from "@/api/ipc";

export interface V4qr {
  id: string;
  projectId: string;
  filename: string;
  hoSoRef: string | null;
  pageCount: number | null;
  status: string;
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
}): Promise<SignBatchResult> {
  return invoke<SignBatchResult>("sign_batch", { args });
}

export function signBatchCancel(): Promise<void> {
  return invoke<void>("sign_batch_cancel");
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
