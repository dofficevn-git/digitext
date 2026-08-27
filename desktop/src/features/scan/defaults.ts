// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3y4, V3y3, V3y5 } from "@/domain/types";

export interface V4of {
  dpi: V3y3;
  color: V3y4;
  source: V3y5;

  duplex: boolean;
}

export const V4og: V4of = {
  dpi: 300,
  color: "rgb",
  source: "adf",
  duplex: true,
};
