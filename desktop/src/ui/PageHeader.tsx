// Copyright (c) 2026 DigiText. All rights reserved.
// Đầu màn: tiêu đề + nút (i) giải thích + mô tả MỘT câu ngắn + hành động bên phải. Thay `.page-head h2 + .sub`.
// 0.25 "ít chữ": mô tả ≤ 70 ký tự (hoặc bỏ); căn cứ pháp lý, cách dùng đưa vào `info` — chỉ hiện khi bấm (i).

import type { ReactNode } from "react";
import { V462 as cn } from "@/lib/utils";
import IconButton from "./IconButton";
import InfoButton from "./InfoButton";

/** Trần độ dài mô tả dưới tiêu đề (ký tự). Dài hơn thì chuyển phần giải thích sang `info`. */
export const MO_TA_TOI_DA = 70;

export default function PageHeader({
  title,
  description,
  info,
  infoTitle,
  actions,
  meta,
  onBack,
  className,
}: {
  title: ReactNode;
  description?: ReactNode;
  /** Nội dung khung giải thích mở từ nút (i) cạnh tiêu đề. */
  info?: ReactNode;
  infoTitle?: ReactNode;
  actions?: ReactNode;
  /** Badge / số liệu nhỏ cạnh tiêu đề. */
  meta?: ReactNode;
  onBack?: () => void;
  className?: string;
}) {
  if (import.meta.env.DEV && typeof description === "string" && description.length > MO_TA_TOI_DA) {
    console.warn(`[PageHeader] mô tả ${description.length} ký tự (> ${MO_TA_TOI_DA}) — rút gọn, phần giải thích chuyển sang prop info: "${description.slice(0, 60)}…"`);
  }
  return (
    <header className={cn("mb-5 flex flex-wrap items-start gap-3", className)}>
      {onBack && <IconButton label="Quay lại" icon="truoc" variant="secondary" onClick={onBack} className="mt-0.5" />}
      <div className="min-w-0 flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <h2 className="text-2xl font-semibold text-fg-strong">{title}</h2>
          {info && (
            <InfoButton title={infoTitle} label="Giải thích về màn này">
              {info}
            </InfoButton>
          )}
          {meta}
        </div>
        {description && <p className="mt-1 max-w-[80ch] text-base text-fg-subtle">{description}</p>}
      </div>
      {actions && <div className="flex flex-none flex-wrap items-center gap-2">{actions}</div>}
    </header>
  );
}
