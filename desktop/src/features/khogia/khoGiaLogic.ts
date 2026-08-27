// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4lp, V4lr, V4lo, V4lq } from "@/api/khaithac";

export function V4ng(k: Pick<V4lq, "tenKho">): string | null {
  if (!k.tenKho.trim()) return "Thiếu tên kho lưu trữ.";
  return null;
}

export function V4nh(g: Pick<V4lr, "tenGia" | "maKho" | "tuHop" | "denHop">): string | null {
  if (!g.tenGia.trim()) return "Thiếu tên giá lưu trữ.";
  if (!g.maKho || g.maKho <= 0) return "Chưa chọn kho lưu trữ.";
  if (g.tuHop != null && g.denHop != null && g.denHop < g.tuHop)
    return "Đến hộp phải ≥ Từ hộp.";
  return null;
}

export function V4ni(k: V4lo, q: string): boolean {
  const s = q.trim().toLowerCase();
  if (!s) return true;
  return k.tenKho.toLowerCase().includes(s) || (k.ghiChu ?? "").toLowerCase().includes(s);
}

export function V4nj(g: V4lp, q: string): boolean {
  const s = q.trim().toLowerCase();
  if (!s) return true;
  return (
    g.tenGia.toLowerCase().includes(s) ||
    (g.tenKho ?? "").toLowerCase().includes(s) ||
    (g.tenDuAn ?? "").toLowerCase().includes(s) ||
    (g.tenDonVi ?? "").toLowerCase().includes(s) ||
    (g.ghiChu ?? "").toLowerCase().includes(s)
  );
}

export function V4nk(V1zm?: number, V1zn?: number): string {
  if (V1zm == null && V1zn == null) return "";
  return `${V1zm ?? "?"} – ${V1zn ?? "?"}`;
}

export function V4nl(v: string): number | undefined {
  const t = v.trim();
  if (!t) return undefined;
  const n = Number(t);
  return Number.isFinite(n) ? n : undefined;
}
