// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4ew, V4ey } from "@/api/docs";

function V4jx(V1nt: number, V1nu: number): number {
  if (V1nu <= 0) return 0;
  return Math.max(0, Math.min(100, Math.round((100 * V1nt) / V1nu)));
}

export function V4jy(p: V4ey): number {
  return V4jx(p.pagesOcrDone, p.pagesTotal);
}

export function V4jz(p: { docsPdf2Done: number; docsOcrDone: number }): number {
  return V4jx(p.docsPdf2Done, p.docsOcrDone);
}

export function V4k0(p: { docsFieldsDone?: number; docs: number }): number {
  return V4jx(p.docsFieldsDone ?? 0, p.docs);
}

export function V4k1(status: string): string {
  switch (status) {
    case "uploaded":
      return "xJDDoyB04bqjaSBsw6pu";
    case "ocr_pending":
      return "Q2jhu50gT0NS";
    case "ocr_done":
      return "OCR xong";
    case "to_sign":
      return "Q2jhu50ga8O9";
    case "signing":
      return "xJBhbmcga8O9";
    case "signed":
      return "xJDDoyBrw70=";
    case "sign_rejected":
      return "S8O9IGzhu5dp";
    case "archived":
      return "xJDDoyBsxrB1IHRy4buv";
    case "disposed":
      return "xJDDoyBodeG7tw==";
    default:
      return status;
  }
}

export function V4k2(r: V4ew): string {
  const V1nv: string[] = [];
  if (r.skippedNoPageCount > 0)
    V1nv.push(`${r.skippedNoPageCount}IHTDoGkgbGnhu4d1IGNoxrBhIGJp4bq/dCBz4buRIHRyYW5n`);
  if (r.skippedPageFailed > 0)
    V1nv.push(
      `${r.skippedPageFailed}IHTDoGkgbGnhu4d1IGjhu49uZyB2xKluaCB2aeG7hW4gKHPhu61hIG5ndXnDqm4gbmjDom4gcuG7k2kgYuG6pW0gIlRo4butIGzhuqFpIHTDoGkgbGnhu4d1IGjhu49uZyIp`,
    );
  const V1nw = V1nv.length > 0 ? `IELhu48gcXVhIA==${V1nv.join("; ")}.` : "";
  const V1nx =
    r.documents > 0
      ? `xJDDoyDEkcawYSA=${r.documents}IHTDoGkgbGnhu4d1IHbDoG8gaMOgbmcgxJHhu6NpIGLDs2MgdMOhY2ggMTYgdHLGsOG7nW5nIHRyw6puIG3DoXkgY2jhu6cu`
      : "S2jDtG5nIGPDsyB0w6BpIGxp4buHdSBuw6BvIGPhuqduIGLDs2MgdMOhY2ggMTYgdHLGsOG7nW5nICjEkcOjIGLDs2MgeG9uZyBob+G6t2MgxJFhbmcgY2jhu50geOG7rSBsw70pLg==";
  return V1nx + V1nw;
}
