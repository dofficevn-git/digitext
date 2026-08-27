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
  if (!d) return "xJBhbmcgbeG7nyB0w6BpIGxp4buHdeKApg==";
  if (d.total && d.total > 0) {
    const V1ir = Math.min(100, Math.round((d.received / d.total) * 100));
    return `xJBhbmcgdOG6o2kgdMOgaSBsaeG7h3UgdOG7qyBtw6F5IGNo4bun4oCmIA==${V1ir}% (${V4hx(d.received)}/${V4hx(d.total)})`;
  }
  if (d.received > 0)
    return `xJBhbmcgdOG6o2kgdMOgaSBsaeG7h3UgdOG7qyBtw6F5IGNo4bun4oCmIA==${V4hx(d.received)}`;
  return "xJBhbmcgdOG6o2kgdMOgaSBsaeG7h3UgdOG7qyBtw6F5IGNo4bun4oCm";
}

export function V4i0(V1is: number, V1it: boolean, V1iu: boolean): string {
  if (V1iu) return `xJBhbmcgbuG6oXAgdGjDqm0gdHJhbmfigKYgKMSRw6MgaGnhu4duIA==${V1is})`;
  if (V1it)
    return `xJDDoyBoaeG7h24g${V1is}IHRyYW5nIOKAlCBjdeG7mW4geHXhu5FuZyDEkeG7gyB4ZW0gdGnhur9w`;
  return `xJDDoyBoaeG7h24gxJHhu6cg${V1is} trang`;
}
