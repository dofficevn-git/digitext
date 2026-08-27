// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3xu } from "@/domain/types";

export interface V4za {
  key: V3xu;
  label: string;
}
export interface V4zb {
  group: string;
  items: V4za[];
}

export const V4zc: V4zb[] = [
  {
    group: "Q2jhu4luaCBsw70=",
    items: [
      { key: "laphoso", label: "TOG6rXAgaOG7kyBzxqEgxJFp4buHbiB04but" },
      { key: "hethonghoa", label: "SOG7hyB0aOG7kW5nIGhvw6EgaOG7kyBzxqE=" },
      { key: "capnhat", label: "Q+G6rXAgbmjhuq10IGPGoSBz4bufIGThu68gbGnhu4d1" },
    ],
  },
  {
    group: "U+G7kSBob8Oh",
    items: [
      { key: "sohoa", label: "VOG6oW8gbOG6rXAgQ1NETA==" },
      { key: "kyso", label: "S8O9IHPhu5E=" },
      { key: "ocr", label: "OCR" },
    ],
  },
];

export const V4zd: V3xu[] = V4zc.flatMap((g) => g.items.map((i) => i.key));

export function V4ze(V3fw: V3xu): string {
  for (const g of V4zc) {
    const V3fx = g.items.find((i) => i.key === V3fw);
    if (V3fx) return V3fx.label;
  }
  return "";
}
