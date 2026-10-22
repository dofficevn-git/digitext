// Copyright (c) 2026 DigiText. All rights reserved.
// Đường theo ngày, tối đa 3 chuỗi (phân loại, thứ tự màu cố định) + vạch định mức. Đường 2 px, điểm đánh dấu ≥ 8 px
// chỉ ở x đang hover (viền nền 2 px), tooltip chung cho mọi chuỗi, legend tự hiện khi ≥ 2 chuỗi, bảng số kèm.

import { useMemo } from "react";
import type { Column } from "../Table";
import ChartFrame from "./ChartFrame";
import { useHoverIndex } from "./hooks";
import ChartTooltip from "./ChartTooltip";
import { CHART, fmtDay, fmtNum, niceMax, tickIndexes } from "./palette";

export interface LineSeries {
  key: string;
  label: string;
  /** Giá trị theo thứ tự nhãn x; null = không có số (đứt đoạn). */
  values: (number | null)[];
  color?: string;
}

export default function LineChart({ labels, series, unit = "", height = 180, target, ariaLabel, actions, className, emptyText }: { labels: string[]; series: LineSeries[]; unit?: string; height?: number; target?: { value: number; label: string } | null; ariaLabel: string; actions?: React.ReactNode; className?: string; emptyText?: string }) {
  const n = labels.length;
  const ss = series.slice(0, 4).map((s, k) => ({ ...s, color: s.color ?? CHART.cat[k] }));
  const { i: hov, setI, step, clear } = useHoverIndex(n);
  const all = ss.flatMap((s) => s.values.filter((v): v is number => v != null));
  const max = niceMax(Math.max(...all, target?.value ?? 0, 0));
  const empty = n === 0 || all.every((v) => v === 0);
  type Row = { label: string } & Record<string, number | null | string>;
  const rows: Row[] = useMemo(() => labels.map((l, i) => ({ label: l, ...Object.fromEntries(ss.map((s) => [s.key, s.values[i] ?? null])) })), [labels, ss]);
  const cols: Column<Row>[] = useMemo(
    () => [{ key: "label", header: "Ngày", cell: (r: Row) => fmtDay(String(r.label)) }, ...ss.map((s) => ({ key: s.key, header: s.label, align: "right" as const, cell: (r: Row) => (r[s.key] == null ? "—" : fmtNum(Number(r[s.key]))) }))],
    [ss],
  );
  return (
    <ChartFrame<Row>
      ariaLabel={ariaLabel}
      legend={ss.map((s) => ({ label: s.label, color: s.color }))}
      table={{ columns: cols, rows, getRowId: (r) => String(r.label) }}
      empty={empty}
      emptyText={emptyText}
      actions={actions}
      className={className}
      onKeyStep={step}
      onLeave={clear}
    >
      {(width) => {
        const m = { l: 40, r: 10, t: 10, b: 22 };
        const W = Math.max(0, width - m.l - m.r);
        const H = Math.max(0, height - m.t - m.b);
        const x = (i: number) => (n <= 1 ? m.l + W / 2 : m.l + (i * W) / (n - 1));
        const y = (v: number) => m.t + H - (v / max) * H;
        const ticks = tickIndexes(n, Math.max(2, Math.floor(W / 56)));
        const path = (vals: (number | null)[]) => {
          let d = "";
          let pen = false;
          vals.forEach((v, i) => {
            if (v == null) {
              pen = false;
              return;
            }
            d += `${pen ? "L" : "M"}${x(i).toFixed(1)},${y(v).toFixed(1)}`;
            pen = true;
          });
          return d;
        };
        return (
          <>
            <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} aria-hidden onMouseMove={(e) => {
              const px = e.clientX - e.currentTarget.getBoundingClientRect().left;
              if (n <= 0) return;
              const i = n <= 1 ? 0 : Math.round(((px - m.l) / W) * (n - 1));
              setI(px < m.l - 8 || px > width - m.r + 8 ? null : Math.max(0, Math.min(n - 1, i)));
            }}>
              {[0.25, 0.5, 0.75, 1].map((g) => (
                <g key={g}>
                  <line x1={m.l} x2={width - m.r} y1={y(max * g)} y2={y(max * g)} stroke={CHART.grid} strokeWidth="1" />
                  <text x={m.l - 6} y={y(max * g) + 3} fontSize="10" textAnchor="end" fill={CHART.axis}>{fmtNum(max * g)}</text>
                </g>
              ))}
              <line x1={m.l} x2={width - m.r} y1={y(0)} y2={y(0)} stroke={CHART.axis} strokeWidth="1" />
              {target && target.value > 0 && (
                <>
                  <line x1={m.l} x2={width - m.r} y1={y(target.value)} y2={y(target.value)} stroke={CHART.target} strokeWidth="1.5" strokeDasharray="4 3" />
                  <text x={width - m.r} y={y(target.value) - 4} fontSize="10" textAnchor="end" fill={CHART.target}>{target.label} {fmtNum(target.value)}</text>
                </>
              )}
              {hov != null && <line x1={x(hov)} x2={x(hov)} y1={m.t} y2={m.t + H} stroke={CHART.axis} strokeWidth="1" strokeDasharray="2 2" />}
              {ss.map((s) => (
                <path key={s.key} d={path(s.values)} fill="none" stroke={s.color} strokeWidth="2" strokeLinejoin="round" strokeLinecap="round" />
              ))}
              {hov != null &&
                ss.map((s) => {
                  const v = s.values[hov];
                  return v == null ? null : (
                    <g key={s.key}>
                      <circle cx={x(hov)} cy={y(v)} r="6" fill={CHART.surface} />
                      <circle cx={x(hov)} cy={y(v)} r="4" fill={s.color} />
                    </g>
                  );
                })}
              {ticks.map((i) => (
                <text key={i} x={x(i)} y={height - 6} fontSize="10" textAnchor={i === 0 ? "start" : i === n - 1 ? "end" : "middle"} fill={CHART.axis}>{fmtDay(labels[i])}</text>
              ))}
            </svg>
            {hov != null && (
              <ChartTooltip
                x={x(hov)}
                y={m.t + H / 2}
                width={width}
                title={fmtDay(labels[hov])}
                rows={[...ss.map((s) => ({ label: s.label, value: s.values[hov] == null ? "—" : `${fmtNum(s.values[hov] as number)}${unit ? ` ${unit}` : ""}`, color: s.color })), ...(target && target.value > 0 ? [{ label: target.label, value: fmtNum(target.value) }] : [])]}
              />
            )}
          </>
        );
      }}
    </ChartFrame>
  );
}
