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
        <span className="flex h-8 max-w-72 items-center gap-2 rounded-md border border-border-subtle bg-surface px-2.5 text-sm font-medium text-fg">
          <span className="size-2 flex-none rounded-full bg-run-solid" aria-hidden />
          <span className="truncate">{p.projectLabel}</span>
        </span>
      </Tip>
    );
  }
  const none = !p.projectId;
  return (
    <div className={cn("flex h-8 items-center gap-1.5 rounded-md border border-border-subtle bg-surface", none && "border-warning-300 ring-2 ring-warning-200")}>
      {/* Chấm hổ phách = dự án đang làm (màu "đang chạy" của hệ thống); chưa chọn → biểu tượng cảnh báo. */}
      {none ? <Icon name="duAn" size={14} className="ml-2 flex-none text-warning-700" /> : <span className="ml-2.5 size-2 flex-none rounded-full bg-run-solid" aria-hidden />}
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
