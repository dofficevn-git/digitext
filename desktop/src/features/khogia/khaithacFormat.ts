// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4le, V4lg } from "@/api/khaithac";
import { V4m6 } from "./logic";

export function V4m9(V1uz?: string): string {
  if (!V1uz) return "";
  const m = /^(\d{4})-(\d{1,2})-(\d{1,2})/.exec(V1uz.trim());
  if (!m) return V1uz;
  const p = (s: string) => s.padStart(2, "0");
  return `${p(m[3])}/${p(m[2])}/${m[1]}`;
}

export type V4ma = string | number;
export type V4mb = V4ma[][];
export interface V4mc {
  name: string;
  aoa: V4mb;
  merges?: string[];
}

export const V4md = [
  "SOG7mXAgc+G7kQ==",
  "SOG7kyBzxqEgc+G7kQ==",
  "VGnDqnUgxJHhu4EgaOG7kyBzxqE=",
  "TmfDoHkgdGjDoW5nIELEkC1OZ8OgeSB0aMOhbmcgS1Q=",
  "U+G7kSB04bud",
  "VGjhu51pIGjhuqFuIGLhuqNvIHF14bqjbg==",
  "R2hpIGNow7o=",
];
export const V4me = [
  "STT",
  "U+G7kSBrw70gaGnhu4d1IFZC",
  "TmfDoHkgdGjDoW5nIFZC",
  "VMOhYyBnaeG6ow==",
  "VHLDrWNoIHnhur91IG7hu5lpIGR1bmc=",
  "VOG7nSBz4buR",
  "R2hpIGNow7o=",
];
export const V4mf = [
  "STT",
  "TuG7mWkgZHVuZw==",
  "VOG7qyBo4buTIHPGoQ==",
  "xJDhur9uIGjhu5Mgc8ah",
  "VOG7qyBo4buZcCBz4buR",
  "xJDhur9uIGjhu5lwIHPhu5E=",
  "VHJhbmcgbeG7pWMgbOG7pWM=",
  "R2hpIGNow7o=",
];

export function V4mg(r: Pick<V4le, "thoiGianBatDau" | "thoiGianKetThuc">): string {
  const V1v0 = V4m9(r.thoiGianBatDau);
  const V1v1 = V4m9(r.thoiGianKetThuc);
  if (V1v0 && V1v1) return `${V1v0} - ${V1v1}`;
  return V1v0 || V1v1;
}

function V4mh(V1v2?: string): number {
  const m = /(\d{4})/.exec(V1v2 ?? "");
  return m ? Number(m[1]) : 0;
}

export function V4mi(V1v3: V4le[]): { from: number; to: number } {
  const V1v4: number[] = [];
  for (const r of V1v3) {
    for (const y of [V4mh(r.thoiGianBatDau), V4mh(r.thoiGianKetThuc)]) if (y > 0) V1v4.push(y);
  }
  if (V1v4.length === 0) return { from: 0, to: 0 };
  return { from: Math.min(...V1v4), to: Math.max(...V1v4) };
}

const V4mj = (s?: string): number | null => {
  const m = /\d+/.exec(s ?? "");
  return m ? Number(m[0]) : null;
};

export function V4mk(V1v5: V4le[]): V4mc {
  const V1v6: V4mb = [
    ["TeG6qlUgWFXhuqRUIEhPw4BOIFRISeG7hk4gVEhFTyBT4buQIEjhu5IgU8ag"],
    [],
    [...V4md],
    V4md.map((_V1v7, i) => i + 1),
    ...V1v5.map((r) => [
      r.hopTaiLieuSo ?? "",
      r.hososo,
      r.tieuDeHs,
      V4mg(r),
      r.soTo ?? "",
      r.thoiHanBaoQuanStr ?? "",
      r.ghiChu ?? "",
    ]),
  ];
  return { name: "MLHS", aoa: V1v6, merges: ["A1:G1"] };
}

