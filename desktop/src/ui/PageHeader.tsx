// Copyright (c) 2026 DigiText. All rights reserved.
// Đầu màn: tiêu đề + mô tả một câu + hành động bên phải. Thay `.page-head h2 + .sub`.

import type { ReactNode } from "react";
import { V462 as cn } from "@/lib/utils";
import IconButton from "./IconButton";

export default function PageHeader({
  title,
  description,
  actions,
  meta,
  onBack,
  className,
}: {
  title: ReactNode;
  description?: ReactNode;
  actions?: ReactNode;
  /** Badge / số liệu nhỏ cạnh tiêu đề. */
  meta?: ReactNode;
  onBack?: () => void;
  className?: string;
}) {
  return (
    <header className={cn("mb-5 flex flex-wrap items-start gap-3", className)}>
      {onBack && <IconButton label="Quay lại" icon="truoc" variant="secondary" onClick={onBack} className="mt-0.5" />}
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <h2 className="text-2xl font-semibold text-fg-strong">{title}</h2>
          {meta}
        </div>
        {description && <p className="mt-1 max-w-[80ch] text-base text-fg-subtle">{description}</p>}
      </div>
      {actions && <div className="flex flex-none flex-wrap items-center gap-2">{actions}</div>}
    </header>
  );
}
