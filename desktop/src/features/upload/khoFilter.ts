// Copyright (c) 2026 DigiText. All rights reserved.

import type { V40d } from "@/api/archive";
import type { V4bm } from "@/api/dossier";
import type { V43g } from "@/api/lansync";
import type { V3yo } from "@/domain/types";
import { V4iu, type V4is, type V4ir } from "./syncMatrix";

export interface V4iz {
  maDa: string;

  maDv: string;

  mangCvid: string;

  thbq: string;

  search: string;
}

export const V4j0: V4iz = {
  maDa: "",
  maDv: "",
  mangCvid: "",
  thbq: "",
  search: "",
};

export const V4j1 = (f: V4iz): boolean =>
  f.maDa !== "" || f.maDv !== "" || f.mangCvid !== "" || f.thbq !== "";

export const V4j2 = (f: V4iz): boolean => V4j1(f) || f.search.trim() !== "";

export const V4j3 = (f: V4iz): boolean => f.maDv !== "" || f.mangCvid !== "" || f.thbq !== "";

export function V4j4(V1me: "phongBan" | "mang", V1mf: boolean, V1mg: number): string {
  const ten = V1me === "phongBan" ? "phòng ban" : "mảng công việc";
  if (!V1mf)
    return V1me === "phongBan"
      ? "— Chọn phông trước —"
      : "— Chọn phòng ban trước —";
  if (V1mg === 0) return `— Danh mục chưa có ${ten} —`;
  return `— Tất cả ${ten} —`;
}

export interface V4j5 {
  maDv: string;
  mangCvid: number | null;
  thbq: number | null;
}

export type V4j6 = Map<string, V4j5>;

const V4j7 = (V1mh: string, V1mi: string) => `${V1mh}\u0000${V1mi.trim()}`;

const V4j8 = (s: string) => s.trim().toLowerCase();

export function V4j9(V1mj: V4bm[], V1mk: V40d[] = [], V1ml: V3yo[] = []): V4j6 {
  const V1mm: V4j6 = new Map();

  const V1mn = new Map<string, number>();
  for (const t of V1ml) {
    const ten = V4j8(t.tenThbq ?? "");
    if (ten) V1mn.set(ten, t.thbqid);
  }
  for (const h of V1mk) {
    const V1mo = h.phongId ?? "";
    if (!V1mo || !h.hoSoSo.trim()) continue;
    const V1mp = V1mn.get(V4j8(h.thbq ?? ""));
    if (V1mp == null) continue;
    V1mm.set(V4j7(V1mo, h.hoSoSo), { maDv: "", mangCvid: null, thbq: V1mp });
  }

  for (const d of V1mj) {
    const V1mq = d.maDa ?? "";
    if (!V1mq || !d.soTam.trim()) continue;
    const k = V4j7(V1mq, d.soTam);
    V1mm.set(k, {
      maDv: d.maDv ?? "",
      mangCvid: d.mangCvid ?? null,
      thbq: d.thbq ?? V1mm.get(k)?.thbq ?? null,
    });
  }

  return V1mm;
}

export function V4ja(V1mr: string | null): string {
  if (!V1mr) return "";
  const V1ms = V1mr.indexOf("/");
  return V1ms < 0 ? V1mr : V1mr.slice(V1ms + 1);
}

export function V4jb(r: V4is, V1mt: V40d | null): string {
  return V1mt?.phongId || r.phongMa || "";
}

export function V4jc(r: V4is, V1mu: V40d | null, V1mv: V4j6): V4j5 | null {
  const V1mw = V4jb(r, V1mu);
  const V1mx = V1mu ? V1mu.hoSoSo : V4ja(r.hoSoRef);
  if (!V1mw || !V1mx.trim()) return null;
  return V1mv.get(V4j7(V1mw, V1mx)) ?? null;
}

export function V4jd(a: V4j5, f: V4iz): boolean {
  if (f.maDv && a.maDv !== f.maDv) return false;
  if (f.mangCvid && a.mangCvid !== Number(f.mangCvid)) return false;
  if (f.thbq && a.thbq !== Number(f.thbq)) return false;
  return true;
}

export interface V4je {
  rows: V43g[];

  unknown: number;
}

export function V4jf(V1my: V43g[], filter: V4iz, tree: V4ir, V1mz: V4j6): V4je {
  if (!V4j2(filter)) return { rows: V1my, unknown: 0 };

  const q = V4j8(filter.search);
  const V1n0 = V4j3(filter);
  const V1n1: V43g[] = [];
  let V1n2 = 0;

  for (const r of V1my) {
    const h = V4iu(r, tree);
    if (filter.maDa && V4jb(r, h) !== filter.maDa) continue;
    if (V1n0) {
      const a = V4jc(r, h, V1mz);
      if (!a) {
        V1n2 += 1;
        continue;
      }
      if (!V4jd(a, filter)) continue;
    }
    if (q) {
      const V1n3 = V4j8(
        [r.filename, r.hoSoRef ?? "", r.sourceUsername ?? "", h?.tieuDe ?? ""].join(" "),
      );
      if (!V1n3.includes(q)) continue;
    }
    V1n1.push(r);
  }

  return { rows: V1n1, unknown: V1n2 };
}

export function V4jg(V1n4: V40d[], filter: V4iz, V1n5: V4j6): V40d[] {
  if (!V4j1(filter)) return V1n4;
  const V1n6 = V4j3(filter);
  return V1n4.filter((h) => {
    const V1n7 = h.phongId ?? "";
    if (filter.maDa && V1n7 !== filter.maDa) return false;
    if (!V1n6) return true;
    if (!V1n7 || !h.hoSoSo.trim()) return false;
    const a = V1n5.get(V4j7(V1n7, h.hoSoSo));
    return !!a && V4jd(a, filter);
  });
}
