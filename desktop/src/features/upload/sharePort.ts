// Copyright (c) 2026 DigiText. All rights reserved.

export const V4qb = 45817;

export function V4qc(V2cm: string): string | null {
  const t = V2cm.trim();
  if (t === "") return null;
  if (!/^\d+$/.test(t))
    return `Cổng chia sẻ phải là số 1024–65535 (bỏ trống = cổng chung ${V4qb}).`;
  const n = Number(t);
  if (n < 1024 || n > 65535)
    return "Cổng chia sẻ phải trong khoảng 1024–65535.";
  return null;
}
