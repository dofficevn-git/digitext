// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3yr } from "@/domain/types";

export interface V4ap {
  batDau: string;
  ketThuc: string;
}

const V4aq = /\d{4}-\d{1,2}-\d{1,2}|\d{1,2}\/\d{1,2}\/\d{4}|\d{1,2}\/\d{4}|\d{4}/g;

function V4ar(s: string): string {
  const m = /^(\d{4})-(\d{1,2})-(\d{1,2})$/.exec(s);
  return m ? `${Number(m[3])}/${Number(m[2])}/${m[1]}` : s;
}

export function V4as(Vq6?: string): V4ap {
  const Vq7 = (Vq6 ?? "").trim();
  if (!Vq7) return { batDau: "", ketThuc: "" };
  const Vq8 = Vq7.match(V4aq);
  if (!Vq8 || Vq8.length === 0) return { batDau: Vq7, ketThuc: "" };
  const Vq9 = V4ar(Vq8[0]);
  const Vqa = Vq8.length > 1 ? V4ar(Vq8[Vq8.length - 1]) : "";
  return { batDau: Vq9, ketThuc: Vqa };
}

export function V4at(h: Pick<V3yr, "thoiGian" | "ngayBatDau" | "ngayKetThuc">): V4ap {
  const Vqb = V4as(h.thoiGian);
  const Vqc = (h.ngayBatDau ?? "").trim();
  const Vqd = (h.ngayKetThuc ?? "").trim();
  return {
    batDau: Vqc ? V4ar(Vqc) : Vqb.batDau,
    ketThuc: Vqd ? V4ar(Vqd) : Vqb.ketThuc,
  };
}

export function V4au(Vqe?: string, Vqf?: string): string {
  const Vqg = (Vqe ?? "").trim();
  const Vqh = (Vqf ?? "").trim();
  if (Vqg && Vqh) return `${Vqg} - ${Vqh}`;
  return Vqg || Vqh;
}

export function V4av(s?: string): string {
  const t = (s ?? "").trim();
  const Vqi = /^(\d{4})-(\d{1,2})-(\d{1,2})$/.exec(t);
  if (Vqi) return `${Vqi[1]}-${Vqi[2].padStart(2, "0")}-${Vqi[3].padStart(2, "0")}`;
  const Vqj = /^(\d{1,2})\/(\d{1,2})\/(\d{4})$/.exec(t);
  if (Vqj) return `${Vqj[3]}-${Vqj[2].padStart(2, "0")}-${Vqj[1].padStart(2, "0")}`;
  return t;
}

export function V4aw(h: V3yr): V3yr {
  const Vqk = V4av(h.ngayBatDau);
  const Vql = V4av(h.ngayKetThuc);
  return {
    ...h,
    ngayBatDau: Vqk || undefined,
    ngayKetThuc: Vql || undefined,
    thoiGian: V4au(V4ar(Vqk), V4ar(Vql)),
  };
}

const V4ax = new Intl.Collator("vi", { numeric: true, sensitivity: "base" });

function V4ay(a?: string, b?: string): number {
  const x = (a ?? "").trim();
  const y = (b ?? "").trim();
  if (!x && !y) return 0;
  if (!x) return 1;
  if (!y) return -1;
  return V4ax.compare(x, y);
}

export function V4az(a: V3yr, b: V3yr): number {
  const hop = V4ay(a.hopSo, b.hopSo);
  if (hop !== 0) return hop;
  const Vqm = V4ay(a.hoSoSo, b.hoSoSo);
  if (Vqm !== 0) return Vqm;
  return a.id - b.id;
}

export function V4b0(Vqn: V3yr[]): V3yr[] {
  return [...Vqn].sort(V4az);
}
