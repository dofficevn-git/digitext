// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4fe } from "@/api/ocr";

export const V4fm: Record<string, string> = {
  thuTu: "Số thứ tự VB",
  loaiVanBan: "Loại văn bản",
  soKyHieu: "KH văn bản",
  ngayThang: "Ngày tháng",
  coQuanBanHanh: "CQ ban hành",
  trichYeu: "Trích yếu",
  toSo: "Tờ số",
  soTrang: "Số trang",
  trangSo: "Trang số",
  ngonNgu: "Ngôn ngữ",
  cheDoSuDung: "CĐ sử dụng",
  doTinCay: "Độ tin cậy",
  butTich: "Bút tích",
  tinhTrangVatLy: "TT vật lý",
  taiLieuSo: "Tài liệu số",
  ghiChu: "Ghi chú",
};

export const V4fn: string[] = Object.keys(V4fm);

export function V4fo(V18b: string): string {
  return V4fm[V18b] ?? V18b;
}

const V4fp: string[] = [
  "#e11d48",
  "#db2777",
  "#9333ea",
  "#7c3aed",
  "#4f46e5",
  "#2563eb",
  "#0284c7",
  "#0891b2",
  "#059669",
  "#16a34a",
  "#65a30d",
  "#ca8a04",
  "#d97706",
  "#ea580c",
  "#dc2626",
  "#78716c",
];

export function V4fq(V18c: string): string {
  const i = V4fn.indexOf(V18c);
  if (i >= 0) return V4fp[i % V4fp.length];
  let h = 0;
  for (let k = 0; k < V18c.length; k++) h = (h * 31 + V18c.charCodeAt(k)) >>> 0;
  return V4fp[h % V4fp.length];
}

function V4fr(V18d: string): number {
  const i = V4fn.indexOf(V18d);
  return i < 0 ? Number.MAX_SAFE_INTEGER : i;
}

export interface V4fs {
  pageNo: number;
  fieldKey: string;
  label: string;
  value: string;
  bbox: number[] | null;
  confidence: number | null;

  corrected: boolean;
  verified: boolean;

  engineVersionId: string;

  placeholder: boolean;
}

function V4ft(a: V4fe, b: V4fe): V4fe {
  const V18e = (r: V4fe) => (r.textSource === "ocr_corrected" ? 1 : 0);
  if (V18e(a) !== V18e(b)) return V18e(a) > V18e(b) ? a : b;
  return (b.updatedAt ?? "") > (a.updatedAt ?? "") ? b : a;
}

function V4fu(r: V4fe): V4fs {
  return {
    pageNo: r.pageNo,
    fieldKey: r.fieldKey,
    label: V4fo(r.fieldKey),
    value: r.value,
    bbox: r.bbox,
    confidence: r.confidence,
    corrected: r.textSource === "ocr_corrected",
    verified: r.verified,
    engineVersionId: r.engineVersionId,
    placeholder: false,
  };
}

export function V4fv(V18f: V4fe[]): V4fs[] {
  const V18g = new Map<string, V4fe>();
  for (const r of V18f) {
    const V18h = `${r.pageNo}|${r.fieldKey}`;
    const V18i = V18g.get(V18h);
    V18g.set(V18h, V18i ? V4ft(V18i, r) : r);
  }
  return [...V18g.values()]
    .map(V4fu)
    .sort((a, b) => a.pageNo - b.pageNo || V4fr(a.fieldKey) - V4fr(b.fieldKey));
}

export function V4fw(fields: V4fs[], V18j: number): V4fs[] {
  return fields.filter((f) => f.pageNo === V18j);
}

function V4fx(fields: V4fs[]): number {
  return fields.length === 0 ? 1 : fields.reduce((min, f) => Math.min(min, f.pageNo), Infinity);
}

