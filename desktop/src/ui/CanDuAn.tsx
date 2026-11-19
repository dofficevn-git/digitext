// Copyright (c) 2026 DigiText. All rights reserved.
// "Chọn dự án" — trạng thái rỗng dùng chung cho mọi màn cần dự án đang làm (0.25 "ít chữ"): thay câu "Chọn dự án đang
// làm ở thanh trên…" chép ở ~8 màn. Quản trị chọn ngay tại chỗ (cùng `useProject()` với ô dự án trên thanh trên);
// tài khoản gắn sẵn dự án thì không bao giờ thấy thành phần này.

import { useState } from "react";
import { useProject } from "@/state/project";
import EmptyState from "./EmptyState";
import { Select } from "./Input";
import { notifyError } from "./toast";

export default function CanDuAn({ className }: { className?: string }) {
  const p = useProject();
  const [busy, setBusy] = useState(false);
  if (p.projectId) return null;
  const chonDuoc = !p.fromAccount && p.projects.length > 0;
  return (
    <EmptyState
      className={className}
      icon="duAn"
      title={p.loading ? "Đang nạp dự án…" : "Chọn dự án"}
      description={p.loading || chonDuoc ? undefined : p.fromAccount ? "Tài khoản chưa được gán dự án." : "Chưa có dự án nào."}
      action={
        chonDuoc ? (
          <Select
            value=""
            disabled={busy || p.loading}
            placeholder="— Dự án đang làm —"
            aria-label="Dự án đang làm"
            className="min-w-64"
            options={p.projects.map((x) => ({ value: x.id, label: `${x.code} · ${x.name}` }))}
            onChange={(id) => {
              if (!id) return;
              setBusy(true);
              p.choose(id)
                .catch((e) => notifyError(e, "chọn dự án"))
                .finally(() => setBusy(false));
            }}
          />
        ) : undefined
      }
    />
  );
}
