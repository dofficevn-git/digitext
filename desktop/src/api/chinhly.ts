// Copyright (c) 2026 DigiText. All rights reserved.
// Chỉnh lý (kế hoạch 29/9/2026): bản sao SQLite trên máy (offline được) + proxy máy chủ cho thao tác online.
// Dòng dữ liệu là JSON snake_case đúng tên cột cl_* của máy chủ.

import { invoke } from "@/api/ipc";

/** Một dòng cl_* (snake_case như máy chủ) + các trường làm giàu. */
export type ClRow = Record<string, unknown> & { id: string; _sync?: { ver: number; dirty: number; deletedAt: string | null } };

export type ClTable =
  | "khoi_phong"
  | "phong"
  | "phong_ban"
  | "mang"
  | "thbq"
  | "ngon_ngu"
  | "loai_van_ban"
  | "hop"
  | "bo_tap"
  | "kho"
  | "gia"
  | "ho_so"
  | "van_ban"
  | "van_ban_file"
  | "muon_tra";

export interface ClCatalog {
  khoi_phong: ClRow[];
  phong: ClRow[];
  phong_ban: ClRow[];
  mang: ClRow[];
  thbq: ClRow[];
  ngon_ngu: ClRow[];
  loai_van_ban: ClRow[];
  kho: ClRow[];
  gia: ClRow[];
  hop: ClRow[];
  bo_tap: ClRow[];
}

export interface ClListFilter {
  eq?: [string, unknown][];
  q?: string;
  qCols?: string[];
  includeDeleted?: boolean;
  order?: string;
  limit?: number;
  offset?: number;
}

export interface ClHoSoFilter {
  phongId?: string;
  phongBanId?: string;
  mangId?: string;
  thbqIds?: string[];
  thbqCoThoiHan?: boolean;
  status?: "VV" | "GL" | "HET";
  trungThua?: boolean;
  q?: string;
  khoId?: string;
  giaId?: string;
  hasHop?: boolean;
  hasHoSoSo?: boolean;
  hopId?: string;
  onlyDirty?: boolean;
  includeDeleted?: boolean;
  sort?: "so_tam" | "ho_so_so" | "ngay_kt" | "mang" | "thbq" | "updated";
  desc?: boolean;
  limit?: number;
  offset?: number;
}

export interface ClLocalErr {
  field: string;
  message: string;
}

export interface ClSaveResult {
  ok: boolean;
  id: string;
  row: ClRow | null;
  errors: ClLocalErr[];
}

export interface ClSyncStatus {
  phase: "idle" | "pushing" | "pulling" | "offline" | "locked" | "error" | "no_project" | string;
  pending: number;
  failed: number;
  conflicts: number;
  lastSyncAt: string | null;
  lastError: string | null;
  pushed: number;
  pulled: number;
}

export interface ClConflict {
  id: number;
  tbl: string;
  rowId: string;
  base: ClRow | null;
  mine: ClRow;
  theirs: ClRow | null;
  fields: string[];
  message: string | null;
  createdAt: string;
}

export interface ClOutboxRow {
  id: number;
  opId: string;
  tbl: string;
  rowId: string;
  op: "upsert" | "delete";
  base: ClRow | null;
  new: ClRow;
  status: "pending" | "rejected" | string;
  attempts: number;
  lastError: string | null;
  createdAt: string;
  editedAt: string | null;
}

export const CL_SYNC_EVENT = "chinhly://sync";

// ---- đọc bản sao (offline được)
export const clCatalogList = () => invoke<ClCatalog>("cl_catalog_list");
export const clRows = (table: ClTable | "lich_su", filter?: ClListFilter) =>
  invoke<{ rows: ClRow[]; total: number }>("cl_rows", { table, filter: filter ?? null });
export const clRow = (table: ClTable, id: string) =>
  invoke<{ row: ClRow; ver: number; dirty: number; deletedAt: string | null; base: ClRow | null } | null>("cl_row", { table, id });
export const clHoSoList = (filter?: ClHoSoFilter) => invoke<{ rows: ClRow[]; total: number }>("cl_ho_so_list", { filter: filter ?? null });
export const clVanBanList = (hoSoId: string) => invoke<ClRow[]>("cl_van_ban_list", { hoSoId });
export const clNextStt = (hoSoId: string) => invoke<number>("cl_next_stt", { hoSoId });

