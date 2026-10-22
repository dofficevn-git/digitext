// Copyright (c) 2026 DigiText. All rights reserved.
// Ô số liệu: số to + nhãn; `tone` tô màu số; `onClick` biến thành nút. Có `icon` → bố cục hàng (ô icon + chữ) như thẻ
// số của Tổng quan; `size="lg"` cho con số dẫn đầu; `loading` hiện khung xám khi chưa có số; `error` giữ số cũ + báo nhỏ.

import type { ReactNode } from "react";
import { V462 as cn } from "@/lib/utils";
import { Icon, type V41v as IconName } from "@/components/Icon";
import Skeleton from "./Skeleton";

export type KpiTone = "ok" | "warn" | "err" | "info";
const TONE_TEXT: Record<KpiTone, string> = { ok: "text-success-700", warn: "text-warning-700", err: "text-danger-700", info: "text-info-700" };
const TONE_BG: Record<KpiTone, string> = { ok: "bg-success-50 text-success-700", warn: "bg-warning-50 text-warning-700", err: "bg-danger-50 text-danger-700", info: "bg-info-50 text-info-700" };

export default function KpiTile({
  label,
  value,
  sub,
  tone,
  onClick,
  className,
  icon,
  size = "md",
  loading,
  error,
  extra,
}: {
  label: string;
  value: number | string | undefined;
  sub?: ReactNode;
  tone?: KpiTone;
  onClick?: () => void;
  className?: string;
  icon?: IconName;
  size?: "md" | "lg";
  loading?: boolean;
  error?: unknown;
  /** Nội dung phụ bên phải (sparkline, badge). */
  extra?: ReactNode;
}) {
  const color = tone ? TONE_TEXT[tone] : "text-fg-strong";
  const numCls = cn("block font-semibold tabular-nums leading-tight", size === "lg" ? "text-3xl" : "text-2xl", color);
  const num = loading && value === undefined ? <Skeleton.Line className={cn("mt-1 w-16", size === "lg" ? "h-8" : "h-6")} /> : <span className={numCls}>{value === undefined ? "—" : typeof value === "number" ? value.toLocaleString("vi-VN") : value}</span>;
  const subLine = error ? <span className="block truncate text-xs text-warning-700">Không cập nhật được</span> : sub ? <span className="block truncate text-xs text-fg-subtle">{sub}</span> : null;
  const body = icon ? (
    <>
      <span className={cn("grid size-10 flex-none place-items-center rounded-md", tone ? TONE_BG[tone] : "bg-gray-100 text-fg-muted")}>
        <Icon name={icon} size={18} />
      </span>
      <span className="min-w-0 flex-1">
        <span className="block truncate text-sm text-fg-subtle">{label}</span>
        {num}
        {subLine}
      </span>
      {extra}
      {onClick && <Icon name="sau" size={14} className="flex-none text-fg-subtle" />}
    </>
  ) : (
    <>
      <span className="flex items-start justify-between gap-2">
        <span className="min-w-0">
          {num}
          <span className="block text-sm text-fg-subtle">{label}</span>
          {subLine}
        </span>
        {extra}
      </span>
    </>
  );
  const cls = cn(
    "rounded-lg border border-border-subtle bg-surface text-left",
    icon ? "flex w-full items-center gap-3 p-3" : "block px-3.5 py-2.5 min-w-[7.5rem]",
    onClick && "hover:border-border-strong hover:bg-surface-hover transition-colors outline-none focus-visible:ring-2 focus-visible:ring-ring/40",
    className,
  );
  return onClick ? (
    <button type="button" className={cls} onClick={onClick}>
      {body}
    </button>
  ) : (
    <div className={cls}>{body}</div>
  );
}

export function KpiRow({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cn("flex flex-wrap gap-2", className)}>{children}</div>;
}
