// Copyright (c) 2026 DigiText. All rights reserved.

import type { V43e } from "@/api/lansync";

export function V4ps(r: V43e): string {
  if (r.total === 0) {
    return "Không tìm thấy tệp PDF nào trong lựa chọn — chỉ nhận tệp .pdf (thư mục được duyệt cả cấp con).";
  }

  const V2c3 = [r.direct ? `${r.added} tài liệu ghi sổ Kho (giữ nguyên trên ổ, cây “${r.dest}”)` : `${r.added} tài liệu vào Kho “${r.dest}”`];

  if (r.skipped > 0) V2c3.push(`${r.skipped} đã có sẵn (bỏ qua)`);
  if (r.failed > 0) V2c3.push(`lỗi ${r.failed}`);

  if (r.cancelled > 0) V2c3.push(`${r.cancelled} đã dừng`);

  let m = `Đã thêm ${V2c3.join(", ")} — dự án ${r.projectId}.`;
  if (r.direct) {
    const d = [`${r.pushed} đã lên máy chủ`];
    if (r.pushSkipped > 0) d.push(`${r.pushSkipped} máy chủ đã có`);
    if (r.pushFailed > 0) d.push(`${r.pushFailed} lỗi gửi`);
    if (r.pushRejected > 0) d.push(`${r.pushRejected} bị từ chối`);
    if (r.pushCancelled > 0) d.push(`${r.pushCancelled} chưa gửi`);
    m += ` Đẩy thẳng: ${d.join(", ")}.`;
    if (r.pushFailed + r.pushCancelled > 0) m += " Phần chưa lên vẫn ở sổ Kho, cắm ổ đĩa rồi bấm Đẩy lên máy chủ.";
  } else if (r.added > 0) {
    m += " Bấm Đẩy lên máy chủ để gửi đi.";
  }
  if (r.notes.length) m += ` (${r.notes.join("; ")})`;
  return m;
}
