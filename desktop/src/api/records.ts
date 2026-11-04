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
export function V4sz(filter?: { kinds?: string[]; ref?: string; projectId?: string | null; limit?: number; offset?: number }): Promise<V4sy[]> {
  return invoke<V4sy[]>("records_list", { args: filter ? { kinds: filter.kinds ?? [], ref: filter.ref ?? null, projectId: filter.projectId ?? null, limit: filter.limit ?? null, offset: filter.offset ?? null } : null });
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

/** Mọi biên bản khớp bộ lọc — máy chủ trả tối đa 200 dòng một lần nên lật trang đến hết. */
export async function recordsAll(filter: { kinds?: string[]; ref?: string; projectId?: string | null }): Promise<V4sy[]> {
  const out: V4sy[] = [];
  const LO = 200;
  for (let offset = 0; offset < 100_000; offset += LO) {
    const r = await V4sz({ ...filter, limit: LO, offset });
    out.push(...r);
    if (r.length < LO) break;
  }
  return out;
}

/** Tải biên bản: PDF đã lưu (mặc định) hoặc bản Word dựng lại từ nội dung (`"docx"`, máy chủ từ 4/11/2026). */
export function V4t1(V2rb: string, V2rc: string, dir?: string, format: "pdf" | "docx" = "pdf"): Promise<string> {
  return invoke<string>("record_download", { args: { id: V2rb, fileName: V2rc, dir: dir ?? null, format: format === "docx" ? "docx" : null } });
}

export type V4t2 = "thu_nop" | "giao_nhan" | "tra_lai";

export function V4t3(V2rd: string, V2re?: string, loai: V4t2 = "thu_nop"): Promise<V4sy> {
  return invoke<V4sy>("submission_report", {
    args: { id: V2rd, ghiChu: V2re ?? null, loai },
  });
}
