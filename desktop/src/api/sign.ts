// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export interface V4qr {
  id: string;
  projectId: string;
  filename: string;
  hoSoRef: string | null;
  pageCount: number | null;
  status: string;
}

export interface V4qs {
  index: number;
  label: string;
}

export function V4qt(): Promise<V4qr[]> {
  return invoke<V4qr[]>("sign_list_jobs");
}

export function V4qu(): Promise<string[]> {
  return invoke<string[]>("sign_detect_modules");
}

export function V4qv(V2gw: string): Promise<V4qs[]> {
  return invoke<V4qs[]>("sign_list_tokens", { args: { modulePath: V2gw } });
}

export function V4qw(V2gx: {
  jobId: string;
  modulePath: string;
  slotIndex: number;
  pin: string;
}): Promise<string> {
  return invoke<string>("sign_execute", { args: V2gx });
}
