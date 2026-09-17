// Copyright (c) 2026 DigiText. All rights reserved.

import type { V40d, V40v } from "@/api/archive";
import type { V4ej } from "@/api/docs";
import { V4jd, V4jc, V4j1, V4j3, V4jb, type V4j6, type V4iz } from "@/features/upload/khoFilter";
import { V4iu, type V4is, type V4ir } from "@/features/upload/syncMatrix";

export function V4jh(s: string): string {
  return s
    .normalize("NFD")
    .replace(/[\u0300-\u036f]/g, "")
    .replace(/đ/g, "d")
    .replace(/Đ/g, "D")
    .replace(/[^\p{L}\p{N}]+/gu, " ")
    .trim()
    .toLowerCase();
}

export interface V4ji {
  byName: Map<string, string>;

  byCode: Map<string, string>;

  ambiguous: Set<string>;
}

export function V4jj(V1n8: V40v[]): V4ji {
  const V1n9 = new Map<string, string>();
  const V1na = new Map<string, string>();
  const V1nb = new Set<string>();

  const V1nc = (m: Map<string, string>, V1nd: string, ma: string) => {
    if (!V1nd || V1nb.has(V1nd)) return;
    const cu = m.get(V1nd);
    if (cu !== undefined && cu !== ma) {
      V1nb.add(V1nd);
      m.delete(V1nd);
    } else {
      m.set(V1nd, ma);
    }
  };
  for (const p of V1n8) {
    V1nc(V1n9, V4jh(p.tenDuAn ?? ""), p.maDuAn);
    V1nc(V1na, V4jh(p.maDuAn ?? ""), p.maDuAn);
  }
  return { byName: V1n9, byCode: V1na, ambiguous: V1nb };
}

function V4jk(V1ne: string | null | undefined): string[] {
  return (V1ne ?? "").split("/").filter((s) => s.trim() !== "");
}

export function V4jl(doc: V4ej, V1nf: V4ji): string {
  const V1ng = V4jk(doc.relPath)[0];
  if (!V1ng) return "";
  const k = V4jh(V1ng);
  if (!k || V1nf.ambiguous.has(k)) return "";
  const V1nh = V1nf.byName.get(k) ?? V1nf.byCode.get(k);
  if (V1nh) return V1nh;
  const V1ni = [...V1nf.byName.entries()].filter(([name]) => name.startsWith(k));
  return V1ni.length === 1 ? V1ni[0][1] : "";
}

export function V4jm(doc: V4ej, V1nj: V4ji): V4is {
  const V1nk = V4jk(doc.relPath);
  const V1nl = doc.hoSoRef ?? (V1nk.length >= 3 ? `${V1nk[1]}/${V1nk[2]}` : null);
  return { phongMa: V4jl(doc, V1nj) || null, hoSoRef: V1nl };
}

export interface V4jn {
  docs: V4ej[];

  unknown: number;
}

export function V4jo(docs: V4ej[], filter: V4iz, V1nm: V4ji, tree: V4ir, V1nn: V4j6): V4jn {
  const V1no = docs.filter((d) => d.status !== "disposed");
  if (!V4j1(filter)) return { docs: V1no, unknown: 0 };

  const V1np = V4j3(filter);
  const V1nq: V4ej[] = [];
  let V1nr = 0;

  for (const d of V1no) {
    const r = V4jm(d, V1nm);
    const h = V4iu(r, tree);
    if (filter.maDa && V4jb(r, h) !== filter.maDa) continue;
    if (V1np) {
      const a = V4jc(r, h, V1nn);
      if (!a) {
        V1nr += 1;
        continue;
      }
      if (!V4jd(a, filter)) continue;
    }
    V1nq.push(d);
  }
  return { docs: V1nq, unknown: V1nr };
}

export function V4jp(doc: V4ej, V1ns: V4ji, tree: V4ir): V40d | null {
  return V4iu(V4jm(doc, V1ns), tree);
}

const V4jq = new Set(["uploaded", "ocr_pending"]);
const V4jr = new Set(["ocr_done", "to_sign", "signing", "signed", "sign_rejected", "archived"]);
const V4js = new Set(["to_sign", "signing", "signed", "sign_rejected", "archived"]);

export interface V4jt {
  docs: number;
  docsOcrPending: number;
  docsOcrDone: number;
  docsPdf2Done: number;
  docsFieldsDone: number;
  docsFieldsActive: number;
  docsFieldsFailed: number;
  /** Trang OCR lỗi — chỉ có ở chế độ toàn kho (tiến độ máy chủ), cây lọc không tính được. */
  pagesOcrFailed: number;
  docsOcrFailed: number;
}

const V4ju: V4jt = {
  docs: 0,
  docsOcrPending: 0,
  docsOcrDone: 0,
  docsPdf2Done: 0,
  docsFieldsDone: 0,
  docsFieldsActive: 0,
  docsFieldsFailed: 0,
  pagesOcrFailed: 0,
  docsOcrFailed: 0,
};

export function V4jv(docs: V4ej[]): V4jt {
  const p = { ...V4ju };
  for (const d of docs) {
    if (d.status === "disposed") continue;
    p.docs += 1;
    if (V4jq.has(d.status)) p.docsOcrPending += 1;
    if (V4jr.has(d.status)) p.docsOcrDone += 1;
    if (V4js.has(d.status)) p.docsPdf2Done += 1;
    switch (d.fieldsStatus) {
      case "done":
        p.docsFieldsDone += 1;
        break;
      case "queued":
      case "running":
        p.docsFieldsActive += 1;
        break;
      case "failed_transient":
      case "page_failed":
        p.docsFieldsFailed += 1;
        break;
      default:
        break;
    }
  }
  return p;
}

export function V4jw(docs: V4ej[]): string[] {
  return docs.map((d) => d.id);
}
