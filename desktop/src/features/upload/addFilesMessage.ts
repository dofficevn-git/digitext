// Copyright (c) 2026 DigiText. All rights reserved.

import type { V43e } from "@/api/lansync";

export function V4ps(r: V43e): string {
  if (r.total === 0) {
    return "Không tìm thấy tệp PDF nào trong lựa chọn — chỉ nhận tệp .pdf (thư mục được duyệt cả cấp con).";
  }

  const V2c3 = [`${r.added} tài liệu vào Kho “${r.dest}”`];

  if (r.skipped > 0) V2c3.push(`${r.skipped} đã có sẵn (bỏ qua)`);
  if (r.failed > 0) V2c3.push(`lỗi ${r.failed}`);

  if (r.cancelled > 0) V2c3.push(`${r.cancelled} đã dừng`);

  let m = `Đã thêm ${V2c3.join(", ")} — dự án ${r.projectId}.`;
  if (r.added > 0)
    m += " Bấm Đẩy lên máy chủ để gửi đi.";
  if (r.notes.length) m += ` (${r.notes.join("; ")})`;
  return m;
}
