// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

export type V4fd = "ocr_raw" | "ocr_corrected";

export interface V4fe {
  pageNo: number;
  engineVersionId: string;
  textSource: V4fd;
  fieldKey: string;
  value: string;
  bbox: number[] | null;
  confidence: number | null;
  verified: boolean;
  updatedAt: string;
}

export interface V4ff {
  documentId: string;
  pageNo: number;
  fieldKey: string;
  value: string;

  engineVersionId?: string;
  bbox?: number[] | null;

  verified?: boolean;
}

export function V4fg(V184: string): Promise<V4fe[]> {
  return invoke<V4fe[]>("ocr_fields_list", { args: { documentId: V184 } });
}

export function V4fh(V185: V4ff): Promise<V4fe> {
  return invoke<V4fe>("ocr_field_correct", { args: V185 });
}

export interface V4fi {
  key: string;
  x: number;
  y: number;
  w: number;
  h: number;
  text: string;
  conf: number | null;
}

export interface V4fj {
  pageNo: number;
  pageW: number;
  pageH: number;
  boxes: V4fi[];
}

export function V4fk(V186: string, V187: number): Promise<V4fj> {
  return invoke<V4fj>("ocr_run_page", { args: { filePath: V186, pageNo: V187 } });
}

export function V4fl(V188: string, V189: number, V18a: V4fi[]): Promise<void> {
  return invoke<void>("ocr_save_boxes", { args: { filePath: V188, pageNo: V189, boxes: V18a } });
}
