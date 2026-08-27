// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3xa } from "./topics";

export const V3xg = 80;

type V3xh = {
  topics: ReadonlySet<V3xa>;
  fn: (V3xi: ReadonlySet<V3xa>) => void;
};

const V3xj = new Set<V3xh>();
let V3xk: Set<V3xa> = new Set();
let V3xl: ReturnType<typeof setTimeout> | null = null;

export function V3xm(topics: readonly V3xa[], V1: (V2: ReadonlySet<V3xa>) => void): () => void {
  const V3: V3xh = { topics: new Set(topics), fn: V1 };
  V3xj.add(V3);
  return () => {
    V3xj.delete(V3);
  };
}

export function V3xn(topics: Iterable<V3xa>): void {
  let V4 = false;
  for (const t of topics) {
    V3xk.add(t);
    V4 = true;
  }
  if (!V4 || V3xl !== null) return;
  V3xl = setTimeout(V3xo, V3xg);
}

export function V3xo(): void {
  if (V3xl !== null) {
    clearTimeout(V3xl);
    V3xl = null;
  }
  if (V3xk.size === 0) return;
  const V5: ReadonlySet<V3xa> = V3xk;
  V3xk = new Set();
  for (const l of [...V3xj]) {
    let V6 = false;
    for (const t of V5) {
      if (l.topics.has(t)) {
        V6 = true;
        break;
      }
    }
    if (V6) l.fn(V5);
  }
}

export function V3xp(): void {
  V3xj.clear();
  V3xk = new Set();
  if (V3xl !== null) {
    clearTimeout(V3xl);
    V3xl = null;
  }
}
