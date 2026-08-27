// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useRef, useState } from "react";
import { V3zf } from "@/api/auth";
import type { V3ze } from "@/api/auth";
import { isTauri } from "@/api/ipc";
import { V4wz, V4x0 } from "@/state/captchaCore";

export interface V4x1 {
  challenge: V3ze | null;
  answer: string;
  setAnswer: (v: string) => void;

  refresh: () => void;

  loading: boolean;

  loadError: boolean;

  ready: boolean;
}

export function V4x2(): V4x1 {
  const [V385, V386] = useState<V3ze | null>(null);
  const [V387, V388] = useState("");
  const [V389, V38a] = useState(false);
  const [V38b, V38c] = useState(false);

  const V38d = useRef(true);
  useEffect(() => {
    V38d.current = true;
    return () => {
      V38d.current = false;
    };
  }, []);

  const V38e = useCallback(() => {
    if (!isTauri()) return;
    V388("");
    V38a(true);
    V38c(false);
    V3zf()
      .then((c) => {
        if (!V38d.current) return;
        V386(c);
        V38c(false);
      })
      .catch(() => {
        if (!V38d.current) return;
        V386(null);
        V38c(true);
      })
      .finally(() => {
        if (V38d.current) V38a(false);
      });
  }, []);

  useEffect(() => V38e(), [V38e]);

  const V38f = useCallback((v: string) => V388(V4wz(v)), []);

  return {
    challenge: V385,
    answer: V387,
    setAnswer: V38f,
    refresh: V38e,
    loading: V389,
    loadError: V38b,
    ready: V385 !== null && V4x0(V387),
  };
}
