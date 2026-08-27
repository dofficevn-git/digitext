// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export interface V4tl {
  id: string;
  projectId: string;
  requester: string | null;
  kind: "doc" | "ban_sao" | "ban_sao_ky" | "dip";
  mucDich: string;
  phuongThuc: string | null;
  uuTien: string | null;
  nguoiNhan: string | null;
  status: "submitted" | "approved" | "refused" | "expired";
  reviewNote: string | null;
  phi: number | null;
  paidAt: string | null;
  hanSuDung: string | null;
  approvedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface V4tm {
  id: string;
  documentId: string;
  stt: number;
  verdict: "pending" | "duyet" | "tu_choi";
  lyDo: string | null;
  dieuKien: boolean;
  filename: string | null;
  hoSoRef: string | null;
  pageCount: number | null;
  deleted: boolean;
}

export interface V4tn extends V4tl {
  items: V4tm[];
}

export function V4to(): Promise<V4tl[]> {
  return invoke<V4tl[]>("access_list");
}

export function V4tp(V2vi: string): Promise<V4tn> {
  return invoke<V4tn>("access_get", { args: { id: V2vi } });
}

export interface V4tq {
  projectId?: string | null;
  kind: "doc" | "ban_sao" | "ban_sao_ky" | "dip";
  mucDich: string;
  phuongThuc?: string | null;
  uuTien?: string | null;
  nguoiNhan?: string | null;
  soDinhDanh?: string | null;
  documentIds: string[];
}

export function V4tr(body: V4tq): Promise<V4tn> {
  return invoke<V4tn>("access_create", { args: { body } });
}

export function V4ts(
  V2vj: string,
  V2vk: string,
  V2vl: "duyet" | "tu_choi",
  V2vm?: string,
  V2vn = false,
): Promise<void> {
  return invoke<void>("access_review_item", {
    args: { id: V2vj, itemId: V2vk, verdict: V2vl, lyDo: V2vm ?? null, dieuKien: V2vn },
  });
}

export type V4tt = "approve" | "refuse" | "pay" | "dip";

export function V4tu(
  V2vo: string,
  V2vp: V4tt,
  V2vq?: string,
): Promise<V4tn & { packageId?: string; maXacThuc?: string }> {
  return invoke("access_action", { args: { id: V2vo, action: V2vp, note: V2vq ?? null } });
}

export function V4tv(V2vr: string, V2vs: string): Promise<string> {
  return invoke<string>("access_read", { args: { id: V2vr, itemId: V2vs } });
}

export function V4tw(V2vt: string, V2vu: string, V2vv: string): Promise<string> {
  return invoke<string>("access_download", { args: { id: V2vt, itemId: V2vu, fileName: V2vv } });
}

export interface V4tx {
  projectId: string | null;
  phiDocTaiLieu: number;
  phiSaoTrang: number;
}

export function V4ty(): Promise<V4tx> {
  return invoke<V4tx>("access_fees_get");
}

export function V4tz(V2vw: number, V2vx: number): Promise<V4tx> {
  return invoke<V4tx>("access_fees_set", { args: { phiDocTaiLieu: V2vw, phiSaoTrang: V2vx } });
}
