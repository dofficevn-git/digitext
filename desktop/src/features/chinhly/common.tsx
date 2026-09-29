// Copyright (c) 2026 DigiText. All rights reserved.
// Dùng chung cho các màn chỉnh lý: danh mục (bản sao), trạng thái đồng bộ, ô chọn danh mục, tiện ích hiển thị.

import { useCallback, useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { isTauri } from "@/api/ipc";
import { V41p } from "@/live/useLive";
import { CL_SYNC_EVENT, clCatalogList, clSyncStatus, str, type ClCatalog, type ClRow, type ClSyncStatus } from "@/api/chinhly";
import { useProject } from "@/state/project";
import { Badge, Banner, errorText, Select as UiSelect, Tip } from "@/ui";
import type { V3y0 } from "@/domain/types";

export const errText = (e: unknown) => errorText(e);
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

/** Dải nhắc khi tài khoản quản trị chưa chọn "Dự án đang làm" (chọn ở thanh trên cùng). */
export function ChuaChonDuAn() {
  const p = useProject();
  if (p.fromAccount || p.projectId || p.loading) return null;
  return (
    <Banner tone="warning" className="mb-4">
      Chưa chọn dự án đang làm. Chọn dự án ở ô cạnh tên bạn trên thanh trên cùng để nạp và đồng bộ dữ liệu chỉnh lý.
    </Banner>
  );
}

/** Trạng thái engine đồng bộ (sự kiện `chinhly://sync`). */
export function useClSync(): ClSyncStatus | null {
  const [s, setS] = useState<ClSyncStatus | null>(null);
  useEffect(() => {
    if (!isTauri()) return;
    let alive = true;
    clSyncStatus()
      .then((v) => alive && setS(v))
      .catch(() => {});
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
      return { text: tail || "Ngoại tuyến", dot: "warn", title: `Không tới máy chủ. Thay đổi giữ ở máy, tự đẩy khi có mạng. ${when}` };
    case "locked":
      return { text: "Đang khoá", dot: "unknown", title: "Mở khoá phiên để đồng bộ chỉnh lý." };
    case "no_project":
      return { text: "Chưa chọn dự án", dot: "warn", title: "Chọn dự án đang làm ở thanh trên cùng để đồng bộ chỉnh lý." };
    case "error":
      return { text: "Lỗi", dot: "err", title: `${s.lastError ?? "Lỗi đồng bộ"}. ${when}` };
    default:
      if (s.conflicts > 0) return { text: tail, dot: "err", title: `Có xung đột cần bạn chọn ở màn Đồng bộ. ${when}` };
      if (s.pending > 0 || s.failed > 0) return { text: tail, dot: "warn", title: `${when}` };
      return { text: s.lastSyncAt ? `Đã đồng bộ ${fmtTime(s.lastSyncAt)}` : "Sẵn sàng", dot: "ok", title: `Bản sao chỉnh lý khớp máy chủ. ${when}` };
  }
}

/** Badge trạng thái đồng bộ dùng ở đầu màn. */
export function SyncBadge({ sync }: { sync: ClSyncStatus | null }) {
  const sl = syncLabel(sync);
  const tone: V3y0 = sl.dot === "ok" ? "ok" : sl.dot === "err" ? "err" : sl.dot === "warn" ? "warn" : "gray";
  return (
    <Tip text={sl.title.length > 80 ? sl.title.slice(0, 77) + "…" : sl.title}>
      <span>
        <Badge tone={tone}>{sl.text}</Badge>
      </span>
    </Tip>
  );
}

/** Dải nhắc ngoại tuyến cho màn chỉ làm được khi có máy chủ. */
export function CanMayChu({ sync, viec }: { sync: ClSyncStatus | null; viec: string }) {
  if (sync?.phase !== "offline" && sync?.phase !== "error") return null;
  return (
    <Banner tone="warning" className="mb-4">
      Đang ngoại tuyến. {viec}
    </Banner>
  );
}

export function fmtTime(iso: string | null | undefined): string {
  if (!iso) return "—";
  const d = new Date(iso.includes("T") || iso.includes("Z") ? iso : iso.replace(" ", "T") + "Z");
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString("vi-VN", { hour: "2-digit", minute: "2-digit", day: "2-digit", month: "2-digit" });
}

export function fmtNgay(s: string): string {
  if (!s) return "";
  const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(s);
  return m ? `${m[3]}/${m[2]}/${m[1]}` : s;
}

/** `<select>` danh mục chỉnh lý: option từ mảng dòng, giá trị = cột `valueKey`. Bọc mỏng trên ui/Select. */
export function Select({
  value,
  onChange,
  rows,
  valueKey = "id",
  labelKey = "ten",
  empty = "Chọn",
  disabled,
  render,
  size,
  className,
}: {
  value: string;
  onChange: (v: string) => void;
  rows: ClRow[];
  valueKey?: string;
  labelKey?: string;
  /** Nhãn option rỗng; null = không có option rỗng. */
  empty?: string | null;
  disabled?: boolean;
  render?: (r: ClRow) => string;
  size?: "md" | "sm";
  className?: string;
}) {
  return (
    <UiSelect
      value={value}
      onChange={onChange}
      disabled={disabled}
      size={size}
      className={className}
      placeholder={empty ?? undefined}
      options={rows.map((r) => ({ value: str(r, valueKey), label: render ? render(r) : str(r, labelKey) }))}
    />
  );
}

/** Nhãn phông dạng "MÃ — Tên". */
export const tenPhong = (r: ClRow) => `${str(r, "ma_du_an")} — ${str(r, "ten")}`;

const SYNC_TONE: Record<string, { tone: V3y0; text: string }> = {
  clean: { tone: "ok", text: "Khớp" },
  pending: { tone: "warn", text: "Chờ đẩy" },
  conflict: { tone: "err", text: "Xung đột" },
};

/** Trạng thái đồng bộ của một dòng, dạng Badge ngắn. */
export function SyncBadgeRow({ sync }: { sync: string }) {
  const s = SYNC_TONE[sync];
  if (!s) return null;
  return <Badge tone={s.tone}>{s.text}</Badge>;
}

export const THBQ_BADGE: Record<string, V3y0> = { VV: "err", HET: "gray", GL: "ok" };

/** Chip bật / tắt dùng cho bộ lọc THBQ. */
export function Chip({ on, onClick, children, disabled }: { on: boolean; onClick: () => void; children: React.ReactNode; disabled?: boolean }) {
  return (
    <button
      type="button"
      aria-pressed={on}
      disabled={disabled}
      onClick={onClick}
      className={`h-7 rounded-full border px-2.5 text-sm transition-colors disabled:opacity-50 ${on ? "border-brand-500 bg-brand-50 text-brand-700" : "border-border-subtle bg-surface text-fg-muted hover:bg-surface-hover"}`}
    >
      {children}
    </button>
  );
}
