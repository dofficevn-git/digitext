// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3zc } from "@/api/auth";
import { V3z6 } from "@/domain/permissions";

export type V3zp = "loading" | "signedOut" | "locked" | "active";

export function V3zq(Vx: V3zc | null, Vy: boolean): V3zp {
  if (Vy) return "loading";
  if (!Vx) return "signedOut";
  return Vx.locked ? "locked" : "active";
}

export function V3zr(Vz: string, V10: Date = new Date()): number {
  const V11 = Date.parse(Vz);
  if (Number.isNaN(V11)) return 0;
  const ms = V11 - V10.getTime();
  return ms > 0 ? Math.floor(ms / 86_400_000) : 0;
}

export function V3zs(V12: string, V13: Date = new Date()): boolean {
  return V3zr(V12, V13) < 7;
}

export function V3zt(V14: string, V15: string): string {
  switch (V14) {
    case "auth_failed":
      return "VMOqbiDEkcSDbmcgbmjhuq1wIGhv4bq3YyBt4bqtdCBraOG6qXUga2jDtG5nIMSRw7puZy4=";
    case "too_many_attempts":
      return (
        V15 ||
        "U2FpIG3huq10IGto4bqpdSBuaGnhu4F1IGzhuqduIOKAlCB0aOG7rSBs4bqhaSBzYXUgw610IHBow7p0Lg=="
      );
    case "captcha_invalid":
      return "TcOjIHjDoWMgbmjhuq1uIGtow7RuZyDEkcO6bmcg4oCUIG5o4bqtcCBs4bqhaSBtw6MgdHJvbmcg4bqjbmggbeG7m2ku";
    case "revoked":
      return "VMOgaSBraG/huqNuIMSRw6MgYuG7iyB0aHUgaOG7k2kg4oCUIGxpw6puIGjhu4cgcXXhuqNuIHRy4buLIGThu7Egw6FuLg==";
    case "device_revoked":
      return "TcOheSBuw6B5IMSRw6MgYuG7iyB0aHUgaOG7k2kgcXV54buBbiB0cnV5IGPhuq1wIOKAlCBsacOqbiBo4buHIHF14bqjbiB0cuG7iyBk4buxIMOhbi4=";
    case "offline_expired":
      return "UXXDoSAzMCBuZ8OgeSBjaMawYSB4w6FjIHRo4buxYyBvbmxpbmUg4oCUIGvhur90IG7hu5FpIG3huqFuZyBy4buTaSDEkcSDbmcgbmjhuq1wIGzhuqFpLg==";
    case "server_unreachable":
      return "S2jDtG5nIGvhur90IG7hu5FpIMSRxrDhu6NjIG3DoXkgY2jhu6cuIEtp4buDbSB0cmEgbeG6oW5nLCBob+G6t2MgxJHEg25nIG5o4bqtcCBvZmZsaW5lIG7hur91IMSRw6MgZMO5bmcgbcOheSBuw6B5IHRyxrDhu5tjIMSRw7Mu";
    case "server_not_configured":
      return "Q2jGsGEgY+G6pXUgaMOsbmggxJHhu4thIGNo4buJIG3DoXkgY2jhu6cg4oCUIGLhuqVtIOKAnEPDoGkgxJHhurd04oCdIMSR4buDIG5o4bqtcC4=";
    case "busy":
      return "xJBhbmcgcXXDqXQgZOG7nyBt4buZdCBsw7Qg4oCUIGvhur90IHRow7pjIGzDtCBxdcOpdCB0csaw4bubYyBraGkgxJHhu5VpIG5nxrDhu51pIGTDuW5nLg==";
    case "locked":
      return "TcOgbiBow6xuaCDEkWFuZyBraG/DoSDigJQgbeG7nyBraG/DoSBi4bqxbmcgbeG6rXQga2jhuql1Lg==";
    case "server_mismatch":
      return "Q2jhu68ga8O9IG3DoXkgY2jhu6cga2jDtG5nIGjhu6NwIGzhu4cg4oCUIGtp4buDbSB0cmEgbOG6oWkgxJHhu4thIGNo4buJIHNlcnZlci4=";
    case "auth_cache_corrupt":
      return "ROG7ryBsaeG7h3UgxJHEg25nIG5o4bqtcCB0csOqbiBtw6F5IGLhu4sgaOG7j25nIOKAlCBj4bqnbiDEkcSDbmcgbmjhuq1wIG9ubGluZSBs4bqhaS4=";
    default:
      return V15 || "Q8OzIGzhu5dpIHjhuqN5IHJhLCB0aOG7rSBs4bqhaS4=";
  }
}

export function V3zu(): V3zc {
  return {
    userId: "dev-user",
    username: "dev",
    displayName: "Dev (browser)",
    role: "admin",
    projectId: null,
    projectCode: "DEV",
    projectName: "Q2jhur8gxJHhu5kgdHLDrG5oIGR1eeG7h3Q=",
    mode: "offline",
    offlineExpiry: new Date(Date.now() + 365 * 86_400_000).toISOString(),
    locked: false,

    permissions: V3z6("admin"),
  };
}
