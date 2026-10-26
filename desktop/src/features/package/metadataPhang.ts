// Copyright (c) 2026 DigiText. All rights reserved.
// Đọc Excel "metadata phẳng" (26/10/2026): mỗi dòng một VĂN BẢN, mang luôn cột phông + hồ sơ + tài liệu (mẫu
// Metadata-<PHÔNG>.xlsx của LTLSBN; cùng mẫu với crates/digitext-extimport/src/excel.rs nhưng đọc ngay trên máy
// trạm). Tên cột lặp giữa ba khối (Ngôn ngữ, Ghi chú, Số lượng tờ…) nên dò theo hai mốc: "Mã hồ sơ" mở khối hồ sơ,
// "Mã định danh tài liệu" / "Mã lưu trữ tài liệu" mở khối tài liệu, rồi tìm cột TRONG từng khối theo tên đã bỏ dấu.
// Mã lưu trữ tài liệu (H05.13.24.01.2005.01.01.0000001.pdf) chính là tên tệp đã đẩy lên máy chủ → ghép theo tên tệp.

export interface HoSoPhang {
  maHoSo: string;
  tieuDe: string;
  thbq: string;
  batDau: string;
  ketThuc: string;
  soTo: string;
  soTrang: string;
  maGocGiay: string;
  /** Tên tệp (chữ thường, có .pdf) của các văn bản thuộc hồ sơ, theo thứ tự trong Excel. */
  tep: string[];
  /** Dòng Excel đầu tiên của hồ sơ. */
  dong: number;
}
export interface MetadataPhang {
  fondCode: string;
  fondName: string;
  hoSos: HoSoPhang[];
  /** tên tệp (chữ thường) → mã hồ sơ. */
  tepToiHoSo: Map<string, string>;
  dongTaiLieu: number;
  dongKhongTenTep: number;
  dongTieuDe: number;
  /** Tên cột đã nhận, để người dùng soát. */
  mapping: [string, string][];
}

/** Bỏ dấu + thường hoá + gộp khoảng trắng (như excel.rs::fold). */
export function fold(s: string): string {
  return s
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/đ/g, "d")
    .replace(/Đ/g, "D")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, " ")
    .trim();
}
const text = (v: unknown): string => {
  if (v == null) return "";
  if (v instanceof Date) return `${String(v.getDate()).padStart(2, "0")}/${String(v.getMonth() + 1).padStart(2, "0")}/${v.getFullYear()}`;
  return String(v).replace(/\s+/g, " ").trim();
};

/** Có phải mẫu metadata phẳng không: một dòng trong 15 dòng đầu có cả "mã hồ sơ" lẫn "mã lưu trữ/định danh tài liệu". */
export function laMetadataPhang(rows: unknown[][]): number {
  for (let i = 0; i < Math.min(rows.length, 15); i++) {
    const f = (rows[i] ?? []).map((c) => fold(text(c)));
    if (f.includes("ma ho so") && f.some((x) => x.startsWith("ma luu tru tai lieu") || x.startsWith("ma dinh danh tai lieu"))) return i;
  }
  return -1;
}

const COT_PHONG: [string, string[]][] = [
  ["fondCode", ["ma phong"]],
  ["fondName", ["ten phong"]],
];
const COT_HS: [string, string[]][] = [
  ["maHoSo", ["ma ho so"]],
  ["tieuDe", ["tieu de ho so"]],
  ["thbq", ["thoi han luu tru", "thoi han bao quan"]],
  ["batDau", ["thoi gian bat dau"]],
  ["ketThuc", ["thoi gian ket thuc"]],
  ["soTo", ["so luong to"]],
  ["soTrang", ["so luong trang"]],
  ["maGocGiay", ["ma ho so goc", "ma ho so giay"]],
];
const COT_TL: [string, string[]][] = [
  ["maLuuTru", ["ma luu tru tai lieu"]],
  ["maDinhDanh", ["ma dinh danh tai lieu"]],
  ["tepTin", ["tep tin tai lieu", "duong dan"]],
];
function timCot(folded: string[], from: number, to: number, tenCot: string[]): number {
  for (let j = from; j < to; j++) for (const t of tenCot) if (folded[j] === t || folded[j].startsWith(t)) return j;
  return -1;
}

