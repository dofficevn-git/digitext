// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4ld, V4le } from "@/api/khaithac";
import { V4dr } from "@/features/dossier/logic";

export function V4m0(status: string): { label: string; cls: string } {
  switch (status) {
    case "VV":
      return { label: "Vĩnh viễn", cls: "badge-vv" };
    case "HET":
      return { label: "Hết THBQ", cls: "badge-het" };
    default:
      return { label: "Giữ lại", cls: "badge-gl" };
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
    "Mã hồ sơ": r.maHoSo ?? "",
    "Hộp số": r.hopTaiLieuSo ?? "",
    "Số tạm": r.hososo,
    "Tiêu đề hồ sơ": r.tieuDeHs,
    THBQ: r.thoiHanBaoQuanStr ?? "",
    "Số tờ": r.soTo ?? "",
    "Số lượng trang": r.soLuongTrang ?? "",
    "Ngày bắt đầu": V4dr(r.thoiGianBatDau),
    "Ngày kết thúc": V4dr(r.thoiGianKetThuc),
    "Trùng thừa": r.trungThua ? "x" : "",
    "Số văn bản": r.soVanBan,
    "Trạng thái": V4m0(r.status).label,
    Kho: r.tenKho ?? "",
    "Giá/kệ": r.tenGia ?? "",
    "Ghi chú": r.ghiChu ?? "",
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
