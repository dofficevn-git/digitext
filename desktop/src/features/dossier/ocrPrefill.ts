// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4eg } from "@/api/docs";
import type { V4fe } from "@/api/ocr";
import type { V4bp } from "@/api/dossier";
import { V4fn, V4fv, V4fm } from "../upload/ocrLogic";

export function V4gd(path: string): string {
  const V195 = path.split(/[\\/]/);
  return (V195[V195.length - 1] ?? "").toLowerCase();
}

export function V4ge(d: { hopSo?: string | null; soTam: string }): string {
  return `${(d.hopSo ?? "").trim()}/${d.soTam.trim()}`;
}

export function V4gf(docs: V4eg[], V196: string, V197: string): V4eg | null {
  const V198 = V4gd(V197);
  if (!V198) return null;
  const V199 = docs.filter((d) => V4gd(d.filename) === V198);
  const V19a = V199.filter((d) => (d.hoSoRef ?? "") === V196);
  if (V19a.length > 0) return V19a[0];
  return V199.length === 1 ? V199[0] : null;
}

const V4gg = ["thuTu", "toSo", "soTrang", "trangSo"] as const;

const V4gh = [
  "loaiVanBan",
  "soKyHieu",
  "ngayThang",
  "coQuanBanHanh",
  "trichYeu",
  "ngonNgu",
  "cheDoSuDung",
  "doTinCay",
  "butTich",
  "tinhTrangVatLy",
  "taiLieuSo",
  "ghiChu",
] as const;

export function V4gi(V19b: V4bp, V19c: V4fe[]): { draft: V4bp; filled: string[] } {
  const V19d = new Map<string, string>();
  for (const f of V4fv(V19c)) {
    if (!V19d.has(f.fieldKey)) V19d.set(f.fieldKey, f.value);
  }

  const V19e: V4bp = { ...V19b };
  const V19f: string[] = [];
  for (const k of V4gh) {
    const v = V19d.get(k)?.trim();
    if (v && !(V19e[k] ?? "").trim()) {
      V19e[k] = v;
      V19f.push(k);
    }
  }
  for (const k of V4gg) {
    const V19g = k === "thuTu" ? !V19e.thuTu : V19e[k] == null;
    const m = V19d.get(k)?.match(/\d+/);
    if (V19g && m) {
      V19e[k] = Number(m[0]);
      V19f.push(k);
    }
  }
  V19f.sort((a, b) => V4fn.indexOf(a) - V4fn.indexOf(b));
  return { draft: V19e, filled: V19f };
}

export function V4gj(V19h: string[]): string {
  return V19h.map((k) => V4fm[k] ?? k).join(", ");
}
