// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4uu, V4us } from "@/api/infoops";
import type { V41v } from "@/components/Icon";

export const V4v6: Record<V4us["hinhThuc"], string> = {
  danh_muc:
    "RGFuaCBt4bulYyB0w6BpIGxp4buHdSAoxJBp4buBdSA0NC4xIOKAlCB04buRaSDEkWEgNCB0csaw4budbmcgdGluKQ==",
  trich_xuat:
    "VHLDrWNoIHh14bqldCBuZ3V5w6puIHbEg24gKMSQaeG7gXUgNDQuMiDigJQga8OobSDEkeG7i2EgY2jhu4kgbMawdSB0cuG7ryk=",
  tong_hop:
    "VGjDtG5nIHRpbiB04buVbmcgaOG7o3AgKMSQaeG7gXUgNDQuMyDigJQgYsOhbyBjw6FvIHThu7EgxJHhu5luZyk=",
};

export const V4v7: Record<V4us["status"], string> = {
  submitted: "Q2jhu50geMOpdCBkdXnhu4d0IChTTEEgNiBuZ8OgeSBsw6BtIHZp4buHYyDigJQgNDYuNSk=",
  approved: "xJDDoyBkdXnhu4d0",
  refused: "VOG7qyBjaOG7kWk=",
  fulfilled: "xJDDoyB0cuG6oyBr4bq/dCBxdeG6ow==",
};

export const V4v8: Record<string, string> = {
  gia_tang: "R2lhIHTEg25nIGjhurFuZyBuZ8OgeSAoMzAuMS5hKQ==",
  day_du: "xJDhuqd5IMSR4bunIGjhurFuZyB0aMOhbmcgKDMwLjEuYSk=",
  day_du_3_nam: "xJDhuqd5IMSR4bunIDAzIG7Eg20gKDMwLjEuYik=",
};

export function V4v9(V30c: V4uu): string[] {
  const V30d: string[] = [];
  if (!V30c.chuDe.trim()) {
    V30d.push("UGjhuqNpIG7DqnUgY2jhu6cgxJHhu4EgY+G7pSB0aOG7gyAoxJBp4buBdSA0NC4xLmEpLg==");
  }
  if (!V30c.mucDich.trim()) {
    V30d.push("UGjhuqNpIGtoYWkgbeG7pWMgxJHDrWNoIHPhu60gZOG7pW5nICjEkGnhu4F1IDQ1LjMpLg==");
  }
  return V30d;
}

export interface V4va {
  action: "approve" | "refuse" | "pay" | "generate" | "fulfill";
  icon: V41v;
  label: string;
}

export function V4vb(row: V4us): V4va[] {
  const V30e: V4va[] = [];
  if (row.status === "submitted") {
    V30e.push({
      action: "approve",
      icon: "duyet",
      label: "RHV54buHdCAocGjDrSB04buxIHTDrW5oIOKAlCA0Ni4xLmMp",
    });
    V30e.push({ action: "refuse", icon: "tuChoi", label: "VOG7qyBjaOG7kWk=" });
  }
  if (row.status === "approved") {
    if ((row.phi ?? 0) > 0 && !row.paidAt) {
      V30e.push({
        action: "pay",
        icon: "thanhToan",
        label: "WMOhYyBuaOG6rW4gdGhhbmggdG/DoW4gKDQ2LjQp",
      });
    }
    V30e.push({
      action: "generate",
      icon: row.coKetQua ? "lamMoi" : "ketXuat",
      label: row.coKetQua
        ? "S+G6v3QgeHXhuqV0IGzhuqFpICg0Ny4xKQ=="
        : "S+G6v3QgeHXhuqV0IHThu6sgSOG7hyB0aOG7kW5nICg0Ny4xKQ==",
    });
    if (row.coKetQua) {
      V30e.push({
        action: "fulfill",
        icon: "guiKetQua",
        label: "UGjDqiBkdXnhu4d0ICsgdHLhuqMga+G6v3QgcXXhuqMgKDQ3LjMvNDcuNCk=",
      });
    }
  }
  return V30e;
}
