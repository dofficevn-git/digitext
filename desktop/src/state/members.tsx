// Copyright (c) 2026 DigiText. All rights reserved.

import { createContext, useContext, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { V420 } from "@/mocks/fixtures";
import type { V3yi } from "@/domain/types";

interface V42f {
  members: V3yi[];
  setMembers: (m: V3yi[]) => void;
}

const V42g = createContext<V42f | null>(null);

export function V42h({ children: V6m }: { children: ReactNode }) {
  const [V6n, V6o] = useState<V3yi[]>(V420);
  const V6p = useMemo<V42f>(() => ({ members: V6n, setMembers: V6o }), [V6n]);
  return <V42g.Provider value={V6p}>{V6m}</V42g.Provider>;
}

export function V42i(): V42f {
  const c = useContext(V42g);
  if (!c) throw new Error("useMembers must be used within <MembersProvider>");
  return c;
}
