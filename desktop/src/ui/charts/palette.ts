// Copyright (c) 2026 DigiText. All rights reserved.
// Bảng màu + định dạng dùng chung cho mọi biểu đồ. Màu lấy từ token theme.css (--color-chart-*), không hardcode hex
// trong thành phần: đổi theme là biểu đồ đổi theo. Quy tắc (skill dataviz): phân loại ≤ 4 chuỗi theo thứ tự cố định,
// tuần tự một màu nhạt → đậm, chữ luôn màu mực (không tô theo chuỗi), màu trạng thái chỉ cho trạng thái.

export const CHART = {
  /** Chuỗi phân loại, thứ tự cố định — chuỗi 1 = xanh ngọc. Quá 4 chuỗi thì gộp "Khác" hoặc tách biểu đồ. */
  cat: ["var(--color-chart-1)", "var(--color-chart-2)", "var(--color-chart-3)", "var(--color-chart-4)"] as const,
  /** Dải tuần tự nhạt → đậm (5 bậc). */
  seq: ["var(--color-chart-seq-1)", "var(--color-chart-seq-2)", "var(--color-chart-seq-3)", "var(--color-chart-seq-4)", "var(--color-chart-seq-5)"] as const,
  grid: "var(--color-chart-grid)",
  axis: "var(--color-chart-axis)",
  target: "var(--color-chart-target)",
  muted: "var(--color-chart-muted)",
  surface: "var(--color-surface)",
  status: { ok: "var(--color-success-600)", warn: "var(--color-warning-600)", err: "var(--color-danger-600)" } as const,
} as const;

export function useChartPalette() {
  return CHART;
}

export const fmtNum = (n: number) => (Number.isInteger(n) ? n.toLocaleString("vi-VN") : n.toLocaleString("vi-VN", { maximumFractionDigits: 1 }));

/** `2026-10-22` → `22/10`. Chuỗi không phải ngày thì trả nguyên. */
export function fmtDay(iso: string): string {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso);
  return m ? `${m[3]}/${m[2]}` : iso;
}
/** `2026-10-22` → `22/10/2026`. */
export function fmtDayFull(iso: string | null | undefined): string {
  if (!iso) return "—";
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(iso);
  return m ? `${m[3]}/${m[2]}/${m[1]}` : iso;
}

/** Trần trục y "đẹp": 0 → 1; còn lại làm tròn lên bậc 1/2/5 × 10^n. */
export function niceMax(v: number): number {
  if (!(v > 0)) return 1;
  const p = 10 ** Math.floor(Math.log10(v));
  const r = v / p;
  const m = r <= 1 ? 1 : r <= 2 ? 2 : r <= 5 ? 5 : 10;
  return m * p;
}

/** Chọn chỉ số nhãn trục x sao cho không quá `maxTicks` nhãn: luôn có đầu và cuối. */
export function tickIndexes(n: number, maxTicks: number): number[] {
  if (n <= 0) return [];
  if (n <= maxTicks) return Array.from({ length: n }, (_, i) => i);
  const step = Math.ceil((n - 1) / (maxTicks - 1));
  const out: number[] = [];
  for (let i = 0; i < n; i += step) out.push(i);
  if (out[out.length - 1] !== n - 1) out.push(n - 1);
  return out;
}
