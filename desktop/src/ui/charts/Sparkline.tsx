// Copyright (c) 2026 DigiText. All rights reserved.
// Đường nhỏ 14–30 điểm trong ô bảng / thẻ số: 2 px, chấm cuối, vạch tham chiếu đứt (định mức / trung bình).
// Không có trục; con số cụ thể nằm ở Tip và ở cột số bên cạnh.

import Tip from "../Tooltip";
import { CHART, fmtNum } from "./palette";

export default function Sparkline({ values, width = 96, height = 24, refValue, color = CHART.cat[0], ariaLabel, className }: { values: number[]; width?: number; height?: number; refValue?: number | null; color?: string; ariaLabel: string; className?: string }) {
  const n = values.length;
  const max = Math.max(1, ...values, refValue ?? 0);
  const pad = 3;
  const sx = (i: number) => (n <= 1 ? width / 2 : pad + (i * (width - pad * 2)) / (n - 1));
  const sy = (v: number) => height - pad - (v / max) * (height - pad * 2);
  const pts = values.map((v, i) => `${sx(i).toFixed(1)},${sy(v).toFixed(1)}`).join(" ");
  const last = n ? values[n - 1] : 0;
  const hi = n ? Math.max(...values) : 0;
  const lo = n ? Math.min(...values) : 0;
  const text = n ? `Cao nhất ${fmtNum(hi)} · thấp nhất ${fmtNum(lo)} · mới nhất ${fmtNum(last)}` : "Chưa có số liệu";
  return (
    <Tip text={text}>
      <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} role="img" aria-label={`${ariaLabel}: ${text}`} className={className}>
        {refValue != null && refValue > 0 && <line x1={pad} x2={width - pad} y1={sy(refValue)} y2={sy(refValue)} stroke={CHART.target} strokeWidth="1" strokeDasharray="3 3" />}
        {n > 1 && <polyline points={pts} fill="none" stroke={color} strokeWidth="2" strokeLinejoin="round" strokeLinecap="round" />}
        {n > 0 && (
          <>
            <circle cx={sx(n - 1)} cy={sy(last)} r="4" fill={CHART.surface} />
            <circle cx={sx(n - 1)} cy={sy(last)} r="3" fill={color} />
          </>
        )}
      </svg>
    </Tip>
  );
}
