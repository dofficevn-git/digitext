// Copyright (c) 2026 DigiText. All rights reserved.
// Thanh ngang phần–toàn thể: tệp theo trạng thái, tồn đọng theo nhóm tuổi. Khe 2 px giữa các đoạn, nhãn trực tiếp ở
// chú giải (tên + số), hover từng đoạn. Màu: dải tuần tự khi các phần có thứ tự (nhạt → đậm), hoặc phân loại.

import { useState } from "react";
import ChartLegend from "./ChartLegend";
import { useWidth } from "./hooks";
import ChartTooltip from "./ChartTooltip";
import { CHART, fmtNum } from "./palette";

export interface Segment {
  key: string;
  label: string;
  value: number;
  color?: string;
}

export default function StackedBar({ segments, total, height = 12, ariaLabel, legend = true, palette = "seq", className }: { segments: Segment[]; total?: number; height?: number; ariaLabel: string; legend?: boolean; palette?: "seq" | "cat"; className?: string }) {
  const [hov, setHov] = useState<number | null>(null);
  const [ref, width] = useWidth<HTMLDivElement>();
  const sum = segments.reduce((a, s) => a + Math.max(0, s.value), 0);
  const denom = Math.max(total ?? sum, sum, 1);
  const colors = palette === "cat" ? CHART.cat : segments.length <= 5 ? CHART.seq.slice(5 - segments.length) : CHART.seq;
  const segs = segments.map((s, i) => ({ ...s, color: s.color ?? colors[i % colors.length] }));
  const text = segs.map((s) => `${s.label} ${fmtNum(s.value)}`).join(", ");
  const before = (i: number) => segs.slice(0, i).reduce((a, s) => a + Math.max(0, s.value), 0);
  return (
    // eslint-disable-next-line jsx-a11y/no-static-element-interactions -- chỉ tắt tooltip khi rời chuột; số liệu luôn có ở chú giải
    <div className={`relative min-w-0 ${className ?? ""}`} onMouseLeave={() => setHov(null)}>
      <div ref={ref} className="flex w-full overflow-hidden rounded" style={{ height, gap: 2, background: CHART.muted }} role="img" aria-label={`${ariaLabel}: ${sum ? text : "chưa có số liệu"}`}>
        {segs.map((s, i) =>
          s.value > 0 ? (
            // eslint-disable-next-line jsx-a11y/no-static-element-interactions -- đoạn chỉ hiện tooltip; số liệu luôn có ở chú giải bên dưới
            <span key={s.key} className="h-full min-w-[2px] rounded-[2px] transition-opacity" style={{ width: `${(s.value / denom) * 100}%`, background: s.color, opacity: hov == null || hov === i ? 1 : 0.55 }} onMouseEnter={() => setHov(i)} />
          ) : null,
        )}
      </div>
      {hov != null && segs[hov] && width > 0 && (
        <ChartTooltip x={((before(hov) + segs[hov].value / 2) / denom) * width} y={-8} width={width} title={segs[hov].label} rows={[{ label: "Số lượng", value: `${fmtNum(segs[hov].value)} · ${Math.round((segs[hov].value / denom) * 100)} %`, color: segs[hov].color }]} />
      )}
      {legend && <ChartLegend className="mt-1.5" items={segs.map((s) => ({ label: s.label, color: s.color, value: fmtNum(s.value) }))} />}
    </div>
  );
}
