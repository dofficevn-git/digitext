// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3xv, V3xu, V3xw, V3xx, V3xt } from "@/domain/types";
import type { V41v } from "@/components/Icon";
import { CATALOG_NAV } from "@/features/catalog/catalogNav";
import { KHOGIA_NAV } from "@/features/khogia/khoGiaNav";
import { REPORT_NAV } from "@/features/report/reportNav";

export type V46r =
  | { kind: "view"; view: V3xt }
  | { kind: "hoso"; pane: V3xu }
  | { kind: "catalog"; tab: V3xv }
  | { kind: "khogia"; tab: V3xw }
  | { kind: "report"; tab: V3xx };

export interface V46s {
  id: string;
  label: string;

  icon?: V41v;

  gate: V3xt;
  leaf?: V46r;
  children?: V46s[];
}

export const NAV_TREE: V46s[] = [
  {
    id: "khotailieu",
    label: "Kho tài liệu",
    icon: "kho",
    gate: "drive",
    leaf: { kind: "view", view: "drive" },
  },
  {
    id: "chinhly",
    label: "Chỉnh lý",
    icon: "chinhly",
    gate: "dossier",
    children: [
      {
        id: "danhmuc",
        label: "Quản lý danh mục",
        icon: "danhmuc",
        gate: "catalog",
        children: CATALOG_NAV.map((c) => ({
          id: `catalog:${c.key}`,
          label: c.label,
          gate: "catalog" as V3xt,
          leaf: { kind: "catalog", tab: c.key } as V46r,
        })),
      },
      {
        id: "laphoso",
        label: "Lập hồ sơ điện tử",
        gate: "dossier",
        leaf: { kind: "hoso", pane: "laphoso" },
      },
      {
        id: "hethonghoa",
        label: "Hệ thống hoá hồ sơ",
        gate: "dossier",
        leaf: { kind: "hoso", pane: "hethonghoa" },
      },
      {
        id: "capnhat",
        label: "Cập nhật cơ sở dữ liệu",
        gate: "dossier",
        leaf: { kind: "hoso", pane: "capnhat" },
      },
    ],
  },
  {
    id: "sohoa",
    label: "Số hoá",
    icon: "sohoa",
    gate: "dossier",
    children: [
      {
        id: "taolap",
        label: "Tạo lập cơ sở dữ liệu",
        gate: "dossier",
        leaf: { kind: "hoso", pane: "sohoa" },
      },
      {
        id: "dongbo",
        label: "Đồng bộ dữ liệu",
        gate: "upload",
        leaf: { kind: "view", view: "upload" },
      },
      { id: "ocr", label: "OCR", gate: "dossier", leaf: { kind: "hoso", pane: "ocr" } },
      { id: "kyso", label: "Ký số", gate: "sign", leaf: { kind: "view", view: "sign" } },
      {
        id: "donggoi",
        label: "Đóng gói theo tiêu chuẩn",
        gate: "package",
        leaf: { kind: "view", view: "package" },
      },
    ],
  },
  {
    id: "khaithac",
    label: "Khai thác hồ sơ",
    icon: "khaithac",
    gate: "khogia",
    children: KHOGIA_NAV.map((k) => ({
      id: `khogia:${k.key}`,
      label: k.label,
      gate: "khogia" as V3xt,
      leaf: { kind: "khogia", tab: k.key } as V46r,
    })),
  },
  {
    id: "thongke",
    label: "Thống kê",
    icon: "thongke",
    gate: "report",
    children: [
      ...REPORT_NAV.map((r) => ({
        id: `report:${r.key}`,
        label: r.label,
        gate: "report" as V3xt,
        leaf: { kind: "report", tab: r.key } as V46r,
      })),

      {
        id: "bienban",
        label: "Biên bản (Phụ lục VII)",
        gate: "records" as V3xt,
        leaf: { kind: "view", view: "records" } as V46r,
      },
    ],
  },
  {
    id: "mayho",
    label: "Máy chủ",
    icon: "mayChu",
    gate: "server",
    leaf: { kind: "view", view: "server" },
  },
  {
    id: "huy",
    label: "Hủy tài liệu hết giá trị",
    icon: "huy",
    gate: "disposal",
    leaf: { kind: "view", view: "disposal" },
  },
  {
    id: "baoquan",
    label: "Bảo quản & sao lưu",
    icon: "baoquan",
    gate: "bk",
    leaf: { kind: "view", view: "bk" },
  },
  {
    id: "users",
    label: "Quản lý người dùng",
    icon: "nguoiDung",
    gate: "users",
    leaf: { kind: "view", view: "users" },
  },
];

export function V46t(Vdd: V46s[] = NAV_TREE): V46r[] {
  const Vde: V46r[] = [];
  const Vdf = (Vdg: V46s[]) => {
    for (const n of Vdg) {
      if (n.leaf) Vde.push(n.leaf);
      if (n.children) Vdf(n.children);
    }
  };
  Vdf(Vdd);
  return Vde;
}
