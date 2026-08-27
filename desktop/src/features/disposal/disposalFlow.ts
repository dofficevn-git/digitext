// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4u9, V4u8 } from "@/api/disposal";

export const V4ul: Record<V4u8["lyDo"], string> = {
  trung_lap: "VHLDuW5nIGzhurdwICjEkGnhu4F1IDQ4LjIp",
  het_thoi_han: "SOG6v3QgdGjhu51pIGjhuqFuIGzGsHUgdHLhu68gKMSQaeG7gXUgNDguMyk=",
};

export const V4um: Record<V4u8["status"], string> = {
  xem_xet: "WGVtIHjDqXQgaOG7p3k=",
  giu_lai: "R2nhu68gbOG6oWkgKMSRw6MgbG/huqFpIGto4buPaSBEYW5oIG3hu6VjKQ==",
  da_huy: "xJDDoyBo4buneQ==",
};

export const V4un: Record<V4u9["status"], string> = {
  hoi_dong: "SOG7mWkgxJHhu5NuZyB4w6l0IGjhu6d5ICjEkGnhu4F1IDQ5LjIp",
  tham_dinh: "Q2jhu50gdGjhuqltIMSR4buLbmggKMSQaeG7gXUgNDkuNCk=",
  quyet_dinh: "xJDDoyBjw7MgUXV54bq/dCDEkeG7i25oIGjhu6d5ICjEkGnhu4F1IDQ5LjYp",
  da_huy: "xJDDoyBo4buneSB0cm9uZyBI4buHIHRo4buRbmcgKMSQaeG7gXUgNDkuNyk=",
};

export const V4uo: Record<string, string> = {
  phat_hien_trung_lap: "SOG7hyB0aOG7kW5nIHBow6F0IGhp4buHbiB0csO5bmcgbOG6t3A=",
  het_thoi_han_luu_tru: "SOG7hyB0aOG7kW5nIGdoaSBuaOG6rW4gaOG6v3QgdGjhu51pIGjhuqFuIGzGsHUgdHLhu68=",
  giu_lai: "R2nhu68gbOG6oWkg4oCUIGxv4bqhaSBraOG7j2kgRGFuaCBt4bulYw==",
  lap_dot_xet_huy: "TOG6rXAgxJHhu6N0IHjDqXQgaOG7p3k=",
  hoi_dong_tong_hop: "SOG7mWkgxJHhu5NuZyB04buVbmcgaOG7o3Aga+G6v3QgcXXhuqM=",
  y_kien_tham_dinh: "w50ga2nhur9uIHRo4bqpbSDEkeG7i25o",
  quyet_dinh_huy: "UXV54bq/dCDEkeG7i25oIGjhu6d5",
  bat_dau_huy: "QuG6r3QgxJHhuqd1IHRoYW8gdMOhYyBo4buneQ==",
  da_huy_tai_lieu: "xJDDoyBo4buneSB0w6BpIGxp4buHdSAoeMOzYSBraOG7j2kga2hvKQ==",
};

export function V4up(
  V2ym: V4u9,
  V2yn: boolean,
  V2yo: boolean,
): { action: "council" | "appraise" | "decide" | "execute"; label: string }[] {
  const V2yp: { action: "council" | "appraise" | "decide" | "execute"; label: string }[] = [];
  if (V2ym.status === "hoi_dong" && V2yn) {
    V2yp.push({
      action: "council",
      label: "Q2jhu5F0IGvhur90IHF14bqjIEjhu5lpIMSR4buTbmcgKDQ5LjIuYyk=",
    });
  }
  if (V2ym.status === "tham_dinh") {
    if (V2yn && !V2ym.thamDinhNote) {
      V2yp.push({
        action: "appraise",
        label: "Tmjhuq1wIMO9IGtp4bq/biB0aOG6qW0gxJHhu4tuaCAoNDkuNCk=",
      });
    }
    if (V2yo) {
      V2yp.push({ action: "decide", label: "UXV54bq/dCDEkeG7i25oIGjhu6d5ICg0OS42KQ==" });
    }
  }
  if (V2ym.status === "quyet_dinh" && V2yo) {
    V2yp.push({
      action: "execute",
      label: "VGhhbyB0w6FjIGjhu6d5IHRyb25nIEjhu4cgdGjhu5FuZyAoNDkuNyk=",
    });
  }
  return V2yp;
}

export function V4uq(V2yq: V4u9): boolean {
  return !V2yq.thamDinhNote;
}
