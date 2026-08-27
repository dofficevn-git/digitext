// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";
import type { V409, V408 } from "@/api/archive";
import type { V49v } from "@/api/sync";

export interface V49y {
  publishing: boolean;
  code?: string;
  port?: number;
  project?: string;
  version?: number;
}

export interface V49z {
  instance: string;
  addr: string;
  port: number;
  project: string;
  version: number;

  fingerprint: string;

  source: string;
}

export const V4a0 = 45821;

export function V4a1(Vpb: string, Vpc: number, Vpd: V408, Vpe: V409[], Vpf = false): Promise<V49y> {
  return invoke<V49y>("lan_publish_start", {
    args: { project: Vpb, version: Vpc, phong: Vpd, hoSos: Vpe, stripAccents: Vpf },
  });
}

export function V4a2(): Promise<V49y> {
  return invoke<V49y>("lan_publish_stop");
}

export function V4a3(): Promise<V49y> {
  return invoke<V49y>("lan_status");
}

export function V4a4(Vpg?: number): Promise<V49z[]> {
  return invoke<V49z[]>("lan_discover", { args: { timeoutMs: Vpg ?? null } });
}

export function V4a5(Vph: string, Vpi: number, Vpj: string): Promise<V49v> {
  return invoke<V49v>("lan_pull", { args: { addr: Vph, port: Vpi, code: Vpj } });
}
