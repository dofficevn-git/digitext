// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4ld } from "@/api/khaithac";
import { V4lk, V4ll } from "@/api/khaithac";
import { isTauri } from "@/api/ipc";
import { V4e0 } from "@/api/exportFile";
import { V4e3, V4e5 } from "@/features/dossier/excel";
import { V4m6, V4m5 } from "./logic";
import { V4mq, V4mx, V4ms, V4mk, V4mt, V4mm, V4mu, V4mp } from "./khaithacFormat";

export type V4my = (V4mz?: number) => string;

class V4n0 extends Error {
  constructor() {
    super(
      "S2jDtG5nIGPDsyBk4buvIGxp4buHdSBwaMO5IGjhu6NwIGLhu5kgbOG7jWMgxJHhu4Mga+G6v3QgeHXhuqV0Lg==",
    );
    this.name = "NoDataError";
  }
}

function V4n1(V1wq: string, V1wr: string): string {
  return (
    `<!doctype html><html><head><meta charset="utf-8"><title>${V4m6(V1wq)}</title>` +
    `<style>${V4mq}</style></head><body>${V1wr}` +
    `<script>window.onload=function(){setTimeout(function(){window.print()},350)}</script>` +
    `</body></html>`
  );
}

function V4n2(V1ws: string, V1wt: string): void {
  const V1wu = V4n1(V1ws, V1wt);
  if (isTauri()) {
    void V4e0(V1wu);
    return;
  }
  const V1wv = document.createElement("iframe");
  V1wv.setAttribute("aria-hidden", "true");
  V1wv.style.cssText = "position:fixed;right:0;bottom:0;width:0;height:0;border:0;";
  document.body.appendChild(V1wv);
  const doc = V1wv.contentDocument;
  const V1ww = V1wv.contentWindow;
  if (!doc || !V1ww) {
    V1wv.remove();
    return;
  }
  doc.open();
  doc.write(V1wu);
  doc.close();
  const V1wx = () => setTimeout(() => V1wv.remove(), 1500);
  V1ww.onafterprint = V1wx;
  setTimeout(() => {
    V1ww.focus();
    V1ww.print();
    V1wx();
  }, 200);
}

export async function V4n3(filter: V4ld): Promise<number> {
  const V1wy = await V4lk(filter);
  if (V1wy.length === 0) throw new V4n0();
  await V4e3(V4m5(V1wy), "Metadata", "Export_Metadata.xlsx");
  return V1wy.length;
}

export async function V4n4(filter: V4ld): Promise<number> {
  const V1wz = await V4lk(filter);
  if (V1wz.length === 0) throw new V4n0();
  await V4e5([V4mk(V1wz)], "MucLucHoSo.xlsx");
  return V1wz.length;
}

export async function V4n5(filter: V4ld): Promise<number> {
  const [V1x0, V1x1] = await Promise.all([V4lk(filter), V4ll(filter)]);
  if (V1x1.length === 0) throw new V4n0();
  const V1x2 = V4mm(V1x0, V1x1);
  await V4e5(V1x2, "MucLucVanBan.xlsx");
  return V1x2.length;
}

export async function V4n6(filter: V4ld, V1x3: V4my): Promise<number> {
  const V1x4 = await V4lk(filter);
  if (V1x4.length === 0) throw new V4n0();
  await V4e5([V4mp(V1x4, V1x3)], "PhuLuc.xlsx");
  return V1x4.length;
}

export async function V4n7(filter: V4ld, V1x5: string): Promise<number> {
  const V1x6 = await V4lk(filter);
  if (V1x6.length === 0) throw new V4n0();
  V4n2("TeG7pWMgbOG7pWMgaOG7kyBzxqE=", V4ms(V1x6, V1x5));
  return V1x6.length;
}

export async function V4n8(filter: V4ld): Promise<number> {
  const [V1x7, V1x8] = await Promise.all([V4lk(filter), V4ll(filter)]);
  if (V1x8.length === 0) throw new V4n0();
  V4n2("TeG7pWMgbOG7pWMgdsSDbiBi4bqjbg==", V4mt(V1x7, V1x8));
  return V1x8.length;
}

export async function V4n9(filter: V4ld, V1x9: V4my): Promise<number> {
  const V1xa = await V4lk(filter);
  if (V1xa.length === 0) throw new V4n0();
  V4n2("UGjhu6UgbOG7pWM=", V4mu(V1xa, V1x9));
  return V1xa.length;
}

export async function V4na(filter: V4ld, V1xb: string): Promise<number> {
  const V1xc = await V4lk(filter);
  if (V1xc.length === 0) throw new V4n0();
  V4n2("TmjDo24gaOG7mXA=", V4mx(V1xc, V1xb));
  return V1xc.length;
}
