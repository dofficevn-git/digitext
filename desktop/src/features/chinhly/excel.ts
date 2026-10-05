// Copyright (c) 2026 DigiText. All rights reserved.
// Excel chỉnh lý: mục lục tạm (đầu ra hệ thống hoá, đầu vào gán hộp/bó/tập), mẫu lập hồ sơ, mẫu mục lục
// văn bản — giữ đúng bố cục cột của bản cũ để tệp cũ nhập lại được.

import { isTauri } from "@/api/ipc";
import { V4dz } from "@/api/exportFile";
import { V4b7 } from "@/domain/catalogImport";
import { str, type ClRow } from "@/api/chinhly";

type Cell = string | number | null;

export interface Sheet {
  name: string;
  aoa: Cell[][];
  cols?: { wch?: number; hidden?: boolean }[];
  merges?: string[];
}

export async function writeXlsx(sheets: Sheet[], fileName: string): Promise<void> {
  const X = await import("xlsx");
  const wb = X.utils.book_new();
  for (const s of sheets) {
    const ws = X.utils.aoa_to_sheet(s.aoa.map((r) => r.map((c) => (c === null ? "" : c))));
    if (s.cols) ws["!cols"] = s.cols;
    if (s.merges?.length) ws["!merges"] = s.merges.map((r) => X.utils.decode_range(r));
    X.utils.book_append_sheet(wb, ws, (s.name || "Sheet").slice(0, 31));
  }
  if (isTauri()) {
    const buf = X.write(wb, { type: "array", bookType: "xlsx" }) as ArrayBuffer;
    await V4dz(new Uint8Array(buf), fileName, "xlsx");
  } else {
    X.writeFile(wb, fileName);
  }
}

export async function readFirstSheet(file: File): Promise<string[][]> {
  const wb = await V4b7(file);
  return wb.sheets[0]?.rows ?? [];
}

// ------------------------------------------------------------------ mục lục tạm (bản cũ: XuatFileTam / Input)

export interface PreviewRow {
  stt: number;
  /** Loại bảo quản — mỗi loại đánh số riêng (0015). */
  vinhVien?: boolean;
  hoSoId: string;
  soTam: string;
  tieuDe: string;
  ngayBatDau: string | null;
  ngayKetThuc: string | null;
  soTo: string | null;
  thbqId: string | null;
  tenThbq: string | null;
  thbqStatus: string | null;
  phongBanId: string | null;
  maDonVi: string | null;
  tenPhongBan: string | null;
  mangId: string | null;
  maMang: string | null;
  tenMang: string | null;
  ghiChu: string | null;
  hoSoSoCu: string | null;
  hopSo: string | null;
  boSo: number | null;
  tapSo: number | null;
  maHoSoMoi: string | null;
  warnings: string[];
}

/** Cột A–P như bản cũ; ô P1 = fileType (1 hộp/hồ sơ, 2 bó/tập). Dòng 1 tiêu đề, dòng 2 số 1..15, dữ liệu từ dòng 3. */
export async function xuatMucLucTam(rows: PreviewRow[], fileType: 1 | 2, phongId: string, tenPhong: string, orderLabel: string): Promise<void> {
  const h1 = fileType === 1 ? ["Hộp số", "Hồ sơ số"] : ["Bó số", "Tập số"];
  const head: Cell[] = [...h1, "Mã mảng công việc", "ID Hồ sơ", "Số tạm", "Tiêu đề hồ sơ", "Ngày tháng bắt đầu", "Ngày tháng kết thúc", "Số tờ", "Thời hạn bảo quản", "Ghi chú", "Mã phông lưu trữ", "Mã phòng ban", "Mảng", "Thời hạn bq", fileType];
  const nums: Cell[] = Array.from({ length: 15 }, (_, i) => i + 1);
  const aoa: Cell[][] = [head, nums];
  for (const r of rows) {
    aoa.push([
      fileType === 1 ? r.hopSo ?? "" : r.boSo ?? "",
      fileType === 1 ? r.stt : r.tapSo ?? "",
      r.maMang ?? "",
      r.hoSoId,
      r.soTam,
      r.tieuDe,
      r.ngayBatDau ?? "",
      r.ngayKetThuc ?? "",
      r.soTo && r.soTo !== "0" ? r.soTo : "",
      r.tenThbq ?? "",
      r.ghiChu ?? "",
      phongId,
      r.phongBanId ?? "",
      r.mangId ?? "",
      r.thbqId ?? "",
    ]);
  }
  const cols = [{ wch: 9 }, { wch: 9 }, { wch: 10 }, { wch: 38 }, { wch: 9 }, { wch: 50 }, { wch: 12 }, { wch: 12 }, { wch: 7 }, { wch: 18 }, { wch: 24 }, { hidden: true }, { hidden: true }, { hidden: true }, { hidden: true }, { wch: 4 }];
  const ten = `HoSoTam_${orderLabel.replace(/[^\p{L}\p{N}]+/gu, "_")}_${tenPhong.replace(/[^\p{L}\p{N}]+/gu, "_")}_${new Date().toISOString().slice(0, 10)}.xlsx`;
  await writeXlsx([{ name: "Muc luc tam", aoa, cols }], ten);
}

