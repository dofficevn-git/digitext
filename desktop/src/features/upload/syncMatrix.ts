// Copyright (c) 2026 DigiText. All rights reserved.

import type { V40d, V40v } from "@/api/archive";
import type { V43g } from "@/api/lansync";

export const V4i8 = "S2jDtG5nIHLDtSBuZ8aw4budaSBxdcOpdA==";
export const V4i9 = "Q2jGsGEgZ8OhbiBwaMO0bmc=";
export const V4ia = "\u0000unmatched";
export const V4ib = "Q2jGsGEga2jhu5twIGPDonkgdGjGsCBt4bulYw==";

export type V4ic = "forwarded" | "rejected" | "pending";

export function V4id(r: V43g): V4ic {
  if (r.forwarded) return "forwarded";
  if (r.forwardFatal) return "rejected";
  return "pending";
}

export function V4ie(r: V43g): boolean {
  const V1l5 = V4id(r);
  return V1l5 === "rejected" || (V1l5 === "pending" && r.forwardTries > 0);
}

export interface V4if {
  total: number;
  pending: number;
  forwarded: number;
  rejected: number;
  sizeBytes: number;
}

const V4ig = (): V4if => ({
  total: 0,
  pending: 0,
  forwarded: 0,
  rejected: 0,
  sizeBytes: 0,
});

function V4ih(t: V4if, r: V43g): void {
  t.total += 1;
  t[V4id(r)] += 1;
  t.sizeBytes += r.sizeBytes;
}

export interface V4ii {
  key: string;
  label: string;

  isAdmin: boolean;
  tally: V4if;
}

export interface V4ij {
  files: V43g[];
  tally: V4if;
}

export interface V4ik {
  key: string;
  hopSo: string;
  hoSoSo: string;
  tieuDe: string;

  inTree: boolean;
  cells: Map<string, V4ij>;
  tally: V4if;
}

export interface V4il {
  key: string;
  label: string;
  rows: V4ik[];
  tally: V4if;
}

export interface V4im {
  columns: V4ii[];
  groups: V4il[];

  unmatched: number;

  hiddenRows: number;
}

export interface V4in {
  phong: V40v[];
  hoSo: V40d[];
  collected: V43g[];

  adminUsername?: string | null;

  hideEmptyRows?: boolean;
}

const V4io = new Intl.Collator("vi", { numeric: true });

const V4ip = (V1l6: string, V1l7: string) => `${V1l6}/${V1l7}`;
const V4iq = (V1l8: string, V1l9: string) => `${V1l8}\u0000${V1l9}`;

export interface V4ir {
  byKey: Map<string, V40d>;

  byRef: Map<string, V40d[]>;
}

export interface V4is {
  phongMa: string | null;
  hoSoRef: string | null;
}

export function V4it(V1la: V40d[]): V4ir {
  const V1lb = new Map<string, V40d>();
  const V1lc = new Map<string, V40d[]>();
  for (const h of V1la) {
    const V1ld = V4ip(h.hopSo, h.hoSoSo);
    V1lb.set(V4iq(h.phongId ?? "", V1ld), h);
    V1lc.set(V1ld, [...(V1lc.get(V1ld) ?? []), h]);
  }
  return { byKey: V1lb, byRef: V1lc };
}

export function V4iu(r: V4is, V1le: V4ir): V40d | null {
  const V1lf = r.hoSoRef;
  if (!V1lf) return null;
  if (r.phongMa) return V1le.byKey.get(V4iq(r.phongMa, V1lf)) ?? null;

  const V1lg = V1le.byRef.get(V1lf);
  return V1lg && V1lg.length === 1 ? V1lg[0] : null;
}

