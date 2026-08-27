// Copyright (c) 2026 DigiText. All rights reserved.

import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { isTauri } from "@/api/ipc";
import { V42s } from "@/api/version";

export type V42t =
  | { phase: "idle" }
  | { phase: "checking"; manual?: boolean }
  | { phase: "uptodate"; manual?: boolean; version?: string }
  | { phase: "available"; version: string }
  | { phase: "downloading"; version: string; downloaded: number; total: number }
  | { phase: "installing"; version: string }
  | { phase: "relaunching"; version: string }
  | { phase: "error"; error: string; manual?: boolean };

export async function V42u(V73: (s: V42t) => void, V74: { manual?: boolean } = {}): Promise<void> {
  const V75 = V74.manual ?? false;

  if (!isTauri()) {
    if (V75) V73({ phase: "uptodate", manual: V75, version: await V42s() });
    return;
  }

  V73({ phase: "checking", manual: V75 });
  try {
    const V76 = await check();
    if (!V76) {
      V73({ phase: "uptodate", manual: V75, version: await V42s() });
      return;
    }

    const V77 = V76.version;
    V73({ phase: "available", version: V77 });

    let V78 = 0;
    let V79 = 0;
    await V76.downloadAndInstall((V7a) => {
      switch (V7a.event) {
        case "Started":
          V79 = V7a.data.contentLength ?? 0;
          V73({ phase: "downloading", version: V77, downloaded: V78, total: V79 });
          break;
        case "Progress":
          V78 += V7a.data.chunkLength;
          V73({ phase: "downloading", version: V77, downloaded: V78, total: V79 });
          break;
        case "Finished":
          V73({ phase: "installing", version: V77 });
          break;
      }
    });

    V73({ phase: "relaunching", version: V77 });
    await relaunch();
  } catch (V7b) {
    V73({ phase: "error", error: String(V7b), manual: V75 });
  }
}
