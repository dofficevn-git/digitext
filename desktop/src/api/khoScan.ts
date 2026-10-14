// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

/** Một phông trong Kho máy này (thư mục cấp 1 dưới thư mục dự án). `phong` rỗng = tệp ngay gốc dự án. */
export interface KhoPhong {
  projectId: string | null;
  projectDir: string;
  phong: string;
  path: string;
  total: number;
  /** Có trong thư mục Kho mà chưa ghi sổ (vd. chép tay bằng Explorer). */
  chuaGhiSo: number;
  chuaGhiSoBytes: number;
  /** Đã ghi sổ, chưa lên máy chủ. */
  chuaDay: number;
  loi: number;
  daDay: number;
}

/** Dò Kho theo dự án → phông, đối chiếu sổ Kho. */
export function khoScan(projectId?: string): Promise<KhoPhong[]> {
  return invoke<KhoPhong[]>("kho_scan", { args: projectId ? { projectId } : null });
}

/** Id các tệp đã ghi sổ, chưa lên máy chủ, nằm dưới thư mục phông `path`. */
export function khoPendingIds(path: string): Promise<number[]> {
  return invoke<number[]>("kho_pending_ids", { args: { path } });
}
