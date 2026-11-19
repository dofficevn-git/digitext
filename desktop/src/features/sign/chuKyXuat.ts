// Copyright (c) 2026 DigiText. All rights reserved.
// Xuất báo cáo Excel cho Tiện ích › Kiểm chữ ký số. Ba sheet: Tổng hợp, Tệp, Chữ ký.
//
// Vì sao tách sheet "Chữ ký" riêng: một tệp ký nhiều lần (VGCA: phê duyệt, số công văn, ngày,
// dấu tổ chức) có nhiều chữ ký, mỗi chữ ký một người ký và một kết luận. Nhồi tất cả vào một
// dòng thì không lọc được theo người ký, mà lọc theo người ký là việc hay làm nhất.

import type { KetQuaQuet, TepDaKy } from "@/api/chuky";
import { V4dz } from "@/api/exportFile";

/** Màu tô ô Kết luận. ARGB, theo nghĩa: đỏ = phải sửa, vàng = xem lại, xanh = xong, xám = chưa làm. */
const MAU = {
  khongDat: "FFFFC7CE",
  canhBao: "FFFFEB9C",
  dat: "FFC6EFCE",
  chuaKy: "FFE7E6E6",
} as const;

const mauKetLuan = (k: string): string | null =>
  k === "Không đạt" || k === "Bỏ qua" ? MAU.khongDat : k === "Đạt, có cảnh báo" ? MAU.canhBao : k === "Đạt" ? MAU.dat : k === "Chưa ký" ? MAU.chuaKy : null;

const co = (b: number): string => (b >= 1048576 ? `${(b / 1048576).toFixed(1)} MB` : `${Math.max(1, Math.round(b / 1024))} KB`);
const co2 = (b: boolean): string => (b ? "Có" : "Không");

/** Cỡ cột theo nội dung dài nhất, kẹp trong khoảng đọc được. */
function doRong(tieuDe: string[], dong: (string | number)[][]): number[] {
  return tieuDe.map((h, i) => {
    const dai = Math.max(h.length, ...dong.map((r) => String(r[i] ?? "").split("\n")[0].length));
    return Math.min(60, Math.max(9, dai + 2));
  });
}

export async function xuatExcel(kq: KetQuaQuet, goc: string[]): Promise<string | null> {
  const ExcelJS = (await import("exceljs")).default;
  const wb = new ExcelJS.Workbook();
  wb.creator = "DigiText";
  wb.created = new Date();

  // ---- Sheet 1: Tổng hợp ----
  const ws0 = wb.addWorksheet("Tổng hợp");
  const luc = new Date().toLocaleString("vi-VN");
  const tong: [string, string | number][] = [
    ["Báo cáo kiểm chữ ký số tệp PDF", ""],
    ["Lập lúc", luc],
    ["Thư mục đã quét", goc.join("\n")],
    ["", ""],
    ["Số tệp PDF đã kiểm", kq.soPdf],
    ["Đạt", kq.soDat],
    ["Đạt, có cảnh báo", kq.soCanhBao],
    ["Không đạt", kq.soKhongDat],
    ["Chưa ký", kq.soChuaKy],
    ["Tổng số chữ ký tìm thấy", kq.tep.reduce((s, t) => s + t.soChuKy, 0)],
    ["", ""],
    ["Mục bỏ qua vì không phải .pdf", kq.boQuaKhongPdf],
    ["Tệp bỏ qua vì quá lớn", kq.boQuaQuaLon],
    ...(kq.biCat ? ([["Ghi chú", "Quét đã dừng ở trần 50.000 tệp — chia nhỏ thư mục rồi quét lại."]] as [string, string][]) : []),
    ["", ""],
    [
      "Phạm vi của báo cáo",
      "Báo cáo này nói về CHỮ KÝ SỐ: chữ ký có toàn vẹn không, ai ký, ký lúc nào, có dấu thời gian và dữ liệu xác minh dài hạn không. Nó KHÔNG kiểm chất lượng tệp — không PDF/A, không độ phân giải, không lớp chữ. Dùng Tiện ích › Kiểm PDF/A cho việc đó.",
    ],
    [
      "Chữ ký không phủ hết tệp",
      "Tệp ký nhiều lần thì chỉ chữ ký SAU CÙNG phủ tới hết tệp; các chữ ký trước phủ tới bản sửa của chính nó. Đó là cách PDF hoạt động, KHÔNG phải dấu hiệu bị sửa. Dấu hiệu bị sửa thật là cột Toàn vẹn ghi Không, hoặc có byte nối thêm sau dấu kết thúc tệp.",
    ],
  ];
  for (const [a, b] of tong) ws0.addRow([a, b]);
  ws0.getColumn(1).width = 34;
  ws0.getColumn(2).width = 96;
  ws0.getColumn(2).alignment = { wrapText: true, vertical: "top" };
  ws0.getRow(1).font = { bold: true, size: 14 };
  for (const r of [5, 6, 7, 8, 9, 10]) ws0.getRow(r).getCell(1).font = { bold: true };
  for (const [r, k] of [
    [6, "Đạt"],
    [7, "Đạt, có cảnh báo"],
    [8, "Không đạt"],
    [9, "Chưa ký"],
  ] as const) {
    const c = ws0.getRow(r).getCell(2);
    const m = mauKetLuan(k);
    if (m) c.style = { ...c.style, fill: { type: "pattern", pattern: "solid", fgColor: { argb: m } } };
  }

  // ---- Sheet 2: Tệp ----
  const td1 = ["Kết luận", "Tên tệp", "Số chữ ký", "Mức chữ ký", "Cỡ", "Thư mục", "Lỗi", "Cần xem lại"];
  const d1 = kq.tep.map((t) => [
    t.ketLuan,
    t.ten,
    t.soChuKy,
    mucChuKy(t),
    co(t.co),
    t.thuMuc,
    t.loi.join("\n"),
    t.canhBao.join("\n"),
  ]);
  bang(wb.addWorksheet("Tệp"), td1, d1, 0, (r) => mauKetLuan(String(d1[r][0])));

  // ---- Sheet 3: Chữ ký ----
  const td2 = [
    "Tệp",
    "#",
    "Nhãn",
    "Loại",
    "Khuôn dạng",
    "Người ký (theo chứng thư)",
    "Tổ chức",
    "Nhà cấp chứng thư",
    "Chứng thư hiệu lực tới",
    "Thời điểm ký (có dấu thời gian)",
    "Thời điểm tệp tự khai",
    "Dấu thời gian",
    "Gắn chứng thư",
    "Toàn vẹn",
    "Phủ hết tệp",
    "Đạt",
    "Không xác minh được vì",
  ];
  const d2: (string | number)[][] = [];
  const dat2: boolean[] = [];
  for (const t of kq.tep) {
    for (const c of t.chuKy) {
      d2.push([
        t.ten,
        c.thuTu,
        c.nhan,
        c.loai === "DocTimeStamp" ? "Dấu thời gian tài liệu" : "Chữ ký",
        c.khuonDang ?? "",
        c.nguoiKy ?? "",
        c.toChuc ?? "",
        c.nhaCap ?? "",
        c.hieuLucDen ?? "",
        c.thoiDiemTsa ?? "",
        c.thoiDiemKhai ?? "",
        co2(c.coTsa),
        co2(c.ganChungThu),
        co2(c.toanVen),
        co2(c.phuTronTep),
        c.dat ? "Đạt" : "Không đạt",
        c.loi ?? "",
      ]);
      dat2.push(c.dat);
    }
  }
  bang(wb.addWorksheet("Chữ ký"), td2, d2, 15, (r) => (dat2[r] ? MAU.dat : MAU.khongDat));

  const bytes = new Uint8Array(await wb.xlsx.writeBuffer());
  const ten = `KiemChuKySo_${new Date().toISOString().slice(0, 10)}`;
  return V4dz(bytes, ten, "xlsx");
}

