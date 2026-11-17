// Copyright (c) 2026 DigiText. All rights reserved.
// Kết xuất của màn «Nhập mục lục hồ sơ»: (1) tệp mẫu lập hồ sơ 14 cột + sheet "Kiểm tra";
// (2) chính tệp gốc đã điền thời gian, tô màu và ghi chú vào ô.
//
// Tách khỏi `mucLucImport.ts` ngày 10/11/2026 khi bộ ĐỌC chuyển lên máy chủ: đây là phần **ghi**,
// phải ở lại máy trạm (ghi tệp xuống đĩa, giữ nguyên bố cục tệp gốc) và không chứa luật kiểm nào —
// mọi kết luận lỗi/cảnh báo/thời gian đã do máy chủ quyết, ở đây chỉ tô và chép ra.

import { V4dz } from "@/api/exportFile";
import type { ClRow } from "@/api/chinhly";
import { MAU_HS_KEYS, mauLapHoSoDauTrang, sheetDanhMucThbq, writeXlsx, type Sheet } from "./excel";
import { vaDinhDangDieuKien } from "./xlsxVa";
import {
  KET_LUAN_LABEL,
  mangCua,
  pbCua,
  soTamCua,
  TEN_LOAI,
  type PhanTichMucLuc,
} from "./mucLucMayChu";

const COT_KIEM_TRA = [
  "Dòng gốc",
  "Loại",
  "Hộp số",
  "Hồ sơ số",
  "Số tạm",
  "Phòng ban",
  "Mảng",
  "Tiêu đề",
  "Thời gian gốc",
  "Bắt đầu",
  "Kết thúc",
  "THBQ",
  "Kết luận",
  "Ghi chú kiểm tra",
];

function dongKiemTra(pt: PhanTichMucLuc, soTam: Map<number, string>): (string | number)[][] {
  const out: (string | number)[][] = pt.rows.map((r) => {
    const pb = pbCua(pt, r);
    const mang = mangCua(pt, r);
    return [
      r.dong,
      TEN_LOAI(r.vinhVien),
      r.hopSo,
      r.hoSoSo,
      soTam.get(r.dong) ?? "",
      pb ? `${pb.ma} · ${pb.ten}` : "",
      mang ? `${mang.ma} · ${mang.ten}${mang.tuDong ? " (tự tạo)" : ""}` : "",
      r.tieuDe,
      r.thoiGianGoc.replace(/\s*\n\s*/g, " "),
      r.tg.batDau,
      r.tg.ketThuc,
      r.thbq ?? r.thbqGoc,
      r.loi.length ? "Lỗi" : r.canhBao.length ? "Cảnh báo" : KET_LUAN_LABEL[r.tg.ketLuan],
      [...r.loi, ...r.canhBao, ...r.tg.ghiChu].join("; "),
    ];
  });
  for (const v of pt.vanDe) {
    out.push([v.dong || "", "", "", "", "", "", "", "", "", "", "", "", v.muc === "loi" ? "Lỗi" : "Cảnh báo", v.noiDung]);
  }
  return out.sort((a, b) => Number(a[0] || 0) - Number(b[0] || 0));
}

/**
 * Tệp đã chuyển: sheet 1 đúng mẫu lập hồ sơ 14 cột (đọc lại được bằng "Nhập hồ sơ từ Excel"),
 * sheet 2 "Kiểm tra", sheet 3 danh mục THBQ.
 *
 * `rowsNhap` là **đúng mảng máy chủ trả về** (`KetQuaDocMucLuc.rowsNhap`), xếp cùng thứ tự
 * `pt.rows` — không dựng lại ở máy trạm, kẻo tệp tải xuống khác dữ liệu sắp nhập.
 */
export async function xuatTepDaChuyen(
  pt: PhanTichMucLuc,
  rowsNhap: Record<string, unknown>[],
  maPhong: string,
  thbqs: ClRow[],
): Promise<void> {
  const soTam = soTamCua(pt);
  const dau = mauLapHoSoDauTrang(maPhong);
  const aoa = [...dau.aoa];
  for (const o of rowsNhap) {
    aoa.push(MAU_HS_KEYS.map((k) => (o[k] === undefined ? "" : String(o[k]))));
  }
  const kt: Sheet = {
    name: "Kiểm tra",
    aoa: [COT_KIEM_TRA, ...dongKiemTra(pt, soTam)],
    cols: [
      { wch: 8 },
      { wch: 12 },
      { wch: 7 },
      { wch: 8 },
      { wch: 10 },
      { wch: 22 },
      { wch: 22 },
      { wch: 50 },
      { wch: 22 },
      { wch: 11 },
      { wch: 11 },
      { wch: 11 },
      { wch: 10 },
      { wch: 60 },
    ],
  };
  const goc = pt.tenTep.replace(/\.[^.]+$/, "").replace(/[^\p{L}\p{N}]+/gu, "-");
  await writeXlsx(
    [{ name: "Lập hồ sơ điện tử", aoa, cols: dau.cols, merges: dau.merges }, kt, sheetDanhMucThbq(thbqs)],
    `mau-lap-ho-so-${maPhong}-${goc}.xlsx`,
  );
}

