// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export interface V42z {
  sharing: boolean;
  port: number | null;
  fileCount: number;
  hasSiteCode: boolean;

  siteCodeWeak: boolean;

  deviceFingerprint: string;

  blocked: V430[];
}

export interface V430 {
  hoSoRef: string;
  reason: string;
}

export function V431(): Promise<V42z> {
  return invoke<V42z>("data_share_status");
}

export function V432(): Promise<string | null> {
  return invoke<string | null>("lan_site_code_get");
}

export function V433(V7k: string, V7l = false): Promise<boolean> {
  return invoke<boolean>("lan_site_code_set", { args: { code: V7k, allowWeak: V7l } });
}

export function V434(): Promise<string> {
  return invoke<string>("lan_site_code_generate");
}

export function V435(): Promise<number | null> {
  return invoke<number | null>("lan_share_port_get");
}

export function V436(V7m: string): Promise<boolean> {
  return invoke<boolean>("lan_share_port_set", { args: { port: V7m } });
}

export function V437(): Promise<string | null> {
  return invoke<string | null>("lan_manual_peers_get");
}

export function V438(V7n: string): Promise<number> {
  return invoke<number>("lan_manual_peers_set", { args: { peers: V7n } });
}

export interface V439 {
  peers: number;
  local: number;
  collected: number;
  skipped: number;
  failed: number;
  total: number;
  scannedLan: boolean;
  mirrored: number;

  peersReachable: number;
  manifestFailed: number;
  advertised: number;
  notes: string[];

  foreign: number;

  projectGateStale: boolean;

  scannedRanges: string[];

  scanClamped: boolean;

  orphans: number;

  khoTruncated: boolean;

  cancelled: number;
}

export function V43a(): Promise<V439> {
  return invoke<V439>("data_collect", { args: {} });
}

export interface V43b {
  forwarded: number;
  skipped: number;
  failed: number;
  total: number;

  retagged: number;

  rejected: number;

  cancelled: number;
}

export function V43c(V7o?: number[]): Promise<V43b> {
  return invoke<V43b>("data_forward", { args: { ids: V7o ?? null } });
}

export interface V43d {
  paths: string[];

  projectId?: string;

  destRel?: string;

  phongMa?: string;
}

export interface V43e {
  total: number;
  added: number;

  skipped: number;
  failed: number;

  cancelled: number;

  projectId: string;

  dest: string;
  notes: string[];
}

export function V43f(V7p: V43d): Promise<V43e> {
  return invoke<V43e>("collect_add_files", { args: V7p });
}

export interface V43g {
  id: number;
  sourceUsername: string | null;
  projectId: string;

  phongMa: string | null;
  hoSoRef: string | null;
  filename: string;
  pageCount: number | null;
  sizeBytes: number;
  forwarded: boolean;
  forwardTries: number;

  forwardFatal: boolean;

  collectedAt: string;

  forwardError: string | null;

  storagePath: string;
}

export interface V43h {
  total: number;
  pending: number;
  forwarded: number;

  rejected: number;
  rows: V43g[];
}

export function V43i(V7q?: number): Promise<V43h> {
  return invoke<V43h>("collected_list", { args: { limit: V7q } });
}

export function V43j(): Promise<number> {
  return invoke<number>("collected_retry_rejected");
}
