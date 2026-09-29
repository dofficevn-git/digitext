// Copyright (c) 2026 DigiText. All rights reserved.

import type { V43b } from "@/api/lansync";

export function V4pu(r: V43b): string {
  let V2c8 = `Đẩy lên máy chủ: ${r.forwarded} mới, ${r.skipped} đã có, lỗi ${r.failed}/${r.total}.`;
  if (r.retagged > 0) {
    V2c8 += ` ${r.retagged} bản đã bị sửa lại trong Kho sau khi gom — đã đẩy bản mới nhất.`;
  }

  if (r.cancelled > 0) {
    V2c8 +=
      ` ${r.cancelled} tệp bạn đã hủy — phần đã gửi vẫn nằm trên máy chủ, ` +
      "bấm Đẩy lên máy chủ lần nữa là tải tiếp từ chỗ dừng.";
  }
  if (r.rejected > 0) {
    V2c8 +=
      ` ${r.rejected} bản bị máy chủ từ chối (thường do người quét không thuộc dự án bạn ` +
      "quản lý) — đã ngừng thử lại. Sửa phía máy chủ rồi chọn Gom lại bản bị từ chối trong menu.";
  }
  return V2c8;
}
