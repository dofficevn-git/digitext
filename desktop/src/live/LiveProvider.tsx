// Copyright (c) 2026 DigiText. All rights reserved.

import { useEffect, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauri } from "@/api/ipc";
import { V45t } from "@/api/live";
import { V407 } from "@/state/session";
import { V3xn } from "./bus";
import { V45r, V45s, V45q, V45p, V45l, type V45o } from "./pollCore";
import { V45v, type V45u } from "./status";
import { V3x8, isTopic } from "./topics";

interface V45x {
  topics: string[];
}

export function V45y({ children: Vc3 }: { children: ReactNode }) {
  const { status } = V407();
  const [Vc4, Vc5] = useState<boolean | null>(null);
  const [Vc6, Vc7] = useState<number | null>(null);

  useEffect(() => {
    if (!isTauri()) return;
    let Vc8 = false;
    let Vc9: UnlistenFn | undefined;
    void listen<V45x>(V3x8, (Vca) => {
      const topics = (Vca.payload?.topics ?? []).filter(isTopic);
      if (topics.length > 0) V3xn(topics);
    }).then((u) => {
      if (Vc8) u();
      else Vc9 = u;
    });
    return () => {
      Vc8 = true;
      Vc9?.();
    };
  }, []);

  const Vcb = useRef<(() => void) | null>(null);
  useEffect(() => {
    if (!isTauri() || status !== "active") {
      Vc5(null);
      return;
    }
    let Vcc = true;
    let Vcd: ReturnType<typeof setTimeout> | undefined;
    let state: V45o = V45p;

    const Vce = (ms: number) => {
      if (!Vcc) return;
      Vcd = setTimeout(() => void Vcf(), ms);
    };

    const Vcf = async () => {
      if (!Vcc) return;

      if (typeof document !== "undefined" && document.visibilityState === "hidden") {
        Vce(V45l);
        return;
      }
      try {
        const Vcg = await V45t(state.since);
        const Vch = V45r(state, Vcg);
        state = Vch.state;
        if (Vch.publish.length > 0) {
          V3xn(Vch.publish);
          Vc7(Date.now());
        }
        Vc5(true);
      } catch {
        state = V45s(state);
        Vc5(false);
      }
      Vce(V45q(state.errors));
    };

    Vcb.current = () => {
      if (Vcd !== undefined) clearTimeout(Vcd);
      void Vcf();
    };
    void Vcf();
    return () => {
      Vcc = false;
      Vcb.current = null;
      if (Vcd !== undefined) clearTimeout(Vcd);
    };
  }, [status]);

  useEffect(() => {
    if (typeof document === "undefined") return;
    const Vci = () => {
      if (document.visibilityState === "visible") Vcb.current?.();
    };
    document.addEventListener("visibilitychange", Vci);
    return () => document.removeEventListener("visibilitychange", Vci);
  }, []);

  const Vcj = useMemo<V45u>(() => ({ serverLive: Vc4, lastChangeAt: Vc6 }), [Vc4, Vc6]);
  return <V45v.Provider value={Vcj}>{Vc3}</V45v.Provider>;
}
