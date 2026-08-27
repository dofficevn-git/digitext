// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4f9, V4ej } from "@/api/docs";
import { V4vp } from "./serverTree";

export type V4w3 = "missing" | "invalid" | "not-pdfa";

export interface V4w4 {
  id: string;
  filename: string;

  folderPath: string;

  path: string;
  kind: V4w3;
  reason: string;
  status: string;
}

export function V4w5(V336: V4w3): string {
  switch (V336) {
    case "missing":
      return "VGhp4bq/dSB04buHcCBQREY=";
    case "invalid":
      return "UERGIGjhu49uZw==";
    case "not-pdfa":
      return "Q2jGsGEgxJHhuqF0IFBERi9BLTJi";
  }
}

export function V4w6(r: V4f9): V4w3 | null {
  if (r.state === "missing") return "missing";
  if (r.state === "invalid") return "invalid";
  if (r.state === "ok" && !r.pdfa2b) return "not-pdfa";
  return null;
}

export function V4w7(r: V4f9, V337: V4w3): string {
  const base =
    r.reason && r.reason.trim() !== ""
      ? r.reason.trim()
      : V337 === "missing"
        ? "S2jDtG5nIHTDrG0gdGjhuqV5IHThu4dwIFBERiAyIGzhu5twIHRyw6puIGtobyBtw6F5IGNo4bun"
        : V337 === "invalid"
          ? "VOG7h3AgUERGIDIgbOG7m3Agc2FpIGPhuqV1IHRyw7pj"
          : "R2jDqXAgYuG7n2kgYuG6o24gY8WpIOKAlCB0aGnhur91IFhNUCBwZGZhaWQvT3V0cHV0SW50ZW50L0lEL2ZvbnQgbmjDum5n";
  const V338 = [base];
  if (r.expectedPages !== null && r.pages !== r.expectedPages) {
    V338.push(`xJHhu41jIMSRxrDhu6NjIA==${r.pages}/${r.expectedPages} trang`);
  }
  if (V337 !== "missing" && r.pagesWithText === 0 && r.pages > 0) {
    V338.push("a2jDtG5nIHRyYW5nIG7DoG8gY8OzIGzhu5twIGNo4buv");
  }
  return V338.join(" · ");
}

const V4w8: Record<V4w3, number> = { missing: 0, invalid: 1, "not-pdfa": 2 };

export function V4w9(V339: readonly V4f9[], docs: readonly V4ej[]): V4w4[] {
  const V33a = new Map(docs.map((d) => [d.id, d]));
  const V33b: V4w4[] = [];
  for (const r of V339) {
    const V33c = V4w6(r);
    if (V33c === null) continue;
    const d = V33a.get(r.id);
    const V33d = d ? V4vp(d) : "";
    V33b.push({
      id: r.id,
      filename: r.filename,
      folderPath: V33d,
      path: V33d === "" ? r.filename : `${V33d}/${r.filename}`,
      kind: V33c,
      reason: V4w7(r, V33c),
      status: r.status,
    });
  }
  V33b.sort((a, b) => V4w8[a.kind] - V4w8[b.kind] || a.path.localeCompare(b.path, "vi"));
  return V33b;
}
