// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useRef, useState } from "react";
import { V3xm } from "./bus";
import { V41n, V41l, type V41m } from "./gate";
import type { V3xa } from "./topics";

export interface V41o {
  topics: readonly V3xa[];

  enabled?: boolean;

  pollMs?: number;

  skipInitial?: boolean;
}

function hidden(): boolean {
  return typeof document !== "undefined" && document.visibilityState === "hidden";
}

export function V41p(
  V3x: () => unknown,
  { topics, enabled: V3y = true, pollMs: V3z = 0, skipInitial: V40 = false }: V41o,
): { reload: () => void; busy: boolean } {
  const [V41, V42] = useState(false);
  const V43 = useRef(V3x);
  V43.current = V3x;
  const V44 = useRef(true);
  const V45 = useRef(false);
  const V46 = useRef<V41m | null>(null);

  const V47 = useRef<number[]>([]);
  const V48 = useRef(false);

  if (V46.current === null) {
    V46.current = V41n(async () => {
      if (V44.current) V42(true);
      try {
        await V43.current();
      } finally {
        if (V44.current) V42(false);
      }
    });
  }
  const V49 = V46.current;

  useEffect(() => {
    V44.current = true;
    return () => {
      V44.current = false;
    };
  }, []);

  const reload = useCallback(() => {
    if (!V3y) return;
    if (hidden()) {
      V45.current = true;
      return;
    }
    V49.trigger();
  }, [V3y, V49]);

  useEffect(() => {
    if (!V3y || V40) return;
    if (V41l(V47.current, Date.now()) && !V48.current) {
      V48.current = true;

      console.error(
        "[live] Hàm nạp đổi định danh mỗi lần render nên useLiveEffect nạp lặp vô hạn. " +
          "Bọc nó trong useCallback với đúng các phụ thuộc.",
      );
    }
    V49.trigger();
  }, [V3x, V3y, V49, V40]);

  const V4a = topics.join("|");
  useEffect(() => {
    if (!V3y) return;
    return V3xm(V4a.split("|").filter(Boolean) as V3xa[], () => {
      if (hidden()) {
        V45.current = true;
        return;
      }
      V49.trigger();
    });
  }, [V4a, V3y, V49]);

  useEffect(() => {
    if (!V3y || typeof document === "undefined") return;
    const V4b = () => {
      if (!hidden() && V45.current) {
        V45.current = false;
        V49.trigger();
      }
    };
    document.addEventListener("visibilitychange", V4b);
    return () => document.removeEventListener("visibilitychange", V4b);
  }, [V3y, V49]);

  useEffect(() => {
    if (!V3y || V3z <= 0) return;
    const V4c = setInterval(() => {
      if (!hidden()) V49.trigger();
    }, V3z);
    return () => clearInterval(V4c);
  }, [V3y, V3z, V49]);

  return { reload, busy: V41 };
}

export interface V41q<T> {
  data: T | undefined;

  error: string | null;

  loading: boolean;

  refreshing: boolean;
  reload: () => void;

  setData: (V41r: T | ((V41s: T | undefined) => T)) => void;
}

export function V41t<T>(
  V4d: () => Promise<T>,
  V4e: V41o & { errorText?: (V4f: unknown) => string },
): V41q<T> {
  const { errorText: V4g } = V4e;
  const [data, V4h] = useState<T | undefined>(undefined);
  const [V4i, V4j] = useState<string | null>(null);
  const V4k = useRef(false);
  const [V4l, V4m] = useState(false);
  const V4n = useRef(true);
  useEffect(() => {
    V4n.current = true;
    return () => {
      V4n.current = false;
    };
  }, []);

  const V4o = useRef(V4g);
  V4o.current = V4g;

  const V4p = useCallback(async () => {
    try {
      const v = await V4d();
      if (!V4n.current) return;
      V4h(v);
      V4k.current = true;
      V4j(null);
    } catch (V4q) {
      if (!V4n.current) return;
      const f = V4o.current;
      V4j(f ? f(V4q) : V4q instanceof Error ? V4q.message : String(V4q));
    } finally {
      if (V4n.current) V4m(true);
    }
  }, [V4d]);

  const { reload, busy: V4r } = V41p(V4p, V4e);

  return {
    data,
    error: V4i,
    loading: !V4l && (V4e.enabled ?? true),
    refreshing: V4r && V4l,
    reload,
    setData: (V4s) =>
      V4h((V4t) => {
        V4k.current = true;
        return typeof V4s === "function" ? (V4s as (p: T | undefined) => T)(V4t) : V4s;
      }),
  };
}
