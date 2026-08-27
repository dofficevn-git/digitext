// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "./ipc";
import type { V45n } from "@/live/pollCore";

export async function V45t(Vc2: number | null): Promise<V45n> {
  return invoke<V45n>("changes_poll", { since: Vc2 });
}
