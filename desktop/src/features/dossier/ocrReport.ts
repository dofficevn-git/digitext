// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4eg, V4f0, V4eh } from "@/api/docs";

export interface V4he {
  from: string;
  to: string;
}

export function V4hf(d: Date): string {
  const p = (n: number) => String(n).padStart(2, "0");
  return `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())}`;
}

export function V4hg(V1hq: Date = new Date()): V4he {
  return {
    from: V4hf(new Date(V1hq.getFullYear(), V1hq.getMonth(), 1)),
    to: V4hf(V1hq),
  };
}

export function V4hh(r: V4he): string | null {
  const V1hr = (s: string) => s === "" || /^\d{4}-\d{2}-\d{2}$/.test(s);
  if (!V1hr(r.from) || !V1hr(r.to))
    return "Ngày phải theo dạng năm-tháng-ngày.";
  if (r.from !== "" && r.to !== "" && r.from > r.to) {
    return "Ngày bắt đầu phải trước hoặc bằng ngày kết thúc.";
  }
  return null;
}

export function V4hi(c: V4f0): number {
  if (c.docs <= 0) return 0;
  return (c.approved / c.docs) * 100;
}

export function V4hj(c: V4f0): string {
  const p = V4hi(c);
  if (p >= 100) return "100%";
  return `${Math.floor(p)}%`;
}

export const V4hk: Record<V4eh, { label: string; tone: "ok" | "err" | "warn" }> = {
  approved: { label: "Đã duyệt", tone: "ok" },
  rejected: { label: "Từ chối", tone: "err" },
  pending: { label: "Chờ xác nhận", tone: "warn" },
};

export function V4hl(status: string) {
  return V4hk[status as V4eh] ?? V4hk.pending;
}

export function V4hm(docs: V4eg[], q: string, V1hs: string): V4eg[] {
  const V1ht = q.trim().toLowerCase();
  return docs.filter((d) => {
    if (V1hs && d.projectId !== V1hs) return false;
    if (!V1ht) return true;
    return (
      d.filename.toLowerCase().includes(V1ht) || (d.hoSoRef ?? "").toLowerCase().includes(V1ht)
    );
  });
}

export interface V4hn {
  known: boolean;

  docs: number;

  total: number;

  old: number;

  percent: number;
}

export function pdfaSummary(c: V4f0): V4hn {
  const docs = Math.max(0, c.pdfaDocs ?? 0);
  const V1hu = Math.max(0, c.docs);
  const p = V1hu <= 0 ? 0 : (docs / V1hu) * 100;
  return {
    known: c.pdfaDocs != null,
    docs,
    total: V1hu,
    old: Math.max(0, c.pdfaOld ?? 0),

    percent: p >= 100 ? 100 : Math.floor(p),
  };
}