// ------------------------------------------------------------------ tệp gốc đã điền + đánh dấu

/**
 * Chỉ 3 kiểu đánh dấu (anh Hiếu 2/10: "đánh dấu cái cần đánh dấu thôi, lỗi nổi bật khác hẳn"):
 * xanh nhạt = ô đã sửa nội dung (DienThoiGian C6EFCE), vàng = cần xem lại, ĐỎ ĐẬM chữ trắng đậm = lỗi phải sửa.
 * Ô đúng sẵn (chỉ khác cách viết 7/6 ↔ 07/06, xuống dòng ↔ dấu cách) để nguyên, không tô, không ghi đè.
 */
export const MAU_DANH_DAU = {
  daDien: "FFC6EFCE",
  canhBao: "FFFFFF00",
  loi: "FFFF0000",
} as const;

export interface ThongKeDanhDau {
  daDien: number;
  khongDien: number;
  canhBao: number;
  loi: number;
}

/**
 * Xuất lại CHÍNH tệp gốc (giữ nguyên bố cục, định dạng) với cột thời gian đã điền
 * "dd/mm/yyyy dd/mm/yyyy", ô được sửa tô màu + ghi chú — như `<tên>_DADIEN.xlsx` của DienThoiGian.
 * Ô lỗi chặn nhập (cột Tiêu đề / THBQ / Hồ sơ số) cũng tô + ghi chú để sửa thẳng trên tệp rồi chọn lại.
 */
export async function xuatTepDanhDau(pt: PhanTichMucLuc): Promise<ThongKeDanhDau> {
  const { bytes, tk } = await danhDauTepGoc(pt);
  await V4dz(bytes, `${pt.tenTep.replace(/\.[^.]+$/, "")}_DADIEN.xlsx`, "xlsx");
  return tk;
}

