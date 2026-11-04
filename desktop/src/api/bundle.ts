// Copyright (c) 2026 DigiText. All rights reserved.
// Nén thư mục bộ hồ sơ nghiệm thu thành <thư mục>.zip cạnh nó — src-tauri/src/bundle.rs.
import { invoke } from "@/api/ipc";

export function zipDir(dir: string): Promise<string> {
  return invoke<string>("zip_dir", { args: { dir } });
}
