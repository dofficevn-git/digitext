// Copyright (c) 2026 DigiText. All rights reserved.
// Vòng phần trăm (chuyển từ ProjectProgressView `Vong`): một tỉ lệ so với 100 %. Màu theo mức: < 50 cảnh báo,
// ≥ 50 xanh ngọc, 100 xanh lá; null = chưa có số liệu (xám).

export default function Ring({ value, label, size = 84, className }: { value: number | null; label: string; size?: number; className?: string }) {
  const r = size * 0.405;
  const c = 2 * Math.PI * r;
  const v = Math.max(0, Math.min(100, value ?? 0));
  const mau = value == null ? "var(--color-gray-300)" : v >= 100 ? "var(--color-success-600)" : v >= 50 ? "var(--color-brand-500)" : "var(--color-warning-600)";
  const half = size / 2;
  return (
    <svg width={size} height={size} viewBox={`0 0 ${size} ${size}`} role="img" aria-label={`${label}: ${value == null ? "chưa có số liệu" : `${Math.round(value)}%`}`} className={className}>
      <circle cx={half} cy={half} r={r} fill="none" stroke="var(--color-border-subtle)" strokeWidth={size * 0.095} />
      <circle cx={half} cy={half} r={r} fill="none" stroke={mau} strokeWidth={size * 0.095} strokeDasharray={`${(c * v) / 100} ${c}`} transform={`rotate(-90 ${half} ${half})`} strokeLinecap="round" />
      <text x={half} y={half + size * 0.06} textAnchor="middle" fontSize={size * 0.19} fontWeight="700" fill="currentColor">
        {value == null ? "—" : `${Math.round(value)}%`}
      </text>
    </svg>
  );
}
