// Copyright (c) 2026 DigiText. All rights reserved.

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke, isTauri } from "@/api/ipc";

export interface V4eg {
  id: string;
  projectId: string;
  filename: string;
  hoSoRef: string | null;
  pageCount: number | null;
  sizeBytes: number;

  status: string;
  uploadedByName: string | null;
  uploadedAt: string;

  reviewStatus: V4eh;
  reviewedAt: string | null;

  reviewNote: string | null;
}

export type V4eh = "pending" | "approved" | "rejected";

export function V4ei(): Promise<V4eg[]> {
  return invoke<V4eg[]>("docs_list");
}

export interface V4ej {
  id: string;
  filename: string;

  relPath: string | null;
  hoSoRef: string | null;
  pageCount: number | null;
  sizeBytes: number;
  status: string;

  projectId?: string | null;
  projectCode?: string | null;
  projectName?: string | null;

  fieldsStatus?: string | null;

  reviewStatus?: V4eh;

  ocrPages?: number;

  fieldPages?: number;

  pdfaPages?: number | null;

  pdfa2b?: boolean | null;
}

export function V4ek(): Promise<V4ej[]> {
  return invoke<V4ej[]>("docs_tree");
}

/** Loại bản tải: bản gốc đã tải lên / PDF/A (lớp chữ, chưa ký) / bản đã ký số. */
export type DownloadKind = "original" | "pdfa" | "signed";

export function V4el(
  V17h: string,
  V17i: string,
  V17j?: string,
  pdfa = false,
  destDir?: string,
  variant?: DownloadKind,
): Promise<string> {
  return invoke<string>("docs_download", {
    args: { id: V17h, filename: V17i, relPath: V17j, pdfa, destDir: destDir ?? null, variant: variant ?? null },
  });
}

export interface V4em {
  path: string;

  cached: boolean;
  sizeBytes: number;
}

export function V4en(V17k: string, V17l?: number): Promise<V4em> {
  return invoke<V4em>("docs_pdf_for_view", { args: { id: V17k, sizeBytes: V17l } });
}

export interface V4eo {
  id: string;
  received: number;

  total: number | null;
}

export async function V4ep(V17m: (V17n: V4eo) => void): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  return listen<V4eo>("docs://download", (V17o) => V17m(V17o.payload));
}

export interface V4eq {
  id: string;
  filename: string;

  duplicate: boolean;
}

export function V4er(V17p: string, path: string, V17q?: string): Promise<V4eq> {
  return invoke<V4eq>("docs_upload_local", {
    args: { projectId: V17p, path, hoSoRef: V17q },
  });
}

export interface V4es {
  documents: number;

  pages: number;

  skippedNoPageCount: number;
}

export function V4et(V17r?: string): Promise<V4es> {
  return invoke<V4es>("docs_ocr_all", { args: { projectId: V17r } });
}

export interface V4eu {
  documents: number;
  /** Bỏ qua vì đã bị từ chối vĩnh viễn (vd. bản quét < 200 dpi — cần quét lại). */
  skippedRejected?: number;
}

export function V4ev(V17s?: string, V17t?: string[]): Promise<V4eu> {
  return invoke<V4eu>("docs_make_pdf2_all", {
    args: { projectId: V17s, documentIds: V17t },
  });
}

export interface OcrRetryFailedResult {
  documents: number;
  pages: number;
  skippedSigned: number;
}

/** Xếp hàng lại trang OCR lỗi. `includePageFailed` kèm cả trang hỏng vĩnh viễn. */
/** Soát & duyệt: một dòng mỗi hồ sơ, số liệu gộp sẵn trên máy chủ (thay cho tải cả cây tài liệu). */
export interface ReviewSummaryRow {
  projectId: string;
  projectCode: string;
  projectName: string;
  phong: string;
  hop: string;
  hoSo: string;
  files: number;
  pages: number;
  unknownPages: number;
  sizeBytes: number;
  ocrPages: number;
  fieldDocs: number;
  pdfaDocs: number;
  pdfaPages: number;
  pdfaCu: number;
  ocrDenom: number;
  approved: number;
  rejected: number;
}
export function docsReviewSummary(args: { projectId?: string; q?: string; pendingOnly?: boolean }): Promise<ReviewSummaryRow[]> {
  return invoke<ReviewSummaryRow[]>("docs_review_summary", { args: { projectId: args.projectId ?? null, q: args.q ?? null, pendingOnly: args.pendingOnly ?? false } });
}
/** Tài liệu của các hồ sơ (phông, hộp, hồ sơ) đã chọn trong một dự án — nạp khi mở hoặc tích chọn hồ sơ. */
export function docsReviewGroupDocs(args: { projectId: string; groups: { phong: string; hop: string; hoSo: string }[] }): Promise<V4ej[]> {
  return invoke<V4ej[]>("docs_review_group_docs", { args });
}

