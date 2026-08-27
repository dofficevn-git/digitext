// Copyright (c) 2026 DigiText. All rights reserved.

import { createContext, useCallback, useContext, useMemo, useState } from "react";
import type { ReactNode } from "react";
import type { V3y1 } from "@/domain/types";

export type V42j = "create" | "members" | null;

interface V42k {
  dialog: V42j;
  dialogTitle: string;
  openDialog: (V42l: Exclude<V42j, null>, V42m?: string) => void;
  closeDialog: () => void;
  previewDoc: V3y1 | null;
  openPreview: (n: V3y1) => void;
  closePreview: () => void;
}

const V42n = createContext<V42k | null>(null);

export function V42o({ children: V6q }: { children: ReactNode }) {
  const [dialog, V6r] = useState<V42j>(null);
  const [V6s, V6t] = useState("");
  const [V6u, V6v] = useState<V3y1 | null>(null);

  const V6w = useCallback((V6x: Exclude<V42j, null>, V6y = "") => {
    V6r(V6x);
    V6t(V6y);
  }, []);
  const V6z = useCallback(() => V6r(null), []);
  const V70 = useCallback((n: V3y1) => V6v(n), []);
  const V71 = useCallback(() => V6v(null), []);

  const V72 = useMemo<V42k>(
    () => ({
      dialog,
      dialogTitle: V6s,
      openDialog: V6w,
      closeDialog: V6z,
      previewDoc: V6u,
      openPreview: V70,
      closePreview: V71,
    }),
    [dialog, V6s, V6w, V6z, V6u, V70, V71],
  );
  return <V42n.Provider value={V72}>{V6q}</V42n.Provider>;
}

export function V42p(): V42k {
  const c = useContext(V42n);
  if (!c) throw new Error("useUi must be used within <UiProvider>");
  return c;
}
