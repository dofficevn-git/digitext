// Copyright (c) 2026 DigiText. All rights reserved.

export const V4hs = 3;

export const V4ht = 6;

export function V4hu(V1im: number): { from: number; count: number } {
  return { from: V1im + 1, count: V1im === 0 ? V4hs : V4ht };
}

export function V4hv(V1in: number, V1io: number): boolean {
  return V1in >= V1io && V1io > 0;
}

export function V4hw(
  V1ip: { scrollTop: number; clientHeight: number; scrollHeight: number },
  margin = 400,
): boolean {
  return V1ip.scrollTop + V1ip.clientHeight >= V1ip.scrollHeight - margin;
}

export function V4hx(V1iq: number): string {
  if (!Number.isFinite(V1iq) || V1iq <= 0) return "0 MB";
  return `${(V1iq / 1_048_576).toFixed(1)} MB`;
}

export interface V4hy {
  received: number;
  total: number | null;
}

export function V4hz(d: V4hy | null): string {
  if (!d) return "Đang mở tài liệu…";
  if (d.total && d.total > 0) {
    const V1ir = Math.min(100, Math.round((d.received / d.total) * 100));
    return `Đang tải tài liệu từ máy chủ… ${V1ir}% (${V4hx(d.received)}/${V4hx(d.total)})`;
  }
  if (d.received > 0)
    return `Đang tải tài liệu từ máy chủ… ${V4hx(d.received)}`;
  return "Đang tải tài liệu từ máy chủ…";
}

export function V4i0(V1is: number, V1it: boolean, V1iu: boolean): string {
  if (V1iu) return `Đang nạp thêm trang… (đã hiện ${V1is})`;
  if (V1it)
    return `Đã hiện ${V1is} trang — cuộn xuống để xem tiếp`;
  return `Đã hiện đủ ${V1is} trang`;
}
