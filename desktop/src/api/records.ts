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
}

export function V4sz(): Promise<V4sy[]> {
  return invoke<V4sy[]>("records_list");
}

export function V4t0(V2ra: {
  projectId?: string | null;
  kind: string;
  payload: unknown;
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
