// Copyright (c) 2026 DigiText. All rights reserved.
// Nhập kho tài liệu TỪ BÊN NGOÀI (thư mục PDF + Excel metadata) → kiểm khớp → đóng gói TT05.
// Mặt TS của desktop/src-tauri/src/extimport.rs.

import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { invoke, isTauri } from "@/api/ipc";

export type ExtSeverity = "error" | "warning";

export interface ExtIssue {
  severity: ExtSeverity;
  code: string;
  dossier: string;
  hsCode: string;
  line: number | null;
  file: string | null;
  message: string;
  count: number;
}

export interface ExtCodeSummary {
  severity: ExtSeverity;
  code: string;
  count: number;
  dossiers: number;
  example: string;
}

export interface ExtCheckSummary {
  root: string;
  sheet: string;
  deep: boolean;
  rows: number;
  filesOnDisk: number;
  dossiers: number;
  dossiersOk: number;
  bytes: number;
  errors: number;
  warnings: number;
  pdfa: number | null;
  signed: number | null;
  pages: number | null;
  byCode: ExtCodeSummary[];
  issues: ExtIssue[];
  issuesTotal: number;
  columnMapping: [string, string][];
  reportDir: string;
  reportFiles: string[];
}

export interface ExtPackResult {
  dir: string;
  hsCode: string;
  status: string;
  zipName: string | null;
  docs: number;
  error: string | null;
}

export interface ExtPackSummary {
  check: ExtCheckSummary;
  started: boolean;
  built: number;
  existing: number;
  held: number;
  failed: number;
  bytes: number;
  failures: ExtPackResult[];
  outDir: string;
  resultFile: string | null;
}

export interface ExtProgress {
  phase: "excel" | "check" | "precheck" | "pack";
  done: number;
  total: number;
  note: string | null;
}

export function extimportCheck(args: { root: string; excel: string; deep: boolean }): Promise<ExtCheckSummary> {
  return invoke<ExtCheckSummary>("extimport_check", { args });
}

export function extimportPack(args: {
  root: string;
  excel: string;
  outDir: string;
  kind: "aip_hoso" | "sip_hoso";
  agreement: string | null;
  orgCode: string | null;
  allowPartial: boolean;
  force: boolean;
}): Promise<ExtPackSummary> {
  return invoke<ExtPackSummary>("extimport_pack", { args });
}

export async function onExtimportProgress(cb: (p: ExtProgress) => void): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  return listen<ExtProgress>("extimport://progress", (e) => cb(e.payload));
}
