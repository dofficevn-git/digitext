// Copyright (c) 2026 DigiText. All rights reserved.
// Chú giải: ô màu + tên chuỗi (+ giá trị khi có). Chữ màu mực, chỉ ô vuông mang màu chuỗi.

export interface LegendItem {
  label: string;
  color: string;
  value?: string;
}

export default function ChartLegend({ items, className }: { items: LegendItem[]; className?: string }) {
  return (
    <ul className={`m-0 flex flex-wrap items-center gap-x-4 gap-y-1 p-0 text-xs text-fg-muted ${className ?? ""}`} aria-label="Chú giải">
      {items.map((it) => (
        <li key={it.label} className="flex items-center gap-1.5">
          <span className="inline-block size-2.5 rounded-sm" style={{ background: it.color }} aria-hidden />
          <span>{it.label}</span>
          {it.value != null && <span className="font-medium tabular-nums text-fg">{it.value}</span>}
        </li>
      ))}
    </ul>
  );
}
