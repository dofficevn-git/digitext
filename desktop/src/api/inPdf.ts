// Copyright (c) 2026 DigiText. All rights reserved.
// In biểu mẫu ra PDF tại máy trạm (27/11/2026, đợt 4 nhóm 3C): dựng mảng khối ManualBlock (cùng bộ dựng với báo cáo / phiếu máy chủ)
// rồi gọi Tauri pdf_manual → hộp thoại Lưu. Chạy được ngoại tuyến.

import { invoke } from "@/api/ipc";

/** Khối in — khớp `digitext_pdf::manual::ManualBlock` (serde tag "t", content "v"). */
export type KhoiIn =
  | { t: "title"; v: string }
  | { t: "subtitle"; v: string }
  | { t: "section_bar"; v: string }
  | { t: "key_value"; v: [string, string][] }
  | { t: "table"; v: { headers: string[]; widths: number[]; rows: string[][] } }
  | { t: "gap"; v: number }
  | { t: "footer"; v: string }
  | { t: "official_header"; v: { left: string[]; right: string[]; dated: string } }
  | { t: "paragraph"; v: string }
  | { t: "signatures"; v: { left: [string, string, string]; right: [string, string, string] } }
  | { t: "page_break" }
  | { t: "centered"; v: { text: string; size: number; bold: boolean } }
  | { t: "labels2x2"; v: NhanIn[] };

/** Một nhãn hộp (khớp `digitext_pdf::manual::Label`): dòng đầu (dòng 1 đậm), số hộp lớn, dòng cuối nhãn: giá trị. */
export interface NhanIn {
  top: string[];
  big: string;
  bottom: [string, string][];
}

export const kb = {
  title: (v: string): KhoiIn => ({ t: "title", v }),
  subtitle: (v: string): KhoiIn => ({ t: "subtitle", v }),
  bar: (v: string): KhoiIn => ({ t: "section_bar", v }),
  kv: (v: [string, string][]): KhoiIn => ({ t: "key_value", v }),
  table: (headers: string[], widths: number[], rows: string[][]): KhoiIn => ({ t: "table", v: { headers, widths, rows } }),
  gap: (v: number): KhoiIn => ({ t: "gap", v }),
  footer: (v: string): KhoiIn => ({ t: "footer", v }),
  header: (left: string[], right: string[], dated = ""): KhoiIn => ({ t: "official_header", v: { left, right, dated } }),
  para: (v: string): KhoiIn => ({ t: "paragraph", v }),
  sign: (left: [string, string, string], right: [string, string, string]): KhoiIn => ({ t: "signatures", v: { left, right } }),
  pageBreak: (): KhoiIn => ({ t: "page_break" }),
  center: (text: string, size = 12, bold = false): KhoiIn => ({ t: "centered", v: { text, size, bold } }),
  labels: (v: NhanIn[]): KhoiIn => ({ t: "labels2x2", v }),
};

/** Dựng và lưu PDF; trả đường dẫn đã ghi, null nếu người dùng huỷ. */
export const pdfManual = (blocks: KhoiIn[], defaultName: string, ngang = false, dir?: string) =>
  invoke<string | null>("pdf_manual", { args: { blocks, ngang, defaultName, dir: dir ?? null } });
