// Copyright (c) 2026 DigiText. All rights reserved.

import type { V43e, V439 } from "@/api/lansync";
import type { V44q, V44n, V44o, V44s, V44r } from "@/api/collectProgress";
import { V44a, V449 } from "./uploadsCore";
import type { V41v } from "@/components/Icon";

export interface V44v {
  key: string;
  filename: string;
  sourceUsername: string | null;
  peer: string;
  hoSoRef: string | null;
  sizeBytes: number;
  receivedBytes: number;
  state: V44n;
  message: string | null;
  updatedAt: number;

  bps: number;
}

export interface V44w {
  sessionId: number;

  kind: V44o;
  running: boolean;

  stage: string;

  peers: number;
  items: V44v[];
  summary: V439 | V43e | null;
  dismissed: boolean;
  collapsed: boolean;
}

export function V44x(): V44w {
  return {
    sessionId: 0,
    kind: "lan",
    running: false,
    stage: "",
    peers: 0,
    items: [],
    summary: null,
    dismissed: false,
    collapsed: false,
  };
}

const V44y = 0.3;

export function V44z(Vab: V44v): boolean {
  return Vab.state !== "downloading";
}

export function V450(state: V44w, Vac: V44s): V44w {
  if (Vac.sessionId < state.sessionId) return state;
  if (Vac.phase === "start") {
    return {
      ...V44x(),
      collapsed: state.collapsed,
      sessionId: Vac.sessionId,
      kind: Vac.kind,
      running: true,
      stage: "xJBhbmcgY2h14bqpbiBi4buL4oCm",
    };
  }
  return { ...state, running: false, stage: "", summary: Vac.summary };
}

export function V451(state: V44w, Vad: V44r): V44w {
  if (Vad.sessionId < state.sessionId) return state;
  const base =
    Vad.sessionId > state.sessionId
      ? { ...V44x(), collapsed: state.collapsed, sessionId: Vad.sessionId, running: true }
      : state;

  const Vae: V44o = Vad.stage === "disk" ? "disk" : base.kind;
  return { ...base, kind: Vae, stage: Vad.label, peers: Vad.peers || base.peers };
}

export function V452(state: V44w, Vaf: V44q, Vag: number): V44w {
  if (Vaf.sessionId < state.sessionId) return state;
  const base =
    Vaf.sessionId > state.sessionId
      ? { ...V44x(), collapsed: state.collapsed, sessionId: Vaf.sessionId, running: true }
      : state;

  const Vah = base.items.find((i) => i.key === Vaf.key);
  const Vai: V44v = {
    key: Vaf.key,
    filename: Vaf.filename,
    sourceUsername: Vaf.sourceUsername,
    peer: Vaf.peer,
    hoSoRef: Vaf.hoSoRef,
    sizeBytes: Vaf.sizeBytes,
    receivedBytes: Vaf.receivedBytes,
    state: Vaf.state,
    message: Vaf.message,
    updatedAt: Vag,
    bps: V453(Vah, Vaf, Vag),
  };
  const Vaj = Vah ? base.items.map((i) => (i.key === Vaf.key ? Vai : i)) : [...base.items, Vai];
  return { ...base, items: Vaj, dismissed: Vah ? base.dismissed : false };
}

function V453(Vak: V44v | undefined, Val: V44q, Vam: number): number {
  if (!Vak || Val.state !== "downloading") return Vak?.bps ?? 0;
  const Van = (Vam - Vak.updatedAt) / 1000;
  const Vao = Val.receivedBytes - Vak.receivedBytes;
  if (Van <= 0 || Vao <= 0) return Vak.bps;
  const Vap = Vao / Van;
  return Vak.bps > 0 ? Vak.bps * (1 - V44y) + Vap * V44y : Vap;
}

export function V454(Vaq: V44v): number {
  if (Vaq.state === "done" || Vaq.state === "skipped") return 100;
  if (Vaq.sizeBytes <= 0) return 0;
  return Math.min(100, Math.max(0, Math.round((Vaq.receivedBytes / Vaq.sizeBytes) * 100)));
}