export interface BoxRowIn {
  hoSoId: string;
  hopSo?: string;
  boSo?: number;
  tapSo?: number;
}
export interface DocMucLucTam {
  fileType: 1 | 2;
  rows: BoxRowIn[];
  errors: { row: number; message: string }[];
}

/** Đọc lại tệp mục lục tạm đã điền hộp (hoặc bó/tập). Dữ liệu từ dòng 3; bỏ dòng trống. */
export async function docMucLucTam(file: File): Promise<DocMucLucTam> {
  const rows = await readFirstSheet(file);
  const p1 = (rows[0]?.[15] ?? "").trim();
  const fileType: 1 | 2 = p1 === "2" ? 2 : 1;
  const out: DocMucLucTam = { fileType, rows: [], errors: [] };
  const isUuid = (s: string) => /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(s);
  for (let i = 2; i < rows.length; i++) {
    const r = rows[i];
    const line = i + 1;
    const id = (r[3] ?? "").trim();
    if (!id && !(r[0] ?? "").trim() && !(r[4] ?? "").trim()) continue;
    if (!isUuid(id)) {
      out.errors.push({ row: line, message: "Cột D (ID hồ sơ) không hợp lệ — dùng tệp do phần mềm xuất" });
      continue;
    }
    if (fileType === 1) {
      const hop = (r[0] ?? "").trim();
      if (!hop) {
        out.errors.push({ row: line, message: "Chưa điền Hộp số (cột A)" });
        continue;
      }
      out.rows.push({ hoSoId: id, hopSo: hop });
    } else {
      const bo = Number.parseInt((r[0] ?? "").trim(), 10);
      const tap = Number.parseInt((r[1] ?? "").trim(), 10);
      if (Number.isNaN(bo) && Number.isNaN(tap)) {
        out.errors.push({ row: line, message: "Chưa điền Bó số / Tập số (cột A, B) hoặc không phải số" });
        continue;
      }
      out.rows.push({ hoSoId: id, boSo: Number.isNaN(bo) ? undefined : bo, tapSo: Number.isNaN(tap) ? undefined : tap });
    }
  }
  return out;
}

// ------------------------------------------------------------------ mẫu lập hồ sơ (bản cũ: DownloadTemplateLapHS / InputTemp)

export const MAU_HS_COLS = ["Mã phòng ban", "Mã mảng công việc", "Số tạm", "Tiêu đề hồ sơ", "Ngày bắt đầu", "Ngày kết thúc", "Mã THBQ", "Số tờ", "Ghi chú", "Tổng số VB", "Số lượng trang", "Chế độ sử dụng", "Ngôn ngữ", "TT Vật lý"];
export const MAU_HS_KEYS = ["ma_don_vi", "ma_mang", "so_tam", "tieu_de", "ngay_bat_dau", "ngay_ket_thuc", "thbq", "so_to", "ghi_chu", "tong_so_vb", "so_luong_trang", "che_do_su_dung", "ngon_ngu", "tinh_trang_vat_ly"];

/** 4 dòng đầu của mẫu lập hồ sơ (A1 = mã phông; dòng 2 tên cột; dòng 3 số 1–14; dòng 4 kiểu dữ liệu). Dữ liệu từ dòng 5. */
export function mauLapHoSoDauTrang(maPhong: string): { aoa: Cell[][]; cols: { wch: number }[]; merges: string[] } {
  return {
    aoa: [
      [maPhong, "", "Mô tả hồ sơ"],
      [...MAU_HS_COLS],
      Array.from({ length: 14 }, (_, i) => i + 1),
      ["Text", "Text", "String", "String", "Date (dd/mm/yyyy)", "Date (dd/mm/yyyy)", "String (tên hoặc số năm)", "Number", "String", "Number", "Number", "01/02/03", "01..11", "01/02/03"],
    ],
    cols: MAU_HS_COLS.map((c) => ({ wch: Math.max(12, c.length + 2) })),
    merges: ["A1:B1", "C1:N1"],
  };
}

/** Sheet "Danh mục THBQ" kèm mẫu: tên và loại của từng thời hạn bảo quản. */
export function sheetDanhMucThbq(thbqs: ClRow[]): Sheet {
  const ds: Cell[][] = [["Mã THBQ / tên", "Loại"], ...thbqs.map((t) => [str(t, "ten"), str(t, "trang_thai") === "1" ? "Vĩnh viễn" : str(t, "trang_thai") === "2" ? `${str(t, "gia_tri")} năm` : "Đánh giá lại"])];
  return { name: "Danh mục THBQ", aoa: ds };
}

export async function xuatMauLapHoSo(maPhong: string, thbqs: ClRow[]): Promise<void> {
  const dau = mauLapHoSoDauTrang(maPhong);
  await writeXlsx([{ name: "Lập hồ sơ điện tử", aoa: dau.aoa, cols: dau.cols, merges: dau.merges }, sheetDanhMucThbq(thbqs)], `mau-lap-ho-so-${maPhong}.xlsx`);
}

