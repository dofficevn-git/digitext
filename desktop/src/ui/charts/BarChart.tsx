// Copyright (c) 2026 DigiText. All rights reserved.
// Cột theo ngày (một chuỗi): so độ lớn giữa các ngày. Một màu, cột đang hover đậm hơn, khe 2 px, vạch định mức đứt,
// tooltip từng cột, bàn phím ←/→, bảng số kèm.

import { useMemo } from "react";
import type { Column } from "../Table";
import ChartFrame from "./ChartFrame";
import { useHoverIndex } from "./hooks";
import ChartTooltip from "./ChartTooltip";
import { CHART, fmtDay, fmtNum, niceMax, tickIndexes } from "./palette";

export interface BarDatum {
  /** Nhãn trục x (ngày ISO sẽ được rút gọn dd/MM). */
  label: string;
  value: number;
}

export default function BarChart({ data, unit = "", height = 160, target, ariaLabel, color = CHART.cat[0], onBarClick, actions, className, emptyText }: { data: BarDatum[]; unit?: string; height?: number; target?: { value: number; label: string } | null; ariaLabel: string; color?: string; onBarClick?: (d: BarDatum, i: number) => void; actions?: React.ReactNode; className?: string; emptyText?: string }) {
  const n = data.length;
  const { i: hov, setI, step, clear } = useHoverIndex(n);
  const max = niceMax(Math.max(...data.map((d) => d.value), target?.value ?? 0));
  const empty = n === 0 || data.every((d) => d.value === 0);
  const cols: Column<BarDatum>[] = useMemo(
    () => [
      { key: "label", header: "Ngày", cell: (d) => fmtDay(d.label) },
      { key: "value", header: unit || "Giá trị", align: "right", cell: (d) => fmtNum(d.value) },
    ],
    [unit],
  );
  const summary = empty ? "" : `cao nhất ${fmtNum(Math.max(...data.map((d) => d.value)))} ${unit} ngày ${fmtDay(data.reduce((a, b) => (b.value > a.value ? b : a)).label)}`;
  return (
    <ChartFrame<BarDatum> ariaLabel={summary ? `${ariaLabel}: ${summary}` : ariaLabel} table={{ columns: cols, rows: data, getRowId: (d) => d.label }} empty={empty} emptyText={emptyText} actions={actions} className={className} onKeyStep={step} onLeave={clear}>
      {(width) => {
        const m = { l: 40, r: 8, t: 10, b: 22 };
        const W = Math.max(0, width - m.l - m.r);
        const H = Math.max(0, height - m.t - m.b);
        const slot = n ? W / n : 0;
        const gap = Math.min(2, slot / 4);
        const bw = Math.max(1, slot - gap);
        const y = (v: number) => m.t + H - (v / max) * H;
        const ticks = tickIndexes(n, Math.max(2, Math.floor(W / 56)));
        const grid = [0.25, 0.5, 0.75, 1];
        return (
          <>
            <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} aria-hidden onMouseMove={(e) => {
              const x = e.clientX - e.currentTarget.getBoundingClientRect().left - m.l;
              if (slot > 0) setI(x < 0 || x > W ? null : Math.min(n - 1, Math.floor(x / slot)));
            }}>
              {grid.map((g) => (
                <g key={g}>
                  <line x1={m.l} x2={width - m.r} y1={y(max * g)} y2={y(max * g)} stroke={CHART.grid} strokeWidth="1" />
                  <text x={m.l - 6} y={y(max * g) + 3} fontSize="10" textAnchor="end" fill={CHART.axis}>{fmtNum(max * g)}</text>
                </g>
              ))}
              <line x1={m.l} x2={width - m.r} y1={y(0)} y2={y(0)} stroke={CHART.axis} strokeWidth="1" />
              {data.map((d, i) => {
                const h = Math.max(0, y(0) - y(d.value));
                return (
                  <rect
                    key={d.label}
                    x={m.l + i * slot + gap / 2}
                    y={y(0) - h}
                    width={bw}
                    height={h}
                    rx={Math.min(2, bw / 2)}
                    fill={color}
                    opacity={hov == null || hov === i ? 1 : 0.55}
                    style={onBarClick ? { cursor: "pointer" } : undefined}
                    onClick={onBarClick ? () => onBarClick(d, i) : undefined}
                  />
                );
              })}
              {target && target.value > 0 && (
                <>
                  <line x1={m.l} x2={width - m.r} y1={y(target.value)} y2={y(target.value)} stroke={CHART.target} strokeWidth="1.5" strokeDasharray="4 3" />
                  <text x={width - m.r} y={y(target.value) - 4} fontSize="10" textAnchor="end" fill={CHART.target}>{target.label} {fmtNum(target.value)}</text>
                </>
              )}
              {ticks.map((i) => (
                <text key={i} x={m.l + i * slot + slot / 2} y={height - 6} fontSize="10" textAnchor="middle" fill={CHART.axis}>{fmtDay(data[i].label)}</text>
              ))}
            </svg>
            {hov != null && data[hov] && (
              <ChartTooltip x={m.l + hov * slot + slot / 2} y={y(data[hov].value)} width={width} title={fmtDay(data[hov].label)} rows={[{ label: unit || "Giá trị", value: fmtNum(data[hov].value), color }, ...(target && target.value > 0 ? [{ label: target.label, value: fmtNum(target.value) }] : [])]} />
            )}
          </>
        );
      }}
    </ChartFrame>
  );
}
