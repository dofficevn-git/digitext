// Copyright (c) 2026 DigiText. All rights reserved.

import { getVersion } from "@tauri-apps/api/app";
import { isTauri } from "@/api/ipc";

declare const _V42q: string;
export const V42r: string = typeof _V42q !== "undefined" ? _V42q : "dev";

export async function V42s(): Promise<string> {
  if (!isTauri()) return V42r;
  try {
    return await getVersion();
  } catch {
    return V42r;
  }
}
