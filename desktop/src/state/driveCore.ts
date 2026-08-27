// Copyright (c) 2026 DigiText. All rights reserved.

import type { V416 } from "@/api/archive";
import type { V3y1 } from "@/domain/types";

export function setDriveRoot(V4w: Record<string, V3y1>, V4x: string): Record<string, V3y1> {
  return { ...V4w, root: { ...V4w.root, path: V4x, name: "S2hvIHTDoGkgbGnhu4d1" } };
}

export function V421(V4y: Record<string, V3y1>, V4z: string, V50: V416[]): Record<string, V3y1> {
  const V51 = { ...V4y };
  const V52: string[] = [];
  for (const V53 of V50) {
    V51[V53.path] = {
      id: V53.path,
      type: V53.isDir ? "folder" : "doc",
      name: V53.isDir ? V53.name : V53.name.replace(/\.pdf$/i, ""),
      path: V53.path,
      size: V53.size,
      kids: V53.isDir ? (V51[V53.path]?.kids ?? []) : undefined,
      status: V53.isPdf ? "scanned" : undefined,
    };
    V52.push(V53.path);
  }
  V51[V4z] = { ...V51[V4z], kids: V52 };
  return V51;
}
