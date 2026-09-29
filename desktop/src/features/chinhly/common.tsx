// Copyright (c) 2026 DigiText. All rights reserved.
// Dùng chung cho các màn chỉnh lý: danh mục (bản sao), trạng thái đồng bộ, tiện ích hiển thị.

import { useCallback, useEffect, useState, type ReactNode } from "react";
import { listen } from "@tauri-apps/api/event";
import { isTauri, V3xq } from "@/api/ipc";
import { V41p } from "@/live/useLive";
import { CL_SYNC_EVENT, clCatalogList, clProjectGet, clProjectSet, clSyncStatus, str, type ClCatalog, type ClProjectInfo, type ClRow, type ClSyncStatus } from "@/api/chinhly";

export const errText = (e: unknown) => (e instanceof V3xq || e instanceof Error ? e.message : String(e));
export const n = (v: number) => v.toLocaleString("vi-VN");

const EMPTY: ClCatalog = { khoi_phong: [], phong: [], phong_ban: [], mang: [], thbq: [], ngon_ngu: [], loai_van_ban: [], kho: [], gia: [], hop: [], bo_tap: [] };

/** Danh mục chỉnh lý từ bản sao (offline được), tự nạp lại khi đồng bộ. */
export function useClCatalog(): { cat: ClCatalog; reload: () => void; error: string | null } {
  const [cat, setCat] = useState<ClCatalog>(EMPTY);
  const [error, setError] = useState<string | null>(null);
  const load = useCallback(async () => {
    try {
      setCat(await clCatalogList());
      setError(null);
    } catch (e) {
      setError(errText(e));
    }
  }, []);
  const { reload } = V41p(load, { topics: ["chinhLy", "catalog"], enabled: isTauri() });
  return { cat, reload, error };
}