export function docMetadataPhang(rows: unknown[][], dongTieuDe: number): MetadataPhang {
  const hdr = (rows[dongTieuDe] ?? []).map((c) => fold(text(c)));
  const iHs = hdr.indexOf("ma ho so");
  let iTl = hdr.findIndex((x) => x.startsWith("ma dinh danh tai lieu"));
  if (iTl < 0) iTl = hdr.findIndex((x) => x.startsWith("ma luu tru tai lieu"));
  const cot = new Map<string, number>();
  const mapping: [string, string][] = [];
  const nhan = (ds: [string, string[]][], from: number, to: number) => {
    for (const [k, ten] of ds) {
      const j = timCot(hdr, from, to, ten);
      if (j >= 0) {
        cot.set(k, j);
        mapping.push([k, text(rows[dongTieuDe]?.[j])]);
      }
    }
  };
  nhan(COT_PHONG, 0, iHs);
  nhan(COT_HS, iHs, iTl);
  nhan(COT_TL, iTl, hdr.length);
  const lay = (r: unknown[], k: string) => {
    const j = cot.get(k);
    return j === undefined ? "" : text(r[j]);
  };
  const tenTep = (r: unknown[]): string => {
    for (const k of ["maLuuTru", "tepTin", "maDinhDanh"]) {
      let v = lay(r, k);
      if (!v) continue;
      v = v.replace(/\\/g, "/");
      v = v.slice(v.lastIndexOf("/") + 1);
      if (!/\.pdf$/i.test(v)) v = `${v}.pdf`;
      return v.toLowerCase();
    }
    return "";
  };

  const theoMa = new Map<string, HoSoPhang>();
  const tepToiHoSo = new Map<string, string>();
  let fondCode = "";
  let fondName = "";
  let dongTaiLieu = 0;
  let dongKhongTenTep = 0;
  for (let i = dongTieuDe + 1; i < rows.length; i++) {
    const r = rows[i] ?? [];
    const ma = lay(r, "maHoSo");
    if (!ma) continue;
    dongTaiLieu += 1;
    if (!fondCode) fondCode = lay(r, "fondCode");
    if (!fondName) fondName = lay(r, "fondName");
    let hs = theoMa.get(ma);
    if (!hs) {
      hs = { maHoSo: ma, tieuDe: lay(r, "tieuDe"), thbq: lay(r, "thbq"), batDau: lay(r, "batDau"), ketThuc: lay(r, "ketThuc"), soTo: lay(r, "soTo"), soTrang: lay(r, "soTrang"), maGocGiay: lay(r, "maGocGiay"), tep: [], dong: i + 1 };
      theoMa.set(ma, hs);
    }
    const t = tenTep(r);
    if (!t) {
      dongKhongTenTep += 1;
      continue;
    }
    hs.tep.push(t);
    if (!tepToiHoSo.has(t)) tepToiHoSo.set(t, ma);
  }
  return { fondCode, fondName, hoSos: [...theoMa.values()], tepToiHoSo, dongTaiLieu, dongKhongTenTep, dongTieuDe: dongTieuDe + 1, mapping };
}

/** Đọc ma trận ô của sheet đầu (mọi ô thành chuỗi/ngày) — dùng chung cho cả hai mẫu Excel. */
export async function docLuoiExcel(buf: ArrayBuffer): Promise<{ sheet: string; rows: unknown[][] }> {
  const X = await import("xlsx");
  const wb = X.read(buf, { type: "array", cellDates: true });
  const name = wb.SheetNames[0] ?? "";
  const ws = wb.Sheets[name];
  if (!ws) return { sheet: name, rows: [] };
  const ref = ws["!ref"] ? X.utils.decode_range(ws["!ref"]) : { s: { r: 0, c: 0 }, e: { r: 0, c: 0 } };
  const rows = X.utils.sheet_to_json<unknown[]>(ws, { header: 1, raw: true, defval: "", blankrows: true, range: { s: { r: 0, c: 0 }, e: ref.e } });
  return { sheet: name, rows };
}
