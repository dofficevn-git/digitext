// Copyright (c) 2026 DigiText. All rights reserved.

import { createContext, useContext } from "react";

export interface V45u {
  serverLive: boolean | null;

  lastChangeAt: number | null;
}

export const V45v = createContext<V45u>({
  serverLive: null,
  lastChangeAt: null,
});

export function V45w(): V45u {
  return useContext(V45v);
}
