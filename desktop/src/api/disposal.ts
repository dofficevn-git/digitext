// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export interface V4u8 {
  id: string;
  projectId: string;
  documentId: string;
  batchId: string | null;
  lyDo: "trung_lap" | "het_thoi_han";
  dupOf: string | null;
  ghiChu: string | null;
  status: "xem_xet" | "giu_lai" | "da_huy";
  filename: string;
  hoSoRef: string | null;
  pageCount: number | null;
  hetHanLuuTru: string | null;
  docStatus: string;
  createdAt: string;
}

export interface V4u9 {
  id: string;
  projectId: string;
  nam: number;
  status: "hoi_dong" | "tham_dinh" | "quyet_dinh" | "da_huy";
  hoiDongNote: string | null;
  thamDinhNote: string | null;
  tuQuyet: boolean;
  quyetDinhSo: string | null;
  decidedAt: string | null;
  executedAt: string | null;
  createdAt: string;
}

export interface V4ua {
  action: string;
  actor: string | null;
  detail: string | null;
  documentId: string | null;
  createdAt: string;
}

export interface V4ub extends V4u9 {
  entries: V4u8[];
  log: V4ua[];
}

export function V4uc(): Promise<V4u8[]> {
  return invoke<V4u8[]>("disposal_entries");
}

export function V4ud(): Promise<{ trungLap: number; hetThoiHan: number }> {
  return invoke("disposal_sweep");
}

export function V4ue(V2yb: string, V2yc: string | null): Promise<void> {
  return invoke<void>("disposal_set_retention", { args: { documentId: V2yb, hetHan: V2yc } });
}

export function V4uf(V2yd: string, V2ye: "note" | "keep", V2yf?: string): Promise<void> {
  return invoke<void>("disposal_entry_action", {
    args: { id: V2yd, action: V2ye, note: V2yf ?? null },
  });
}

export function V4ug(): Promise<V4u9[]> {
  return invoke<V4u9[]>("disposal_batches");
}

export function V4uh(V2yg: string): Promise<V4ub> {
  return invoke<V4ub>("disposal_batch_get", { args: { id: V2yg } });
}

export function V4ui(V2yh: number, V2yi: string[]): Promise<V4u9> {
  return invoke<V4u9>("disposal_batch_create", { args: { nam: V2yh, entryIds: V2yi } });
}

export type V4uj = "council" | "appraise" | "decide" | "execute";

export function V4uk(
  V2yj: string,
  V2yk: V4uj,
  V2yl?: { note?: string; quyetDinhSo?: string; tuQuyet?: boolean },
): Promise<{ disposed?: number } | null> {
  return invoke("disposal_batch_action", {
    args: {
      id: V2yj,
      action: V2yk,
      note: V2yl?.note ?? null,
      quyetDinhSo: V2yl?.quyetDinhSo ?? null,
      tuQuyet: V2yl?.tuQuyet ?? false,
    },
  });
}