/** Dựng một sheet bảng: hàng tiêu đề in đậm, đóng băng, bật lọc, tô ô ở `cotTo`. */
function bang(
  ws: import("exceljs").Worksheet,
  tieuDe: string[],
  dong: (string | number)[][],
  cotTo: number,
  mau: (r: number) => string | null,
): void {
  ws.addRow(tieuDe);
  for (const r of dong) ws.addRow(r);
  const w = doRong(tieuDe, dong);
  tieuDe.forEach((_, i) => {
    ws.getColumn(i + 1).width = w[i];
  });
  const h = ws.getRow(1);
  h.font = { bold: true };
  h.alignment = { wrapText: true, vertical: "middle" };
  h.height = 30;
  ws.views = [{ state: "frozen", xSplit: 0, ySplit: 1 }];
  if (dong.length > 0) {
    ws.autoFilter = { from: { row: 1, column: 1 }, to: { row: dong.length + 1, column: tieuDe.length } };
  }
  // Cột lý do dài: cho xuống dòng, để người đọc không phải kéo ngang.
  for (let i = 0; i < tieuDe.length; i++) {
    if (/Lỗi|xem lại|vì$/.test(tieuDe[i])) ws.getColumn(i + 1).alignment = { wrapText: true, vertical: "top" };
  }
  dong.forEach((_, r) => {
    const m = mau(r);
    if (!m) return;
    const c = ws.getRow(r + 2).getCell(cotTo + 1);
    // ExcelJS cho các ô cùng định dạng DÙNG CHUNG một đối tượng style: gán c.fill sẽ tô cả loạt
    // ô khác. Phải thay cả style bằng bản sao riêng cho đúng ô này.
    c.style = { ...c.style, fill: { type: "pattern", pattern: "solid", fgColor: { argb: m } } };
  });
}

/** Tệp nào cần người xem trước: không đạt xếp trên, rồi cảnh báo, rồi chưa ký. */
export function xepTheoViecCanLam(ds: TepDaKy[]): TepDaKy[] {
  const hang = (k: string) => (k === "Không đạt" || k === "Bỏ qua" ? 0 : k === "Đạt, có cảnh báo" ? 1 : k === "Chưa ký" ? 2 : 3);
  return [...ds].sort((a, b) => hang(a.ketLuan) - hang(b.ketLuan) || a.ten.localeCompare(b.ten, "vi"));
}

/**
 * Mức chữ ký theo PAdES, nói bằng thứ người lưu trữ cần biết: chữ ký này còn chứng minh được
 * bao lâu.
 *
 * LTA (có dấu thời gian cho cả tài liệu) > LT (có dữ liệu xác minh nhúng kèm) > B-T (chỉ có dấu
 * thời gian) > B-B (không có gì). TT05 đòi mức LT trở lên cho tài liệu bảo quản vĩnh viễn.
 */
export function mucChuKy(t: TepDaKy): string {
  if (t.soChuKy === 0) return "—";
  if (t.coDauThoiGianTaiLieu) return "LTA — có dấu thời gian cả tài liệu";
  if (t.coDss) return "LT — có dữ liệu xác minh nhúng kèm";
  if (t.chuKy.some((c) => c.coTsa)) return "B-T — chỉ có dấu thời gian";
  return "B-B — không có dấu thời gian";
}
