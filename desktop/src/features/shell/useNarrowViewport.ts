// Copyright (c) 2026 DigiText. All rights reserved.

import { useEffect, useState } from "react";

export const V46x = "(max-width: 1100px)";

export function shouldUseRail(collapsed: boolean, Vdh: boolean): boolean {
  return collapsed || Vdh;
}

export function useNarrowViewport(): boolean {
  const [Vdi, Vdj] = useState(
    () => typeof window !== "undefined" && window.matchMedia(V46x).matches,
  );

  useEffect(() => {
    const Vdk = window.matchMedia(V46x);
    const Vdl = (Vdm: MediaQueryListEvent) => Vdj(Vdm.matches);
    Vdj(Vdk.matches);
    Vdk.addEventListener("change", Vdl);
    return () => Vdk.removeEventListener("change", Vdl);
  }, []);

  return Vdi;
}
