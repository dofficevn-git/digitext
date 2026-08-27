// Copyright (c) 2026 DigiText. All rights reserved.

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import type { ReactNode } from "react";
import { V43c, type V43b } from "@/api/lansync";
import { V43o, V43n } from "@/api/uploadProgress";
import { V43x, V43w, V44e, V43r, V43u, V44d, type V43q } from "./uploadsCore";

interface V44f {
  uploads: V43q;

  running: boolean;

  startForward: (V44g?: number[]) => Promise<V43b>;

  cancel: (V44h?: number) => Promise<void>;

  retry: (V44i: number | number[]) => Promise<void>;

  queuedRetry: number[];

  lastError: string | null;

  notice: string | null;

  dismissItem: (V44j: number) => void;

  dismiss: () => void;
  toggleCollapsed: () => void;
}

const V44k = createContext<V44f | null>(null);

export function V44l({ children: V93 }: { children: ReactNode }) {
  const [V94, V95] = useState<V43q>(V43r);

  const V96 = useRef(V94);
  V96.current = V94;

  const [V97, V98] = useState(false);
  const [V99, V9a] = useState<string | null>(null);
  const [V9b, V9c] = useState<string | null>(null);

  const [V9d, V9e] = useState<number[]>([]);

  useEffect(() => {
    let V9f: (() => void) | null = null;
    let V9g = false;
    void V43n({
      onSession: (V9h) => V95((s) => V43w(s, V9h)),
      onFile: (V9i) => V95((s) => V43x(s, V9i, Date.now())),
    }).then((V9j) => {
      if (V9g) V9j();
      else V9f = V9j;
    });
    return () => {
      V9g = true;
      V9f?.();
    };
  }, []);

  const V9k = useCallback(async (V9l?: number[]) => {
    V98(true);
    V9a(null);
    V9c(null);
    try {
      return await V43c(V9l);
    } catch (V9m) {
      V9a(V9m instanceof Error ? V9m.message : String(V9m));
      throw V9m;
    } finally {
      V98(false);
    }
  }, []);

  const V9n = V94.running || V97;

  const V9o = useRef(V9n);
  V9o.current = V9n;

  const V9p = useCallback(async (V9q?: number) => {
    const { sessionId: V9r } = V96.current;
    if (V9r === 0) return;
    await V43o(V9r, V9q);
  }, []);

  const V9s = useCallback(
    async (V9t: number[]) => {
      V95(V43u);

      const r = await V9k(V9t);

      if (r.total === 0) {
        V95((s) => V9t.reduce(V44e, s));
        V9c(
          V9t.length === 1
            ? "VOG7h3AgbsOgeSBraMO0bmcgY8OybiB0cm9uZyBow6BuZyBjaOG7nSAoxJHDoyBsw6puIG3DoXkgY2jhu6cgaG/hurdjIMSRw6MgYuG7iyBn4buhKSDigJQgxJHDoyBi4buPIGto4buPaSBi4bqjbmcu"
            : `${V9t.length}IHThu4dwIGtow7RuZyBjw7JuIHRyb25nIGjDoG5nIGNo4budIOKAlCDEkcOjIGLhu48ga2jhu49pIGLhuqNuZy4=`,
        );
      }
    },
    [V9k],
  );

  const V9u = useCallback(
    async (V9v: number | number[]) => {
      const V9w = Array.isArray(V9v) ? V9v : [V9v];
      if (V9w.length === 0) return;

      if (V9o.current) {
        V9e((q) => V44d(q, V9w));
        V9c(
          "xJDDoyB44bq/cCBow6BuZyBn4butaSBs4bqhaSDigJQgc+G6vSBjaOG6oXkgbmdheSBraGkgcGhpw6puIGhp4buHbiB04bqhaSB4b25nLg==",
        );
        return;
      }
      await V9s(V9w);
    },
    [V9s],
  );

  useEffect(() => {
    if (V9n || V9d.length === 0) return;
    const V9x = V9d;
    V9e([]);
    void V9s(V9x).catch(() => {});
  }, [V9n, V9d, V9s]);

  const V9y = useMemo<V44f>(
    () => ({
      uploads: V94,
      running: V9n,
      startForward: V9k,
      cancel: V9p,
      retry: V9u,
      queuedRetry: V9d,
      lastError: V99,
      notice: V9b,
      dismissItem: (V9z: number) => V95((s) => V44e(s, V9z)),
      dismiss: () => V95((s) => ({ ...s, dismissed: true })),
      toggleCollapsed: () => V95((s) => ({ ...s, collapsed: !s.collapsed })),
    }),
    [V94, V9n, V9d, V99, V9b, V9k, V9p, V9u],
  );

  return <V44k.Provider value={V9y}>{V93}</V44k.Provider>;
}

export function V44m(): V44f {
  const v = useContext(V44k);
  if (!v) throw new Error("dXNlVXBsb2FkcyBwaOG6o2kgbuG6sW0gdHJvbmcgPFVwbG9hZHNQcm92aWRlcj4=");
  return v;
}
