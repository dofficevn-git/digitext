// Copyright (c) 2026 DigiText. All rights reserved.

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke, isTauri } from "@/api/ipc";
import type { V43b } from "@/api/lansync";

export type V43k = "uploading" | "done" | "skipped" | "error" | "cancelled";

export interface V43l {
  sessionId: number;

  id: number;
  filename: string;
  projectId: string;
  hoSoRef: string | null;

  relPath: string | null;
  sizeBytes: number;
  sentBytes: number;
  state: V43k;
  message: string | null;

  permanent: boolean;
}

export interface V43m {
  sessionId: number;
  phase: "start" | "end";
  total: number;
  summary: V43b | null;
}

export async function V43n(V7r: {
  onSession: (V7s: V43m) => void;
  onFile: (V7t: V43l) => void;
}): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  const V7u: UnlistenFn[] = [
    await listen<V43m>("upload://session", (V7v) => V7r.onSession(V7v.payload)),
    await listen<V43l>("upload://file", (V7w) => V7r.onFile(V7w.payload)),
  ];
  return () => V7u.forEach((V7x) => V7x());
}

export function V43o(V7y: number, V7z?: number): Promise<boolean> {
  return invoke<boolean>("data_forward_cancel", { args: { sessionId: V7y, id: V7z ?? null } });
}
