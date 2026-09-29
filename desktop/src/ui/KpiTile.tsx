// Copyright (c) 2026 DigiText. All rights reserved.
// Ô số liệu: số to + nhãn; `tone` tô màu số; `onClick` biến thành nút.

import type { ReactNode } from "react";
import { V462 as cn } from "@/lib/utils";

export default function KpiTile({
  label,
  value,
  sub,
  tone,
  onClick,
  className,
}: {
  label: string;
  value: number | string;
  sub?: ReactNode;
  tone?: "ok" | "warn" | "err" | "info";
  onClick?: () => void;
  className?: string;
}) {
  const color = tone === "ok" ? "text-success-700" : tone === "warn" ? "text-warning-700" : tone === "err" ? "text-danger-700" : tone === "info" ? "text-info-700" : "text-fg-strong";
  const body = (
    <>
      <span className={cn("block text-2xl font-semibold tabular-nums leading-tight", color)}>{typeof value === "number" ? value.toLocaleString("vi-VN") : value}</span>
      <span className="block text-sm text-fg-subtle">{label}</span>
      {sub && <span className="block text-xs text-fg-subtle">{sub}</span>}
    </>
  );
  const cls = cn("rounded-md border border-border-subtle bg-surface px-3 py-2 text-left min-w-[7.5rem]", onClick && "hover:border-border-strong hover:bg-surface-hover transition-colors", className);
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