export function V455(s: V44w): V44v[] {
  const Var = (Vas: V44v) => (Vas.state === "error" ? 0 : Vas.state === "cancelled" ? 1 : 2);
  return s.items
    .map((Vat, i) => ({ it: Vat, i }))
    .sort((a, b) => Var(a.it) - Var(b.it) || a.i - b.i)
    .map((x) => x.it);
}

export function V456(s: V44w): boolean {
  return !s.dismissed && s.sessionId > 0;
}

export function V457(s: V44w): {
  done: number;
  skipped: number;
  errors: number;
  cancelled: number;
  downloading: number;
} {
  return {
    done: s.items.filter((i) => i.state === "done").length,
    skipped: s.items.filter((i) => i.state === "skipped").length,
    errors: s.items.filter((i) => i.state === "error").length,
    cancelled: s.items.filter((i) => i.state === "cancelled").length,
    downloading: s.items.filter((i) => i.state === "downloading").length,
  };
}

export function V458(s: V44w): string {
  const t = V457(s);
  const Vau = s.kind === "disk";
  if (s.running) {
    if (t.downloading > 0) {
      return Vau
        ? `xJBhbmcgdGjDqm0g${t.downloading}IHThu4dwIHbDoG8gS2hv`
        : `xJBhbmcgZ29tIA==${t.downloading}IHThu4dwIHbhu4EgbcOheQ==`;
    }
    if (Vau) return "xJBhbmcgdGjDqm0gdMOgaSBsaeG7h3UgdOG7qyDhu5UgY+G7qW5n";
    return s.peers > 0
      ? `xJBhbmcgdGh1IHRo4bqtcCB04burIA==${s.peers}IG3DoXkgbmjDom4gdmnDqm4=`
      : "xJBhbmcgdGh1IHRo4bqtcCBk4buvIGxp4buHdQ==";
  }
  const Vav: string[] = [];
  if (t.done > 0)
    Vav.push(Vau ? `xJDDoyB0aMOqbSA=${t.done}IHThu4dw` : `xJDDoyBnb20g${t.done}IHThu4dw`);
  if (t.skipped > 0) Vav.push(`${t.skipped}IHThu4dwIMSRw6MgY8OzIHPhurVu`);
  if (t.errors > 0) Vav.push(`${t.errors}IHThu4dwIGzhu5dp`);
  if (t.cancelled > 0) Vav.push(`${t.cancelled}IHThu4dwIMSRw6MgaOG7p3k=`);
  if (Vav.length === 0)
    return Vau
      ? "S2jDtG5nIHRow6ptIMSRxrDhu6NjIHThu4dwIG7DoG8gbeG7m2k="
      : "S2jDtG5nIGdvbSDEkcaw4bujYyB04buHcCBuw6BvIG3hu5tp";
  return Vav.join(" · ");
}

export function V459(Vaw: V44v, Vax: number, Vay: V44o = "lan"): string {
  const nguon =
    Vay === "disk"
      ? Vaw.hoSoRef
        ? `→ ${Vaw.hoSoRef}`
        : ""
      : Vaw.sourceUsername
        ? `${Vaw.sourceUsername} · ${Vaw.peer}`
        : Vaw.peer;
  const Vaz = (...Vb0: string[]) => [nguon, ...Vb0].filter((s) => s !== "").join(" · ");
  switch (Vaw.state) {
    case "downloading":
      return Vaz(
        `${V44a(Vaw.receivedBytes)}/${V44a(Vaw.sizeBytes)}`,
        V45c(Vaw.bps),
        V45d(Vaw, Vax),
      );
    case "done":
      return Vaz(`${Vay === "disk" ? "xJHDoyBjaMOpcA==" : "xJHDoyB24buB"} ${V44a(Vaw.sizeBytes)}`);
    default:
      return Vaz(Vaw.message ?? "");
  }
}

export interface V45a {
  title: string;

  icon: V41v;

  empty: string;

  foot: string;

  stop: string;

  closeBusy: string;

  cancelRow: string;
}

