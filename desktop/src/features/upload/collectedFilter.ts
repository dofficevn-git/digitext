// Copyright (c) 2026 DigiText. All rights reserved.

import type { V43g } from "@/api/lansync";
import { V4id } from "./syncMatrix";

export interface V4pv {
  from: string;
  to: string;
}

export const V4pw: V4pv = { from: "", to: "" };

export const V4px = (r: V4pv) => r.from !== "" || r.to !== "";

const V4py = (n: number) => String(n).padStart(2, "0");

export function V4pz(V2c9: string): string {
  const s = (V2c9 ?? "").trim();
  if (!s) return "";
  const V2ca = /^\d{4}-\d{2}-\d{2}[ T]\d{2}:\d{2}:\d{2}/.test(s)
    ? `${s.slice(0, 10)}T${s.slice(11, 19)}Z`
    : s;
  const d = new Date(V2ca);
  if (Number.isNaN(d.getTime())) return "";
  return `${d.getFullYear()}-${V4py(d.getMonth() + 1)}-${V4py(d.getDate())}`;
}

export function V4q0(V2cb: string): string {
  const s = (V2cb ?? "").trim();
  if (!s) return "—";
  const V2cc = /^\d{4}-\d{2}-\d{2}[ T]\d{2}:\d{2}:\d{2}/.test(s)
    ? `${s.slice(0, 10)}T${s.slice(11, 19)}Z`
    : s;
  const d = new Date(V2cc);
  if (Number.isNaN(d.getTime())) return "—";
  return `${V4py(d.getDate())}/${V4py(d.getMonth() + 1)}/${d.getFullYear()} ${V4py(d.getHours())}:${V4py(d.getMinutes())}`;
}

export function V4q1(row: V43g, V2cd: V4pv): boolean {
  if (!V4px(V2cd)) return true;
  const V2ce = V4pz(row.collectedAt);
  if (!V2ce) return false;
  if (V2cd.from && V2ce < V2cd.from) return false;
  if (V2cd.to && V2ce > V2cd.to) return false;
  return true;
}

export function V4q2(V2cf: V43g[], V2cg: V4pv): V43g[] {
  return V4px(V2cg) ? V2cf.filter((r) => V4q1(r, V2cg)) : V2cf;
}

export interface V4q3 {
  collected: number;

  forwarded: number;

  notForwarded: number;

  rejected: number;
}

export function V4q4(V2ch: V43g[]): V4q3 {
  const t: V4q3 = { collected: 0, forwarded: 0, notForwarded: 0, rejected: 0 };
  for (const r of V2ch) {
    t.collected += 1;
    const V2ci = V4id(r);
    if (V2ci === "forwarded") t.forwarded += 1;
    else {
      t.notForwarded += 1;
      if (V2ci === "rejected") t.rejected += 1;
    }
  }
  return t;
}

export function V4q5(r: V43g): boolean {
  const V2cj = V4id(r);
  return V2cj === "rejected" || (V2cj === "pending" && r.forwardTries > 0);
}

export function V4q6(V2ck: V43g[]): V43g[] {
  return V2ck.filter(V4q5).sort(
    (a, b) =>
      Number(b.forwardFatal) - Number(a.forwardFatal) ||
      b.forwardTries - a.forwardTries ||
      a.filename.localeCompare(b.filename, "vi", { numeric: true }),
  );
}
