// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";
import type { V3yq, V3ym, V3yp, V3yl, V3yo } from "@/domain/types";

export interface V48i {
  phongBans: V3yl[];
  mangs: V3ym[];
  thbqs: V3yo[];
  ngonNgus: V3yp[];
  loaiVanBans: V3yq[];
}

export function V48j(): Promise<V48i> {
  return invoke<V48i>("catalog_list");
}
