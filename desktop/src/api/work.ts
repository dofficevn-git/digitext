// Copyright (c) 2026 DigiText. All rights reserved.
// Gói việc & cộng tác viên xác nhận trường (kế hoạch 29/9/2026).

import { invoke } from "@/api/ipc";

export type WorkKind = "xac_nhan" | "kiem_tra";
export type WorkMode = "hoso" | "ngau_nhien";
export type PackageStatus = "open" | "under_review" | "done" | "closed";
export type ItemStatus =
  | "pending"
  | "done"
  | "redone"
  | "checked_ok"
  | "checked_err"
  | "approved"
  | "rejected"
  | "cancelled";

export interface WorkMember {
  userId: string;
  username: string;
  displayName: string;
}

export interface WorkPackage {
  id: string;
  projectId: string;
  name: string;
  kind: WorkKind;
  mode: WorkMode;
  status: PackageStatus;
  dueAt: string | null;
  note: string | null;
  sourcePackageId: string | null;
  samplePct: number | null;
  createdAt: string;
  nTotal: number;
  nPending: number;
  nDone: number;
  nChecked: number;
  nRejected: number;
  nApproved: number;
  members: WorkMember[];
}

export interface WorkItem {
  id: string;
  packageId: string;
  documentId: string;
  ord: number;
  status: ItemStatus;
  lockedBy: string | null;
  leaseUntil: string | null;
  doneBy: string | null;
  doneByName: string | null;
  doneAt: string | null;
  checkedByName: string | null;
  checkNote: string | null;
  reviewNote: string | null;
  filename: string;
  relPath: string | null;
  hoSoRef: string | null;
  pageCount: number | null;
}

export interface WorkPackageDetail extends WorkPackage {
  items: WorkItem[];
}

export interface CreatePackageReq {
  projectId: string;
  name?: string;
  kind: WorkKind;
  mode?: WorkMode;
  prefixes?: string[];
  size?: number;
  count?: number;
  memberIds?: string[];
  dueAt?: string | null;
  note?: string | null;
  sourcePackageId?: string | null;
  samplePct?: number | null;
  dryRun?: boolean;
}

export interface CreatePackageResp {
  available: number;
  sizes: number[];
  packageIds: string[];
}

export const KIND_LABEL: Record<WorkKind, string> = { xac_nhan: "Xác nhận trường", kiem_tra: "Kiểm tra chéo" };
export const MODE_LABEL: Record<WorkMode, string> = { hoso: "Theo hồ sơ liền mạch", ngau_nhien: "Ngẫu nhiên" };
export const PKG_STATUS_LABEL: Record<PackageStatus, string> = {
  open: "Đang làm",
  under_review: "Chờ duyệt",
  done: "Hoàn thành",
  closed: "Đã đóng",
};
export const ITEM_STATUS_LABEL: Record<ItemStatus, string> = {
  pending: "Chờ làm",
  done: "Đã xác nhận",
  redone: "Đã làm lại",
  checked_ok: "Kiểm tra: đạt",
  checked_err: "Kiểm tra: có lỗi",
  approved: "Đã duyệt",
  rejected: "Bị trả lại",
  cancelled: "Đã huỷ",
};

export const workListPackages = (projectId?: string) =>
  invoke<WorkPackage[]>("work_list_packages", { projectId: projectId ?? null });
export const workCreatePackage = (body: CreatePackageReq) => invoke<CreatePackageResp>("work_create_package", { body });
export const workGetPackage = (id: string) => invoke<WorkPackageDetail>("work_get_package", { id });
export const workPatchPackage = (id: string, body: { name?: string; dueAt?: string | null; note?: string }) =>
  invoke<WorkPackage>("work_patch_package", { id, body });
export const workSetMembers = (id: string, userIds: string[]) => invoke<WorkPackage>("work_set_members", { id, userIds });
export const workReview = (id: string, body: { itemIds?: string[]; decision: "approve" | "reject"; note?: string }) =>
  invoke<{ changed: number; skipped: number }>("work_review", { id, body });
export const workClosePackage = (id: string) => invoke<WorkPackage>("work_close_package", { id });
export const workMyPackages = () => invoke<WorkPackage[]>("work_my_packages");
export const workRejectedItems = () => invoke<WorkItem[]>("work_rejected_items");
export const workNextItem = (packageId: string) => invoke<WorkItem | null>("work_next_item", { packageId });
export const workClaimItem = (itemId: string) => invoke<WorkItem | null>("work_claim_item", { itemId });
export const workNavigate =(itemId: string, direction: "prev" | "next") =>
  invoke<WorkItem | null>("work_navigate", { itemId, direction });
export const workRelease = (itemId: string) => invoke<unknown>("work_release", { itemId });
export const workDone = (itemId: string) => invoke<WorkItem>("work_done", { itemId });
export const workCheck = (itemId: string, ok: boolean, note?: string) =>
  invoke<WorkItem>("work_check", { itemId, ok, note: note ?? null });
