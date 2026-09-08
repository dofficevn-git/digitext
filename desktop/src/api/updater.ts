// Copyright (c) 2026 DigiText. All rights reserved.
//
// TÍNH NĂNG TỰ CẬP NHẬT ĐANG TẮT.
//
// Bản trước dùng tauri-plugin-updater trỏ tới releases của một tài khoản GitHub
// bên ngoài, kèm khoá công khai minisign của tài khoản đó. Nghĩa là ứng dụng
// TIN và tự cài bất kỳ bản cập nhật nào được ký bằng khoá riêng tương ứng —
// một quyền rất lớn giao cho bên thứ ba. Đã gỡ toàn bộ: plugin, endpoint,
// khoá công khai, và cả createUpdaterArtifacts lúc đóng gói.
//
// Kiểu dữ liệu và chữ ký hàm giữ nguyên để phần giao diện không phải sửa gì.
// Muốn bật lại: cài @tauri-apps/plugin-updater, khai endpoint + pubkey CỦA MÌNH
// trong tauri.conf.json, rồi khôi phục phần gọi check()/downloadAndInstall().

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

export async function V42u(V73: (s: V42t) => void, V74: { manual?: boolean } = {}): Promise<void> {
  const V75 = V74.manual ?? false;

  // Lượt kiểm tra TỰ ĐỘNG lúc khởi động: im lặng tuyệt đối. UpdateOverlay hiện
  // hộp thoại với mọi phase "error" bất kể tự động hay thủ công, nên báo lỗi ở
  // đây sẽ bật hộp thoại vô cớ mỗi lần mở ứng dụng.
  if (!V75) return;

  // Người dùng tự bấm "Kiểm tra cập nhật" thì phải nói thật, không im lặng và
  // cũng không giả vờ "đã là bản mới nhất".
  V73({
    phase: "error",
    manual: true,
    error: `Bản dựng này không bật tự cập nhật. Phiên bản đang chạy: ${await V42s()}.`,
  });
}