// ---- ghi bản sao (offline được; tự đẩy lên máy chủ)
export const clSave = (table: ClTable, id: string | null, patch: Record<string, unknown>) =>
  invoke<ClSaveResult>("cl_save", { table, id, patch });
export const clDelete = (table: ClTable, id: string) => invoke<number>("cl_delete", { table, id });

// ---- đồng bộ
export const clSyncNow = () => invoke<ClSyncStatus>("cl_sync_now");
export const clSyncStatus = () => invoke<ClSyncStatus>("cl_sync_status");
export const clConflicts = () => invoke<ClConflict[]>("cl_conflicts");
export const clConflictResolve = (id: number, choice: "mine" | "theirs" | "merged", merged?: Record<string, unknown>) =>
  invoke<void>("cl_conflict_resolve", { id, choice, merged: merged ?? null });
export const clOutbox = () => invoke<ClOutboxRow[]>("cl_outbox");
export const clOutboxRetry = (id: number) => invoke<void>("cl_outbox_retry", { id });
export const clOutboxDiscard = (id: number) => invoke<void>("cl_outbox_discard", { id });

// ---- chuyển dữ liệu cũ trên máy lên bản sao / máy chủ
export interface ClMigrateReport {
  phong: number;
  thbq: number;
  phongBan: number;
  mang: number;
  hoSo: number;
  vanBan: number;
  kho: number;
  gia: number;
  errors: string[];
}
export const clMigratePreview = () => invoke<ClMigrateReport>("cl_migrate_preview");
export const clMigrateRun = () => invoke<ClMigrateReport>("cl_migrate_run");

// ---- máy chủ (chỉ online)
export const clRemote = <T = unknown>(method: "GET" | "POST" | "PUT" | "PATCH" | "DELETE", path: string, body?: unknown) =>
  invoke<T>("cl_remote", { method, path, body: body ?? null });

export function qs(params: Record<string, unknown>): string {
  const p = new URLSearchParams();
  for (const [k, v] of Object.entries(params)) {
    if (v === undefined || v === null || v === "") continue;
    p.set(k, Array.isArray(v) ? v.join(",") : String(v));
  }
  const s = p.toString();
  return s ? `?${s}` : "";
}

/** Ghi qua máy chủ (đường online, có `ifVer` chống ghi đè). */
export const clRemoteSave = (table: ClTable, projectId: string, id: string | null, newRow: Record<string, unknown>, ifVer?: number) =>
  id
    ? clRemote<ClRow>("PUT", `/cl/rows/${table}/${id}`, { projectId, ifVer, new: newRow })
    : clRemote<ClRow>("POST", `/cl/rows/${table}`, { projectId, new: newRow });
export const clRemoteDelete = (table: ClTable, projectId: string, id: string, ifVer?: number) =>
  clRemote<ClRow>("DELETE", `/cl/rows/${table}/${id}${qs({ projectId, ifVer })}`);
export const clRemoteRestore = (table: ClTable, projectId: string, id: string) =>
  clRemote<ClRow>("POST", `/cl/rows/${table}/${id}/restore${qs({ projectId })}`);

// ---- nhãn
export const THBQ_STATUS_LABEL: Record<string, string> = { VV: "Vĩnh viễn", GL: "Giữ lại", HET: "Hết THBQ" };
export const TIEP_CAN: [string, string][] = [
  ["01", "Công khai"],
  ["02", "Sử dụng có điều kiện"],
  ["03", "Mật"],
];
export const TINH_TRANG_VAT_LY: [string, string][] = [
  ["01", "Tốt"],
  ["02", "Bình thường"],
  ["03", "Hỏng"],
];
export const TIN_CAY: [string, string][] = [
  ["01", "Gốc điện tử"],
  ["02", "Số hoá"],
  ["03", "Hỗn hợp"],
];

export const str = (r: ClRow | null | undefined, k: string): string => {
  const v = r?.[k];
  return v === null || v === undefined ? "" : String(v);
};
export const num = (r: ClRow | null | undefined, k: string): number | null => {
  const v = r?.[k];
  if (v === null || v === undefined || v === "") return null;
  const n = Number(v);
  return Number.isFinite(n) ? n : null;
};
