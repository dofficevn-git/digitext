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
  verapdf: boolean;
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
  pdfaFlavours: Record<string, number>;
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
  phase: "excel" | "check" | "precheck" | "pack" | "pdfa";
  done: number;
  total: number;
  note: string | null;
}

export function extimportCheck(args: { root: string; excel: string; deep: boolean; useVerapdf: boolean }): Promise<ExtCheckSummary> {
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
  useVerapdf: boolean;
}): Promise<ExtPackSummary> {
  return invoke<ExtPackSummary>("extimport_pack", { args });
}

export async function onExtimportProgress(cb: (p: ExtProgress) => void): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  return listen<ExtProgress>("extimport://progress", (e) => cb(e.payload));
}

// ---- Kiểm tra PDF/A cho tệp / thư mục bất kỳ ----

export interface PdfaVera {
  flavour: string;
  compliant: boolean;
  rules: string[];
}

export interface PdfaFileReport {
  path: string;
  name: string;
  bytes: number;
  pdfVersion: string;
  declared: string | null;
  hasXmp: boolean;
  hasOutputIntent: boolean;
  pages: number;
  textPages: number;
  minDpi: number | null;
  minBitDepth: number | null;
  signed: boolean;
  producer: string | null;
  verdict: string;
  looksPdfa: boolean;
  problems: string[];
  verapdf: PdfaVera | null;
  error: string | null;
}

export interface PdfaSummary {
  total: number;
  truncated: boolean;
  verapdfUsed: boolean;
  verapdfAvailable: boolean;
  byKind: Record<string, number>;
  ok: number;
  notOk: number;
  files: PdfaFileReport[];
  reportFile: string | null;
}

export function pdfaCheck(args: { paths: string[]; useVerapdf: boolean }): Promise<PdfaSummary> {
  return invoke<PdfaSummary>("pdfa_check", { args });
}

export function pdfaVerapdfAvailable(): Promise<boolean> {
  return invoke<boolean>("pdfa_verapdf_available");
}
