// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3xw } from "@/domain/types";

export interface V46l {
  key: V3xw;
  label: string;
}

export const KHOGIA_NAV: V46l[] = [
  { key: "kho", label: "S2hvIGzGsHUgdHLhu68=" },
  { key: "gia", label: "R2nDoSBsxrB1IHRy4buv" },
  { key: "khaithac", label: "S2hhaSB0aMOhYyBk4buvIGxp4buHdQ==" },
];

export const V46m: V3xw[] = KHOGIA_NAV.map((i) => i.key);

export function V46n(Vdb: V3xw): string {
  return KHOGIA_NAV.find((i) => i.key === Vdb)?.label ?? "";
}
