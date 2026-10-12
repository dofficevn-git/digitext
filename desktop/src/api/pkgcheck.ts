// Copyright (c) 2026 DigiText. All rights reserved.
// Kiểm tra gói tin SIP/AIP/DIP của phần mềm khác (TT05 + E-ARK CSIP) và xuất báo cáo PDF — src-tauri/src/pkgcheck.rs.
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke, isTauri } from "@/api/ipc";
import { V4dz } from "@/api/exportFile";

export type PkgLevel = "ok" | "warn" | "err";

export interface PkgFinding {
  level: PkgLevel;
  group: string;
  message: string;
}

export interface PkgPdfRow {
  file: string;
  bytes: number;
  pdfVersion: string;
  pages: number;
  textPages: number;
  pdfa: string | null;
  outputIntent: boolean;
  minDpi: number | null;
  bitDepth: number | null;
  producer: string | null;
  signed: boolean;
  sigValid: boolean | null;
  signer: string | null;
  issuer: string | null;
  certUntil: string | null;
  signingTime: string | null;
  tsa: boolean;
  ltv: boolean;
  problems: string[];
  verdict: string;
}

export interface PkgResult {
  source: string;
  name: string;
  root: string;
  sha256: string | null;
  size: number;
  kind: string;
  objid: string;
  profile: string;
  creatorOrg: string;
  creatorSoftware: string;
  created: string;
  fileCode: string;
  title: string;
  totalDoc: number | null;
  pages: number | null;
  dataFiles: number;
  findings: PkgFinding[];
  pdfs: PkgPdfRow[];
  errors: number;
  warnings: number;
  verdict: string;
}

export interface PkgReportHeader {
  parentOrg: string;
  org: string;
  place: string;
  inspector: string;
  inspectorTitle: string;
  approver: string;
  approverTitle: string;
}

export interface PkgProgress {
  done: number;
  total: number;
  name: string;
}

export function pkgcheckRun(paths: string[]): Promise<PkgResult[]> {
  return invoke<PkgResult[]>("pkgcheck_run", { args: { paths } });
}
export function pkgcheckCancel(): Promise<void> {
  return invoke<void>("pkgcheck_cancel");
}
/** Dựng báo cáo PDF rồi mở hộp thoại lưu; trả đường dẫn đã lưu (null = người dùng huỷ). */
export async function pkgcheckReport(results: PkgResult[], header: PkgReportHeader, defaultName: string): Promise<string | null> {
  const bytes = await invoke<number[]>("pkgcheck_report", { args: { results, header } });
  return V4dz(Uint8Array.from(bytes), defaultName, "pdf");
}
export async function onPkgcheckProgress(cb: (p: PkgProgress) => void): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  return listen<PkgProgress>("pkgcheck://progress", (e) => cb(e.payload));
}
