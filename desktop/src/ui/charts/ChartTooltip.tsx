// Copyright (c) 2026 DigiText. All rights reserved.
// Tooltip của biểu đồ: đặt tuyệt đối trong khung, lật sang trái khi gần mép phải. Chữ màu mực; ô màu mang nhận diện chuỗi.

import type { ReactNode } from "react";

export interface TooltipRow {
  label: string;
  value: string;
  color?: string;
}

export default function ChartTooltip({ x, y, width, title, rows }: { x: number; y: number; width: number; title: ReactNode; rows: TooltipRow[] }) {
  const flip = x > width * 0.62;
  return (
    <div
      role="tooltip"
      className="pointer-events-none absolute z-10 min-w-[7rem] rounded-md border border-border-subtle bg-surface px-2.5 py-1.5 text-xs shadow-md"
      style={{ left: x, top: y, transform: `translate(${flip ? "calc(-100% - 10px)" : "10px"}, -50%)` }}
    >
      <div className="mb-0.5 font-medium text-fg-strong">{title}</div>
      <ul className="m-0 space-y-0.5 p-0">
        {rows.map((r) => (
          <li key={r.label} className="flex items-center gap-1.5 whitespace-nowrap text-fg-muted">
            {r.color && <span className="inline-block size-2 rounded-sm" style={{ background: r.color }} aria-hidden />}
            <span>{r.label}</span>
            <span className="ml-auto pl-3 font-medium tabular-nums text-fg">{r.value}</span>
          </li>
        ))}
      </ul>
    </div>
  );
}
