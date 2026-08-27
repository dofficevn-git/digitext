// Copyright (c) 2026 DigiText. All rights reserved.

import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { V42u, type V42t } from "@/api/updater";
import { V42s as resolveAppVersion } from "@/api/version";

interface V42v {
  appVersion: string;
  update: V42t;
  checkForUpdates: () => void;
  dismissUpdate: () => void;
}

const V42w = createContext<V42v | null>(null);

export function V42x({ children: V7c }: { children: ReactNode }) {
  const [V7d, V7e] = useState("");
  const [V7f, V7g] = useState<V42t>({ phase: "idle" });

  useEffect(() => {
    resolveAppVersion().then(V7e);
    V42u(V7g);
  }, []);

  const V7h = useCallback(() => V42u(V7g, { manual: true }), []);
  const V7i = useCallback(() => V7g({ phase: "idle" }), []);

  const V7j = useMemo<V42v>(
    () => ({ appVersion: V7d, update: V7f, checkForUpdates: V7h, dismissUpdate: V7i }),
    [V7d, V7f, V7h, V7i],
  );
  return <V42w.Provider value={V7j}>{V7c}</V42w.Provider>;
}

export function V42y(): V42v {
  const c = useContext(V42w);
  if (!c) throw new Error("useUpdate must be used within <UpdateProvider>");
  return c;
}
