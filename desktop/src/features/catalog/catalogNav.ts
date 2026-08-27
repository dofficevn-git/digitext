// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3xv } from "@/domain/types";

export interface V46i {
  key: V3xv;
  label: string;
}

export const CATALOG_NAV: V46i[] = [
  { key: "phong", label: "Phông lưu trữ" },
  { key: "phongban", label: "Phòng ban" },
  { key: "mang", label: "Mảng công việc" },
  { key: "thbq", label: "Thời hạn bảo quản" },
  { key: "ngonngu", label: "Ngôn ngữ" },
  { key: "loai", label: "Loại tài liệu" },
  { key: "hoso", label: "Hồ sơ (mục lục)" },
];

export const V46j: V3xv[] = CATALOG_NAV.map((i) => i.key);

export function V46k(Vda: V3xv): string {
  return CATALOG_NAV.find((i) => i.key === Vda)?.label ?? "";
}
