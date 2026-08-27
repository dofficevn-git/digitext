// Copyright (c) 2026 DigiText. All rights reserved.

import type { V41v } from "@/components/Icon";

export type V47j = "ok" | "warn" | "err" | "unknown";

export interface V47k {
  locked: boolean;
  mode: "online" | "offline";
}

export interface V47l {
  loading: boolean;

  session: V47k | null;

  scanners: number | null;

  serverUp: boolean | null;

  storage: { configured: boolean; exists: boolean } | null;

  live: boolean | null;
}

export type V47m = "scanner" | "token" | "server" | "live" | "upload" | "drive";

export interface V47n {
  key: V47m;
  icon: V41v;
  label: string;
  value: string;
  dot: V47j;
  title: string;
}

function V47o(i: V47l): V47n {
  const base = { key: "scanner" as const, icon: "scanner" as const, label: "Scanner" };
  if (i.scanners === null)
    return {
      ...base,
      dot: "unknown",
      value: "—",
      title: "xJBhbmcgZMOyIG3DoXkgcXXDqXQgKGhv4bq3YyBraMO0bmcgY8OzIHF1eeG7gW4pLg==",
    };
  if (i.scanners > 0)
    return {
      ...base,
      dot: "ok",
      value: i.scanners === 1 ? "xJDDoyBr4bq/dCBu4buRaQ==" : `${i.scanners}IG3DoXk=`,
      title: `RMOyIHRo4bqleSA=${i.scanners}IG3DoXkgcXXDqXQu`,
    };
  return {
    ...base,
    dot: "err",
    value: "S2jDtG5nIHRo4bqleQ==",
    title: "S2jDtG5nIGTDsiB0aOG6pXkgbcOheSBxdcOpdCBuw6BvLg==",
  };
}

function token(i: V47l): V47n {
  const base = { key: "token" as const, icon: "token" as const, label: "Token" };
  if (i.loading)
    return {
      ...base,
      dot: "unknown",
      value: "—",
      title: "xJBhbmcga2nhu4NtIHRyYSBwaGnDqm4gxJHEg25nIG5o4bqtcC4=",
    };
  if (!i.session)
    return {
      ...base,
      dot: "err",
      value: "Q2jGsGEgxJHEg25nIG5o4bqtcA==",
      title: "Q2jGsGEgY8OzIHBoacOqbiDEkcSDbmcgbmjhuq1wLg==",
    };
  if (i.session.locked)
    return {
      ...base,
      dot: "warn",
      value: "xJBhbmcga2hvw6E=",
      title: "UGhpw6puIMSRYW5nIGtob8OhIOKAlCBt4bufIGtob8OhIGLhurFuZyBt4bqtdCBraOG6qXUu",
    };
  return {
    ...base,
    dot: "ok",
    value: i.session.mode === "online" ? "VHLhu7FjIHR1eeG6v24=" : "Tmdv4bqhaSB0dXnhur9u",
    title:
      i.session.mode === "online"
        ? "VG9rZW4gxJHEg25nIG5o4bqtcCBo4bujcCBs4buHICh0cuG7sWMgdHV54bq/biku"
        : "VG9rZW4gxJHEg25nIG5o4bqtcCBjw7JuIGjhuqFuIChuZ2/huqFpIHR1eeG6v24pLg==",
  };
}

function server(i: V47l): V47n {
  const base = { key: "server" as const, icon: "server" as const, label: "Server" };
  if (i.serverUp === null)
    return {
      ...base,
      dot: "unknown",
      value: "—",
      title: "xJBhbmcga2nhu4NtIHRyYSBr4bq/dCBu4buRaSBtw6F5IGNo4bunLg==",
    };
  return i.serverUp
    ? {
        ...base,
        dot: "ok",
        value: "S+G6v3QgbuG7kWk=",
        title: "TcOheSBjaOG7pyBwaOG6o24gaOG7k2kgKC9oZWFsdGh6KS4=",
      }
    : {
        ...base,
        dot: "err",
        value: "TeG6pXQga+G6v3QgbuG7kWk=",
        title: "S2jDtG5nIGvhur90IG7hu5FpIMSRxrDhu6NjIG3DoXkgY2jhu6cu",
      };
}