export async function danhDauTepGoc(pt: PhanTichMucLuc): Promise<{ bytes: Uint8Array; tk: ThongKeDanhDau }> {
  if (!pt.buf) throw new Error("Không còn nội dung tệp gốc — chọn lại tệp.");
  if (!/\.xlsx$|\.xlsm$/i.test(pt.tenTep)) {
    throw new Error("Chỉ đánh dấu được tệp .xlsx. Mở tệp .xls bằng Excel, lưu lại dạng .xlsx rồi chọn lại.");
  }
  const ExcelJS = (await import("exceljs")).default;
  const wb = new ExcelJS.Workbook();
  await wb.xlsx.load(pt.buf.slice(0));
  const ws = wb.getWorksheet(pt.sheet) ?? wb.worksheets[0];
  if (!ws) throw new Error("Tệp không có sheet nào.");
  const tk: ThongKeDanhDau = { daDien: 0, khongDien: 0, canhBao: 0, loi: 0 };

  const o = (dong: number, cot0: number | undefined) => {
    if (cot0 === undefined) return null;
    const c = ws.getCell(dong, cot0 + 1);
    return c.master ?? c; // ô trong vùng gộp → ghi vào ô gốc (trên-trái)
  };
  const daGhi = new Map<string, string>();
  const danhDau = (dong: number, cot0: number | undefined, mau: string, ghiChu: string) => {
    const c = o(dong, cot0);
    if (!c) return;
    const laLoi = mau === MAU_DANH_DAU.loi;
    // ExcelJS cho các ô cùng định dạng DÙNG CHUNG một đối tượng style: gán c.fill sẽ tô cả loạt ô khác
    // (cả cột Tiêu đề đỏ, cột Số tờ xanh). Phải thay style bằng bản sao riêng cho đúng ô này.
    c.style = {
      ...c.style,
      fill: { type: "pattern", pattern: "solid", fgColor: { argb: mau } },
      ...(laLoi ? { font: { ...(c.style.font ?? {}), bold: true, color: { argb: "FFFFFFFF" } } } : {}),
    };
    const cu = daGhi.get(c.address);
    const text = cu ? `${cu}\n\n${ghiChu}` : ghiChu;
    daGhi.set(c.address, text);
    c.note = { texts: [{ text }], margins: { insetmode: "auto" } } as never;
  };
  const cotTg = pt.cot.thoiGian ?? pt.cot.ngayBatDau;
  const RE_NGAY = /(\d{1,2})\s*\/\s*(\d{1,2})\s*\/\s*(\d{4})/g;
  const pad = (d: string, m: string, y: string) => `${d.padStart(2, "0")}/${m.padStart(2, "0")}/${y}`;

  for (const r of pt.rows) {
    const { batDau, ketThuc, ketLuan, ghiChu } = r.tg;
    const goc = r.thoiGianGoc.replace(/\s*\n\s*/g, " ").trim();
    const moi = batDau && ketThuc ? `${batDau} ${ketThuc}` : "";
    // Ô đã ghi đủ 2 ngày đúng thứ tự (chỉ khác cách viết) → giữ nguyên, không ghi đè, không tô.
    const ngays = [...goc.matchAll(RE_NGAY)].map((m) => pad(m[1], m[2], m[3]));
    const conLai = goc.replace(RE_NGAY, "").replace(/['\s\-–—;,]/g, "");
    const giuNguyen = ngays.length === 2 && !conLai && ngays[0] === batDau && ngays[1] === ketThuc;
    const doi = !!moi && !giuNguyen;
    if (doi) {
      if (pt.cot.thoiGian !== undefined) {
        const c = o(r.dong, pt.cot.thoiGian);
        if (c) c.value = moi;
      } else {
        const a = o(r.dong, pt.cot.ngayBatDau);
        const b = o(r.dong, pt.cot.ngayKetThuc);
        if (a) a.value = batDau;
        if (b) b.value = ketThuc;
      }
    }
    const lyDo = ghiChu.join("\n");
    if (ketLuan === "loi" || !moi) {
      tk.khongDien++;
      danhDau(
        r.dong,
        cotTg,
        MAU_DANH_DAU.loi,
        `LỖI — không điền được thời gian\nGiá trị: ${goc || "(trống)"}\nLý do: ${lyDo || "không đọc được năm"}`,
      );
    } else if (ketLuan === "canh_bao") {
      tk.canhBao++;
      danhDau(
        r.dong,
        cotTg,
        MAU_DANH_DAU.canhBao,
        `Cần xem lại\n${doi ? `Cũ : ${goc || "(trống)"}\nMới: ${moi}\n` : ""}Lý do: ${lyDo}`,
      );
    } else if (doi) {
      tk.daDien++;
      danhDau(r.dong, cotTg, MAU_DANH_DAU.daDien, `Đã sửa\nCũ : ${goc || "(trống)"}\nMới: ${moi}${lyDo ? `\nLý do: ${lyDo}` : ""}`);
    }
    // Mọi hồ sơ có lỗi đều tô đỏ — kể cả cả đoạn hồ sơ không thuộc mảng / phòng ban nào (anh Hiếu 2/10).
    const cotCua = (s: string) =>
      /^Số tờ|^Thiếu số tờ/.test(s)
        ? pt.cot.soTo
        : /THBQ|bảo quản/i.test(s)
          ? pt.cot.thbq
          : /Hồ sơ số/i.test(s)
            ? pt.cot.hoSoSo
            : /Hộp/.test(s)
              ? (pt.cot.hopSo ?? pt.cot.tieuDe)
              : pt.cot.tieuDe;
    for (const loi of r.loi) {
      tk.loi++;
      danhDau(r.dong, cotCua(loi), MAU_DANH_DAU.loi, `LỖI: ${loi}`);
    }
    // Cảnh báo (vẫn nhập): tô vàng ô liên quan để người lập xem lại.
    for (const cb of r.canhBao) {
      tk.canhBao++;
      danhDau(r.dong, cotCua(cb), MAU_DANH_DAU.canhBao, `Cần xem lại: ${cb}`);
    }
  }
  for (const v of pt.vanDe) {
    if (v.muc !== "loi" || !v.dong) continue;
    tk.loi++;
    danhDau(v.dong, pt.cot.tieuDe, MAU_DANH_DAU.loi, `LỖI: ${v.noiDung}`);
  }

  // ExcelJS ghi thẻ định dạng có điều kiện rỗng cho loại nó không hiểu (Duplicate Values…) → Excel "Repaired" và
  // xoá luôn ghi chú. Vá bằng khối gốc trước khi trả (xlsxVa.ts).
  const { bytes } = await vaDinhDangDieuKien(pt.buf, new Uint8Array(await wb.xlsx.writeBuffer()));
  return { bytes, tk };
}
