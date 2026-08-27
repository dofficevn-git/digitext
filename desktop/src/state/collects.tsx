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
import { V43f, V43a, type V43d, type V43e, type V439 } from "@/api/lansync";
import { V44u, V44t } from "@/api/collectProgress";
import { V452, V450, V451, V45f, V44x, type V44w } from "./collectsCore";

interface V45g {
  collects: V44w;

  running: boolean;
  startCollect: () => Promise<V439>;

  startAddFiles: (V45h: V43d) => Promise<V43e>;

  cancel: (key?: string) => Promise<void>;

  lastError: string | null;
  dismissItem: (key: string) => void;
  dismiss: () => void;
  toggleCollapsed: () => void;
}

const V45i = createContext<V45g | null>(null);

export function V45j({ children: Vb9 }: { children: ReactNode }) {
  const [Vba, Vbb] = useState<V44w>(V44x);

  const Vbc = useRef(Vba);
  Vbc.current = Vba;
  const [Vbd, Vbe] = useState(false);
  const [Vbf, Vbg] = useState<string | null>(null);

  useEffect(() => {
    let Vbh: (() => void) | null = null;
    let Vbi = false;
    void V44t({
      onSession: (Vbj) => Vbb((s) => V450(s, Vbj)),
      onStage: (Vbk) => Vbb((s) => V451(s, Vbk)),
      onFile: (Vbl) => Vbb((s) => V452(s, Vbl, Date.now())),
    }).then((Vbm) => {
      if (Vbi) Vbm();
      else Vbh = Vbm;
    });
    return () => {
      Vbi = true;
      Vbh?.();
    };
  }, []);

  const Vbn = useCallback(async () => {
    Vbe(true);
    Vbg(null);
    try {
      return await V43a();
    } catch (Vbo) {
      Vbg(Vbo instanceof Error ? Vbo.message : String(Vbo));
      throw Vbo;
    } finally {
      Vbe(false);
    }
  }, []);

  const Vbp = useCallback(async (Vbq: V43d) => {
    Vbe(true);
    Vbg(null);
    try {
      return await V43f(Vbq);
    } catch (Vbr) {
      Vbg(Vbr instanceof Error ? Vbr.message : String(Vbr));
      throw Vbr;
    } finally {
      Vbe(false);
    }
  }, []);

  const Vbs = useCallback(async (Vbt?: string) => {
    const { sessionId: Vbu } = Vbc.current;
    if (Vbu === 0) return;
    await V44u(Vbu, Vbt);
  }, []);

  const Vbv = useMemo<V45g>(
    () => ({
      collects: Vba,
      running: Vba.running || Vbd,
      startCollect: Vbn,
      startAddFiles: Vbp,
      cancel: Vbs,
      lastError: Vbf,
      dismissItem: (Vbw: string) => Vbb((s) => V45f(s, Vbw)),
      dismiss: () => Vbb((s) => ({ ...s, dismissed: true })),
      toggleCollapsed: () => Vbb((s) => ({ ...s, collapsed: !s.collapsed })),
    }),
    [Vba, Vbd, Vbf, Vbn, Vbp, Vbs],
  );

  return <V45i.Provider value={Vbv}>{Vb9}</V45i.Provider>;
}

export function V45k(): V45g {
  const v = useContext(V45i);
  if (!v) throw new Error("useCollects phải nằm trong <CollectsProvider>");
  return v;
}
