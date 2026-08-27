// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4ld, V4le } from "@/api/khaithac";
import { V4dr } from "@/features/dossier/logic";

export function V4m0(status: string): { label: string; cls: string } {
  switch (status) {
    case "VV":
      return { label: "VsSpbmggdmnhu4Vu", cls: "badge-vv" };
    case "HET":
      return { label: "SOG6v3QgVEhCUQ==", cls: "badge-het" };
    default:
      return { label: "R2nhu68gbOG6oWk=", cls: "badge-gl" };
  }
}

export function V4m1(r: { thoiGianBatDau?: string; thoiGianKetThuc?: string }): string {
  const V1us = V4dr(r.thoiGianBatDau);
  const V1ut = V4dr(r.thoiGianKetThuc);
  if (V1us && V1ut) return `${V1us} - ${V1ut}`;
  return V1us || V1ut || "";
}

const V4m2 = "khaithac.filters";

export function V4m3(f: V4ld): void {
  try {
    const V1uu: V4ld = { ...f };
    delete V1uu.page;
    delete V1uu.pageSize;
    localStorage.setItem(V4m2, JSON.stringify(V1uu));
  } catch {}
}

export function V4m4(): Partial<V4ld> {
  try {
    const V1uv = localStorage.getItem(V4m2);
    if (!V1uv) return {};
    const o = JSON.parse(V1uv);
    return o && typeof o === "object" ? (o as Partial<V4ld>) : {};
  } catch {
    return {};
  }
}

export function V4m5(V1uw: V4le[]): Record<string, unknown>[] {
  return V1uw.map((r, i) => ({
    STT: i + 1,
    TcOjIGjhu5Mgc8ah: r.maHoSo ?? "",
    "SOG7mXAgc+G7kQ==": r.hopTaiLieuSo ?? "",
    "U+G7kSB04bqhbQ==": r.hososo,
    "VGnDqnUgxJHhu4EgaOG7kyBzxqE=": r.tieuDeHs,
    THBQ: r.thoiHanBaoQuanStr ?? "",
    "U+G7kSB04bud": r.soTo ?? "",
    "U+G7kSBsxrDhu6NuZyB0cmFuZw==": r.soLuongTrang ?? "",
    TmfDoHkgYuG6r3QgxJHhuqd1: V4dr(r.thoiGianBatDau),
    "TmfDoHkga+G6v3QgdGjDumM=": V4dr(r.thoiGianKetThuc),
    "VHLDuW5nIHRo4burYQ==": r.trungThua ? "x" : "",
    "U+G7kSB2xINuIGLhuqNu": r.soVanBan,
    "VHLhuqFuZyB0aMOhaQ==": V4m0(r.status).label,
    Kho: r.tenKho ?? "",
    R2nDoS9r4buH: r.tenGia ?? "",
    "R2hpIGNow7o=": r.ghiChu ?? "",
  }));
}

export function V4m6(v: unknown): string {
  return String(v ?? "")
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function V4m7(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

export function V4m8(text: string, V1ux: string): { text: string; mark: boolean }[] {
  const t = V1ux.trim();
  if (!t || !text) return [{ text, mark: false }];
  const V1uy = new RegExp(`(${V4m7(t)})`, "gi");
  return text
    .split(V1uy)
    .filter((p) => p !== "")
    .map((p) => ({ text: p, mark: p.toLowerCase() === t.toLowerCase() }));
}
