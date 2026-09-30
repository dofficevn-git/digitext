// Copyright (c) 2026 DigiText. All rights reserved.

import type { V43b } from "@/api/lansync";
import type { V43l, V43k, V43m } from "@/api/uploadProgress";

export interface V43p {
  id: number;
  filename: string;
  projectId: string;
  hoSoRef: string | null;

  relPath: string | null;
  sizeBytes: number;
  sentBytes: number;
  state: V43k;
  message: string | null;
  permanent: boolean;

  updatedAt: number;

  bps: number;
}

export interface V43q {
  sessionId: number;
  running: boolean;

  total: number;

  items: V43p[];
  /** Bộ đếm tích luỹ của phiên — không suy từ `items` vì items bị cắt còn MAX_ROWS dòng. */
  counts: { done: number; errors: number; cancelled: number };
  summary: V43b | null;

  dismissed: boolean;

  collapsed: boolean;

  keepOnNextSession: boolean;
}

/** Số dòng tối đa giữ trong khung tiến độ: phiên 125.000 tệp mà vẽ lại cả danh sách mỗi sự kiện
 *  làm WebView ngốn trọn một lõi CPU. Dòng xong (done/skipped) cũ nhất bị bỏ trước; lỗi, huỷ và
 *  đang gửi luôn giữ. */
export const MAX_ROWS = 300;

function isTerminal(st: V43k): boolean {
  return st === "done" || st === "skipped" || st === "error" || st === "cancelled";
}

function bump(c: V43q["counts"], st: V43k, d: number): V43q["counts"] {
  if (st === "done" || st === "skipped") return { ...c, done: c.done + d };
  if (st === "error") return { ...c, errors: c.errors + d };
  if (st === "cancelled") return { ...c, cancelled: c.cancelled + d };
  return c;
}

function prune(items: V43p[]): V43p[] {
  if (items.length <= MAX_ROWS) return items;
  let drop = items.length - MAX_ROWS;
  const out: V43p[] = [];
  for (const it of items) {
    if (drop > 0 && (it.state === "done" || it.state === "skipped")) {
      drop -= 1;
      continue;
    }
    out.push(it);
  }
  return out;
}

export function V43r(): V43q {
  return {
    sessionId: 0,
    running: false,
    total: 0,
    items: [],
    counts: { done: 0, errors: 0, cancelled: 0 },
    summary: null,
    dismissed: false,
    collapsed: false,
    keepOnNextSession: false,
  };
}

const V43s = 0.3;

export function V43t(V80: V43p): boolean {
  return V80.state !== "uploading";
}

export function V43u(state: V43q): V43q {
  return { ...state, keepOnNextSession: true };
}

function V43v(state: V43q, V81: number, V82: number): V43q {
  return {
    ...V43r(),

    collapsed: state.collapsed,

    items: state.keepOnNextSession ? state.items.filter(V43t) : [],
    counts: state.keepOnNextSession
      ? state.items.filter(V43t).reduce((c, i) => bump(c, i.state, 1), { done: 0, errors: 0, cancelled: 0 })
      : { done: 0, errors: 0, cancelled: 0 },
    sessionId: V81,
    running: true,
    total: V82,
  };
}

export function V43w(state: V43q, V83: V43m): V43q {
  if (V83.sessionId < state.sessionId) return state;
  if (V83.phase === "start") return V43v(state, V83.sessionId, V83.total);
  return { ...state, running: false, total: V83.total, summary: V83.summary };
}

export function V43x(state: V43q, V84: V43l, V85: number): V43q {
  if (V84.sessionId < state.sessionId) return state;
  const base: V43q = V84.sessionId > state.sessionId ? V43v(state, V84.sessionId, 0) : state;

  const V86 = base.items.find((i) => i.id === V84.id);
  const V87: V43p = {
    id: V84.id,
    filename: V84.filename,
    projectId: V84.projectId,
    hoSoRef: V84.hoSoRef,
    relPath: V84.relPath,
    sizeBytes: V84.sizeBytes,
    sentBytes: V84.sentBytes,
    state: V84.state,
    message: V84.message,
    permanent: V84.permanent,
    updatedAt: V85,
    bps: V43y(V86, V84, V85),
  };
  const V88 = V86 ? base.items.map((i) => (i.id === V84.id ? V87 : i)) : [...base.items, V87];
  // Bộ đếm: trừ trạng thái cũ (nếu đã kết thúc), cộng trạng thái mới.
  let counts = base.counts;
  if (V86 && isTerminal(V86.state)) counts = bump(counts, V86.state, -1);
  if (isTerminal(V87.state)) counts = bump(counts, V87.state, 1);

  return { ...base, items: prune(V88), counts, dismissed: V86 ? base.dismissed : false };
}

function V43y(V89: V43p | undefined, V8a: V43l, V8b: number): number {
  if (!V89 || V8a.state !== "uploading") return V89?.bps ?? 0;
  const V8c = (V8b - V89.updatedAt) / 1000;
  const V8d = V8a.sentBytes - V89.sentBytes;
  if (V8c <= 0 || V8d <= 0) return V89.bps;
  const V8e = V8d / V8c;
  return V89.bps > 0 ? V89.bps * (1 - V43s) + V8e * V43s : V8e;
}

