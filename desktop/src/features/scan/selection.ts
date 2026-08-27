// Copyright (c) 2026 DigiText. All rights reserved.

import type { V40a } from "@/api/archive";
import type { V3yc } from "@/domain/types";

export function V4oh(V22p: ReadonlySet<number>, V22q: number): Set<number> {
  const V22r = new Set(V22p);
  if (V22r.has(V22q)) V22r.delete(V22q);
  else V22r.add(V22q);
  return V22r;
}

export function V4oi(V22s: V3yc[], V22t: ReadonlySet<number>): V40a[] {
  return V22s.filter((p) => V22t.has(p.index))
    .sort((a, b) => a.index - b.index)
    .map((p) => ({ sourcePath: p.sourcePath, dpi: p.dpi, paper: p.paper }));
}
