// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3zc } from "@/api/auth";
import { V3z6 } from "@/domain/permissions";

export type V3zp = "loading" | "signedOut" | "locked" | "active";

export function V3zq(Vx: V3zc | null, Vy: boolean): V3zp {
  if (Vy) return "loading";
  if (!Vx) return "signedOut";
  return Vx.locked ? "locked" : "active";
}

export function V3zr(Vz: string, V10: Date = new Date()): number {
  const V11 = Date.parse(Vz);
  if (Number.isNaN(V11)) return 0;
  const ms = V11 - V10.getTime();
  return ms > 0 ? Math.floor(ms / 86_400_000) : 0;
}

export function V3zs(V12: string, V13: Date = new Date()): boolean {
  return V3zr(V12, V13) < 7;
}

export function V3zt(V14: string, V15: string): string {
  switch (V14) {
    case "auth_failed":
      return "Tên đăng nhập hoặc mật khẩu không đúng.";
    case "too_many_attempts":
      return (
        V15 ||
        "Sai mật khẩu nhiều lần. Thử lại sau ít phút."
      );
    case "captcha_invalid":
      return "Mã xác nhận không đúng — nhập lại mã trong ảnh mới.";
    case "revoked":
      return "Tài khoản đã bị thu hồi — liên hệ quản trị dự án.";
    case "device_revoked":
      return "Máy này đã bị thu hồi quyền truy cập — liên hệ quản trị dự án.";
    case "offline_expired":
      return "Quá 30 ngày chưa xác thực online — kết nối mạng rồi đăng nhập lại.";
    case "server_unreachable": {
      // Kèm lỗi thật (hết giờ chờ / đọc phản hồi lỗi…) để biết đâu là nguyên nhân.
      const chiTiet = V15.match(/\((.*)\)\.?$/)?.[1];
      return "Không kết nối được máy chủ. Kiểm tra mạng, hoặc đăng nhập offline nếu đã dùng máy này trước đó." + (chiTiet ? ` Chi tiết: ${chiTiet}` : "");
    }
    case "server_not_configured":
      return "Chưa có địa chỉ máy chủ. Bấm nút “Máy chủ” bên dưới để nhập.";
    case "busy":
      return "Đang quét dở một lô — kết thúc lô quét trước khi đổi người dùng.";
    case "locked":
      return "Màn hình đang khoá — mở khoá bằng mật khẩu.";
    case "server_mismatch":
      return "Chữ ký máy chủ không hợp lệ — kiểm tra lại địa chỉ server.";
    case "auth_cache_corrupt":
      return "Dữ liệu đăng nhập trên máy bị hỏng — cần đăng nhập online lại.";
    default:
      return V15 || "Có lỗi xảy ra, thử lại.";
  }
}

export function V3zu(): V3zc {
  return {
    userId: "dev-user",
    username: "dev",
    displayName: "Dev (browser)",
    role: "admin",
    projectId: null,
    projectCode: "DEV",
    projectName: "Chế độ trình duyệt",
    mode: "offline",
    offlineExpiry: new Date(Date.now() + 365 * 86_400_000).toISOString(),
    locked: false,

    permissions: V3z6("admin"),
  };
}