export function V43z(V8f: V43p): number {
  if (V8f.state === "done" || V8f.state === "skipped") return 100;
  return V440(V8f);
}

function V440(V8g: V43p): number {
  if (V8g.sizeBytes <= 0) return 0;
  const p = Math.round((V8g.sentBytes / V8g.sizeBytes) * 100);
  return Math.min(100, Math.max(0, p));
}

export function V441(s: V43q): V43p[] {
  const V8h = (V8i: V43p) => (V8i.state === "error" ? 0 : V8i.state === "cancelled" ? 1 : 2);
  return s.items
    .map((V8j, i) => ({ it: V8j, i }))
    .sort((a, b) => V8h(a.it) - V8h(b.it) || a.i - b.i)
    .map((x) => x.it);
}

export function V442(s: V43q): boolean {
  if (s.dismissed) return false;
  return s.items.length > 0 || (s.running && s.total > 0);
}

export function V443(s: V43q): number {
  const V8k = s.counts.done + s.counts.errors + s.counts.cancelled;
  return Math.max(0, Math.max(s.total, V8k) - V8k);
}

export function V444(s: V43q): {
  done: number;
  errors: number;
  cancelled: number;
} {
  return s.counts;
}

export function V445(s: V43q): string {
  const t = V444(s);
  if (s.running) {
    const con = V443(s);
    return con > 0 ? `Đang tải ${con} mục lên` : "Đang hoàn tất…";
  }
  const V8l: string[] = [];
  if (t.done > 0) V8l.push(`Đã hoàn tất ${t.done} tệp tải lên`);
  if (t.errors > 0) V8l.push(`${t.errors} tệp lỗi`);
  if (t.cancelled > 0) V8l.push(`${t.cancelled} tệp đã hủy`);
  if (V8l.length === 0) return "Không có tệp nào để tải lên";
  return V8l.join(" · ");
}

export function V446(V8m: V43p, V8n: number): string {
  switch (V8m.state) {
    case "uploading": {
      const size = `${V44a(V8m.sentBytes)}/${V44a(V8m.sizeBytes)}`;
      const V8o = V447(V8m.bps);
      const V8p = V448(V8m, V8n);
      return [size, V8o, V8p].filter((s) => s !== "").join(" · ");
    }
    case "done":
      return `Đã tải lên · ${V44a(V8m.sizeBytes)}`;
    case "skipped":
      return V8m.message ?? "Máy chủ đã có tài liệu này.";
    case "cancelled":
      return V8m.message ?? "Đã hủy tải lên.";
    case "error":
      return V8m.message ?? "Tải lên thất bại.";
  }
}

export function V447(V8q: number): string {
  if (!Number.isFinite(V8q) || V8q <= 0) return "";
  return `${V44a(V8q)}/s`;
}

export function V448(V8r: V43p, V8s: number): string {
  if (V8r.state !== "uploading" || V8r.bps <= 0 || V8r.sizeBytes <= 0) return "";

  if (V8s - V8r.updatedAt > 30_000) return "";
  const left = V8r.sizeBytes - V8r.sentBytes;
  if (left <= 0) return "";
  return `còn ${V449(Math.ceil(left / V8r.bps))}`;
}

export function V449(V8t: number): string {
  if (V8t < 60) return `${V8t} giây`;
  if (V8t < 3600) {
    const m = Math.floor(V8t / 60);
    const s = V8t % 60;
    return s > 0 ? `${m} phút ${String(s).padStart(2, "0")} giây` : `${m} phút`;
  }
  const h = Math.floor(V8t / 3600);
  const m = Math.round((V8t % 3600) / 60);
  return m > 0 ? `${h} giờ ${m} phút` : `${h} giờ`;
}

export function V44a(n: number): string {
  if (!Number.isFinite(n) || n <= 0) return "0 B";
  const V8u = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  let v = n;
  while (v >= 1024 && i < V8u.length - 1) {
    v /= 1024;
    i += 1;
  }
  const V8v = v >= 100 || i === 0 ? 0 : 1;
  return `${v.toFixed(V8v).replace(".", ",")} ${V8u[i]}`;
}

export function V44b(V8w: V43p): boolean {
  return V8w.state === "uploading";
}

export function V44c(V8x: V43p): boolean {
  return V8x.state === "error" || V8x.state === "cancelled";
}

export function V44d(V8y: number[], V8z: number[]): number[] {
  const V90 = [...V8y];
  for (const V91 of V8z) if (!V90.includes(V91)) V90.push(V91);
  return V90;
}

export function V44e(s: V43q, V92: number): V43q {
  const it = s.items.find((i) => i.id === V92);
  return { ...s, items: s.items.filter((i) => i.id !== V92), counts: it && isTerminal(it.state) ? bump(s.counts, it.state, -1) : s.counts };
}
