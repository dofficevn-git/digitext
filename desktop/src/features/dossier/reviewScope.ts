// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4eh, V4ej } from "@/api/docs";

export interface V4k6 {
  docs: number;
  pending: number;
  approved: number;
  rejected: number;
}

export interface V4k7 {
  files: number;

  pages: number;

  unknownPages: number;
  sizeBytes: number;

  ocrPages: number;

  fieldDocs: number;

  unknownOcr: number;

  pdfaDocs: number;

  pdfaPages: number;

  pdfaCu: number;

  ocrDenom: number;
}

export interface V4k8 {
  id: string;
  filename: string;
  pages: number | null;
  sizeBytes: number;
  status: string;

  ocrPages: number;

  fieldPages: number;

  pdfaPages: number;

  pdfa2b: boolean | null;

  ocrKnown: boolean;
  review: V4eh;
}

export type V4k9 = "unknown" | "none" | "fields" | "partial" | "done";

export function bocTachState(f: V4k8): V4k9 {
  if (!f.ocrKnown) return "unknown";
  if (f.ocrPages <= 0) return f.fieldPages > 0 ? "fields" : "none";
  if (f.pages != null && f.ocrPages < f.pages) return "partial";
  return "done";
}

export type V4ka = "none" | "pdfa2b" | "cu" | "chuaRo";

export function pdfaState(f: V4k8): V4ka {
  if (f.pdfaPages <= 0) return "none";
  if (f.pdfa2b == null) return "chuaRo";
  return f.pdfa2b ? "pdfa2b" : "cu";
}

export interface V4kb {
  key: string;
  /** Phông = tầng đầu của relPath (cây Dự án/Phông/Hộp/Hồ sơ). "" nếu không xác định. */
  phong: string;
  hopSo: string;
  hoSoSo: string;
  files: V4k8[];

  ids: string[];
  tally: V4k7;
  counts: V4k6;
}

export interface V4kc {
  key: string;
  label: string;
  rows: V4kb[];
  ids: string[];
  tally: V4k7;
  counts: V4k6;
}

export const V4kd = "(Chưa rõ dự án)";
export const V4ke = "(Chưa rõ hộp số)";
export const V4kf = "(Chưa gán hồ sơ)";

export type V4kg = "none" | "some" | "all";

export function V4kh(d: V4ej): V4eh {
  return d.reviewStatus === "approved" || d.reviewStatus === "rejected"
    ? d.reviewStatus
    : "pending";
}

/** Phông của tài liệu: tầng đầu relPath khi cây có đủ phông/hộp/hồ sơ (≥ 2 tầng). */
export function phongOf(doc: V4ej): string {
  const parts = (doc.relPath ?? "").split("/").filter((s) => s.trim() !== "");
  return parts.length >= 2 ? parts[0].trim() : "";
}

export function V4ki(doc: V4ej): { hop: string; hoSo: string } {
  const V1pe = (doc.hoSoRef ?? "").trim();
  if (V1pe !== "") {
    const i = V1pe.indexOf("/");
    if (i > 0) return { hop: V1pe.slice(0, i).trim(), hoSo: V1pe.slice(i + 1).trim() };

    return { hop: "", hoSo: V1pe };
  }
  const V1pf = (doc.relPath ?? "").split("/").filter((s) => s.trim() !== "");
  if (V1pf.length >= 3) return { hop: V1pf[1].trim(), hoSo: V1pf[2].trim() };
  return { hop: "", hoSo: "" };
}

const V4kj = (): V4k6 => ({ docs: 0, pending: 0, approved: 0, rejected: 0 });
const V4kk = (): V4k7 => ({
  files: 0,
  pages: 0,
  unknownPages: 0,
  sizeBytes: 0,
  ocrPages: 0,
  fieldDocs: 0,
  unknownOcr: 0,
  pdfaDocs: 0,
  pdfaPages: 0,
  pdfaCu: 0,
  ocrDenom: 0,
});

