// Copyright (c) 2026 DigiText. All rights reserved.
// Dữ liệu giả CHỈ cho bản dev chạy trong trình duyệt (xem trước giao diện, chụp màn). Không vào bản phát hành:
// ipc.ts chỉ nạp tệp này khi import.meta.env.DEV và không có Tauri. Lệnh không có ở đây vẫn báo lỗi như cũ.
import type { ReviewSummaryRow } from "./docs";

function row(phong: string, hop: string, hoSo: string, pages: number, mb: number, ocr: number, pdfa: boolean, signed: boolean): ReviewSummaryRow {
  return {
    projectId: "dev-ltlsbn", projectCode: "LTLSBNinh", projectName: "Lưu trữ lịch sử Bắc Ninh",
    phong, hop, hoSo, files: 1, pages, unknownPages: 0, sizeBytes: Math.round(mb * 1024 * 1024),
    ocrPages: ocr, fieldDocs: 0, pdfaDocs: pdfa ? 1 : 0, pdfaPages: pdfa ? pages : 0, pdfaCu: 0, ocrDenom: pages,
    approved: 0, rejected: 0, ocrPendingDocs: ocr < pages ? 1 : 0, ocrDoneDocs: ocr >= pages ? 1 : 0,
    pdf2DoneDocs: pdfa ? 1 : 0, fieldsDone: 0, fieldsActive: 0, fieldsFailed: 0, signedDocs: signed ? 1 : 0, pdfaOnlyDocs: 0,
  };
}

const SUMMARY: ReviewSummaryRow[] = [
  row("PHONG SNN&PTNT BG", "HOP 84", "HO SO 419", 48, 53.7, 45, false, false),
  row("PHONG SNN&PTNT BG", "HOP 84", "HO SO 420", 33, 26.2, 33, true, false),
  row("PHONG SNN&PTNT BG", "HOP 06", "HO SO 34", 11, 18.4, 10, false, false),
  row("UBND TINH BAC GIANG (1997 - 2017)", "HOP 13", "HO SO 102", 126, 141, 52, false, false),
  row("UBND TINH BAC GIANG (1997 - 2017)", "HOP 12", "HO SO 97", 64, 70.3, 64, true, true),
  row("PHONG SO NONG NGHIEP HB", "HOP 31", "HO SO 211", 2, 1.9, 2, true, true),
  row("PHONG SKHCNMTBG MOI", "HOP 05", "HO SO 18", 22, 24.8, 0, false, false),
];

let taiThu = 0;

export function devMock(cmd: string): unknown {
  switch (cmd) {
    case "docs_download":
      // Tải giả: 0,8 giây mỗi tệp, cứ tệp thứ 7 thì lỗi — để xem khung tiến độ tải xuống.
      return new Promise((ok, loi) => setTimeout(() => (++taiThu % 7 === 0 ? loi(new Error("Máy chủ trả 404: chưa có bản PDF/A")) : ok("D:/Tai ve/LTLSBN/tep.pdf")), 800));
    case "docs_review_summary":
      return SUMMARY;
    default:
      return undefined;
  }
}
