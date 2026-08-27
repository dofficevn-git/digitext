// Copyright (c) 2026 DigiText. All rights reserved.

import { V4a0 } from "@/api/lan";

export function V4a6(Vpk: string): { addr: string; port: number } | null {
  const t = Vpk.trim();
  if (t === "") return null;
  const m = /^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})(?::(\d{1,5}))?$/.exec(t);
  if (!m) return null;
  if (m.slice(1, 5).some((o) => Number(o) > 255)) return null;
  const Vpl = m[5] === undefined ? V4a0 : Number(m[5]);
  if (Vpl < 1 || Vpl > 65535) return null;
  return { addr: m.slice(1, 5).join("."), port: Vpl };
}

export function V4a7(Vpm: string): string | null {
  const t = Vpm.trim();
  if (t === "") return null;
  return V4a6(t)
    ? null
    : `xJDhu4thIGNo4buJICI=${t}IiBwaOG6o2kgY8OzIGThuqFuZyBJUCBob+G6t2MgSVA6Y+G7lW5nICh2ZCAxOTIuMTY4LjEuMTAgaG/hurdjIDE5Mi4xNjguMS4xMDo=${V4a0}).`;
}