/** Thanh chọn "Dự án đang làm": chỉ hiện với tài khoản không gắn dự án (quản trị hệ thống). */
export function ChonDuAn() {
  const [info, setInfo] = useState<ClProjectInfo | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const load = useCallback(async () => {
    if (!isTauri()) return;
    try {
      setInfo(await clProjectGet());
      setErr(null);
    } catch (e) {
      setErr(errText(e));
    }
  }, []);
  useEffect(() => {
    void load();
  }, [load]);
  if (!info || info.fromAccount) return null;
  const chon = async (id: string) => {
    if (!id) return;
    setBusy(true);
    try {
      await clProjectSet(id);
      await load();
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  };
  const chua = !info.projectId;
  return (
    <div className={`card mt-3.5 flex items-center gap-3 flex-wrap ${chua ? "border-warning" : ""}`} style={{ padding: "8px 12px" }}>
      <span className="text-sm font-medium">Dự án đang làm:</span>
      <select className="min-w-72" value={info.projectId ?? ""} disabled={busy} onChange={(e) => void chon(e.target.value)}>
        <option value="">— chọn dự án —</option>
        {info.projects.map((p) => (
          <option key={p.id} value={p.id}>
            {p.code} — {p.name}
          </option>
        ))}
      </select>
      <span className="muted text-sm">
        {chua
          ? "Tài khoản quản trị không gắn dự án: chọn dự án để nạp và đồng bộ dữ liệu chỉnh lý."
          : "Dữ liệu chỉnh lý (danh mục, hồ sơ, kho/giá…) thuộc dự án này; đổi dự án sẽ đồng bộ lại."}
        {info.cached ? " (danh sách lấy từ bộ nhớ đệm — không tới máy chủ)" : ""}
      </span>
      {err && <span className="text-danger text-sm">{err}</span>}
    </div>
  );
}

/** Trạng thái engine đồng bộ (sự kiện `chinhly://sync`). */
export function useClSync(): ClSyncStatus | null {
  const [s, setS] = useState<ClSyncStatus | null>(null);
  useEffect(() => {
    if (!isTauri()) return;
    let alive = true;
    clSyncStatus().then((v) => alive && setS(v)).catch(() => {});
    const un = listen<ClSyncStatus>(CL_SYNC_EVENT, (ev) => {
      if (alive) setS(ev.payload);
    });
    return () => {
      alive = false;
      void un.then((f) => f());
    };
  }, []);
  return s;
}

export function syncLabel(s: ClSyncStatus | null): { text: string; dot: "ok" | "warn" | "err" | "unknown"; title: string } {
  if (!s) return { text: "—", dot: "unknown", title: "Chưa có thông tin đồng bộ chỉnh lý." };
  const tail = [s.pending > 0 ? `${n(s.pending)} chờ đẩy` : "", s.conflicts > 0 ? `${n(s.conflicts)} xung đột` : "", s.failed > 0 ? `${n(s.failed)} bị từ chối` : ""].filter(Boolean).join(" · ");
  const when = s.lastSyncAt ? `Lần cuối: ${fmtTime(s.lastSyncAt)}.` : "Chưa đồng bộ lần nào.";
  switch (s.phase) {
    case "pushing":
      return { text: "Đang đẩy…", dot: "warn", title: `Đang gửi thay đổi lên máy chủ. ${when}` };
    case "pulling":
      return { text: "Đang kéo…", dot: "warn", title: `Đang nhận thay đổi từ máy chủ. ${when}` };
    case "offline":
      return { text: tail || "Ngoại tuyến", dot: "warn", title: `Không tới máy chủ — thay đổi giữ ở máy, sẽ tự đẩy khi có mạng. ${when}` };
    case "locked":
      return { text: "Đang khoá", dot: "unknown", title: "Mở khoá phiên để đồng bộ chỉnh lý." };
    case "no_project":
      return { text: "Chưa chọn dự án", dot: "warn", title: "Tài khoản không gắn dự án — chọn \"Dự án đang làm\" ở đầu màn Chỉnh lý." };
    case "error":
      return { text: "Lỗi", dot: "err", title: `${s.lastError ?? "Lỗi đồng bộ"}. ${when}` };
    default:
      if (s.conflicts > 0) return { text: tail, dot: "err", title: `Có xung đột cần bạn chọn (menu Chỉnh lý → Đồng bộ & xung đột). ${when}` };
      if (s.pending > 0 || s.failed > 0) return { text: tail, dot: "warn", title: `${when}` };
      return { text: s.lastSyncAt ? `Đã đồng bộ ${fmtTime(s.lastSyncAt)}` : "Sẵn sàng", dot: "ok", title: `Bản sao chỉnh lý khớp máy chủ. ${when}` };
  }
}

export function fmtTime(iso: string | null | undefined): string {
  if (!iso) return "—";
  const d = new Date(iso.includes("T") || iso.includes("Z") ? iso : iso.replace(" ", "T") + "Z");
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString("vi-VN", { hour: "2-digit", minute: "2-digit", day: "2-digit", month: "2-digit" });
}

export function Field({ label, children, className }: { label: string; children: ReactNode; className?: string }) {
  return (
    <label className={`grid gap-1 text-sm min-w-0 ${className ?? ""}`}>
      <span className="muted truncate">{label}</span>
      {children}
    </label>
  );
}

/** `<select>` danh mục: option từ mảng dòng, giá trị = cột `valueKey`. */
export function Select({
  value,
  onChange,
  rows,
  valueKey = "id",
  labelKey = "ten",
  empty = "— Chọn —",
  disabled,
  render,
}: {
  value: string;
  onChange: (v: string) => void;
  rows: ClRow[];
  valueKey?: string;
  labelKey?: string;
  empty?: string;
  disabled?: boolean;
  render?: (r: ClRow) => string;
}) {
  return (
    <select className="w-full min-w-0" value={value} disabled={disabled} onChange={(e) => onChange(e.target.value)}>
      <option value="">{empty}</option>
      {rows.map((r) => (
        <option key={str(r, valueKey)} value={str(r, valueKey)}>
          {render ? render(r) : str(r, labelKey)}
        </option>
      ))}
    </select>
  );
}

export function SyncDot({ sync }: { sync: string }) {
  const m: Record<string, [string, string]> = {
    clean: ["✓", "Đã đồng bộ với máy chủ"],
    pending: ["●", "Chờ đẩy lên máy chủ"],
    conflict: ["⚠", "Xung đột — vào Đồng bộ & xung đột để chọn"],
  };
  const [t, title] = m[sync] ?? ["", ""];
  const cls = sync === "conflict" ? "text-danger" : sync === "pending" ? "text-warning-700" : "text-success-700";
  return (
    <span className={cls} title={title}>
      {t}
    </span>
  );
}

export const THBQ_BADGE: Record<string, string> = { VV: "err", HET: "gray", GL: "ok" };