export function V4fy(fields: V4fs[]): V4fs[] {
  const V18k = new Set(fields.map((f) => f.fieldKey));
  const V18l = V4fx(fields);
  const V18m = V4fn.filter((k) => !V18k.has(k)).map((V18n) => ({
    pageNo: V18l,
    fieldKey: V18n,
    label: V4fo(V18n),
    value: "",
    bbox: null,
    confidence: null,
    corrected: false,
    verified: false,
    engineVersionId: "",
    placeholder: true,
  }));
  return [...fields, ...V18m].sort(
    (a, b) => a.pageNo - b.pageNo || V4fr(a.fieldKey) - V4fr(b.fieldKey),
  );
}

export function V4fz(fields: V4fs[]): number {
  const V18o = new Set(
    fields.filter((f) => f.value.trim() !== "" && V4fn.includes(f.fieldKey)).map((f) => f.fieldKey),
  );
  return V18o.size;
}

const V4g0 = /\u200B|\u200C|\u200D|\uFEFF|\u00AD/gu;

export function V4g1(s: string): string {
  const V18p = s
    .normalize("NFC")
    .replace(V4g0, "")

    .replace(/\r\n?/g, "\n")
    .replace(/[\u2028\u2029]/g, "\n");
  const V18q = V18p.split("\n").map((l) => l.replace(/\s+/g, " ").trim());

  const V18r: string[] = [];
  for (const l of V18q) {
    if (l === "" && V18r[V18r.length - 1] === "") continue;
    V18r.push(l);
  }
  while (V18r.length > 0 && V18r[0] === "") V18r.shift();
  while (V18r.length > 0 && V18r[V18r.length - 1] === "") V18r.pop();
  return V18r.join("\n");
}

export const V4g2 = 2;

export function V4g3(a: number[] | null, b: number[] | null): boolean {
  if (a == null && b == null) return true;
  if (a == null || b == null) return false;
  if (a.length !== 4 || b.length !== 4) return false;
  return a.every((v, i) => Math.abs(v - b[i]) <= V4g2);
}

export interface V4g4 {
  drafts: Record<string, string>;

  bboxDrafts: Record<string, number[]>;
}

export interface V4g5 {
  pageNo: number;
  fieldKey: string;
  label: string;
  value: string;
  bbox: number[] | null;

  engineVersionId: string;
}

function V4g6(f: V4fs, V18s: V4g4): V4g5 | null {
  const V18t = `${f.pageNo}|${f.fieldKey}`;
  const V18u = V4g1(V18s.drafts[V18t] ?? f.value);
  const V18v = V18s.bboxDrafts[V18t] ?? f.bbox;
  if (V18u === f.value && V4g3(V18v, f.bbox)) return null;
  return {
    pageNo: f.pageNo,
    fieldKey: f.fieldKey,
    label: f.label,
    value: V18u,
    bbox: V18v,
    engineVersionId: f.engineVersionId,
  };
}

export function V4g7(f: V4fs, V18w: V4g4): boolean {
  return V4g6(f, V18w) != null;
}

export function V4g8(fields: V4fs[], V18x: V4g4): V4g5[] {
  return fields.map((f) => V4g6(f, V18x)).filter((s): s is V4g5 => s != null);
}

export const V4g9 = 34;
export const V4ga = 320;

export function V4gb(V18y: number, max: number = V4ga): { height: number; scroll: boolean } {
  const V18z = Number.isFinite(V18y) ? Math.max(V4g9, Math.ceil(V18y)) : V4g9;
  const V190 = Math.max(V4g9, max);
  return { height: Math.min(V18z, V190), scroll: V18z > V190 };
}

export function V4gc(
  V191: number[],
  V192: number,
  V193: number,
): { left: string; top: string; width: string; height: string } | null {
  if (V191.length < 4 || V192 <= 0 || V193 <= 0) return null;
  const [x, y, w, h] = V191;
  const V194 = (v: number, d: number) => `${Math.max(0, Math.min(100, (v / d) * 100))}%`;
  return { left: V194(x, V192), top: V194(y, V193), width: V194(w, V192), height: V194(h, V193) };
}
