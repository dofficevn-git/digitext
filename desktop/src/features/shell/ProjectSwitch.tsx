// Copyright (c) 2026 DigiText. All rights reserved.
// Ô "Dự án đang làm" ở Topbar. Tài khoản gắn dự án → chỉ hiện tên. Quản trị không gắn → chọn từ danh sách.

import { useState } from "react";
import { Icon } from "@/components/Icon";
import { useProject } from "@/state/project";
import { V462 as cn } from "@/lib/utils";
import { notifyError, Select, Tip } from "@/ui";

export default function ProjectSwitch() {
  const p = useProject();
  const [busy, setBusy] = useState(false);
  if (p.fromAccount) {
    if (!p.projectLabel) return null;
    return (
      <Tip text="Dự án của tài khoản">
        <span className="flex h-7 max-w-72 items-center gap-1.5 rounded border border-border-subtle bg-gray-50 px-2.5 text-sm text-fg-muted">
          <Icon name="duAn" size={14} className="flex-none text-fg-subtle" />
          <span className="truncate">{p.projectLabel}</span>
        </span>
      </Tip>
    );
  }
  const none = !p.projectId;
  return (
    <div className={cn("flex items-center gap-1.5", none && "rounded-md ring-2 ring-warning-300")}>
      <Icon name="duAn" size={14} className={cn("ml-2 flex-none", none ? "text-warning-700" : "text-fg-subtle")} />
      <Select
        size="sm"
        value={p.projectId ?? ""}
        onChange={(id) => {
          if (!id) return;
          setBusy(true);
          p.choose(id)
            .catch((e) => notifyError(e, "chọn dự án"))
            .finally(() => setBusy(false));
        }}
        disabled={busy || p.loading}
        placeholder={p.projects.length ? "— Chọn dự án đang làm —" : "Chưa có dự án"}
        options={p.projects.map((x) => ({ value: x.id, label: `${x.code} · ${x.name}` }))}
        aria-label="Dự án đang làm"
        className="w-auto max-w-80 border-transparent bg-transparent pl-1 font-medium text-fg hover:bg-surface-hover"
        title={p.cached ? "Danh sách dự án lấy từ bộ nhớ đệm (chưa tới máy chủ)" : undefined}
      />
    </div>
  );
}
