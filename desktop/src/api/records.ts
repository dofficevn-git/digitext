// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export interface V4sy {
  id: string;
  projectId: string;

  kind: string;
  soBienBan: string;
  title: string;
  submissionId: string | null;
  createdAt: string;
  /** Khoá phụ (0024): mã phông / ngày YYYY-MM-DD / "lần n". Máy chủ cũ không trả. */
  ref?: string | null;
}

/** Sổ biên bản; lọc theo loại / khoá phụ / dự án (máy chủ từ 0024, máy chủ cũ trả toàn bộ). */
export function V4sz(filter?: { kinds?: string[]; ref?: string; projectId?: string | null; limit?: number }): Promise<V4sy[]> {
  return invoke<V4sy[]>("records_list", { args: filter ? { kinds: filter.kinds ?? [], ref: filter.ref ?? null, projectId: filter.projectId ?? null, limit: filter.limit ?? null } : null });
}

export interface RecordDetail extends V4sy {
  payload: Record<string, unknown>;
}
export function recordGet(id: string): Promise<RecordDetail> {
  return invoke<RecordDetail>("record_get", { id });
}

export function V4t0(V2ra: {
  projectId?: string | null;
  kind: string;
  payload: unknown;
  ref?: string | null;
}): Promise<V4sy> {
  return invoke<V4sy>("record_create", { args: V2ra });
}

export function V4t1(V2rb: string, V2rc: string): Promise<string> {
  return invoke<string>("record_download", { args: { id: V2rb, fileName: V2rc } });
}

export type V4t2 = "thu_nop" | "giao_nhan" | "tra_lai";

export function V4t3(V2rd: string, V2re?: string, loai: V4t2 = "thu_nop"): Promise<V4sy> {
  return invoke<V4sy>("submission_report", {
    args: { id: V2rd, ghiChu: V2re ?? null, loai },
  });
}
