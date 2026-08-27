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
    label: "S2hvIHTDoGkgbGnhu4d1",
    icon: "kho",
    gate: "drive",
    leaf: { kind: "view", view: "drive" },
  },
  {
    id: "chinhly",
    label: "Q2jhu4luaCBsw70=",
    icon: "chinhly",
    gate: "dossier",
    children: [
      {
        id: "danhmuc",
        label: "UXXhuqNuIGzDvSBkYW5oIG3hu6Vj",
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
        label: "TOG6rXAgaOG7kyBzxqEgxJFp4buHbiB04but",
        gate: "dossier",
        leaf: { kind: "hoso", pane: "laphoso" },
      },
      {
        id: "hethonghoa",
        label: "SOG7hyB0aOG7kW5nIGhvw6EgaOG7kyBzxqE=",
        gate: "dossier",
        leaf: { kind: "hoso", pane: "hethonghoa" },
      },
      {
        id: "capnhat",
        label: "Q+G6rXAgbmjhuq10IGPGoSBz4bufIGThu68gbGnhu4d1",
        gate: "dossier",
        leaf: { kind: "hoso", pane: "capnhat" },
      },
    ],
  },
  {
    id: "sohoa",
    label: "U+G7kSBob8Oh",
    icon: "sohoa",
    gate: "dossier",
    children: [
      {
        id: "taolap",
        label: "VOG6oW8gbOG6rXAgY8ahIHPhu58gZOG7ryBsaeG7h3U=",
        gate: "dossier",
        leaf: { kind: "hoso", pane: "sohoa" },
      },
      {
        id: "dongbo",
        label: "xJDhu5NuZyBi4buZIGThu68gbGnhu4d1",
        gate: "upload",
        leaf: { kind: "view", view: "upload" },
      },
      { id: "ocr", label: "OCR", gate: "dossier", leaf: { kind: "hoso", pane: "ocr" } },
      { id: "kyso", label: "S8O9IHPhu5E=", gate: "sign", leaf: { kind: "view", view: "sign" } },
      {
        id: "donggoi",
        label: "xJDDs25nIGfDs2kgdGhlbyB0acOqdSBjaHXhuqlu",
        gate: "package",
        leaf: { kind: "view", view: "package" },
      },
    ],
  },
  {
    id: "khaithac",
    label: "S2hhaSB0aMOhYyBo4buTIHPGoQ==",
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
    label: "VGjhu5FuZyBrw6o=",
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
        label: "QmnDqm4gYuG6o24gKFBo4bulIGzhu6VjIFZJSSk=",
        gate: "records" as V3xt,
        leaf: { kind: "view", view: "records" } as V46r,
      },
    ],
  },
  {
    id: "mayho",
    label: "TcOheSBjaOG7pw==",
    icon: "mayChu",
    gate: "server",
    leaf: { kind: "view", view: "server" },
  },
  {
    id: "huy",
    label: "SOG7p3kgdMOgaSBsaeG7h3UgaOG6v3QgZ2nDoSB0cuG7iw==",
    icon: "huy",
    gate: "disposal",
    leaf: { kind: "view", view: "disposal" },
  },
  {
    id: "baoquan",
    label: "QuG6o28gcXXhuqNuICYgc2FvIGzGsHU=",
    icon: "baoquan",
    gate: "bk",
    leaf: { kind: "view", view: "bk" },
  },
  {
    id: "users",
    label: "UXXhuqNuIGzDvSBuZ8aw4budaSBkw7luZw==",
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
