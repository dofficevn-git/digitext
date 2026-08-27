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
    group: "Chỉnh lý",
    items: [
      { key: "laphoso", label: "Lập hồ sơ điện tử" },
      { key: "hethonghoa", label: "Hệ thống hoá hồ sơ" },
      { key: "capnhat", label: "Cập nhật cơ sở dữ liệu" },
    ],
  },
  {
    group: "Số hoá",
    items: [
      { key: "sohoa", label: "Tạo lập CSDL" },
      { key: "kyso", label: "Ký số" },
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
