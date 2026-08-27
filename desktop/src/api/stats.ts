// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export interface V4qz {
  pagesScanned: number;
  hoSoCount: number;
  pdfCount: number;
  pdfBytes: number;
}

export interface V4r0 extends V4qz {
  userId: string;
  username: string;
  displayName: string;
  projectId: string | null;

  filesUploaded: number;
  bytesUploaded: number;
  reportedAt: string | null;
}

export function V4r1(): Promise<V4qz> {
  return invoke<V4qz>("stats_local");
}

export function V4r2(): Promise<V4qz> {
  return invoke<V4qz>("stats_push");
}

export function V4r3(): Promise<V4r0[]> {
  return invoke<V4r0[]>("stats_summary");
}
