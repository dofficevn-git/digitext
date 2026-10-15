// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export interface PurgeCounts {
  documents: number;
  pages: number;
  sourceBytes: number;
  signed: number;
  packages: number;
  hoSo: number;
  vanBan: number;
  submissions: number;
  activeJobs: number;
  openSignSessions: number;
}

export interface PurgePreview {
  projectCode: string;
  projectName: string;
  counts: PurgeCounts;
  /** Lý do chưa cho xoá (rỗng = được xoá). */
  blockers: string[];
  /** Chưa có gói bàn giao nào. */
  undelivered: boolean;
}

export interface PurgeStatus {
  state: "running" | "done" | "failed";
  docsTotal: number;
  docsDone: number;
  objectsDeleted: number;
  error: string | null;
  startedAt: string | null;
  finishedAt: string | null;
}

/** Chạy thử: số liệu sẽ xoá + lý do chưa cho xoá. Không xoá gì. */
export function projectPurgePreview(projectId: string): Promise<PurgePreview> {
  return invoke<PurgePreview>("project_purge", { args: { projectId, dryRun: true } });
}

/** Bắt đầu xoá thật (chạy nền trên máy chủ). */
export function projectPurgeStart(args: { projectId: string; confirmCode: string; reason: string; allowUndelivered: boolean }): Promise<{ started: boolean }> {
  return invoke<{ started: boolean }>("project_purge", { args: { ...args, dryRun: false } });
}

export function projectPurgeStatus(projectId: string): Promise<PurgeStatus | null> {
  return invoke<PurgeStatus | null>("project_purge_status", { projectId });
}
