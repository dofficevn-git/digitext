// Copyright (c) 2026 DigiText. All rights reserved.

export function V51e(V3oh: string): string | null {
  for (const V3oi of V3oh.split(/[\s,;]+/)) {
    const t = V3oi.trim();
    if (t === "") continue;
    const V3oj = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})\/(\d{1,2})$/.exec(t);
    if (V3oj) {
      if (V3oj.slice(1, 5).some((o) => Number(o) > 255)) {
        return `ROG6o2kgIg==${t}IiBjw7MgxJHhu4thIGNo4buJIElQdjQga2jDtG5nIGjhu6NwIGzhu4cu`;
      }
      if (Number(V3oj[5]) > 32)
        return `ROG6o2kgIg==${t}IiBjw7MgxJHhu5kgZMOgaSB0aeG7gW4gdOG7kSBraMO0bmcgaOG7o3AgbOG7hyAoMOKAkzMyKS4=`;
      continue;
    }
    const m = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})(?::(\d{1,5}))?$/.exec(t);
    if (!m) {
      return `TeG7pWMgIg==${t}IiBwaOG6o2kgY8OzIGThuqFuZyBJUCwgSVA6Y+G7lW5nIGhv4bq3YyBk4bqjaSBDSURSICh2ZCAxOTIuMTY4LjEuMTg6NjQwNDAsIDE5Mi4xNjguNS4wLzI0KS4=`;
    }
    if (m.slice(1, 5).some((o) => Number(o) > 255)) {
      return `TeG7pWMgIg==${t}IiBjw7MgxJHhu4thIGNo4buJIElQdjQga2jDtG5nIGjhu6NwIGzhu4cu`;
    }
    if (m[5] !== undefined) {
      const V3ok = Number(m[5]);
      if (V3ok < 1 || V3ok > 65535)
        return `TeG7pWMgIg==${t}IiBjw7MgY+G7lW5nIGtow7RuZyBo4bujcCBs4buHICgx4oCTNjU1MzUpLg==`;
    }
  }
  return null;
}
