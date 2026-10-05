// Copyright (c) 2026 DigiText. All rights reserved.
// Kho ví dụ dạy máy bóc trường (máy chủ /kie/examples, 5/10/2026).

import { invoke } from "@/api/ipc";

export interface KieExample {
  id: string;
  source: string;
  projectId: string | null;
  projectCode: string | null;
  documentId: string | null;
  filename: string | null;
  extRef: string | null;
  pageNo: number | null;
  ocrText: string;
  fields: Record<string, string>;
  active: boolean;
  usedCount: number;
  lastUsedAt: string | null;
  createdByName: string | null;
  updatedAt: string;
}

/** all = ví dụ của dự án + ví dụ chung; project = chỉ của dự án; general = chỉ ví dụ chung (plakie). */
export type KieScope = "all" | "project" | "general";

export interface KieListArgs {
  projectId?: string | null;
  scope?: KieScope;
  active?: boolean | null;
  q?: string;
  order?: "used" | "recent";
  limit?: number;
  offset?: number;
}

export interface KieListResp {
  total: number;
  items: KieExample[];
}

export const kieExamplesList = (args: KieListArgs) => invoke<KieListResp>("kie_examples_list", { args });
export const kieExampleSetActive = (id: string, active: boolean) =>
  invoke<{ id: string; active: boolean }>("kie_example_set_active", { id, active });