export function V45b(s: V44w, Vb1: boolean): V45a {
  const t = V457(s);
  const Vb2 = s.kind === "disk";

  const Vb3 = Vb1
    ? Vb2
      ? `${t.done}IHThu4dwIMSRw6MgdsOgbyBLaG8=`
      : `${t.done}IHThu4dwIMSRw6MgduG7gQ==${s.peers > 0 ? ` · ${s.peers}IG3DoXk=` : ""}`
    : [
        Vb2 ? `${t.done}IHThu4dwIHRow6ptIG3hu5tp` : `${t.done}IHThu4dwIGdvbSBt4bubaQ==`,
        t.skipped > 0 ? `${t.skipped}IMSRw6MgY8OzIHPhurVu` : "",
        t.errors > 0 ? `${t.errors}IGzhu5dp` : "",
        t.cancelled > 0 ? `${t.cancelled}IMSRw6MgaOG7p3k=` : "",
      ]
        .filter((x) => x !== "")
        .join(" · ");

  return {
    title: Vb2
      ? "VHLhuqFuZyB0aMOhaSB0aMOqbSB0w6BpIGxp4buHdSB04burIOG7lSBj4bupbmc="
      : "VHLhuqFuZyB0aMOhaSB0aHUgdGjhuq1w",
    icon: Vb2 ? "them" : "nhapKho",
    empty: Vb2
      ? Vb1
        ? "Q2jGsGEgY2jDqXAgdOG7h3AgbsOgbyDigJQgxJFhbmcgZHV54buHdCBs4buxYSBjaOG7jW4gKHRoxrAgbeG7pWMgbOG7m24gbeG6pXQgbeG7mXQgbMO6YynigKY="
        : "TMaw4bujdCBuw6B5IGtow7RuZyBjw7MgdOG7h3AgbsOgbyDEkeG7gyB0aMOqbS4="
      : Vb1
        ? "Q2jGsGEgY8OzIHThu4dwIG7DoG8gduG7gSDigJQgxJFhbmcgdMOsbSB2w6AgxJHhu41jIGRhbmggc8OhY2ggY+G7p2EgbcOheSBuaMOibiB2acOqbuKApg=="
        : "TMaw4bujdCBuw6B5IGtow7RuZyBjw7MgdOG7h3AgbeG7m2kgbsOgbyDEkeG7gyBnb20u",
    foot: Vb3,
    stop: Vb2 ? "ROG7q25nIHRow6ptIHTDoGkgbGnhu4d1" : "ROG7q25nIHRodSB0aOG6rXA=",
    closeBusy: Vb2
      ? "xJBhbmcgdGjDqm0gdMOgaSBsaeG7h3Ug4oCUIHRodSBuaOG7jyBi4bqjbmcsIGhv4bq3YyBk4burbmcgdHLGsOG7m2Mga2hpIMSRw7NuZw=="
      : "xJBhbmcgdGh1IHRo4bqtcCDigJQgdGh1IG5o4buPIGLhuqNuZywgaG/hurdjIGjhu6d5IHRyxrDhu5tjIGtoaSDEkcOzbmc=",
    cancelRow: Vb2
      ? "QuG7jyBxdWEgdOG7h3AgbsOgeSB0cm9uZyBsxrDhu6N0IHRow6pt"
      : "QuG7jyBxdWEgdOG7h3AgbsOgeSB0cm9uZyBsxrDhu6N0IGdvbQ==",
  };
}

export function V45c(Vb4: number): string {
  if (!Number.isFinite(Vb4) || Vb4 <= 0) return "";
  return `${V44a(Vb4)}/s`;
}

export function V45d(Vb5: V44v, Vb6: number): string {
  if (Vb5.state !== "downloading" || Vb5.bps <= 0 || Vb5.sizeBytes <= 0) return "";

  if (Vb6 - Vb5.updatedAt > 30_000) return "";
  const left = Vb5.sizeBytes - Vb5.receivedBytes;
  if (left <= 0) return "";
  return `Y8OybiA=${V449(Math.ceil(left / Vb5.bps))}`;
}

export function V45e(Vb7: V44v): boolean {
  return Vb7.state === "downloading";
}

export function V45f(s: V44w, Vb8: string): V44w {
  return { ...s, items: s.items.filter((i) => i.key !== Vb8) };
}
