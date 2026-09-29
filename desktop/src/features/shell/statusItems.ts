// Copyright (c) 2026 DigiText. All rights reserved.
// Tính nội dung các ô ở thanh trạng thái (thuần, có thể test): Máy chủ · Phiên · Máy quét · Ổ lưu trữ.

import type { V41v } from "@/components/Icon";

export type V47j = "ok" | "warn" | "err" | "unknown";

export interface V47k {
  locked: boolean;
  mode: "online" | "offline";
  /** Số ngày còn dùng được ngoại tuyến. */
  offlineDays: number;
}

export interface V47l {
  loading: boolean;
  session: V47k | null;
  scanners: number | null;
  serverUp: boolean | null;
  storage: { configured: boolean; exists: boolean } | null;
  live: boolean | null;
}

export type V47m = "server" | "session" | "scanner" | "drive";

export interface V47n {
  key: V47m;
  icon: V41v;
  label: string;
  value: string;
  dot: V47j;
  /** Một câu ngắn hiện khi rê chuột. */
  title: string;
}

function server(i: V47l): V47n {
  const base = { key: "server" as const, icon: "server" as const, label: "Máy chủ" };
  if (i.serverUp === null) return { ...base, dot: "unknown", value: "Đang kiểm tra", title: "Đang kiểm tra kết nối máy chủ." };
  if (!i.serverUp) return { ...base, dot: "err", value: "Mất kết nối", title: "Không tới được máy chủ. Bấm để mở Cài đặt." };
  if (i.live === false) return { ...base, dot: "warn", value: "Không tự cập nhật", title: "Kết nối được nhưng màn hình không tự cập nhật. Bấm để xem cài đặt." };
  return { ...base, dot: "ok", value: "Đã kết nối", title: "Máy chủ hoạt động, màn hình tự cập nhật." };
}

function session(i: V47l): V47n {
  const base = { key: "session" as const, icon: "token" as const, label: "Phiên" };
  if (i.loading) return { ...base, dot: "unknown", value: "Đang kiểm tra", title: "Đang kiểm tra phiên đăng nhập." };
  if (!i.session) return { ...base, dot: "err", value: "Chưa đăng nhập", title: "Chưa có phiên đăng nhập." };
  if (i.session.locked) return { ...base, dot: "warn", value: "Đang khoá", title: "Mở khoá bằng mật khẩu để tiếp tục." };
  if (i.session.mode === "online") return { ...base, dot: "ok", value: "Trực tuyến", title: "Đăng nhập trực tuyến." };
  const d = i.session.offlineDays;
  return {
    ...base,
    dot: d < 7 ? "warn" : "ok",
    value: `Ngoại tuyến · còn ${d} ngày`,
    title: d < 7 ? "Sắp hết hạn dùng ngoại tuyến. Kết nối mạng và đăng nhập lại." : "Đang dùng ngoại tuyến, sẽ tự đồng bộ khi có mạng.",
  };
}

function scanner(i: V47l): V47n {
  const base = { key: "scanner" as const, icon: "scanner" as const, label: "Máy quét" };
  if (i.scanners === null) return { ...base, dot: "unknown", value: "Đang dò", title: "Đang dò máy quét." };
  if (i.scanners > 0) return { ...base, dot: "ok", value: i.scanners === 1 ? "Sẵn sàng" : `${i.scanners} máy`, title: "Bấm để mở màn Quét." };
  return { ...base, dot: "err", value: "Không thấy", title: "Không dò thấy máy quét. Kiểm tra cáp và nguồn." };
}

function drive(i: V47l): V47n | null {
  if (!i.storage || !i.storage.configured || i.storage.exists) return null;
  return { key: "drive", icon: "drive", label: "Ổ lưu trữ", dot: "err", value: "Không thấy", title: "Ổ lưu trữ đã cấu hình không truy cập được. Bấm để mở Cài đặt." };
}

export function V47q(i: V47l, opts: { showScanner: boolean }): V47n[] {
  const out: V47n[] = [server(i), session(i)];
  if (opts.showScanner) out.push(scanner(i));
  const d = drive(i);
  if (d) out.push(d);
  return out;
}