export function V4iv(V1lh: V4in): V4im {
  const { phong: V1li, hoSo: V1lj, collected, adminUsername: V1lk, hideEmptyRows: V1ll } = V1lh;

  const V1lm = V1lk ?? "";
  const V1ln = new Map<string, V4if>([[V1lm, V4ig()]]);
  for (const r of collected) {
    const k = r.sourceUsername ?? "";
    let t = V1ln.get(k);
    if (!t) {
      t = V4ig();
      V1ln.set(k, t);
    }
    V4ih(t, r);
  }

  const columns: V4ii[] = [...V1ln.entries()]
    .map(([V1lo, V1lp]) => ({
      key: V1lo,
      label: V1lo === V1lm ? V1lo || "TcOheSBxdeG6o24gdHLhu4s=" : V1lo || V4i8,
      isAdmin: V1lo === V1lm,
      tally: V1lp,
    }))
    .sort((a, b) =>
      a.isAdmin !== b.isAdmin
        ? Number(b.isAdmin) - Number(a.isAdmin)
        : a.label.localeCompare(b.label, "vi"),
    );

  const V1lq = new Map(V1li.map((p) => [p.maDuAn, p.tenDuAn || p.maDuAn]));
  const V1lr = new Map<string, V4il>();
  const V1ls = new Map<string, V4ik>();
  const tree = V4it(V1lj);

  const V1lt = (V1lu: string, V1lv: string): V4il => {
    let g = V1lr.get(V1lu);
    if (!g) {
      g = { key: V1lu, label: V1lv, rows: [], tally: V4ig() };
      V1lr.set(V1lu, g);
    }
    return g;
  };
  const V1lw = (
    g: V4il,
    V1lx: string,
    V1ly: string,
    V1lz: string,
    V1m0: string,
    V1m1: boolean,
  ): V4ik => {
    let row = V1ls.get(V1lx);
    if (!row) {
      row = {
        key: V1lx,
        hopSo: V1ly,
        hoSoSo: V1lz,
        tieuDe: V1m0,
        inTree: V1m1,
        cells: new Map(),
        tally: V4ig(),
      };
      V1ls.set(V1lx, row);
      g.rows.push(row);
    }
    return row;
  };

  for (const h of V1lj) {
    const V1m2 = h.phongId ?? "";
    const g = V1lt(V1m2, V1m2 ? (V1lq.get(V1m2) ?? V1m2) : V4i9);
    const V1m3 = V4iq(V1m2, V4ip(h.hopSo, h.hoSoSo));
    V1lw(g, V1m3, h.hopSo, h.hoSoSo, h.tieuDe, true);
  }

  let V1m4 = 0;
  for (const r of collected) {
    const h = V4iu(r, tree);
    let V1m5 = h ? V4iq(h.phongId ?? "", V4ip(h.hopSo, h.hoSoSo)) : null;
    if (V1m5 === null) {
      V1m4 += 1;
      const g = V1lt(V4ia, V4ib);
      const V1m6 = r.hoSoRef ?? "—";
      V1m5 = V4iq(V4ia, `${r.phongMa ?? "?"}\u0000${V1m6}`);
      const [V1m7, V1m8] = V1m6.includes("/") ? V1m6.split("/") : [V1m6, ""];
      V1lw(
        g,
        V1m5,
        V1m7,
        V1m8,
        r.phongMa ? `UGjDtG5nIA==${r.phongMa}` : "S2jDtG5nIHLDtSBwaMO0bmc=",
        false,
      );
    }
    const row = V1ls.get(V1m5)!;
    const V1m9 = r.sourceUsername ?? "";
    let cell = row.cells.get(V1m9);
    if (!cell) {
      cell = { files: [], tally: V4ig() };
      row.cells.set(V1m9, cell);
    }
    cell.files.push(r);
    V4ih(cell.tally, r);
    V4ih(row.tally, r);
  }

  for (const row of V1ls.values()) {
    for (const cell of row.cells.values()) {
      cell.files.sort((a, b) => V4io.compare(a.filename, b.filename) || a.id - b.id);
    }
  }

  for (const g of V1lr.values()) {
    for (const row of g.rows) {
      g.tally.total += row.tally.total;
      g.tally.pending += row.tally.pending;
      g.tally.forwarded += row.tally.forwarded;
      g.tally.rejected += row.tally.rejected;
      g.tally.sizeBytes += row.tally.sizeBytes;
    }
    g.rows.sort(
      (a, b) =>
        a.hopSo.localeCompare(b.hopSo, "vi", { numeric: true }) ||
        a.hoSoSo.localeCompare(b.hoSoSo, "vi", { numeric: true }),
    );
  }

  let V1ma = 0;
  const V1mb: V4il[] = [];
  for (const g of V1lr.values()) {
    const V1mc = V1ll ? g.rows.filter((r) => r.tally.total > 0) : g.rows;
    V1ma += g.rows.length - V1mc.length;
    if (V1mc.length > 0) V1mb.push({ ...g, rows: V1mc });
  }

  V1mb.sort(
    (a, b) =>
      (a.key === V4ia ? 1 : 0) - (b.key === V4ia ? 1 : 0) || a.label.localeCompare(b.label, "vi"),
  );

  return { columns, groups: V1mb, unmatched: V1m4, hiddenRows: V1ma };
}

export function V4iw(t: V4if): V4ic {
  if (t.rejected > 0) return "rejected";
  if (t.pending > 0) return "pending";
  return "forwarded";
}

export function V4ix(t: V4if): string {
  const V1md = [`${t.total}IHTDoGkgbGnhu4d1`];
  if (t.forwarded > 0) V1md.push(`${t.forwarded}IMSRw6MgbMOqbiBtw6F5IGNo4bun`);
  if (t.pending > 0) V1md.push(`${t.pending}IGNo4budIMSR4bqpeQ==`);
  if (t.rejected > 0) V1md.push(`${t.rejected}IG3DoXkgY2jhu6cgdOG7qyBjaOG7kWk=`);
  return V1md.join(" · ");
}

export function V4iy(cell: V4ij): number[] {
  return cell.files.filter(V4ie).map((r) => r.id);
}
