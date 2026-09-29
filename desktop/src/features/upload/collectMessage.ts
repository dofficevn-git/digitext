// Copyright (c) 2026 DigiText. All rights reserved.

import type { V439 } from "@/api/lansync";

export function V4pt(r: V439): string {
  const V2c4: string[] = [];
  if (r.collected > 0)
    V2c4.push(
      `${r.collected} mới (máy này: ${r.local}, LAN: ${r.collected - r.local - r.orphans}` +
        (r.orphans > 0 ? `, tài liệu rời trong Kho: ${r.orphans}` : "") +
        ")",
    );

  if (r.mirrored > 0) V2c4.push(`${r.mirrored} dựng vào mục lục`);
  if (r.skipped > 0) V2c4.push(`${r.skipped} đã có`);
  if (r.failed > 0) V2c4.push(`lỗi ${r.failed}`);
  if (r.cancelled > 0) V2c4.push(`${r.cancelled} đã hủy`);

  let m = V2c4.length
    ? `Thu thập: ${V2c4.join(", ")}.`
    : "Không có tài liệu mới để gom.";

  if (r.cancelled > 0) {
    m +=
      ` Bạn đã dừng ${r.cancelled} tài liệu — phần đã tải được giữ lại, bấm Thu thập` +
      " lần nữa là tải tiếp từ chỗ dừng.";
  }

  if (r.khoTruncated) {
    m +=
      " Kho có rất nhiều tệp nên lượt này mới quét được một phần — bấm Thu thập thêm vài lần nữa để nhặt hết tài liệu rời.";
  }

  if (r.projectGateStale) {
    m +=
      " Chưa lấy được danh sách dự án từ máy chủ nên đang dùng bản lưu — có thể bỏ sót tài liệu thuộc dự án khác bạn quản lý. Hãy kết nối máy chủ rồi thu thập lại.";
  }

  if (r.foreign > 0) {
    m += ` Bỏ qua ${r.foreign} tài liệu thuộc dự án máy này không quản lý (quản trị viên phụ trách dự án đó sẽ thu thập).`;
  }

  if (!r.scannedLan) {
    m += r.notes.length
      ? ` ${r.notes.join("; ")}`
      : " Chưa đặt mã site LAN nên chưa quét máy nhân viên khác.";
    return m;
  }

  if (r.manifestFailed > 0) {
    const V2c5 = r.notes.length ? ` (${r.notes.join("; ")})` : "";
    const V2c6 = r.notes.some((n) => n.includes("bản cũ"));
    m += ` Có ${r.manifestFailed} máy không lấy được danh sách tài liệu${V2c5} — `;
    m += V2c6
      ? "máy nhân viên đang chạy bản cũ. Cập nhật DigiText trên mọi máy trong site rồi thu thập lại."
      : "thường do mã site trên máy nhân viên khác mã trên máy quản trị, hoặc máy nhân viên chưa cập nhật. Phát lại cây thư mục để họ nhận đúng mã và kiểm tra hai máy cùng phiên bản.";
    return m;
  }

  if (r.peers === 0) {
    const V2c7 = r.scannedRanges?.length
      ? r.scannedRanges.join(", ")
      : "không xác định được";
    m += ` Không thấy máy nhân viên nào chia sẻ trong mạng nội bộ (đã quét dải ${V2c7}).` + " Xem mục trợ giúp bên dưới để kiểm tra mạng và tường lửa.";
    if (r.scanClamped) {
      m += " Dải mạng của máy này quá rộng nên chỉ quét được một phần; đặt hai máy trong cùng một dải hẹp.";
    }
    if (r.notes.length) m += ` (${r.notes.join("; ")})`;
    return m;
  }

  if (r.advertised === 0 && V2c4.length === 0) {
    m +=
      " Đã kết nối máy nhân viên nhưng chưa có tài liệu nào đã đóng gói để chia sẻ. Nhân viên cần bấm Đóng gói và lưu PDF vào hồ sơ sau khi quét.";
    if (r.notes.length) m += ` (${r.notes.join("; ")})`;
    return m;
  }

  if (r.notes.length) m += ` (${r.notes.join("; ")})`;
  return m;
}
