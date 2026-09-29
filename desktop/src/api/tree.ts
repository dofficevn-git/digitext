// Copyright (c) 2026 DigiText. All rights reserved.
// Cây thư mục quét: lấy hồ sơ từ mục lục chỉnh lý của dự án đang làm vào bảng hồ sơ trên máy.

import { invoke } from "@/api/ipc";

export interface TreeSeedResult {
  hoSo: number;
  dossier: number;
}

export function treeSeedFromCl(maDuAn: string): Promise<TreeSeedResult> {
  return invoke<TreeSeedResult>("tree_seed_from_cl", { args: { maDuAn } });
}