function V4ml(base: string, V1v8: Set<string>): string {
  const n =
    (base || "HS")
      .replace(/[:\\/?*[\]]/g, " ")
      .slice(0, 28)
      .trim() || "HS";
  let name = n;
  let k = 1;
  while (V1v8.has(name.toLowerCase())) name = `${n} ${++k}`.slice(0, 31);
  V1v8.add(name.toLowerCase());
  return name;
}

export function V4mm(V1v9: V4le[], V1va: V4lg[]): V4mc[] {
  const V1vb = new Map(V1v9.map((r) => [r.hoSoId, r]));
  const V1vc = new Map<number, V4lg[]>();
  for (const v of V1va) {
    if (!V1vc.has(v.dossierId)) V1vc.set(v.dossierId, []);
    V1vc.get(v.dossierId)!.push(v);
  }
  const V1vd = new Set<string>();
  const V1ve: V4mc[] = [];
  for (const [V1vf, V1vg] of V1vc) {
    const r = V1vb.get(V1vf);
    const hop = r?.hopTaiLieuSo ?? V1vg[0]?.hopSo ?? "";
    const so = r?.hososo ?? V1vg[0]?.hoSo ?? "";
    const V1vh = r?.tieuDeHs ?? "";
    const V1vi: V4mb = [
      ["TeG7pEMgTOG7pEMgVsSCTiBC4bqiTg=="],
      [],
      [`SOG7mXAgc+G7kTog${hop}`],
      [`SOG7kyBzxqEgc+G7kTog${so}`],
      [`VGnDqnUgxJHhu4EgaOG7kyBzxqE6IA==${V1vh}`],
      [],
      [...V4me],
      V4me.map((_V1vj, i) => i + 1),
      ...V1vg.map((v, i) => [
        i + 1,
        v.soKyHieu ?? "",
        V4m9(v.ngayThang) || (v.ngayThang ?? ""),
        v.coQuanBanHanh ?? "",
        v.trichYeu ?? "",
        v.trangSo ?? v.toSo ?? "",
        v.ghiChu ?? "",
      ]),
    ];
    V1ve.push({ name: V4ml(String(so), V1vd), aoa: V1vi, merges: ["A1:G1"] });
  }
  return V1ve;
}

export interface V4mn {
  noiDung: string;
  tuHoSo: string;
  denHoSo: string;
  tuHop: string;
  denHop: string;
}

export function V4mo(V1vk: V4le[], V1vl: (V1vm?: number) => string): V4mn[] {
  const V1vn: number[] = [];
  const V1vo = new Map<number, V4le[]>();
  for (const r of V1vk) {
    const V1vp = r.mangCvid ?? -1;
    if (!V1vo.has(V1vp)) {
      V1vo.set(V1vp, []);
      V1vn.push(V1vp);
    }
    V1vo.get(V1vp)!.push(r);
  }
  return V1vn.map((V1vq) => {
    const V1vr = V1vo.get(V1vq)!;
    const V1vs = V1vr.map((r) => V4mj(r.hopTaiLieuSo)).filter((n): n is number => n != null);
    const V1vt = V1vr[0];
    const last = V1vr[V1vr.length - 1];
    return {
      noiDung: V1vq === -1 ? "KENoxrBhIHBow6JuIG3huqNuZyk=" : V1vl(V1vq) || `TeG6o25nIA==${V1vq}`,
      tuHoSo: V1vt?.hososo ?? "",
      denHoSo: last?.hososo ?? "",
      tuHop: V1vs.length ? String(Math.min(...V1vs)) : (V1vt?.hopTaiLieuSo ?? ""),
      denHop: V1vs.length ? String(Math.max(...V1vs)) : (last?.hopTaiLieuSo ?? ""),
    };
  });
}

export function V4mp(V1vu: V4le[], V1vv: (V1vw?: number) => string): V4mc {
  const data = V4mo(V1vu, V1vv);
  const V1vx: V4mb = [
    [...V4mf],
    ...data.map((g, i) => [i + 1, g.noiDung, g.tuHoSo, g.denHoSo, g.tuHop, g.denHop, "", ""]),
  ];
  return { name: "PhuLuc", aoa: V1vx };
}

