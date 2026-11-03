// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";

/** Lưu tệp kết xuất: mở hộp thoại Lưu; có `dir` thì ghi thẳng vào thư mục đó (bộ hồ sơ nghiệm thu). */
export async function V4dz(Vzl: Uint8Array, Vzm: string, Vzn: string, dir?: string): Promise<string | null> {
  return await invoke<string | null>("save_export", {
    args: { data: Array.from(Vzl), ext: Vzn, defaultName: Vzm, open: false, dir: dir ?? null },
  });
}

export async function V4e0(Vzo: string): Promise<void> {
  const Vzp = new TextEncoder().encode(Vzo);
  await invoke<string | null>("save_export", {
    args: { data: Array.from(Vzp), ext: "html", open: true },
  });
}
