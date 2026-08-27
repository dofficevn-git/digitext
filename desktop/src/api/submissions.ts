// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";
import type { V4rt } from "@/api/packaging";

export interface V4rx {
  id: string;
  projectId: string;
  maYeuCau: string;
  orgName: string;
  orgCode: string;
  soLanNop: number;
  fondCode: string;
  cachThuc: string;
  status: string;
  decisionNote: string | null;
  reconcileNote: string | null;
  approvalDoc: string | null;

  hanNop: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface V4ry {
  id: string;
  stt: number;
  maHoSo: string;
  tieuDe: string;
  thoiGianBatDau: string | null;
  thoiGianKetThuc: string | null;
  soTrang: number | null;
  mucDoTiepCan: string | null;
  ghiChu: string | null;
  reviewResult: "pending" | "dat" | "khong_dat";
  reviewNote: string | null;
}

export interface V4rz extends V4rx {
  summary: string | null;
  receptionNote: string | null;
  approvalNote: string | null;
  items: V4ry[];
  packages: V4rt[];
  duplicateInSystem: string[];
}

export interface V4s0 {
  maHoSo: string;
  tieuDe: string;
  thoiGianBatDau?: string | null;
  thoiGianKetThuc?: string | null;
  soTrang?: number | null;
  mucDoTiepCan?: string | null;
  ghiChu?: string | null;
}

export interface V4s1 {
  projectId?: string | null;
  orgName: string;
  orgCode: string;
  daNopGiay?: boolean;
  summary?: string | null;
  fondCode: string;
  cachThuc?: "truc_tiep" | "truc_tuyen";
  totalPages?: number | null;

  hanNop?: string | null;
  items: V4s0[];
}

export function V4s2(): Promise<V4rx[]> {
  return invoke<V4rx[]>("submissions_list");
}

export function V4s3(V2nw: string): Promise<V4rz> {
  return invoke<V4rz>("submission_get", { args: { id: V2nw } });
}

export function V4s4(body: V4s1): Promise<V4rz> {
  return invoke<V4rz>("submission_register", { args: { body } });
}

export type V4s5 = "confirm" | "refuse" | "reconcile" | "receive" | "complete";

export function V4s6(V2nx: string, V2ny: V4s5, V2nz?: string): Promise<V4rx> {
  return invoke<V4rx>("submission_action", {
    args: { id: V2nx, action: V2ny, note: V2nz ?? null },
  });
}

export function V4s7(
  V2o0: string,
  V2o1: string,
  V2o2: "dat" | "khong_dat",
  V2o3?: string,
): Promise<void> {
  return invoke<void>("submission_review_item", {
    args: { id: V2o0, itemId: V2o1, result: V2o2, note: V2o3 ?? null },
  });
}

export interface V4s8 {
  submission: V4rx;
  convertedAipIds: string[];
}

export function V4s9(V2o4: string, V2o5: string, V2o6?: string): Promise<V4s8> {
  return invoke<V4s8>("submission_approve", {
    args: { id: V2o4, quyetDinhSo: V2o5, note: V2o6 ?? null },
  });
}

export interface V4sa {
  id: string;
  submissionId: string | null;
  kind: string;
  message: string;
  readAt: string | null;
  createdAt: string;
}

export interface V4sb {
  unread: number;
  items: V4sa[];
}

export function V4sc(): Promise<V4sb> {
  return invoke<V4sb>("notifications_list");
}

export function V4sd(V2o7: string): Promise<void> {
  return invoke<void>("notification_read", { args: { id: V2o7 } });
}

export function V4se(): Promise<void> {
  return invoke<void>("notifications_read_all");
}

export interface V4sf extends V4ry {
  sipPackageId: string | null;
  sipZipName: string | null;
}

export interface V4sg {
  submission: V4rx;
  totalRegistered: number;
  totalApproved: number;
  totalReturned: number;
  returnedItems: V4sf[];
  sipPackages: V4rt[];
}

export function V4sh(V2o8: string): Promise<V4sg> {
  return invoke<V4sg>("submission_returned", { args: { id: V2o8 } });
}

export function V4si(V2o9: string, V2oa: string): Promise<string> {
  return invoke<string>("submission_dossier", { args: { id: V2o9, fileName: V2oa } });
}