export const V4mq = `
  * { box-sizing: border-box; }
  body { font-family: "Times New Roman", serif; font-size: 13px; color:#000; margin:16px; }
  .rpt-head { text-align:center; margin-bottom:10px; }
  .rpt-head .co { font-weight:bold; text-transform:uppercase; }
  .rpt-head h2 { font-size:18px; margin:6px 0 2px; text-transform:uppercase; }
  .rpt-head .sub { font-style:italic; }
  table { width:100%; border-collapse:collapse; margin-top:6px; page-break-inside:auto; }
  th, td { border:1px solid #000; padding:4px 6px; vertical-align:top; }
  th { background:#eee; text-align:center; }
  td.c, th.c { text-align:center; }
  .colnum td { text-align:center; font-style:italic; }
  .hs-block { page-break-inside:avoid; margin-bottom:14px; }
  .hs-meta { margin:8px 0 2px; }
  .labels { display:flex; flex-wrap:wrap; gap:0; }
  .label { width:50%; height:48vh; border:1px solid #000; padding:14px; text-align:center;
           page-break-inside:avoid; display:flex; flex-direction:column; justify-content:center; }
  .label .co { font-weight:bold; text-transform:uppercase; font-size:15px; }
  .label .hop { font-size:56px; font-weight:bold; margin:14px 0; }
  .label .rng { font-size:15px; margin:4px 0; }
  @media print { body { margin:0; } }
`;

function V4mr(V1vy: string[], V1vz: V4ma[][]): string {
  const th = `<tr>${V1vy.map((h) => `<th>${V4m6(h)}</th>`).join("")}</tr>`;
  const V1w0 = `<tr class="colnum">${V1vy.map((_V1w1, i) => `<td>(${i + 1})</td>`).join("")}</tr>`;
  const body = V1vz.map((r) => `<tr>${r.map((c) => `<td>${V4m6(c)}</td>`).join("")}</tr>`).join("");
  return `<table><thead>${th}${V1w0}</thead><tbody>${body}</tbody></table>`;
}

export function V4ms(V1w2: V4le[], V1w3: string): string {
  const { from, to } = V4mi(V1w2);
  const V1w4 = from && to ? `VOG7qiBOxIJNIA==${from}IMSQ4bq+TiBOxIJNIA==${to}` : "";
  const V1w5 =
    `<div class="rpt-head"><div class="co">${V4m6(V1w3)}</div>` +
    `PGgyPk3hu6VjIGzhu6VjIGjhu5Mgc8ahPC9oMj4=` +
    `PGRpdiBjbGFzcz0ic3ViIj5QSMOUTkcgTMavVSBUUuG7riA=${V4m6(V1w3)}</div>` +
    (V1w4 ? `<div class="sub">${V1w4}</div>` : "") +
    `</div>`;
  const body = V1w2.map((r) => [
    r.hopTaiLieuSo ?? "",
    r.hososo,
    r.tieuDeHs,
    V4mg(r),
    r.soTo ?? "",
    r.thoiHanBaoQuanStr ?? "",
    r.ghiChu ?? "",
  ]);
  return V1w5 + V4mr(V4md, body);
}

