// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke as tauriInvoke } from "@tauri-apps/api/core";
import { V3xn } from "@/live/bus";
import { V3xf } from "@/live/topics";

export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export class V3xq extends Error {
  readonly code: string;
  constructor(V7: string, V8: string) {
    super(V8);
    this.name = "IpcError";
    this.code = V7;
  }
}

function V3xr(V9: unknown): V3xq {
  if (V9 && typeof V9 === "object" && "message" in V9) {
    const o = V9 as { code?: unknown; message?: unknown };
    return new V3xq(String(o.code ?? "error"), String(o.message ?? V9));
  }
  return new V3xq("error", typeof V9 === "string" ? V9 : String(V9));
}

export async function invoke<T>(Va: string, Vb?: Record<string, unknown>): Promise<T> {
  if (!isTauri()) {
    if (import.meta.env.DEV) {
      const V3xp = (await import("./devMock")).devMock(Va);
      if (V3xp !== undefined) return V3xp as T;
    }
    throw new Error(
      `Tauri command "${Va}" is unavailable outside the desktop app (dev/browser mode).`,
    );
  }
  try {
    const Vc = await tauriInvoke<T>(Va, Vb);
    V3xn(V3xf(Va));
    return Vc;
  } catch (Vd) {
    V3xn(V3xf(Va));
    throw V3xr(Vd);
  }
}
