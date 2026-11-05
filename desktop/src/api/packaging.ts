// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export interface V4rt {
  id: string;
  projectId: string;
  kind: string;
  status: string;
  hoSoRef: string | null;
  label: string;

  fondCode: string | null;
  fondName: string | null;
  objid: string | null;
  zipName: string | null;
  zipSha256: string | null;
  zipSize: number | null;
  documentCount: number;
  error: string | null;

  virusScan: string | null;

  submissionId: string | null;

  convertedFrom: string | null;

  sttFrom: number | null;
  sttTo: number | null;
  createdAt: string;
  updatedAt: string;
}

export function V4ru(): Promise<V4rt[]> {
  return invoke<V4rt[]>("packages_list");
}

export function V4rv(V2nt: {
  projectId?: string | null;
  hoSoRef?: string | null;

  submissionId?: string | null;

  sttFrom?: number | null;
  sttTo?: number | null;
  request: unknown;
}): Promise<V4rt> {
  return invoke<V4rt>("package_create", { args: V2nt });
}

export function V4rw(V2nu: string, V2nv: string, dir?: string): Promise<string> {
  return invoke<string>("package_download", { args: { id: V2nu, zipName: V2nv, dir: dir ?? null } });
}