export interface DocMauHoSo {
  maPhong: string;
  rows: Record<string, unknown>[];
  errors: { row: number; message: string }[];
}

/** Đọc mẫu lập hồ sơ (cũ và mới): bỏ 4 dòng đầu (dòng 4 nếu là dòng "kiểu dữ liệu"). */
export async function docMauLapHoSo(file: File): Promise<DocMauHoSo> {
  const rows = await readFirstSheet(file);
  const maPhong = (rows[0]?.[0] ?? "").trim();
  const out: DocMauHoSo = { maPhong, rows: [], errors: [] };
  const start = /^(text|string|date|number)/i.test((rows[3]?.[0] ?? "").trim()) ? 4 : 3;
  for (let i = start; i < rows.length; i++) {
    const r = rows[i];
    if (r.every((c) => !c || !c.trim())) continue;
    const o: Record<string, unknown> = {};
    MAU_HS_KEYS.forEach((k, j) => {
      const v = (r[j] ?? "").trim();
      if (v !== "") o[k] = v;
    });
    if (!o.so_tam) {
      out.errors.push({ row: i + 1, message: "Thiếu số tạm (cột C)" });
      continue;
    }
    // Số tờ chỉ nhận số tự nhiên > 0 (5/10/2026) — "01 quyển" bị từ chối, dòng không nhập.
    const soTo = String(o.so_to ?? "");
    if (soTo && (!/^\d+$/.test(soTo) || +soTo === 0)) {
      const cot = String.fromCharCode(65 + MAU_HS_KEYS.indexOf("so_to"));
      out.errors.push({ row: i + 1, message: `Số tờ "${soTo}" không hợp lệ (cột ${cot}) — chỉ ghi số tự nhiên, bỏ chữ như "quyển", "tờ"` });
      continue;
    }
    out.rows.push(o);
  }
  return out;
}

// ------------------------------------------------------------------ mẫu mục lục văn bản (bản cũ: DownloadTemplateLapVB / InputVanBan)

export const MAU_VB_COLS = ["ID Hồ sơ", "Số thứ tự VB trong HS", "Tên loại VB", "Số của tài liệu", "Ký hiệu văn bản", "Ngày tháng VB", "Cơ quan ban hành VB", "Trích yếu nội dung VB", "Tờ số", "Số lượng trang VB", "Ghi chú", "Ngôn ngữ", "Chế độ sử dụng", "Mức độ tin cậy", "Bút tích", "Tình trạng vật lý"];
const MAU_VB_KEYS = ["", "so_thu_tu", "loai_vb", "so_van_ban", "so_ky_hieu", "ngay_thang", "co_quan_ban_hanh", "trich_yeu", "to_so", "so_trang", "ghi_chu", "ngon_ngu", "che_do_su_dung", "do_tin_cay", "but_tich", "tinh_trang_vat_ly"];

export async function xuatMauMlvb(hoSoId: string, maHoSo: string, tieuDe: string): Promise<void> {
  const aoa: Cell[][] = [
    [`ID Hồ sơ: ${hoSoId}`, "", maHoSo, tieuDe],
    [...MAU_VB_COLS],
    Array.from({ length: 16 }, (_, i) => i + 1),
    ["Text", "Number", "Text (tên hoặc mã 01..32)", "Text", "Text", "Date (dd/mm/yyyy · mm/yyyy · yyyy)", "Text", "Text", "Number", "Number", "Text", "01..11", "01/02/03", "01/02/03", "Text", "01/02/03"],
    [hoSoId, 1],
  ];
  await writeXlsx([{ name: "Mục lục văn bản", aoa, cols: MAU_VB_COLS.map((c) => ({ wch: Math.max(10, Math.min(30, c.length + 2)) })) }], `mau-muc-luc-van-ban-${maHoSo || hoSoId.slice(0, 8)}.xlsx`);
}

export interface DocMauVb {
  hoSoId: string | null;
  rows: Record<string, unknown>[];
  errors: { row: number; message: string }[];
}

export async function docMauMlvb(file: File): Promise<DocMauVb> {
  const rows = await readFirstSheet(file);
  const m = /ID Hồ sơ:\s*([0-9a-f-]{36}|\d+)/i.exec(rows[0]?.[0] ?? "");
  const out: DocMauVb = { hoSoId: m ? m[1] : null, rows: [], errors: [] };
  const start = /^(text|number|date)/i.test((rows[3]?.[0] ?? "").trim()) ? 4 : 3;
  for (let i = start; i < rows.length; i++) {
    const r = rows[i];
    if (r.every((c) => !c || !c.trim())) continue;
    const o: Record<string, unknown> = {};
    MAU_VB_KEYS.forEach((k, j) => {
      if (!k) return;
      const v = (r[j] ?? "").trim();
      if (v !== "") o[k] = v;
    });
    if (!o.trich_yeu && !o.so_ky_hieu && !o.so_van_ban) {
      out.errors.push({ row: i + 1, message: "Dòng không có trích yếu / số ký hiệu" });
      continue;
    }
    out.rows.push(o);
  }
  return out;
}
