// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3xx } from "@/domain/types";

export interface V46o {
  key: V3xx;
  label: string;
}

export const REPORT_NAV: V46o[] = [
  { key: "sohoa", label: "VOG7lW5nIGvhur90IHPhu5EgaG/DoQ==" },
  { key: "chinhly", label: "VOG7lW5nIGvhur90IGNo4buJbmggbMO9" },
  { key: "nhanvien", label: "VOG7lW5nIGvhur90IG5ow6JuIHZpw6pu" },
  { key: "phanmem", label: "VOG7lW5nIGvhur90IHBo4bqnbiBt4buBbQ==" },
];

export const V46p: V3xx[] = REPORT_NAV.map((i) => i.key);

export function V46q(Vdc: V3xx): string {
  return REPORT_NAV.find((i) => i.key === Vdc)?.label ?? "";
}