function V4kl(V1pg: V4k7, src: V4k7): void {
  V1pg.files += src.files;
  V1pg.pages += src.pages;
  V1pg.unknownPages += src.unknownPages;
  V1pg.sizeBytes += src.sizeBytes;
  V1pg.ocrPages += src.ocrPages;
  V1pg.fieldDocs += src.fieldDocs;
  V1pg.unknownOcr += src.unknownOcr;
  V1pg.pdfaDocs += src.pdfaDocs;
  V1pg.pdfaPages += src.pdfaPages;
  V1pg.pdfaCu += src.pdfaCu;
  V1pg.ocrDenom += src.ocrDenom;
}

function V4km(V1ph: V4k6, src: V4k6): void {
  V1ph.docs += src.docs;
  V1ph.pending += src.pending;
  V1ph.approved += src.approved;
  V1ph.rejected += src.rejected;
}

const V4kn = new Intl.Collator("vi", { numeric: true, sensitivity: "base" });
function V4ko(a: string, b: string): number {
  if (a === "" && b !== "") return 1;
  if (b === "" && a !== "") return -1;
  return V4kn.compare(a, b);
}

function V4kp(a: V4kb, b: V4kb): number {
  // Phông → hộp → hồ sơ, sắp theo số. "HOP 1" và "HOP 01" bằng nhau về số nên phải tách
  // bằng tên gốc — nếu không hồ sơ của hai hộp đó xếp xen kẽ ("hết hộp này mới sang hộp khác").
  return (
    V4ko(a.phong, b.phong) ||
    V4ko(a.hopSo, b.hopSo) ||
    (a.hopSo < b.hopSo ? -1 : a.hopSo > b.hopSo ? 1 : 0) ||
    V4ko(a.hoSoSo, b.hoSoSo) ||
    (a.hoSoSo < b.hoSoSo ? -1 : a.hoSoSo > b.hoSoSo ? 1 : 0)
  );
}

export function V4kq(docs: V4ej[]): V4kc[] {
  const V1pi = new Map<string, { label: string; rows: Map<string, V4kb> }>();

  for (const d of docs) {
    if (d.status === "disposed") continue;
    const V1pj = d.projectId ?? "";
    let p = V1pi.get(V1pj);
    if (!p) {
      p = {
        label: d.projectName?.trim() || d.projectCode?.trim() || V4kd,
        rows: new Map(),
      };
      V1pi.set(V1pj, p);
    }
    const { hop, hoSo: V1pk } = V4ki(d);
    // Phông phải nằm trong khoá: "HOP 01 / HO SO 5" có ở nhiều phông khác nhau.
    const phong = phongOf(d);

    const V1pl = JSON.stringify([V1pj, phong, hop, V1pk]);
    let row = p.rows.get(V1pl);
    if (!row) {
      row = {
        key: V1pl,
        phong,
        hopSo: hop,
        hoSoSo: V1pk,
        files: [],
        ids: [],
        tally: V4kk(),
        counts: V4kj(),
      };
      p.rows.set(V1pl, row);
    }
    const ocrPages = Math.max(0, d.ocrPages ?? 0);
    const V1pm = Math.max(0, d.fieldPages ?? 0);
    const pdfaPages = Math.max(0, d.pdfaPages ?? 0);

    const V1pn = d.ocrPages != null || d.fieldPages != null;
    row.files.push({
      id: d.id,
      filename: d.filename,
      pages: d.pageCount ?? null,
      sizeBytes: d.sizeBytes,
      status: d.status,
      ocrPages,
      fieldPages: V1pm,
      pdfaPages,

      pdfa2b: d.pdfa2b ?? null,
      ocrKnown: V1pn,
      review: V4kh(d),
    });
    row.tally.files += 1;
    if (d.pageCount == null) row.tally.unknownPages += 1;
    else row.tally.pages += d.pageCount;
    row.tally.sizeBytes += d.sizeBytes;
    row.tally.ocrPages += ocrPages;
    if (V1pm > 0) row.tally.fieldDocs += 1;
    if (!V1pn) row.tally.unknownOcr += 1;

    if (pdfaPages > 0) {
      row.tally.pdfaDocs += 1;
      row.tally.pdfaPages += pdfaPages;
      if (d.pdfa2b === false) row.tally.pdfaCu += 1;
    }
    row.tally.ocrDenom += d.pageCount ?? ocrPages;
    row.counts.docs += 1;
    row.counts[V4kh(d)] += 1;
  }

  const V1po: V4kc[] = [];
  for (const [V1pp, p] of V1pi) {
    const V1pq = [...p.rows.values()].sort(V4kp);
    const g: V4kc = {
      key: `p:${V1pp}`,
      label: p.label,
      rows: V1pq,
      ids: [],
      tally: V4kk(),
      counts: V4kj(),
    };
    for (const r of V1pq) {
      r.files.sort((a, b) => V4kn.compare(a.filename, b.filename));
      r.ids = r.files.map((f) => f.id);
      g.ids.push(...r.ids);
      V4kl(g.tally, r.tally);
      V4km(g.counts, r.counts);
    }
    V1po.push(g);
  }
  return V1po.sort((a, b) => V4kn.compare(a.label, b.label));
}

