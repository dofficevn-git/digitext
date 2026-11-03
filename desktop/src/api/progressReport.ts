// Copyright (c) 2026 DigiText. All rights reserved.
// Báo cáo tiến độ dự án + báo cáo nhân viên — số liệu lấy từ máy chủ (đề nghị chỉnh sửa mục 10, 11).

import { invoke } from "@/api/ipc";

export interface ProjectContract {
  signedOn: string | null;
  endOn: string | null;
  deadlineOn: string | null;
  contractPages: number | null;
  contractFiles: number | null;
  /** Khối lượng theo TT05 (0024): hộp, hồ sơ, tờ. */
  contractBoxes: number | null;
  contractDossiers: number | null;
  contractSheets: number | null;
  managerName: string | null;
  note: string | null;
}

export interface ProjectProgress {
  projectId: string;
  code: string;
  name: string;
  contract: ProjectContract;
  files: number;
  pages: number;
  pdfaFiles: number;
  signedFiles: number;
  ocrFiles: number;
  approvedFiles: number;
  rejectedFiles: number;
}

export interface RangeArgs {
  projectId?: string;
  from?: string;
  to?: string;
}

export function reportProjectProgress(args: RangeArgs): Promise<ProjectProgress[]> {
  return invoke<ProjectProgress[]>("report_project_progress", { args });
}

export function reportSetContract(projectId: string, contract: ProjectContract): Promise<ProjectContract> {
  return invoke<ProjectContract>("report_set_contract", { projectId, contract });
}

export type StaffTask =
  | "scan"
  | "kiem_tra_scan"
  | "nhap_lieu"
  | "kiem_tra_nhap_lieu"
  | "xac_nhan"
  | "kiem_tra_xac_nhan";

export interface StaffRow {
  userId: string;
  username: string;
  displayName: string;
  task: StaffTask;
  done: number;
  files: number;
  fields: number;
  days: number;
  errors: number;
  target: number | null;
}

export function reportStaff(args: RangeArgs): Promise<StaffRow[]> {
  return invoke<StaffRow[]>("report_staff", { args });
}

export function reportSetTarget(target: {
  projectId: string;
  userId: string;
  task: StaffTask;
  quantity: number | null;
}): Promise<unknown> {
  return invoke<unknown>("report_set_target", { target });
}
