// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";
import type { V409, V408 } from "@/api/archive";

export interface V49v {
  applied: boolean;
  version: number;
  created: number;

  offeredSiteCode?: string;

  peerFingerprint?: string;

  peerDeviceId?: string;
}

export function V49w(Vp7: string, Vp8: V408, Vp9: V409[], Vpa = false): Promise<number> {
  return invoke<number>("tree_publish", {
    args: { projectId: Vp7, phong: Vp8, hoSos: Vp9, stripAccents: Vpa },
  });
}

export function V49x(): Promise<V49v> {
  return invoke<V49v>("tree_pull");
}
