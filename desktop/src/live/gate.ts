// Copyright (c) 2026 DigiText. All rights reserved.

export const V41j = 1_000;

export const V41k = 25;

export function V41l(V3q: number[], V3r: number): boolean {
  V3q.push(V3r);
  while (V3q.length > 0 && V3r - V3q[0] > V41j) V3q.shift();
  return V3q.length > V41k;
}

export interface V41m {
  trigger(): void;

  readonly busy: boolean;

  readonly queued: boolean;
}

export function V41n(V3s: () => Promise<void>): V41m {
  let V3t = false;
  let V3u = false;

  const V3v = (): void => {
    V3t = true;
    void V3s().then(V3w, V3w);
  };

  function V3w(): void {
    V3t = false;
    if (V3u) {
      V3u = false;
      V3v();
    }
  }

  return {
    trigger(): void {
      if (V3t) {
        V3u = true;
        return;
      }
      V3v();
    },
    get busy(): boolean {
      return V3t;
    },
    get queued(): boolean {
      return V3u;
    },
  };
}
