// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export interface V4us {
  id: string;
  projectId: string;
  requester: string | null;
  hinhThuc: "danh_muc" | "trich_xuat" | "tong_hop";
  chuDe: string;
  mucDich: string;
  phuongThuc: string | null;
  status: "submitted" | "approved" | "refused" | "fulfilled";
  reviewNote: string | null;
  phi: number | null;
  paidAt: string | null;
  coKetQua: boolean;
  bienTap: string | null;
  approvedAt: string | null;
  fulfilledAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export function V4ut(): Promise<V4us[]> {
  return invoke<V4us[]>("info_list");
}

export interface V4uu {
  projectId?: string | null;
  hinhThuc: V4us["hinhThuc"];
  chuDe: string;
  mucDich: string;
  phuongThuc?: string | null;
}

export function V4uv(body: V4uu): Promise<V4us> {
  return invoke<V4us>("info_create", { args: { body } });
}

export type V4uw = "approve" | "refuse" | "pay" | "generate" | "fulfill";

export function V4ux(
  V306: string,
  V307: V4uw,
  V308?: { note?: string; bienTap?: string },
): Promise<V4us> {
  return invoke<V4us>("info_action", {
    args: { id: V306, action: V307, note: V308?.note ?? null, bienTap: V308?.bienTap ?? null },
  });
}

export function V4uy(V309: string, V30a: string): Promise<string> {
  return invoke<string>("info_download", { args: { id: V309, fileName: V30a } });
}

export interface V4uz {
  documentId: string;
  filename: string;
  hoSoRef: string | null;
  chiTiet: string | null;
  checkedAt: string;
}

export interface V4v0 {
  tongTaiLieu: number;
  daKiem3Nam: number;
  soLoi: number;
  tepLoi: V4uz[];
}

export function V4v1(): Promise<V4v0> {
  return invoke<V4v0>("ops_integrity_report");
}

export function V4v2(): Promise<{ daKiem: number; loi: number }> {
  return invoke("ops_integrity_run");
}

export interface V4v3 {
  id: string;
  loai: "gia_tang" | "day_du" | "day_du_3_nam";
  nguon: string;
  phuongTien: string | null;
  ketQua: "dat" | "khong_dat";
  chiTiet: string | null;
  createdAt: string;
}

export function V4v4(): Promise<V4v3[]> {
  return invoke<V4v3[]>("ops_backups");
}

export function V4v5(V30b: {
  loai: V4v3["loai"];
  nguon: string;
  phuongTien?: string | null;
  ketQua: V4v3["ketQua"];
  chiTiet?: string | null;
}): Promise<V4v3> {
  return invoke<V4v3>("ops_backup_report", { args: { ...V30b } });
}
