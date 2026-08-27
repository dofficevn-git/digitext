// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4tq, V4tl } from "@/api/access";

export const V4u0: Record<V4tl["kind"], string> = {
  doc: "xJDhu41jIHRyw6puIEjhu4cgdGjhu5FuZyAoxJBp4buBdSAzNeKAkzM3KQ==",
  ban_sao: "QuG6o24gc2FvIGtow7RuZyB4w6FjIHRo4buxYyAoxJBp4buBdSA0MCk=",
  ban_sao_ky:
    "QuG6o24gc2FvIGPDsyB4w6FjIHRo4buxYyDigJQga8O9IHPhu5EgdHLhu7FjIHRp4bq/cCAoxJBp4buBdSA0MSk=",
  dip: "QuG6o24gc2FvIGPDsyB4w6FjIHRo4buxYyDigJQgZ8OzaSBESVAgKMSQaeG7gXUgNDIp",
};

export const V4u1: Record<V4tl["status"], string> = {
  submitted: "Q2jhu50geMOpdCBkdXnhu4d0",
  approved: "xJDDoyBkdXnhu4d0",
  refused: "VOG7qyBjaOG7kWk=",
  expired: "SOG6v3QgaOG6oW4gKMSRw6MgeMOzYSBi4bqjbiBuZ8aw4budaSBkw7luZyk=",
};

export const V4u2: Record<string, string> = {
  tai_khoan: "UXVhIHTDoGkga2hv4bqjbiB0cm9uZyBI4buHIHRo4buRbmc=",
  email: "UXVhIHRoxrAgxJFp4buHbiB04but",
  giay: "QuG6o24gZ2nhuqV5IHThuqFpIGzGsHUgdHLhu68gbOG7i2NoIHPhu60=",
};

export function V4u3(V2vy: V4tq): string[] {
  const V2vz: string[] = [];
  if (!V2vy.mucDich.trim()) {
    V2vz.push("UGjhuqNpIGtoYWkgbeG7pWMgxJHDrWNoIHPhu60gZOG7pW5nICjEkGnhu4F1IDM1LjQpLg==");
  }
  if (V2vy.documentIds.length === 0) {
    V2vz.push("Q2jhu41uIHThu5FpIHRoaeG7g3UgMDEgdMOgaSBsaeG7h3Uu");
  }
  if (V2vy.kind === "dip") {
    if (!V2vy.nguoiNhan?.trim()) {
      V2vz.push(
        "QuG6o24gc2FvIERJUCBwaOG6o2kga2hhaSBo4buNIHTDqm4gbmfGsOG7nWkgbmjhuq1uICjEkGnhu4F1IDQyLjIuYyku",
      );
    }
    if (!V2vy.soDinhDanh?.trim()) {
      V2vz.push(
        "QuG6o24gc2FvIERJUCBwaOG6o2kga2hhaSBtw6MgxJHhu4tuaCBkYW5oIG5nxrDhu51pIG5o4bqtbiAoxJBp4buBdSA0Mi4yLmMpLg==",
      );
    }
  }
  return V2vz;
}

export function V4u4(V2w0: number | null): string {
  if (V2w0 == null) return "—";
  if (V2w0 === 0) return "TWnhu4VuIHBow60=";
  return `${V2w0.toLocaleString("vi-VN")}IMSR`;
}

export function V4u5(row: V4tl): boolean {
  return row.hanSuDung == null || Date.now() <= new Date(row.hanSuDung).getTime();
}

export function V4u6(row: V4tl): boolean {
  return (row.phi ?? 0) === 0 || row.paidAt != null;
}
