// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export interface V4pa {
  id: string;
  forwardedBy: string | null;
  forwardedByName: string | null;
  note: string | null;

  status: "open" | "completed" | "abandoned";
  forwarded: number;
  skipped: number;
  failed: number;
  rejected: number;

  received: number;
  projectCount: number;
  createdAt: string;
  completedAt: string | null;
}

export function V4pb(V2be?: number): Promise<V4pa[]> {
  return invoke<V4pa[]>("upload_batches_list", { args: { limit: V2be } });
}

export interface V4pc {
  id: string;
  name: string | null;
  attention: boolean;
}

export interface V4pd {
  id: string;
  code: string;
  name: string;
  cells: Record<string, number>;
}

export interface V4pe {
  admins: V4pc[];
  projects: V4pd[];
}

export function V4pf(): Promise<V4pe> {
  return invoke<V4pe>("upload_batches_matrix");
}

export interface V4pg {
  admins: V4pc[];
  rows: Array<{ project: V4pd; total: number }>;

  columnTotals: Record<string, number>;
  grandTotal: number;

  isEmpty: boolean;
}

export function V4ph(m: V4pe): V4pg {
  const V2bf: Record<string, number> = {};
  for (const a of m.admins) V2bf[a.id] = 0;

  let V2bg = 0;
  const V2bh = m.projects.map((V2bi) => {
    let V2bj = 0;
    for (const a of m.admins) {
      const n = V2bi.cells[a.id] ?? 0;
      V2bj += n;
      V2bf[a.id] += n;
    }
    V2bg += V2bj;
    return { project: V2bi, total: V2bj };
  });

  return {
    admins: m.admins,
    rows: V2bh,
    columnTotals: V2bf,
    grandTotal: V2bg,
    isEmpty: m.admins.length === 0 || V2bh.length === 0,
  };
}

export type V4pi = "ok" | "in_progress" | "attention";

export function V4pj(b: V4pa): V4pi {
  if (b.status === "abandoned") return "attention";
  if (b.status !== "completed") return "in_progress";
  if (b.failed > 0 || b.rejected > 0) return "attention";

  if (b.received < b.forwarded) return "attention";
  return "ok";
}

export function V4pk(b: V4pa): string {
  if (b.status === "abandoned") {
    return `UGhpw6puIGfhu61pIGLhu4sgYuG7jyBk4bufIOKAlCBtw6F5IGNo4bunIMSRw6MgdOG7sSDEkcOzbmcuIMSQw6Mgbmjhuq1uIA==${b.received}IHTDoGkgbGnhu4d1OyBi4bqlbSDigJzikaEgR+G7rWkgbMOqbiBtw6F5IGNo4bun4oCdIMSR4buDIHRp4bq/cCB04bulYyBwaOG6p24gY8OybiBs4bqhaS4=`;
  }
  if (b.status !== "completed") {
    return "xJBhbmcgZ+G7rWkgaG/hurdjIHBoacOqbiBi4buLIGdpw6FuIMSRb+G6oW4g4oCUIGNoxrBhIGNo4buRdC4=";
  }
  const V2bk: string[] = [`${b.forwarded}IGfhu61pIG3hu5tp`];
  if (b.skipped > 0) V2bk.push(`${b.skipped}IMSRw6MgY8Oz`);
  if (b.failed > 0) V2bk.push(`${b.failed}IGzhu5dpIHThuqFtIChz4bq9IHRo4butIGzhuqFpKQ==`);
  if (b.rejected > 0) V2bk.push(`${b.rejected}IGLhu4sgdOG7qyBjaOG7kWk=`);
  let s = V2bk.join(", ");
  if (b.received < b.forwarded) {
    s += `IOKAlCBtw6F5IGNo4bunIGNo4buJIG5o4bqtbiA=${b.received}/${b.forwarded}LCBraeG7g20gdHJhIGzhuqFp`;
  }
  return s;
}
