// Copyright (c) 2026 DigiText. All rights reserved.
// Thanh lọc MỘT hàng (0.25 "ít chữ"): ô tìm + các ô chọn đặt làm con; bên dưới chỉ hiện chip của bộ lọc ĐANG BẬT
// (bấm × để gỡ) và "Xoá lọc". Thay thẻ lọc nhiều hàng + dải chip luôn hiện.

import type { ReactNode } from "react";
import { Icon } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";

export interface FilterChip {
  key: string;
  label: string;
  onRemove: () => void;
}

export default function FilterBar({ children, active = [], onClear, className }: { children: ReactNode; active?: FilterChip[]; onClear?: () => void; className?: string }) {
  return (
    <div role="search" className={cn("rounded-lg border border-border-subtle bg-surface px-2.5 py-2 shadow-xs", className)}>
      <div className="flex flex-wrap items-center gap-2">{children}</div>
      {active.length > 0 && (
        <div className="mt-2 flex flex-wrap items-center gap-1.5 border-t border-border-subtle pt-2">
          {active.map((c) => (
            <span key={c.key} className="inline-flex h-6 items-center gap-1 rounded-full bg-brand-50 pl-2.5 pr-1 text-sm font-medium text-brand-700">
              {c.label}
              <button type="button" aria-label={`Bỏ lọc ${c.label}`} onClick={c.onRemove} className="grid size-4 place-items-center rounded-full outline-none hover:bg-brand-100 focus-visible:ring-2 focus-visible:ring-ring/40">
                <Icon name="dong" size={11} />
              </button>
            </span>
          ))}
          {onClear && (
            <button type="button" onClick={onClear} className="ml-1 rounded text-sm font-medium text-fg-subtle underline-offset-4 outline-none hover:text-fg hover:underline focus-visible:ring-2 focus-visible:ring-ring/40">
              Xoá lọc
            </button>
          )}
        </div>
      )}
    </div>
  );
}
