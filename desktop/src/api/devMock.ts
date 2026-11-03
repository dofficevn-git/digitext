// Copyright (c) 2026 DigiText. All rights reserved.
// Dữ liệu giả CHỈ cho bản dev chạy trong trình duyệt (xem trước giao diện, chụp màn). Không vào bản phát hành:
// ipc.ts chỉ nạp tệp này khi import.meta.env.DEV và không có Tauri. Lệnh không có ở đây vẫn báo lỗi như cũ.
import type { ReviewSummaryRow } from "./docs";
import { devMockDashboard } from "./devMockDashboard";

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

export function devMock(cmd: string, args?: Record<string, unknown>): unknown {
  switch (cmd) {
    case "docs_download":
      // Tải giả: 0,8 giây mỗi tệp, cứ tệp thứ 7 thì lỗi — để xem khung tiến độ tải xuống.
      return new Promise((ok, loi) => setTimeout(() => (++taiThu % 7 === 0 ? loi(new Error("Máy chủ trả 404: chưa có bản PDF/A")) : ok("D:/Tai ve/LTLSBN/tep.pdf")), 800));
    case "docs_review_summary":
      return SUMMARY;
    // Kiểm tra sản phẩm (0.22): phông mẫu + kết quả đối soát để xem trước `?view=qc`.
    case "cl_catalog_list":
      return { khoi_phong: [], phong: [{ id: "p1", ma_du_an: "CCVTLT", ten: "Chi cục Văn thư – Lưu trữ tỉnh Bắc Giang" }, { id: "p2", ma_du_an: "UBND", ten: "UBND tỉnh Bắc Giang (1997 – 2017)" }], phong_ban: [], mang: [], thbq: [], ngon_ngu: [], loai_van_ban: [], kho: [], gia: [], hop: [] };
    case "records_list":
      return [];
    // Nghiệm thu (0.23): hợp đồng, gói, mục lục mẫu để xem trước `?view=acceptance`.
    case "report_project_progress":
      return [{ projectId: String((args?.args as { projectId?: string } | undefined)?.projectId ?? ""), code: "LTLSBN", name: "Lưu trữ lịch sử Bắc Ninh", contract: { signedOn: "2026-03-01", endOn: "2027-03-25", deadlineOn: "2027-03-25", contractPages: 2_250_000, contractFiles: 220_000, contractBoxes: 1_800, contractDossiers: 14_000, contractSheets: 1_100_000, managerName: "Nguyễn Văn A", note: null }, files: 205_008, pages: 866_703, pdfaFiles: 183_679, signedFiles: 160_375, ocrFiles: 190_000, approvedFiles: 1_200, rejectedFiles: 40 }];
    case "packages_list":
      return [
        { id: "g1", projectId: String((args?.args as { projectId?: string } | undefined)?.projectId ?? ""), kind: "aip", status: "built", hoSoRef: null, label: "CCVTLT · hộp 01–05", objid: "urn:uuid:1", zipName: "CCVTLT-01-05.zip", zipSha256: "9e630ee18a7ad5a1", zipSize: 812_000_000, documentCount: 420, error: null, virusScan: "clean", submissionId: null, convertedFrom: null, sttFrom: 1, sttTo: 420, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() },
        { id: "g2", projectId: "khac", kind: "aip", status: "built", hoSoRef: null, label: "Dự án khác", objid: null, zipName: "x.zip", zipSha256: null, zipSize: 1, documentCount: 1, error: null, virusScan: null, submissionId: null, convertedFrom: null, sttFrom: null, sttTo: null, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() },
      ];
    case "cl_ho_so_list": {
      const pid = String((args?.filter as { phongId?: string } | undefined)?.phongId ?? "");
      const k = pid === "p1" ? 9 : 24;
      const rows = Array.from({ length: k }, (_, i) => ({ id: `${pid}-hs${i + 1}`, hop_so: String(Math.floor(i / 3) + 1).padStart(2, "0"), ho_so_so: String(i + 1), so_to: String(10 + ((i * 7) % 40)), tieu_de: `Hồ sơ mẫu ${i + 1}` }));
      return { rows, total: rows.length };
    }
    case "cl_rows":
      return { rows: [], total: 0 };
    case "docs_pdfa_rejected":
      return [
        { id: "r1", projectCode: "LTLSBN", relPath: "CHI CUC VAN THU LUU TRU/HOP 03", filename: "CCVTLT.03.000012.pdf", pageCount: 6, sizeBytes: 812_000, uploadedAt: new Date().toISOString(), reason: "Độ phân giải 150 dpi (yêu cầu ≥ 200)", outcome: "rejected" },
        { id: "r2", projectCode: "LTLSBN", relPath: "CHI CUC VAN THU LUU TRU/HOP 07", filename: "CCVTLT.07.000041.pdf", pageCount: 14, sizeBytes: 2_310_000, uploadedAt: new Date().toISOString(), reason: "Độ phân giải 180 dpi, đã ghép theo ngoại lệ dự án", outcome: "exception" },
      ];
    case "cl_remote": {
      if (!String(args?.path ?? "").startsWith("/cl/doi-soat")) return undefined;
      const hs = (i: number, hop: string, so: string, tieuDe: string, soTo: number | null, files: number, pages: number, pdfa: number, cu: number, signed: number, rej: number, ketLuan: string) => ({
        hoSoId: `hs${i}`, hopSo: hop, hoSoSo: so, maHoSo: `08.11.2015.${so}`, tieuDe, vinhVien: i % 3 === 0, soTo, files, pages, pdfaDocs: pdfa, pdfaCu: cu, signedDocs: signed, rejectedDocs: rej, ketLuan,
      });
      const hoSo = [
        hs(1, "01", "1", "Tập lưu quyết định của Chi cục năm 2015", 42, 3, 84, 3, 0, 3, 0, "dat"),
        hs(2, "01", "2", "Hồ sơ kiểm tra công tác văn thư, lưu trữ các sở, ngành", 120, 5, 96, 5, 0, 0, 0, "thieu_trang"),
        hs(3, "01", "3", "Báo cáo thống kê công tác lưu trữ năm 2015", 15, 1, 30, 1, 0, 1, 0, "dat"),
        hs(4, "02", "4", "Hồ sơ tiêu huỷ tài liệu hết giá trị đợt 1", 8, 1, 40, 1, 0, 0, 0, "thua_trang"),
        hs(5, "02", "5", "Công văn đi về nghiệp vụ lưu trữ", 60, 4, 118, 2, 0, 0, 0, "khong_pdfa"),
        hs(6, "02", "6", "Hồ sơ hội nghị tập huấn nghiệp vụ", 25, 2, 50, 2, 1, 0, 0, "pdfa_cu"),
        hs(7, "03", "7", "Kế hoạch công tác năm 2016", 10, 1, 20, 1, 0, 0, 1, "bi_tu_choi"),
        hs(8, "03", "8", "Hồ sơ thi đua khen thưởng", null, 0, 0, 0, 0, 0, 0, "chua_quet"),
        hs(9, "03", "9", "Tập lưu công văn đến năm 2016", 88, 6, 176, 6, 0, 6, 0, "dat"),
      ];
      return {
        tongKet: { hoSoMucLuc: 9, hoSoCoTep: 8, dat: 3, chuaQuet: 1, lechSoTo: 2, khongPdfa: 1, pdfaCu: 1, biTuChoi: 1, tep: 23, trang: 614, soToMucLuc: 368, tepKhongKhop: 2 },
        hoSo,
        khongKhop: [
          { documentId: "d1", filename: "CCVTLT.04.000003.pdf", hop: "04", hoSo: "3", lyDo: "không có hồ sơ này trong mục lục" },
          { documentId: "d2", filename: "CCVTLT.01.000099.pdf", hop: "01", hoSo: "99", lyDo: "không có hồ sơ này trong mục lục" },
        ],
        thuMucCo: ["CHI CUC VAN THU LUU TRU", "UBND TINH BAC GIANG (1997 - 2017)"],
      };
    }
    default:
      return devMockDashboard(cmd, args);
  }
}
