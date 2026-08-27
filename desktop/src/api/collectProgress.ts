// Copyright (c) 2026 DigiText. All rights reserved.

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke, isTauri } from "@/api/ipc";
import type { V43e, V439 } from "@/api/lansync";

export type V44n = "downloading" | "done" | "skipped" | "error" | "cancelled";

export type V44o = "lan" | "disk";

export type V44p = "local" | "kho" | "discover" | "peer" | "disk";

export interface V44q {
  sessionId: number;

  key: string;
  filename: string;
  sourceUsername: string | null;

  peer: string;

  hoSoRef: string | null;
  sizeBytes: number;
  receivedBytes: number;
  state: V44n;
  message: string | null;
}

export interface V44r {
  sessionId: number;
  stage: V44p;
  label: string;
  peers: number;
}

export interface V44s {
  sessionId: number;
  phase: "start" | "end";
  kind: V44o;

  summary: V439 | V43e | null;
}

export async function V44t(Va0: {
  onSession: (Va1: V44s) => void;
  onStage: (Va2: V44r) => void;
  onFile: (Va3: V44q) => void;
}): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  const Va4: UnlistenFn[] = [
    await listen<V44s>("collect://session", (Va5) => Va0.onSession(Va5.payload)),
    await listen<V44r>("collect://stage", (Va6) => Va0.onStage(Va6.payload)),
    await listen<V44q>("collect://file", (Va7) => Va0.onFile(Va7.payload)),
  ];
  return () => Va4.forEach((Va8) => Va8());
}

export function V44u(Va9: number, Vaa?: string): Promise<boolean> {
  return invoke<boolean>("data_collect_cancel", { args: { sessionId: Va9, key: Vaa ?? null } });
}
