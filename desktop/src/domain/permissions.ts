// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3xs, V3xt } from "./types";

export type V3yt = "read" | "operate" | "admin";

const V3yu: Record<V3xs, number> = {
  customer: 0,
  user: 1,
  admin: 2,
  super_admin: 3,
};

const V3yv: Record<V3yt, number> = { read: 0, operate: 1, admin: 2 };

export function V3yw(Ve: V3xs, Vf: V3yt): boolean {
  return V3yu[Ve] >= V3yv[Vf];
}

export type V3yx =
  | "dossier.read"
  | "dossier.create"
  | "dossier.update"
  | "dossier.delete"
  | "dossier.export"
  | "catalog.read"
  | "catalog.create"
  | "catalog.update"
  | "catalog.delete"
  | "catalog.import"
  | "catalog.export"
  | "scan.view"
  | "scan.execute"
  | "editor.view"
  | "editor.edit"
  | "upload.execute"
  | "sign.view"
  | "sign.execute"
  | "drive.view"
  | "drive.download"
  | "document.list"
  | "document.download"
  | "document.review"
  | "package.view"
  | "package.create"
  | "package.download"
  | "submission.view"
  | "submission.create"
  | "submission.review"
  | "submission.approve"
  | "record.view"
  | "record.create"
  | "access.view"
  | "access.request"
  | "access.review"
  | "disposal.view"
  | "disposal.review"
  | "disposal.decide"
  | "bk.view"
  | "bk.manage"
  | "tree.view"
  | "tree.assign"
  | "stats.view"
  | "stats.view_project"
  | "user.view"
  | "user.create"
  | "user.update"
  | "user.revoke"
  | "user.reset_password"
  | "user.grant_permissions"
  | "project.view"
  | "project.create"
  | "project.update"
  | "project.assign_admin"
  | "settings.view"
  | "settings.update"
  | "device.view"
  | "device.revoke";

export interface V3yy {
  perm: V3yx;
  label: string;
}

export interface V3yz {
  key: string;
  label: string;
  perms: V3yy[];
}

