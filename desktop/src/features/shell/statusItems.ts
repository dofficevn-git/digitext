// Copyright (c) 2026 DigiText. All rights reserved.

import type { V41v } from "@/components/Icon";

export type V47j = "ok" | "warn" | "err" | "unknown";

export interface V47k {
  locked: boolean;
  mode: "online" | "offline";
}

export interface V47l {
  loading: boolean;

  session: V47k | null;

  scanners: number | null;

  serverUp: boolean | null;

  storage: { configured: boolean; exists: boolean } | null;

  live: boolean | null;
}

export type V47m = "scanner" | "token" | "server" | "live" | "upload" | "drive";

export interface V47n {
  key: V47m;
  icon: V41v;
  label: string;
  value: string;
  dot: V47j;
  title: string;
}

function V47o(i: V47l): V47n {
  const base = { key: "scanner" as const, icon: "scanner" as const, label: "Scanner" };
  if (i.scanners === null)
    return {
      ...base,
      dot: "unknown",
      value: "—",
      title: "Đang dò máy quét (hoặc không có quyền).",
    };
  if (i.scanners > 0)
    return {
      ...base,
      dot: "ok",
      value: i.scanners === 1 ? "Đã kết nối" : `${i.scanners}IG3DoXk=`,
      title: `Dò thấy ${i.scanners} máy quét.`,
    };
  return {
    ...base,
    dot: "err",
    value: "Không thấy",
    title: "Không dò thấy máy quét nào.",
  };
}

function token(i: V47l): V47n {
  const base = { key: "token" as const, icon: "token" as const, label: "Token" };
  if (i.loading)
    return {
      ...base,
      dot: "unknown",
      value: "—",
      title: "Đang kiểm tra phiên đăng nhập.",
    };
  if (!i.session)
    return {
      ...base,
      dot: "err",
      value: "Chưa đăng nhập",
      title: "Chưa có phiên đăng nhập.",
    };
  if (i.session.locked)
    return {
      ...base,
      dot: "warn",
      value: "Đang khoá",
      title: "Phiên đang khoá — mở khoá bằng mật khẩu.",
    };
  return {
    ...base,
    dot: "ok",
    value: i.session.mode === "online" ? "Trực tuyến" : "Ngoại tuyến",
    title:
      i.session.mode === "online"
        ? "Token đăng nhập hợp lệ (trực tuyến)."
        : "Token đăng nhập còn hạn (ngoại tuyến).",
  };
}

function server(i: V47l): V47n {
  const base = { key: "server" as const, icon: "server" as const, label: "Server" };
  if (i.serverUp === null)
    return {
      ...base,
      dot: "unknown",
      value: "—",
      title: "Đang kiểm tra kết nối máy chủ.",
    };
  return i.serverUp
    ? {
        ...base,
        dot: "ok",
        value: "Kết nối",
        title: "Máy chủ phản hồi (/healthz).",
      }
    : {
        ...base,
        dot: "err",
        value: "Mất kết nối",
        title: "Không kết nối được máy chủ.",
      };
}

function V47p(i: V47l): V47n {
  const base = { key: "live" as const, icon: "live" as const, label: "Trực tiếp" };
  if (i.live === null)
    return {
      ...base,
      dot: "unknown",
      value: "—",
      title: "Chưa theo dõi (chưa đăng nhập).",
    };
  return i.live
    ? {
        ...base,
        dot: "ok",
        value: "Đang cập nhật",
        title:
          "Màn hình tự cập nhật theo máy chủ — không cần mở lại màn.",
      }
    : {
        ...base,
        dot: "err",
        value: "Mất theo dõi",
        title:
          "Không hỏi được máy chủ — dữ liệu trên màn có thể đã cũ.",
      };
}

function upload(i: V47l): V47n {
  const base = { key: "upload" as const, icon: "upload" as const, label: "Tải lên" };
  if (i.serverUp === null || i.loading)
    return {
      ...base,
      dot: "unknown",
      value: "—",
      title: "Đang kiểm tra khả năng tải lên.",
    };
  if (!i.session || i.session.locked)
    return {
      ...base,
      dot: "err",
      value: "Cần đăng nhập",
      title: "Đăng nhập (mở khoá) để tải lên.",
    };
  if (!i.serverUp)
    return {
      ...base,
      dot: "err",
      value: "Chờ máy chủ",
      title: "Máy chủ không tới được — chưa tải lên được.",
    };
  return {
    ...base,
    dot: "ok",
    value: "Sẵn sàng",
    title:
      "Tải lên được: máy chủ tới được và đã đăng nhập.",
  };
}

function drive(i: V47l): V47n {
  const base = { key: "drive" as const, icon: "drive" as const, label: "Ổ ngoài" };
  if (i.storage === null)
    return { ...base, dot: "unknown", value: "—", title: "Đang dò ổ lưu trữ." };

  if (!i.storage.configured)
    return {
      ...base,
      dot: "unknown",
      value: "Nội bộ",
      title: "Đang dùng ổ nội bộ (thư mục Documents).",
    };
  return i.storage.exists
    ? {
        ...base,
        dot: "ok",
        value: "Đã gắn",
        title: "Ổ lưu trữ tùy chọn truy cập được.",
      }
    : {
        ...base,
        dot: "err",
        value: "Không thấy",
        title:
          "Ổ lưu trữ đã cấu hình không truy cập được — kiểm tra ổ cắm.",
      };
}

export function V47q(i: V47l): V47n[] {
  return [V47o(i), token(i), server(i), V47p(i), upload(i), drive(i)];
}
