// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3xv } from "@/domain/types";

export interface V46i {
  key: V3xv;
  label: string;
}

export const CATALOG_NAV: V46i[] = [
  { key: "phong", label: "UGjDtG5nIGzGsHUgdHLhu68=" },
  { key: "phongban", label: "UGjDsm5nIGJhbg==" },
  { key: "mang", label: "TeG6o25nIGPDtG5nIHZp4buHYw==" },
  { key: "thbq", label: "VGjhu51pIGjhuqFuIGLhuqNvIHF14bqjbg==" },
  { key: "ngonngu", label: "TmfDtG4gbmfhu68=" },
  { key: "loai", label: "TG/huqFpIHTDoGkgbGnhu4d1" },
  { key: "hoso", label: "SOG7kyBzxqEgKG3hu6VjIGzhu6VjKQ==" },
];

export const V46j: V3xv[] = CATALOG_NAV.map((i) => i.key);

export function V46k(Vda: V3xv): string {
  return CATALOG_NAV.find((i) => i.key === Vda)?.label ?? "";
}
