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
  /** Đẩy thẳng từ ổ đĩa: tệp giữ nguyên trên ổ, chỉ ghi sổ. */
  direct: boolean;
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
    direct: false,
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
      direct: Vac.direct === true,
      running: true,
      stage: "Đang chuẩn bị…",
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
      return s.direct
        ? `Đang ghi sổ ${t.downloading} tệp trên ổ đĩa`
        : Vau
          ? `Đang thêm ${t.downloading} tệp vào Kho`
          : `Đang gom ${t.downloading} tệp về máy`;
    }
    if (s.direct) return "Đang ghi sổ tài liệu trên ổ đĩa (không chép)";
    if (Vau) return "Đang thêm tài liệu từ ổ cứng";
    return s.peers > 0
      ? `Đang thu thập từ ${s.peers} máy nhân viên`
      : "Đang thu thập dữ liệu";
  }
  const Vav: string[] = [];
  if (t.done > 0)
    Vav.push(Vau ? `Đã thêm ${t.done} tệp` : `Đã gom ${t.done} tệp`);
  if (t.skipped > 0) Vav.push(`${t.skipped} tệp đã có sẵn`);
  if (t.errors > 0) Vav.push(`${t.errors} tệp lỗi`);
  if (t.cancelled > 0) Vav.push(`${t.cancelled} tệp đã hủy`);
  if (Vav.length === 0)
    return Vau
      ? "Không thêm được tệp nào mới"
      : "Không gom được tệp nào mới";
  return Vav.join(" · ");
}

export function V459(Vaw: V44v, Vax: number, Vay: V44o = "lan", direct = false): string {
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
      return Vaz(`${direct ? "đã ghi sổ" : Vay === "disk" ? "đã chép" : "đã về"} ${V44a(Vaw.sizeBytes)}`);
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
    ? s.direct
      ? `${t.done} tệp đã ghi sổ (giữ trên ổ)`
      : Vb2
        ? `${t.done} tệp đã vào Kho`
        : `${t.done} tệp đã về${s.peers > 0 ? ` · ${s.peers} máy` : ""}`
    : [
        s.direct ? `${t.done} tệp ghi sổ mới` : Vb2 ? `${t.done} tệp thêm mới` : `${t.done} tệp gom mới`,
        t.skipped > 0 ? `${t.skipped} đã có sẵn` : "",
        t.errors > 0 ? `${t.errors} lỗi` : "",
        t.cancelled > 0 ? `${t.cancelled} đã hủy` : "",
      ]
        .filter((x) => x !== "")
        .join(" · ");

  return {
    title: s.direct
      ? "Trạng thái ghi sổ từ ổ đĩa (đẩy thẳng)"
      : Vb2
        ? "Trạng thái thêm tài liệu từ ổ cứng"
        : "Trạng thái thu thập",
    icon: Vb2 ? "them" : "nhapKho",
    empty: Vb2
      ? Vb1
        ? "Chưa chép tệp nào — đang duyệt lựa chọn (thư mục lớn mất một lúc)…"
        : "Lượt này không có tệp nào để thêm."
      : Vb1
        ? "Chưa có tệp nào về — đang tìm và đọc danh sách của máy nhân viên…"
        : "Lượt này không có tệp mới nào để gom.",
    foot: Vb3,
    stop: Vb2 ? "Dừng thêm tài liệu" : "Dừng thu thập",
    closeBusy: Vb2
      ? "Đang thêm tài liệu — thu nhỏ bảng, hoặc dừng trước khi đóng"
      : "Đang thu thập — thu nhỏ bảng, hoặc hủy trước khi đóng",
    cancelRow: Vb2
      ? "Bỏ qua tệp này trong lượt thêm"
      : "Bỏ qua tệp này trong lượt gom",
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
  return `còn ${V449(Math.ceil(left / Vb5.bps))}`;
}

export function V45e(Vb7: V44v): boolean {
  return Vb7.state === "downloading";
}

export function V45f(s: V44w, Vb8: string): V44w {
  return { ...s, items: s.items.filter((i) => i.key !== Vb8) };
}