export function V4mt(V1w6: V4le[], V1w7: V4lg[]): string {
  const V1w8 = new Map(V1w6.map((r) => [r.hoSoId, r]));
  const V1w9 = new Map<number, V4lg[]>();
  for (const v of V1w7) {
    if (!V1w9.has(v.dossierId)) V1w9.set(v.dossierId, []);
    V1w9.get(v.dossierId)!.push(v);
  }
  const V1wa: string[] = [];
  for (const [V1wb, V1wc] of V1w9) {
    const r = V1w8.get(V1wb);
    const hop = r?.hopTaiLieuSo ?? V1wc[0]?.hopSo ?? "";
    const so = r?.hososo ?? V1wc[0]?.hoSo ?? "";
    const V1wd = r?.tieuDeHs ?? "";
    const V1we = V1wc.map((v, i) => [
      i + 1,
      v.soKyHieu ?? "",
      V4m9(v.ngayThang) || (v.ngayThang ?? ""),
      v.coQuanBanHanh ?? "",
      v.trichYeu ?? "",
      v.trangSo ?? v.toSo ?? "",
      v.ghiChu ?? "",
    ]);
    V1wa.push(
      `PGRpdiBjbGFzcz0iaHMtYmxvY2siPjxkaXYgY2xhc3M9InJwdC1oZWFkIj48aDI+TeG7pWMgbOG7pWMgdsSDbiBi4bqjbjwvaDI+PC9kaXY+` +
        `PGRpdiBjbGFzcz0iaHMtbWV0YSI+SOG7mXAgc+G7kTog${V4m6(hop)}</div>` +
        `PGRpdiBjbGFzcz0iaHMtbWV0YSI+SOG7kyBzxqEgc+G7kTog${V4m6(so)}</div>` +
        `PGRpdiBjbGFzcz0iaHMtbWV0YSI+VGnDqnUgxJHhu4EgaOG7kyBzxqE6IA==${V4m6(V1wd)}</div>` +
        V4mr(V4me, V1we) +
        `</div>`,
    );
  }
  return V1wa.join("");
}

export function V4mu(V1wf: V4le[], V1wg: (V1wh?: number) => string): string {
  const data = V4mo(V1wf, V1wg);
  const body = data.map((g, i) => [
    i + 1,
    g.noiDung,
    g.tuHoSo,
    g.denHoSo,
    g.tuHop,
    g.denHop,
    "",
    "",
  ]);
  return (
    `PGRpdiBjbGFzcz0icnB0LWhlYWQiPjxoMj5IxrDhu5tuZyBk4bqrbiB0cmEgdMOsbSBt4bulYyBs4bulYzwvaDI+PC9kaXY+` +
    V4mr(V4mf, body)
  );
}

export interface V4mv {
  hopSo: string;
  tuHoSo: string;
  denHoSo: string;
  soHoSo: number;
  thbq: string;
}

export function V4mw(V1wi: V4le[]): V4mv[] {
  const V1wj: string[] = [];
  const V1wk = new Map<string, V4le[]>();
  for (const r of V1wi) {
    const hop = r.hopTaiLieuSo ?? "KGNoxrBhIHjhur9wIGjhu5lwKQ==";
    if (!V1wk.has(hop)) {
      V1wk.set(hop, []);
      V1wj.push(hop);
    }
    V1wk.get(hop)!.push(r);
  }
  return V1wj.map((hop) => {
    const V1wl = V1wk.get(hop)!;
    const V1wm = [...new Set(V1wl.map((r) => r.thoiHanBaoQuanStr).filter(Boolean) as string[])];
    return {
      hopSo: hop,
      tuHoSo: V1wl[0]?.hososo ?? "",
      denHoSo: V1wl[V1wl.length - 1]?.hososo ?? "",
      soHoSo: V1wl.length,
      thbq: V1wm.join(", "),
    };
  });
}

export function V4mx(V1wn: V4le[], V1wo: string): string {
  const V1wp = V4mw(V1wn)
    .map(
      (l) =>
        `<div class="label"><div class="co">${V4m6(V1wo)}</div>` +
        `PGRpdiBjbGFzcz0iaG9wIj5I4buYUCBT4buQIA==${V4m6(l.hopSo)}</div>` +
        `PGRpdiBjbGFzcz0icm5nIj5U4burIGjhu5Mgc8ahIA==${V4m6(l.tuHoSo)}IMSR4bq/biBo4buTIHPGoSA=${V4m6(l.denHoSo)}</div>` +
        `PGRpdiBjbGFzcz0icm5nIj5T4buRIGzGsOG7o25nIGjhu5Mgc8ahOiA=${V4m6(l.soHoSo)}</div>` +
        `PGRpdiBjbGFzcz0icm5nIj5UaOG7nWkgaOG6oW4gYuG6o28gcXXhuqNuOiA=${V4m6(l.thbq)}</div>` +
        `</div>`,
    )
    .join("");
  return `<div class="labels">${V1wp}</div>`;
}