function V47p(i: V47l): V47n {
  const base = { key: "live" as const, icon: "live" as const, label: "VHLhu7FjIHRp4bq/cA==" };
  if (i.live === null)
    return {
      ...base,
      dot: "unknown",
      value: "—",
      title: "Q2jGsGEgdGhlbyBkw7VpIChjaMawYSDEkcSDbmcgbmjhuq1wKS4=",
    };
  return i.live
    ? {
        ...base,
        dot: "ok",
        value: "xJBhbmcgY+G6rXAgbmjhuq10",
        title:
          "TcOgbiBow6xuaCB04buxIGPhuq1wIG5o4bqtdCB0aGVvIG3DoXkgY2jhu6cg4oCUIGtow7RuZyBj4bqnbiBt4bufIGzhuqFpIG3DoG4u",
      }
    : {
        ...base,
        dot: "err",
        value: "TeG6pXQgdGhlbyBkw7Vp",
        title:
          "S2jDtG5nIGjhu49pIMSRxrDhu6NjIG3DoXkgY2jhu6cg4oCUIGThu68gbGnhu4d1IHRyw6puIG3DoG4gY8OzIHRo4buDIMSRw6MgY8WpLg==",
      };
}

function upload(i: V47l): V47n {
  const base = { key: "upload" as const, icon: "upload" as const, label: "VOG6o2kgbMOqbg==" };
  if (i.serverUp === null || i.loading)
    return {
      ...base,
      dot: "unknown",
      value: "—",
      title: "xJBhbmcga2nhu4NtIHRyYSBraOG6oyBuxINuZyB04bqjaSBsw6puLg==",
    };
  if (!i.session || i.session.locked)
    return {
      ...base,
      dot: "err",
      value: "Q+G6p24gxJHEg25nIG5o4bqtcA==",
      title: "xJDEg25nIG5o4bqtcCAobeG7nyBraG/DoSkgxJHhu4MgdOG6o2kgbMOqbi4=",
    };
  if (!i.serverUp)
    return {
      ...base,
      dot: "err",
      value: "Q2jhu50gbcOheSBjaOG7pw==",
      title: "TcOheSBjaOG7pyBraMO0bmcgdOG7m2kgxJHGsOG7o2Mg4oCUIGNoxrBhIHThuqNpIGzDqm4gxJHGsOG7o2Mu",
    };
  return {
    ...base,
    dot: "ok",
    value: "U+G6tW4gc8Ogbmc=",
    title:
      "VOG6o2kgbMOqbiDEkcaw4bujYzogbcOheSBjaOG7pyB04bubaSDEkcaw4bujYyB2w6AgxJHDoyDEkcSDbmcgbmjhuq1wLg==",
  };
}

function drive(i: V47l): V47n {
  const base = { key: "drive" as const, icon: "drive" as const, label: "4buUIG5nb8OgaQ==" };
  if (i.storage === null)
    return { ...base, dot: "unknown", value: "—", title: "xJBhbmcgZMOyIOG7lSBsxrB1IHRy4buvLg==" };

  if (!i.storage.configured)
    return {
      ...base,
      dot: "unknown",
      value: "TuG7mWkgYuG7mQ==",
      title: "xJBhbmcgZMO5bmcg4buVIG7hu5lpIGLhu5kgKHRoxrAgbeG7pWMgRG9jdW1lbnRzKS4=",
    };
  return i.storage.exists
    ? {
        ...base,
        dot: "ok",
        value: "xJDDoyBn4bqvbg==",
        title: "4buUIGzGsHUgdHLhu68gdMO5eSBjaOG7jW4gdHJ1eSBj4bqtcCDEkcaw4bujYy4=",
      }
    : {
        ...base,
        dot: "err",
        value: "S2jDtG5nIHRo4bqleQ==",
        title:
          "4buUIGzGsHUgdHLhu68gxJHDoyBj4bqldSBow6xuaCBraMO0bmcgdHJ1eSBj4bqtcCDEkcaw4bujYyDigJQga2nhu4NtIHRyYSDhu5UgY+G6r20u",
      };
}

export function V47q(i: V47l): V47n[] {
  return [V47o(i), token(i), server(i), V47p(i), upload(i), drive(i)];
}
