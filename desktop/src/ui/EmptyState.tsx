// Copyright (c) 2026 DigiText. All rights reserved.
// Trạng thái rỗng thống nhất: "Chưa có …" khi chưa từng có, "Không có … khớp bộ lọc" khi lọc ra rỗng.

import type { ReactNode } from "react";
import { Icon, type V41v as IconName } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";

export default function EmptyState({
  icon = "thuMuc",
  title = "Chưa có dữ liệu",
  description,
  action,
  compact,
  className,
}: {
  icon?: IconName;
  title?: ReactNode;
  description?: ReactNode;
  action?: ReactNode;
  compact?: boolean;
  className?: string;
}) {
  return (
    <div className={cn("flex flex-col items-center justify-center text-center", compact ? "gap-1 py-6" : "gap-2 py-12", className)}>
      <span className="grid size-10 place-items-center rounded-full bg-gray-100 text-fg-subtle">
        <Icon name={icon} size={20} />
      </span>
      <p className="text-base font-medium text-fg">{title}</p>
      {description && <p className="max-w-md text-sm text-fg-subtle">{description}</p>}
      {action && <div className="mt-2">{action}</div>}
    </div>
  );
}
