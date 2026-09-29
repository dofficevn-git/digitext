// Copyright (c) 2026 DigiText. All rights reserved.
// Chuẩn hoá lỗi cho người dùng: một câu tiếng Việt ngắn nói rõ việc không làm được (+ bước tiếp theo),
// phần kỹ thuật (thông điệp gốc, mã lỗi) tách ra `detail` để hiện gập trong banner / toast / hộp "Chi tiết".

import { V3xq } from "@/api/ipc";

export interface UserError {
  /** Câu cho người dùng, không lộ tên lệnh / bảng / stack. */
  message: string;
  /** Thông điệp gốc (kỹ thuật) nếu khác `message`. */
  detail?: string;
  /** Mã lỗi từ máy chủ / Tauri (`V3xq.code`) nếu có. */
  code?: string;
}

/** Mã lỗi → câu tiếng Việt. Mã không có trong bảng dùng thông điệp gốc (nếu là tiếng Việt) hoặc câu chung. */
const BY_CODE: Record<string, string> = {
  unauthorized: "Phiên đăng nhập đã hết. Hãy đăng nhập lại.",
  forbidden: "Bạn không có quyền thực hiện thao tác này.",
  locked: "Phiên đang khoá. Mở khoá rồi thử lại.",
  offline: "Không kết nối được máy chủ. Kiểm tra mạng rồi thử lại.",
  network: "Không kết nối được máy chủ. Kiểm tra mạng rồi thử lại.",
  timeout: "Máy chủ phản hồi quá lâu. Thử lại sau ít phút.",
  server_not_configured: "Chưa khai báo địa chỉ máy chủ. Vào Cài đặt › Máy chủ để khai báo.",
  not_found: "Không tìm thấy dữ liệu. Có thể mục này vừa bị xoá.",
  conflict: "Dữ liệu vừa bị người khác thay đổi. Tải lại rồi làm lại.",
  no_project: "Chưa chọn dự án đang làm. Chọn dự án ở thanh trên cùng.",
  db_poisoned: "Cơ sở dữ liệu trên máy đang bận. Khởi động lại ứng dụng.",
};

/** Thông điệp gốc trông có phải kỹ thuật (tiếng Anh / stack / JSON) không. */
function looksTechnical(msg: string): boolean {
  const m = msg.trim();
  if (!m) return true;
  if (/^(Error|TypeError|RangeError|IpcError):/i.test(m)) return true;
  if (/\b(invoke|panicked|unwrap|serde|sqlx|rusqlite|invalid type|missing field|failed to|error decoding|ECONN|ENOENT|fetch)\b/i.test(m)) return true;
  if (/^[{[]/.test(m)) return true;
  // Không có ký tự tiếng Việt có dấu và toàn từ tiếng Anh → coi là kỹ thuật.
  if (!/[àáạảãâầấậẩẫăằắặẳẵèéẹẻẽêềếệểễìíịỉĩòóọỏõôồốộổỗơờớợởỡùúụủũưừứựửữỳýỵỷỹđ]/i.test(m) && /^[ -~]+$/.test(m) && m.split(/\s+/).length > 2) return true;
  return false;
}

export function describeError(e: unknown, ctx?: string): UserError {
  const fallback = ctx ? `Không ${ctx} được.` : "Không thực hiện được thao tác.";
  if (e instanceof V3xq) {
    const raw = e.message?.trim() ?? "";
    const known = BY_CODE[e.code];
    if (known) return { message: known, detail: raw && raw !== known ? raw : undefined, code: e.code };
    if (e.code === "validation" && raw) return { message: raw, code: e.code };
    if (raw && !looksTechnical(raw)) return { message: raw, code: e.code };
    return { message: fallback, detail: raw || undefined, code: e.code };
  }
  if (e instanceof Error) {
    const raw = e.message?.trim() ?? "";
    if (raw && !looksTechnical(raw)) return { message: raw };
    return { message: fallback, detail: raw || e.name };
  }
  if (typeof e === "string") {
    const raw = e.trim();
    if (raw && !looksTechnical(raw)) return { message: raw };
    return { message: fallback, detail: raw || undefined };
  }
  if (e && typeof e === "object" && "message" in e) {
    return describeError(new Error(String((e as { message: unknown }).message)), ctx);
  }
  return { message: fallback, detail: e === undefined ? undefined : String(e) };
}

/** Rút gọn: chỉ câu cho người dùng. */
export function errorText(e: unknown, ctx?: string): string {
  return describeError(e, ctx).message;
}