export const V3z0: V3yz[] = [
  {
    key: "dossier",
    label: "SOG7kyBzxqE=",
    perms: [
      { perm: "dossier.read", label: "WGVtIGjhu5Mgc8ah" },
      { perm: "dossier.create", label: "VOG6oW8gaOG7kyBzxqE=" },
      { perm: "dossier.update", label: "U+G7rWEgaOG7kyBzxqE=" },
      { perm: "dossier.delete", label: "WG/DoSBo4buTIHPGoQ==" },
      { perm: "dossier.export", label: "WHXhuqV0IGjhu5Mgc8ah" },
    ],
  },
  {
    key: "catalog",
    label: "RGFuaCBt4bulYw==",
    perms: [
      { perm: "catalog.read", label: "WGVtIGRhbmggbeG7pWM=" },
      { perm: "catalog.create", label: "VGjDqm0gbeG7pWM=" },
      { perm: "catalog.update", label: "U+G7rWEgbeG7pWM=" },
      { perm: "catalog.delete", label: "WG/DoSBt4bulYw==" },
      { perm: "catalog.import", label: "Tmjhuq1wIHThu6sgRXhjZWw=" },
      { perm: "catalog.export", label: "WHXhuqV0IGRhbmggbeG7pWM=" },
    ],
  },
  {
    key: "scan",
    label: "UXXDqXQ=",
    perms: [
      { perm: "scan.view", label: "TeG7nyBtw6BuIHF1w6l0" },
      { perm: "scan.execute", label: "VGjhu7FjIGhp4buHbiBxdcOpdA==" },
    ],
  },
  {
    key: "editor",
    label: "Q2jhu4luaCDhuqNuaA==",
    perms: [
      { perm: "editor.view", label: "TeG7nyB0csOsbmggY2jhu4luaCDhuqNuaA==" },
      { perm: "editor.edit", label: "Q2jhu4luaCBz4butYSDhuqNuaA==" },
    ],
  },
  {
    key: "upload",
    label: "VOG6o2kgbMOqbg==",
    perms: [{ perm: "upload.execute", label: "VOG6o2kgdMOgaSBsaeG7h3UgbMOqbiBtw6F5IGNo4bun" }],
  },
  {
    key: "sign",
    label: "S8O9IHPhu5E=",
    perms: [
      { perm: "sign.view", label: "WGVtIGvDvSBz4buR" },
      { perm: "sign.execute", label: "VGjhu7FjIGhp4buHbiBrw70gc+G7kQ==" },
    ],
  },
  {
    key: "drive",
    label: "S2hvIGLDoG4gZ2lhbw==",
    perms: [
      { perm: "drive.view", label: "Xem kho" },
      { perm: "drive.download", label: "VOG6o2kgdOG7qyBraG8=" },
    ],
  },
  {
    key: "document",
    label: "VMOgaSBsaeG7h3UgxJHDoyBu4buZcCAobcOheSBjaOG7pyk=",
    perms: [
      { perm: "document.list", label: "WGVtIGRhbmggc8OhY2ggdMOgaSBsaeG7h3U=" },
      { perm: "document.download", label: "VOG6o2kgdMOgaSBsaeG7h3UgduG7gQ==" },
      {
        perm: "document.review",
        label: "RHV54buHdCAvIHThu6sgY2jhu5FpIGvhur90IHF14bqjIGLDs2MgdMOhY2g=",
      },
    ],
  },
  {
    key: "package",
    label: "xJDDs25nIGfDs2kgU0lQL0FJUC9ESVA=",
    perms: [
      { perm: "package.view", label: "WGVtIGfDs2kgdGlu" },
      { perm: "package.create", label: "VOG6oW8gZ8OzaSB0aW4=" },
      { perm: "package.download", label: "VOG6o2kgZ8OzaSB0aW4=" },
    ],
  },
  {
    key: "submission",
    label: "VGh1IG7hu5lwIGzGsHUgdHLhu68gbOG7i2NoIHPhu60=",
    perms: [
      { perm: "submission.view", label: "WGVtIGzhuqduIG7hu5lw" },
      { perm: "submission.create", label: "xJDEg25nIGvDvSBu4buZcA==" },
      {
        perm: "submission.review",
        label: "WMOhYyBuaOG6rW4gLyB0aeG6v3Agbmjhuq1uIC8gY2jhuqVtIGjhu5Mgc8ah",
      },
      { perm: "submission.approve", label: "UGjDqiBkdXnhu4d0IC8gaG/DoG4gdGjDoG5o" },
    ],
  },
  {
    key: "record",
    label: "QmnDqm4gYuG6o24gKFBo4bulIGzhu6VjIFZJSSk=",
    perms: [
      { perm: "record.view", label: "WGVtIC8gdOG6o2kgYmnDqm4gYuG6o24=" },
      { perm: "record.create", label: "TOG6rXAgYmnDqm4gYuG6o24=" },
    ],
  },
  {
    key: "access",
    label: "S2hhaSB0aMOhYyBsxrB1IHRy4buvIHPhu5E=",
    perms: [
      {
        perm: "access.view",
        label: "WGVtIHnDqnUgY+G6p3UgLyDEkeG7jWMgYuG6o24gbmfGsOG7nWkgZMO5bmc=",
      },
      { perm: "access.request", label: "xJDEg25nIGvDvSB5w6p1IGPhuqd1IGtoYWkgdGjDoWM=" },
      { perm: "access.review", label: "WMOpdCBkdXnhu4d0IC8gcGjDrSAvIGPhuqVwIERJUA==" },
    ],
  },
  {
    key: "disposal",
    label: "SOG7p3kgdMOgaSBsaeG7h3UgaOG6v3QgZ2nDoSB0cuG7iw==",
    perms: [
      {
        perm: "disposal.view",
        label: "WGVtIGRhbmggbeG7pWMgLyDEkeG7o3QgaOG7p3kgKHRow6BuaCB2acOqbiBI4buZaSDEkeG7k25nKQ==",
      },
      {
        perm: "disposal.review",
        label: "UsOgIHNvw6F0IC8gbOG6rXAgxJHhu6N0IC8gdGjhuqltIMSR4buLbmg=",
      },
      { perm: "disposal.decide", label: "UXV54bq/dCDEkeG7i25oICsgdGhhbyB0w6FjIGjhu6d5" },
    ],
  },
  {
    key: "bk",
    label: "QuG6o28gcXXhuqNuICYgc2FvIGzGsHU=",
    perms: [
      { perm: "bk.view", label: "WGVtIGtp4buDbSB0cmEgLyBz4buVIHNhbyBsxrB1" },
      { perm: "bk.manage", label: "Q2jhuqF5IGtp4buDbSB0cmEgLyBnaGkgbmjhuq1uIHNhbyBsxrB1" },
    ],
  },
  {
    key: "tree",
    label: "Q8OieSB0aMawIG3hu6Vj",
    perms: [
      { perm: "tree.view", label: "WGVtIGPDonkgdGjGsCBt4bulYw==" },
      { perm: "tree.assign", label: "UGjDoXQgY8OieSB0aMawIG3hu6Vj" },
    ],
  },
  {
    key: "stats",
    label: "VGjhu5FuZyBrw6o=",
    perms: [
      { perm: "stats.view", label: "QsOhbyBjw6FvIC8geGVtIHPhu5EgbGnhu4d1" },
      { perm: "stats.view_project", label: "WGVtIHPhu5EgbGnhu4d1IHRvw6BuIGThu7Egw6Fu" },
    ],
  },
  {
    key: "user",
    label: "UXXhuqNuIGzDvSBuZ8aw4budaSBkw7luZw==",
    perms: [
      { perm: "user.view", label: "WGVtIGRhbmggc8OhY2g=" },
      { perm: "user.create", label: "VOG6oW8gdMOgaSBraG/huqNu" },
      { perm: "user.update", label: "U+G7rWEgdMOgaSBraG/huqNu" },
      { perm: "user.revoke", label: "VsO0IGhp4buHdSBob8OhIHTDoGkga2hv4bqjbg==" },
      { perm: "user.reset_password", label: "xJDhurd0IGzhuqFpIG3huq10IGto4bqpdQ==" },
      { perm: "user.grant_permissions", label: "UGjDom4gcXV54buBbiBjaG8gbmfGsOG7nWkga2jDoWM=" },
    ],
  },
  {
    key: "project",
    label: "UXXhuqNuIGzDvSBk4buxIMOhbg==",
    perms: [
      { perm: "project.view", label: "WGVtIGThu7Egw6Fu" },
      { perm: "project.create", label: "VOG6oW8gZOG7sSDDoW4=" },
      { perm: "project.update", label: "U+G7rWEgZOG7sSDDoW4=" },
      { perm: "project.assign_admin", label: "R8OhbiBhZG1pbiBwaOG7pSB0csOhY2g=" },
    ],
  },
  {
    key: "device",
    label: "VGhp4bq/dCBi4buL",
    perms: [
      { perm: "device.view", label: "WGVtIHRoaeG6v3QgYuG7iw==" },
      { perm: "device.revoke", label: "VGh1IGjhu5NpIHRoaeG6v3QgYuG7iw==" },
    ],
  },
  {
    key: "settings",
    label: "Q8OgaSDEkeG6t3Q=",
    perms: [
      { perm: "settings.view", label: "WGVtIGPDoGkgxJHhurd0" },
      { perm: "settings.update", label: "xJDhu5VpIGPDoGkgxJHhurd0" },
    ],
  },
];

