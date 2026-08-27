// Copyright (c) 2026 DigiText. All rights reserved.

import { listen } from "@tauri-apps/api/event";
import { invoke, isTauri } from "@/api/ipc";
import type { V3yh, V3yg, V3y7, V3ye, V3yf, V3yb, V3yc, V3y6 } from "@/domain/types";

export interface V46y {
  onState?: (state: V3yh) => void;
  onPage?: (V46z: V3yc) => void;
  onError?: (message: string) => void;

  onEnhanced?: (V470: number, V471: V3yf | null, V472: number, V473: number) => void;
}

export interface V474 {
  batchId: string;

  cancel: () => Promise<void>;

  dispose: () => void;
}

export async function V475(): Promise<V3y6[]> {
  if (!isTauri()) return [];
  return invoke<V3y6[]>("list_scanners");
}

export async function V476(Vep: V3yb, Veq: V46y): Promise<V474> {
  return V47f(Vep, Veq);
}

export async function V477(Ver: string, Ves: V3ye): Promise<V3yf> {
  return invoke<V3yf>("process_scan_page", { sourcePath: Ver, ops: Ves });
}

export async function V478(Vet: string): Promise<V3yf> {
  return invoke<V3yf>("reset_scan_page", { sourcePath: Vet });
}

export async function V479(Veu: string): Promise<V3yf> {
  return invoke<V3yf>("undo_scan_page", { sourcePath: Veu });
}

export async function V47a(Vev: string): Promise<V3yf> {
  return invoke<V3yf>("redo_scan_page", { sourcePath: Vev });
}

export async function V47b(Vew: string): Promise<V3yg> {
  if (!isTauri()) return { canUndo: false, canRedo: false };
  return invoke<V3yg>("scan_page_history", { sourcePath: Vew });
}

export async function V47c(): Promise<string[]> {
  if (!isTauri()) return [];
  return invoke<string[]>("scanner_driver_extensions");
}

export async function V47d(path: string): Promise<V3y7> {
  return invoke<V3y7>("install_scanner_driver", { path });
}

const V47e: ReadonlySet<V3yh["kind"]> = new Set(["completed", "cancelled", "disconnected"]);

async function V47f(Vex: V3yb, Vey: V46y): Promise<V474> {
  const Vez: Array<() => void> = [];
  const Vf0 = () => {
    for (const u of Vez) u();
    Vez.length = 0;
  };

  let Vf1 = "";
  const Vf2 = (Vf3: string) => Vf1 === "" || Vf3 === Vf1;

  Vez.push(
    await listen<{ batchId: string; state: V3yh }>("scan://state", (Vf4) => {
      if (!Vf2(Vf4.payload.batchId)) return;
      Vey.onState?.(Vf4.payload.state);
      if (V47e.has(Vf4.payload.state.kind)) Vf0();
    }),
  );
  Vez.push(
    await listen<{ batchId: string; page: V3yc }>("scan://page", (Vf5) => {
      if (Vf2(Vf5.payload.batchId)) Vey.onPage?.(Vf5.payload.page);
    }),
  );
  Vez.push(
    await listen<{ batchId: string; message: string }>("scan://error", (Vf6) => {
      if (Vf2(Vf6.payload.batchId)) Vey.onError?.(Vf6.payload.message);
    }),
  );
  Vez.push(
    await listen<{
      batchId: string;
      index: number;
      result: V3yf | null;
      done: number;
      total: number;
    }>("scan://enhanced", (Vf7) => {
      const p = Vf7.payload;
      if (Vf2(p.batchId)) Vey.onEnhanced?.(p.index, p.result, p.done, p.total);
    }),
  );

  try {
    Vf1 = await invoke<string>("start_scan_batch", { opts: Vex });
  } catch (Vf8) {
    Vf0();
    throw Vf8;
  }

  return {
    batchId: Vf1,

    cancel: async () => {
      await invoke("cancel_scan_batch", { batchId: Vf1 });
    },
    dispose: Vf0,
  };
}
