// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3xx } from "@/domain/types";

export interface V46o {
  key: V3xx;
  label: string;
}

export const REPORT_NAV: V46o[] = [
  { key: "sohoa", label: "Tổng kết số hoá" },
  { key: "chinhly", label: "Tổng kết chỉnh lý" },
  { key: "nhanvien", label: "Tổng kết nhân viên" },
  { key: "phanmem", label: "Tổng kết phần mềm" },
];

export const V46p: V3xx[] = REPORT_NAV.map((i) => i.key);

export function V46q(Vdc: V3xx): string {
  return REPORT_NAV.find((i) => i.key === Vdc)?.label ?? "";
}