export function V4kr(V1pr: string[], V1ps: ReadonlySet<string>): V4kg {
  if (V1pr.length === 0 || V1ps.size === 0) return "none";
  let V1pt = 0;
  for (const V1pu of V1pr) if (V1ps.has(V1pu)) V1pt += 1;
  if (V1pt === 0) return "none";
  return V1pt === V1pr.length ? "all" : "some";
}

export function V4ks(V1pv: ReadonlySet<string>, V1pw: string[], V1px: boolean): Set<string> {
  const V1py = new Set(V1pv);
  for (const V1pz of V1pw) {
    if (V1px) V1py.add(V1pz);
    else V1py.delete(V1pz);
  }
  return V1py;
}

export function V4kt(V1q0: ReadonlySet<string>, V1q1: V4kc[]): Set<string> {
  const V1q2 = new Set<string>();
  for (const g of V1q1) for (const V1q3 of g.ids) V1q2.add(V1q3);
  const V1q4 = new Set<string>();
  for (const V1q5 of V1q0) if (V1q2.has(V1q5)) V1q4.add(V1q5);
  return V1q4;
}

export function V4ku(V1q6: V4kc[], V1q7: ReadonlySet<string>): string[] {
  const V1q8: string[] = [];
  for (const g of V1q6) for (const V1q9 of g.ids) if (V1q7.has(V1q9)) V1q8.push(V1q9);
  return V1q8;
}

export function V4kv<T>(V1qa: T[], size: number): T[][] {
  if (size <= 0) throw new Error("cỡ lô phải lớn hơn 0");
  const V1qb: T[][] = [];
  for (let i = 0; i < V1qa.length; i += size) V1qb.push(V1qa.slice(i, i + size));
  return V1qb;
}

export function V4kw(docs: V4ej[], V1qc: { chiChoXacNhan?: boolean; q?: string } = {}): V4ej[] {
  const V1qd = (V1qc.q ?? "").trim().toLowerCase();
  return docs.filter((d) => {
    if (V1qc.chiChoXacNhan && V4kh(d) !== "pending") return false;
    if (!V1qd) return true;
    const { hop, hoSo: V1qe } = V4ki(d);
    return (
      d.filename.toLowerCase().includes(V1qd) ||
      (d.hoSoRef ?? "").toLowerCase().includes(V1qd) ||
      hop.toLowerCase().includes(V1qd) ||
      V1qe.toLowerCase().includes(V1qd)
    );
  });
}

export function V4kx(t: V4k7): number {
  if (t.ocrDenom <= 0) return 0;
  const p = (t.ocrPages / t.ocrDenom) * 100;
  return p >= 100 ? 100 : Math.floor(p);
}

export function pdfaPercent(t: V4k7): number {
  if (t.files <= 0) return 0;
  const p = (t.pdfaDocs / t.files) * 100;
  return p >= 100 ? 100 : Math.floor(p);
}
