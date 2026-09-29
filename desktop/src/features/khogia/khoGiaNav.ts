// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3xw } from "@/domain/types";

export interface V46l {
  key: V3xw;
  label: string;
}

export const KHOGIA_NAV: V46l[] = [
  { key: "kho", label: "Kho lưu trữ" },
  { key: "gia", label: "Giá lưu trữ" },
  { key: "khaithac", label: "Khai thác dữ liệu" },
  { key: "muontra", label: "Mượn – trả hồ sơ" },
];

export const V46m: V3xw[] = KHOGIA_NAV.map((i) => i.key);

export function V46n(Vdb: V3xw): string {
  return KHOGIA_NAV.find((i) => i.key === Vdb)?.label ?? "";
}