/** Tài liệu máy chủ từ chối ghép PDF/A (bản quét dưới 200 dpi, OCR không ra chữ…) kèm lý do. */
export interface PdfaRejectedRow {
  id: string;
  projectCode: string;
  relPath: string | null;
  filename: string;
  pageCount: number | null;
  sizeBytes: number;
  uploadedAt: string;
  reason: string;
  /** "rejected" = chưa có PDF/A; "exception" = đã ghép theo ngoại lệ dự án, vẫn cần quét lại. */
  outcome: "rejected" | "exception";
}
export function docsPdfaRejected(): Promise<PdfaRejectedRow[]> {
  return invoke<PdfaRejectedRow[]>("docs_pdfa_rejected");
}

export function docsOcrRetryFailed(
  projectId?: string,
  documentIds?: string[],
  includePageFailed = false,
): Promise<OcrRetryFailedResult> {
  return invoke<OcrRetryFailedResult>("docs_ocr_retry_failed", {
    args: { projectId, documentIds, includePageFailed },
  });
}

export interface V4ew {
  documents: number;

  skippedNoPageCount: number;

  skippedPageFailed: number;
}

export function V4ex(V17u?: string, V17v = false, V17w?: string[]): Promise<V4ew> {
  return invoke<V4ew>("docs_extract_fields_all", {
    args: { projectId: V17u, retryFailed: V17v, documentIds: V17w },
  });
}

export interface V4ey {
  docs: number;
  pagesTotal: number;
  pagesOcrDone: number;
  docsOcrPending: number;
  docsOcrDone: number;
  docsPdf2Done: number;

  /** OCR xong nhưng ghép PDF 2 lớp bị từ chối vĩnh viễn — tổng và tách theo lý do. */
  docsPdf2Rejected?: number;
  docsPdf2RejectedDpi?: number;
  docsPdf2RejectedNoText?: number;
  docsPdf2RejectedOther?: number;

  docsFieldsDone?: number;

  docsFieldsActive?: number;

  docsFieldsFailed?: number;

  pagesOcrFailed?: number;

  pagesOcrTransientFailed?: number;

  pagesOcrQueued?: number;

  pagesOcrRunning?: number;

  docsOcrFailed?: number;

  pagesInkMeasured?: number;

  pagesInkLow?: number;
  byStatus: Record<string, number>;
}

export function V4ez(): Promise<V4ey> {
  return invoke<V4ey>("docs_progress");
}

export interface V4f0 {
  docs: number;

  pages: number;

  ocrDone: number;

  ocrPages?: number;

  fieldDocs?: number;

  pdfaDocs?: number;

  pdfaOld?: number;

  approved: number;

  rejected: number;

  pending: number;
}

export interface V4f1 extends V4f0 {
  projectId: string;
  code: string;
  name: string;
  isActive: boolean;
}

export interface V4f2 {
  projectsCount: number;

  from: string | null;
  to: string | null;
  totals: V4f0;
  projects: V4f1[];
}

export function V4f3(from?: string, to?: string): Promise<V4f2> {
  return invoke<V4f2>("docs_report", { args: { from, to } });
}

export interface V4f4 {
  id: string;
  reviewStatus: V4eh;
  reviewedAt: string | null;
  reviewNote: string | null;
}

export function V4f5(V17x: string, V17y: V4eh, V17z?: string): Promise<V4f4> {
  return invoke<V4f4>("docs_review", { args: { id: V17x, decision: V17y, note: V17z } });
}

export interface V4f6 {
  updated: number;

  skipped: number;
  reviewStatus: V4eh;
}

export const V4f7 = 500;

export function V4f8(V180: string[], V181: V4eh, V182?: string): Promise<V4f6> {
  return invoke<V4f6>("docs_review_bulk", {
    args: { ids: V180, decision: V181, note: V182 },
  });
}

export interface V4f9 {
  id: string;
  filename: string;
  hoSoRef: string | null;
  status: string;

  state: "ok" | "invalid" | "missing" | "pending" | "rejected";
  pages: number;
  expectedPages: number | null;
  hasTextLayer: boolean;
  hasImage: boolean;

  pagesWithText: number;

  pagesWithoutText: number;

  pdfa2b: boolean;
  reason: string | null;
}

export interface V4fa {
  checked: number;
  enqueued: number;

  skippedSigned: number;
}

export function V4fb(V183?: string): Promise<V4fa> {
  return invoke<V4fa>("docs_rebuild_non_pdfa", { args: { projectId: V183 ?? null } });
}

export function V4fc(): Promise<V4f9[]> {
  return invoke<V4f9[]>("docs_pdfa_check");
}
