// Copyright (c) 2026 DigiText. All rights reserved.

export function V51e(V3oh: string): string | null {
  for (const V3oi of V3oh.split(/[\s,;]+/)) {
    const t = V3oi.trim();
    if (t === "") continue;
    const V3oj = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})\/(\d{1,2})$/.exec(t);
    if (V3oj) {
      if (V3oj.slice(1, 5).some((o) => Number(o) > 255)) {
        return `Dải "${t}" có địa chỉ IPv4 không hợp lệ.`;
      }
      if (Number(V3oj[5]) > 32)
        return `Dải "${t}" có độ dài tiền tố không hợp lệ (0–32).`;
      continue;
    }
    const m = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})(?::(\d{1,5}))?$/.exec(t);
    if (!m) {
      return `Mục "${t}" phải có dạng IP, IP:cổng hoặc dải CIDR (vd 192.168.1.18:64040, 192.168.5.0/24).`;
    }
    if (m.slice(1, 5).some((o) => Number(o) > 255)) {
      return `Mục "${t}" có địa chỉ IPv4 không hợp lệ.`;
    }
    if (m[5] !== undefined) {
      const V3ok = Number(m[5]);
      if (V3ok < 1 || V3ok > 65535)
        return `Mục "${t}" có cổng không hợp lệ (1–65535).`;
    }
  }
  return null;
}
