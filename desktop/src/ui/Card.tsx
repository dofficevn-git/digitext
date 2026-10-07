// Copyright (c) 2026 DigiText. All rights reserved.
// Khối nội dung có viền. `title` + `actions` tạo đầu thẻ; `padding="none"` cho bảng tràn viền.

import type { ReactNode } from "react";
import { V462 as cn } from "@/lib/utils";

export default function Card({
  title,
  description,
  actions,
  children,
  className,
  padding = "md",
  tone,
}: {
  title?: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
  children?: ReactNode;
  className?: string;
  padding?: "md" | "sm" | "none";
  tone?: "warning" | "danger" | "info";
}) {
  const pad = padding === "none" ? "" : padding === "sm" ? "p-3" : "p-4";
  const border = tone === "warning" ? "border-warning-border" : tone === "danger" ? "border-danger-border" : tone === "info" ? "border-info-border" : "border-border-subtle";
  return (
    <section className={cn("rounded-lg border bg-surface", border, className)}>
      {(title || actions) && (
        <header className={cn("flex flex-wrap items-center gap-2 border-b border-border-subtle px-4 py-3", padding === "none" && "")}>
          <div className="min-w-0 flex-1">
            {title && <h3 className="text-base font-semibold text-fg-strong">{title}</h3>}
            {description && <p className="text-sm text-fg-subtle">{description}</p>}
          </div>
          {actions && <div className="flex flex-none items-center gap-2">{actions}</div>}
        </header>
      )}
      {children !== undefined && <div className={pad}>{children}</div>}
    </section>
  );
}
