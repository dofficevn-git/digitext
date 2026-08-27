// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4ew, V4ey } from "@/api/docs";

function V4jx(V1nt: number, V1nu: number): number {
  if (V1nu <= 0) return 0;
  return Math.max(0, Math.min(100, Math.round((100 * V1nt) / V1nu)));
}

export function V4jy(p: V4ey): number {
  return V4jx(p.pagesOcrDone, p.pagesTotal);
}

export function V4jz(p: { docsPdf2Done: number; docsOcrDone: number }): number {
  return V4jx(p.docsPdf2Done, p.docsOcrDone);
}

export function V4k0(p: { docsFieldsDone?: number; docs: number }): number {
  return V4jx(p.docsFieldsDone ?? 0, p.docs);
}

export function V4k1(status: string): string {
  switch (status) {
    case "uploaded":
      return "Đã tải lên";
    case "ocr_pending":
      return "Chờ OCR";
    case "ocr_done":
      return "OCR xong";
    case "to_sign":
      return "Chờ ký";
    case "signing":
      return "Đang ký";
    case "signed":
      return "Đã ký";
    case "sign_rejected":
      return "Ký lỗi";
    case "archived":
      return "Đã lưu trữ";
    case "disposed":
      return "Đã huỷ";
    default:
      return status;
  }
}

export function V4k2(r: V4ew): string {
  const V1nv: string[] = [];
  if (r.skippedNoPageCount > 0)
    V1nv.push(`${r.skippedNoPageCount} tài liệu chưa biết số trang`);
  if (r.skippedPageFailed > 0)
    V1nv.push(
      `${r.skippedPageFailed} tài liệu hỏng vĩnh viễn (sửa nguyên nhân rồi bấm "Thử lại tài liệu hỏng")`,
    );
  const V1nw = V1nv.length > 0 ? ` Bỏ qua ${V1nv.join("; ")}.` : "";
  const V1nx =
    r.documents > 0
      ? `Đã đưa ${r.documents} tài liệu vào hàng đợi bóc tách 16 trường trên máy chủ.`
      : "Không có tài liệu nào cần bóc tách 16 trường (đã bóc xong hoặc đang chờ xử lý).";
  return V1nx + V1nw;
}
