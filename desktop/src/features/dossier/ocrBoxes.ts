// Copyright (c) 2026 DigiText. All rights reserved.

export interface V4gk {
  key: string;
  x: number;
  y: number;
  w: number;
  h: number;

  field?: string;
  text?: string;
  conf?: number | null;
}

export interface V4gl {
  left: number;
  top: number;
  width: number;
  height: number;
}

export type V4gm = "nw" | "n" | "ne" | "e" | "se" | "s" | "sw" | "w" | "move";

export const V4gn = 6;

export function V4go(V19i: number, V19j: number): number {
  if (V19i <= 0) return 1;
  return V19j / V19i;
}

export function V4gp(V19k: V4gk, scale: number): V4gl {
  return {
    left: V19k.x * scale,
    top: V19k.y * scale,
    width: V19k.w * scale,
    height: V19k.h * scale,
  };
}

export function V4gq(V19l: V4gk, V19m: number, V19n: number): V4gk {
  const w = Math.max(V4gn, Math.min(V19l.w, V19m));
  const h = Math.max(V4gn, Math.min(V19l.h, V19n));
  const x = Math.max(0, Math.min(V19l.x, V19m - w));
  const y = Math.max(0, Math.min(V19l.y, V19n - h));
  return { ...V19l, x, y, w, h };
}

export function V4gr(V19o: V4gk, V19p: number, V19q: number, V19r: number, V19s: number): V4gk {
  return V4gq({ ...V19o, x: V19o.x + V19p, y: V19o.y + V19q }, V19r, V19s);
}

export function V4gs(
  V19t: V4gk,
  V19u: V4gm,
  V19v: number,
  V19w: number,
  V19x: number,
  V19y: number,
): V4gk {
  if (V19u === "move") return V4gr(V19t, V19v, V19w, V19x, V19y);
  let { x, y, w, h } = V19t;
  const right = x + w;
  const bottom = y + h;

  if (V19u.includes("w")) {
    const V19z = Math.min(x + V19v, right - V4gn);
    w = right - V19z;
    x = V19z;
  }
  if (V19u.includes("e")) {
    w = Math.max(V4gn, w + V19v);
  }
  if (V19u.includes("n")) {
    const V1a0 = Math.min(y + V19w, bottom - V4gn);
    h = bottom - V1a0;
    y = V1a0;
  }
  if (V19u.includes("s")) {
    h = Math.max(V4gn, h + V19w);
  }
  return V4gq({ ...V19t, x, y, w, h }, V19x, V19y);
}

export function V4gt(
  V1a1: string,
  V1a2: number[] | null | undefined,
  text?: string,
  V1a3?: number | null,
  field?: string,
): V4gk | null {
  if (!V1a2 || V1a2.length < 4) return null;
  const [x, y, w, h] = V1a2;
  if (![x, y, w, h].every((n) => Number.isFinite(n))) return null;
  return { key: V1a1, x, y, w, h, text, conf: V1a3, field };
}

export function V4gu(
  V1a4: string,
  field: string,
  V1a5: number,
  V1a6: number,
  V1a7: number,
  V1a8: number,
  V1a9: number,
  V1aa: number,
): V4gk {
  const x = Math.min(V1a5, V1a7);
  const y = Math.min(V1a6, V1a8);
  const w = Math.abs(V1a7 - V1a5);
  const h = Math.abs(V1a8 - V1a6);
  return V4gq({ key: V1a4, field, x, y, w, h }, V1a9, V1aa);
}

export function V4gv(V1ab: V4gk): number[] {
  return [Math.round(V1ab.x), Math.round(V1ab.y), Math.round(V1ab.w), Math.round(V1ab.h)];
}

export function V4gw(V1ac: number | null | undefined): string {
  if (V1ac == null) return "#6b7280";
  if (V1ac >= 0.8) return "#22c55e";
  if (V1ac >= 0.6) return "#eab308";
  return "#ef4444";
}