export const V3z1: V3yx[] = V3z0.flatMap((g) => g.perms.map((p) => p.perm));

const V3z2: Record<V3yx, string> = Object.fromEntries(
  V3z0.flatMap((g) => g.perms.map((p) => [p.perm, p.label])),
) as Record<V3yx, string>;

export function V3z3(Vg: V3yx): string {
  return V3z2[Vg] ?? Vg;
}

const V3z4: Record<V3yx, number> = {
  "dossier.read": 0,
  "catalog.read": 0,
  "drive.view": 0,
  "drive.download": 0,
  "tree.view": 0,
  "settings.view": 0,

  "access.view": 0,
  "access.request": 0,
  "dossier.create": 1,
  "dossier.update": 1,
  "dossier.export": 1,
  "catalog.create": 1,
  "catalog.update": 1,
  "catalog.import": 1,
  "catalog.export": 1,
  "scan.view": 1,
  "scan.execute": 1,
  "editor.view": 1,
  "editor.edit": 1,
  "upload.execute": 1,
  "sign.view": 1,
  "sign.execute": 1,
  "stats.view": 1,
  "settings.update": 1,
  "package.view": 1,
  "package.create": 1,
  "package.download": 1,
  "submission.view": 1,
  "submission.create": 1,
  "submission.review": 2,
  "submission.approve": 2,
  "access.review": 2,

  "disposal.view": 1,
  "disposal.review": 2,
  "disposal.decide": 2,
  "bk.view": 2,
  "bk.manage": 2,
  "record.view": 1,
  "record.create": 1,
  "dossier.delete": 2,
  "catalog.delete": 2,
  "document.list": 2,
  "document.download": 2,
  "document.review": 2,
  "tree.assign": 2,
  "stats.view_project": 2,
  "user.view": 2,
  "user.create": 2,
  "user.update": 2,
  "user.revoke": 2,
  "user.reset_password": 2,
  "user.grant_permissions": 2,
  "project.view": 2,
  "project.create": 2,
  "project.update": 2,
  "device.view": 2,
  "device.revoke": 2,
  "project.assign_admin": 3,
};

export function V3z5(Vh: V3xs, Vi: V3yx): boolean {
  return V3yu[Vh] >= V3z4[Vi];
}

export function V3z6(Vj: V3xs): V3yx[] {
  return V3z1.filter((p) => V3z5(Vj, p));
}

const V3z7: Record<V3xt, V3yx> = {
  drive: "drive.view",
  catalog: "catalog.read",
  dossier: "dossier.read",
  khogia: "dossier.read",
  scan: "scan.view",
  editor: "editor.view",
  upload: "upload.execute",
  sign: "sign.view",
  users: "user.view",
  settings: "settings.view",

  package: "package.view",

  submission: "submission.view",

  records: "record.view",

  access: "access.view",

  disposal: "disposal.view",

  info: "access.view",

  bk: "bk.view",

  server: "document.list",

  report: "stats.view_project",
};

export function V3z8(Vk: V3yx[], view: V3xt): boolean {
  return Vk.includes(V3z7[view]);
}

export function V3z9(Vl: V3yx[] | undefined, Vm: V3yx): boolean {
  return !!Vl?.includes(Vm);
}

export function V3za(Vn: V3xs): string {
  switch (Vn) {
    case "super_admin":
      return "UXXhuqNuIHRy4buLIGjhu4cgdGjhu5FuZw==";
    case "admin":
      return "UXXhuqNuIHRy4buLIGThu7Egw6Fu";
    case "user":
      return "TmjDom4gdmnDqm4=";
    case "customer":
      return "S2jDoWNoIGjDoG5n";
  }
}
