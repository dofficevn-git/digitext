// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export function V47g(): Promise<boolean> {
  return invoke<boolean>("server_ping");
}

export interface V47h {
  configured: boolean;

  exists: boolean;
  path: string | null;
}

export function V47i(): Promise<V47h> {
  return invoke<V47h>("storage_root_status");
}
