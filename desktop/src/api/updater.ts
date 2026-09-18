// Copyright (c) 2026 DigiText. All rights reserved.
//
// TỰ CẬP NHẬT — kênh tự phục vụ của DigiText.
//
// Endpoint và khoá công khai khai ở src-tauri/tauri.conf.json → plugins.updater:
//   https://api.digitext.vn/updates/latest.json  (API phát từ DIGITEXT_UPDATES_DIR)
// Khoá riêng để ký bộ cài nằm NGOÀI repo (~/.tauri/digitext.key) — xem docs/DEPLOY-DESKTOP.md.
// App chỉ cài bản được ký đúng khoá đó; máy chủ bị chiếm cũng không đẩy được bản lạ.
//
// Lượt TỰ ĐỘNG lúc mở app: im lặng khi không có mạng / lỗi (UpdateOverlay hiện hộp
// thoại với mọi phase "error", nên báo lỗi ở đây là bật hộp thoại vô cớ mỗi lần mở).
// Lượt THỦ CÔNG (nút "Kiểm tra cập nhật"): nói thật mọi kết quả.

import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { isTauri } from "@/api/ipc";
import { V42s } from "@/api/version";

export type V42t =
  | { phase: "idle" }
  | { phase: "checking"; manual?: boolean }
  | { phase: "uptodate"; manual?: boolean; version?: string }
  | { phase: "available"; version: string }
  | { phase: "downloading"; version: string; downloaded: number; total: number }
  | { phase: "installing"; version: string }
  | { phase: "relaunching"; version: string }
  | { phase: "error"; error: string; manual?: boolean };

function V42v(e: unknown): string {
  if (e instanceof Error) return e.message;
  if (typeof e === "string") return e;
  try {
    return JSON.stringify(e);
  } catch {
    return String(e);
  }
}

export async function V42u(V73: (s: V42t) => void, V74: { manual?: boolean } = {}): Promise<void> {
  const V75 = V74.manual ?? false;
  if (!isTauri()) {
    if (V75) V73({ phase: "error", manual: true, error: "Chỉ kiểm tra được trong ứng dụng desktop." });
    return;
  }
  V73({ phase: "checking", manual: V75 });
  let update;
  try {
    update = await check({ timeout: 15_000 });
  } catch (e) {
    if (V75) {
      V73({
        phase: "error",
        manual: true,
        error: `Không liên lạc được máy chủ cập nhật: ${V42v(e)}`,
      });
    } else {
      V73({ phase: "idle" });
    }
    return;
  }
  if (!update) {
    V73({ phase: "uptodate", manual: V75, version: await V42s() });
    return;
  }
  const version = update.version;
  V73({ phase: "available", version });
  try {
    let downloaded = 0;
    let total = 0;
    await update.downloadAndInstall((ev) => {
      if (ev.event === "Started") {
        total = ev.data.contentLength ?? 0;
        V73({ phase: "downloading", version, downloaded: 0, total });
      } else if (ev.event === "Progress") {
        downloaded += ev.data.chunkLength;
        V73({ phase: "downloading", version, downloaded, total });
      } else if (ev.event === "Finished") {
        V73({ phase: "installing", version });
      }
    });
    V73({ phase: "relaunching", version });
    await relaunch();
  } catch (e) {
    // Lỗi lúc TẢI/CÀI thì luôn báo (kể cả lượt tự động) — người dùng đã thấy tiến trình.
    V73({ phase: "error", manual: V75, error: `Cập nhật v${version} thất bại: ${V42v(e)}` });
  }
}
