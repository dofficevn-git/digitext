// Copyright (c) 2026 DigiText. All rights reserved.
// Nhập mục lục hồ sơ (Excel 7 cột: Hộp số · Hồ sơ số · Tiêu đề · Thời gian BĐ - KT · Số tờ · THBQ · Ghi chú) có
// dòng nhóm trong cột Tiêu đề: "I. …" = phòng ban, "1. …" = mảng công việc. Phân tích thuần trên máy → dựng dòng
// theo mẫu lập hồ sơ 14 cột (nhập qua /cl/ho-so/import, tải về được) + sheet "Kiểm tra".
// Quyết định 1/10/2026: chỉ nhận I./1. làm phòng ban / mảng (5/10: dòng nhóm khác = mục nhỏ trong mảng, bỏ qua);
// mã phòng ban / mảng = số thứ tự 2 chữ số. 5/10/2026 (theo DigiText 2025): số tạm = Hồ sơ số trong tệp; Hồ sơ số và
// Hộp số ghi luôn vào hồ sơ (Hộp số chỉ ghi ở dòng đầu hộp → mang xuống). Đánh số riêng cho 2 loại: vĩnh viễn / có
// thời hạn (migration 0015) — số tạm, hộp duy nhất trong từng loại. 12/10/2026 (anh Hiếu): phòng ban có hồ sơ đứng ngay
// dưới mà chưa có dòng mảng "1. …" → tự tạo mảng "Vấn đề chung" (mã 00) nhét các hồ sơ đó vào, không báo lỗi nữa.

import { V4dz } from "@/api/exportFile";
import { str, type ClRow } from "@/api/chinhly";
import { MAU_HS_COLS, MAU_HS_KEYS, mauLapHoSoDauTrang, sheetDanhMucThbq, writeXlsx, type Sheet } from "./excel";
import { cellText, KET_LUAN_LABEL, kiemVaDienThoiGian, type KetQuaThoiGian } from "./thoiGianHoSo";

export interface NhomMang {
  dong: number;
  stt: number;
  ma: string;
  ten: string;
  soHoSo: number;
  /** Mảng "Vấn đề chung" do hệ thống tự tạo cho phòng ban không có dòng mảng phía trên hồ sơ. */
  tuDong?: boolean;
}

/** Tên mảng tự tạo khi hồ sơ nằm ngay dưới phòng ban (12/10/2026). Mã 00 để không đụng số thứ tự 01, 02… trong tệp. */
export const MANG_VAN_DE_CHUNG = "Vấn đề chung";
export const MA_MANG_VAN_DE_CHUNG = "00";
/** Tên phòng ban / mảng: gộp khoảng trắng; gạch ngang có khoảng trắng ở ÍT NHẤT một bên → " - " ("Văn phòng- Thống kê"
 *  = "Văn phòng - Thống kê", 12/10/2026 hai tệp viết khác nhau sinh 2 phòng ban). Gạch liền ("Đảng-Đoàn", "2010-2012")
 *  giữ nguyên để khớp mảng đã có. */
export function chuanTenNhom(s: string): string {
  return s
    .replace(/\s+/g, " ")
    .replace(/\s+[-–]\s*|\s*[-–]\s+/g, " - ")
    .trim();
}
/** Tên mảng là "Vấn đề chung" (không phân biệt hoa thường, khoảng trắng, dấu chấm cuối). */
export function laVanDeChung(ten: string): boolean {
  return ten.normalize("NFC").toLowerCase().replace(/\s+/g, " ").replace(/[.:;]+$/, "").trim() === MANG_VAN_DE_CHUNG.toLowerCase();
}
export interface NhomPhongBan {
  dong: number;
  stt: number;
  ma: string;
  ten: string;
  mangs: NhomMang[];
  soHoSo: number;
}
export interface VanDe {
  dong: number;
  muc: "loi" | "canh_bao";
  noiDung: string;
}
export interface DongHoSo {
  dong: number;
  hopSo: string;
  hoSoSo: string;
  tieuDe: string;
  thoiGianGoc: string;
  soTo: string;
  thbqGoc: string;
  /** Tên THBQ đúng như Danh mục (null = không khớp). */
  thbq: string | null;
  /** Loại bảo quản: true vĩnh viễn · false có thời hạn · null chưa biết (THBQ không khớp). */
  vinhVien: boolean | null;
  ghiChu: string;
  phongBan: NhomPhongBan | null;
  mang: NhomMang | null;
  tg: KetQuaThoiGian;
  /** Lỗi chặn nhập của riêng dòng này. */
  loi: string[];
}
export type CotMap = Partial<Record<"hopSo" | "hoSoSo" | "tieuDe" | "thoiGian" | "ngayBatDau" | "ngayKetThuc" | "soTo" | "thbq" | "ghiChu", number>>;

export interface PhanTichMucLuc {
  tenTep: string;
  sheet: string;
  tenPhongTrongTep: string;
  dongTieuDeCot: number;
  cot: CotMap;
  phongBans: NhomPhongBan[];
  rows: DongHoSo[];
  /** Lỗi + cảnh báo cấu trúc (dòng nhóm, thiếu cột…), không gồm ghi chú thời gian từng dòng. */
  vanDe: VanDe[];
  /** THBQ phổ biến nhất trong tệp. */
  thbqTep: string;
  /** Số hộp khác nhau (theo loại). */
  soHop: number;
  soLoi: number;
  soCanhBao: number;
  soDaDien: number;
  /** Nội dung tệp gốc — để xuất lại chính tệp đó đã điền + tô màu + ghi chú (như DienThoiGian / CheckThoiGian). */
  buf?: ArrayBuffer;
}

// ------------------------------------------------------------------ tiện ích

function boDau(s: string): string {
  return s
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/đ/g, "d")
    .replace(/\s+/g, " ")
    .trim();
}

const ROMAN: Record<string, number> = { I: 1, V: 5, X: 10, L: 50, C: 100, D: 500, M: 1000 };
export function romanToInt(s: string): number {
  let n = 0;
  const u = s.toUpperCase();
  for (let i = 0; i < u.length; i++) {
    const a = ROMAN[u[i]] ?? 0;
    const b = ROMAN[u[i + 1]] ?? 0;
    n += a < b ? -a : a;
  }
  return n;
}

const CHUAN_THBQ: Record<string, string> = { vv: "Vĩnh viễn", "vinh vien": "Vĩnh viễn", "vĩnh viễn": "Vĩnh viễn" };

/** Khớp tên THBQ trong tệp với Danh mục: bỏ dấu cách thừa, không phân biệt hoa thường; "20" → "20 năm"; "vv" → Vĩnh viễn. */
export function khopThbq(ten: string, danhMuc: string[]): string | null {
  const t = ten.trim();
  if (!t) return null;
  const k = boDau(t);
  const ung = [t, CHUAN_THBQ[k] ?? "", /^\d+$/.test(k) ? `${k} năm` : "", k.replace(/^0+(\d)/, "$1")].filter(Boolean);
  for (const u of ung) {
    const hit = danhMuc.find((d) => boDau(d) === boDau(u));
    if (hit) return hit;
  }
  return null;
}

/** Số chuẩn: "001" → "1" (số thuần); "12A" giữ nguyên. Dùng cho Hồ sơ số / số tạm. */
export function soChuan(s: string): string {
  const t = s.trim();
  return /^\d+$/.test(t) ? String(+t) : t;
}

export const TEN_LOAI = (vv: boolean | null) => (vv === null ? "chưa rõ loại" : vv ? "vĩnh viễn" : "có thời hạn");

const NHAN_COT: [keyof CotMap, string[]][] = [
  ["hopSo", ["hop so", "so hop"]],
  ["hoSoSo", ["ho so so", "so ho so", "hs so", "dvbq", "so chinh thuc", "so tam", "ma ho so"]],
  ["tieuDe", ["tieu de", "ten ho so", "noi dung", "trich yeu"]],
  ["ngayBatDau", ["ngay bat dau", "bat dau", "ngay bd", "tu ngay"]],
  ["ngayKetThuc", ["ngay ket thuc", "ket thuc", "ngay kt", "den ngay"]],
  ["thoiGian", ["thoi gian", "ngay thang"]],
  ["soTo", ["so to", "so trang"]],
  ["thbq", ["thbq", "thoi han", "bao quan"]],
  ["ghiChu", ["ghi chu"]],
];

/** Tìm dòng tiêu đề cột trong 15 dòng đầu: phải có Hồ sơ số + Tiêu đề + Thời gian (hoặc Ngày bắt đầu). */
export function timTieuDeCot(rows: unknown[][]): { dong: number; cot: CotMap } | null {
  for (let i = 0; i < Math.min(rows.length, 15); i++) {
    const cells = (rows[i] ?? []).map((c) => boDau(cellText(c)));
    const cot: CotMap = {};
    const dung = new Set<number>();
    for (const [k, als] of NHAN_COT) {
      const j = cells.findIndex((c, idx) => !dung.has(idx) && c.length > 0 && als.some((a) => c.includes(a)));
      if (j >= 0) {
        cot[k] = j;
        dung.add(j);
      }
    }
    if (cot.hoSoSo !== undefined && cot.tieuDe !== undefined && (cot.thoiGian !== undefined || cot.ngayBatDau !== undefined)) return { dong: i + 1, cot };
  }
  return null;
}

const RE_PHONG_BAN = /^\s*([IVXLCDM]+)\s*[.\-)]\s*(.+)$/;
const RE_MANG = /^\s*(\d+)\s*[.\-)]\s*(.+)$/;
const RE_DONG_DANH_SO = /^\(?\s*-?\d{1,2}\s*\)?$/;

// ------------------------------------------------------------------ phân tích (thuần)

/** `rows[i]` = dòng Excel i+1 (giữ cả dòng trống để số dòng khớp tệp). */
export function phanTichMucLuc(rows: unknown[][], opts: { tenTep: string; sheet: string; thbqDanhMuc: string[]; thbqVinhVien?: string[] }): PhanTichMucLuc {
  const vanDe: VanDe[] = [];
  const out: PhanTichMucLuc = {
    tenTep: opts.tenTep,
    sheet: opts.sheet,
    tenPhongTrongTep: "",
    dongTieuDeCot: 0,
    cot: {},
    phongBans: [],
    rows: [],
    vanDe,
    thbqTep: "",
    soHop: 0,
    soLoi: 0,
    soCanhBao: 0,
    soDaDien: 0,
  };
  const hd = timTieuDeCot(rows);
  if (!hd) {
    vanDe.push({ dong: 0, muc: "loi", noiDung: "Không tìm thấy dòng tiêu đề cột (cần có Hồ sơ số, Tiêu đề hồ sơ, Thời gian BĐ - KT trong 15 dòng đầu)" });
    out.soLoi = 1;
    return out;
  }
  out.dongTieuDeCot = hd.dong;
  out.cot = hd.cot;
  for (let i = 0; i < hd.dong - 1; i++) {
    for (const c of rows[i] ?? []) {
      const m = /ph[oô]ng\s*[:：]\s*(.+)/i.exec(cellText(c).replace(/\s+/g, " "));
      if (m) out.tenPhongTrongTep = m[1].trim();
    }
  }
  const lay = (r: unknown[], k: keyof CotMap, ngay = false): string => {
    const j = hd.cot[k];
    return j === undefined ? "" : cellText(r[j], ngay);
  };

  let pbHienTai: NhomPhongBan | null = null;
  let mangHienTai: NhomMang | null = null;
  let hopHienTai = "";
  const daCoHoSoSo = new Map<string, number>(); // `${loại}|${hồ sơ số}` → dòng
  const demThbq = new Map<string, number>();
  const tenVv = new Set((opts.thbqVinhVien ?? ["Vĩnh viễn"]).map((t) => t.toLowerCase()));

  for (let i = hd.dong; i < rows.length; i++) {
    const r = rows[i] ?? [];
    const dong = i + 1;
    const nonEmpty = r.map((c) => cellText(c)).filter(Boolean);
    if (nonEmpty.length === 0) continue;
    if (i < hd.dong + 2 && nonEmpty.every((c) => RE_DONG_DANH_SO.test(c))) continue; // dòng (1) (2) (3)…
    const hoSoSo = soChuan(lay(r, "hoSoSo"));
    const tieuDe = lay(r, "tieuDe").replace(/\s+/g, " ").trim();
    // Hộp số chỉ ghi ở dòng đầu hộp → mang xuống các dòng sau tới khi gặp số hộp mới.
    const hopO = lay(r, "hopSo").trim();
    if (hopO) hopHienTai = hopO;

    if (!hoSoSo) {
      if (!tieuDe) continue; // chỉ có hộp số / ghi chú lẻ → bỏ
      let m = RE_PHONG_BAN.exec(tieuDe);
      if (m) {
        const stt = romanToInt(m[1]);
        pbHienTai = { dong, stt, ma: String(stt).padStart(2, "0"), ten: chuanTenNhom(m[2]), mangs: [], soHoSo: 0 };
        mangHienTai = null;
        if (out.phongBans.some((p) => p.ma === pbHienTai!.ma)) vanDe.push({ dong, muc: "loi", noiDung: `Phòng ban ${m[1]}. lặp lại (đã có ở dòng ${out.phongBans.find((p) => p.ma === pbHienTai!.ma)!.dong})` });
        out.phongBans.push(pbHienTai);
        continue;
      }
      m = RE_MANG.exec(tieuDe);
      if (m) {
        if (!pbHienTai) {
          vanDe.push({ dong, muc: "loi", noiDung: `Mảng "${tieuDe}" đứng trước dòng phòng ban (I. …) — thêm dòng phòng ban phía trên` });
          continue;
        }
        const stt = +m[1];
        mangHienTai = { dong, stt, ma: String(stt).padStart(2, "0"), ten: chuanTenNhom(m[2]), soHoSo: 0 };
        if (pbHienTai.mangs.some((x) => x.ma === mangHienTai!.ma)) vanDe.push({ dong, muc: "loi", noiDung: `Mảng ${m[1]}. lặp lại trong phòng ban ${pbHienTai.ten}` });
        pbHienTai.mangs.push(mangHienTai);
        continue;
      }
      // Dòng nhóm không có I./1. = mục nhỏ trong mảng công việc (vd. "Sổ chứng thực"). Anh Hiếu 5/10/2026: tạm
      // thời bỏ qua khi nhập — không báo lỗi, hồ sơ phía dưới vẫn thuộc mảng hiện tại.
      continue;
    }

    const loi: string[] = [];
    if (!tieuDe) loi.push("Thiếu tiêu đề hồ sơ");
    if (!pbHienTai) loi.push("Chưa có dòng phòng ban (I. …) phía trên");
    else if (!mangHienTai) {
      // Hồ sơ đứng ngay dưới phòng ban, chưa có dòng mảng → gom vào mảng "Vấn đề chung" tự tạo (một mảng / phòng ban).
      mangHienTai = pbHienTai.mangs.find((x) => x.tuDong) ?? null;
      if (!mangHienTai) {
        mangHienTai = { dong, stt: 0, ma: MA_MANG_VAN_DE_CHUNG, ten: MANG_VAN_DE_CHUNG, soHoSo: 0, tuDong: true };
        pbHienTai.mangs.push(mangHienTai);
      }
    }
    const thbqGoc = lay(r, "thbq");
    const thbq = khopThbq(thbqGoc, opts.thbqDanhMuc);
    if (!thbqGoc) loi.push("Thiếu thời hạn bảo quản");
    else if (!thbq) loi.push(`Thời hạn bảo quản "${thbqGoc}" không có trong Danh mục`);
    if (thbq) demThbq.set(thbq, (demThbq.get(thbq) ?? 0) + 1);
    const vinhVien = thbq ? tenVv.has(thbq.toLowerCase()) : null;
    // Hồ sơ số (= số tạm) duy nhất trong từng loại: mục lục vĩnh viễn và có thời hạn đều đánh từ 1.
    const khoa = `${vinhVien}|${hoSoSo.toLowerCase()}`;
    const truoc = daCoHoSoSo.get(khoa);
    if (truoc !== undefined) loi.push(`Hồ sơ số ${hoSoSo} trùng với dòng ${truoc} (cùng loại ${TEN_LOAI(vinhVien)})`);
    else daCoHoSoSo.set(khoa, dong);
    if (hd.cot.hopSo !== undefined && !hopHienTai) loi.push("Chưa có Hộp số (dòng đầu mỗi hộp phải ghi số hộp)");

    const thoiGianGoc = hd.cot.thoiGian !== undefined ? lay(r, "thoiGian", true) : [lay(r, "ngayBatDau", true), lay(r, "ngayKetThuc", true)].filter(Boolean).join(" - ");
    const tg = kiemVaDienThoiGian(thoiGianGoc, tieuDe);
    // Không xác định được thời gian (ô trống + tiêu đề không ghi năm, hoặc sai định dạng không đọc được) → lỗi chặn
    // nhập; trước 12/10/2026 chỉ là cảnh báo nên hồ sơ vào hệ thống với thời gian trống (anh Hiếu hỏi).
    if (!tg.batDau || !tg.ketThuc) loi.push(thoiGianGoc.trim() ? `Thời gian "${thoiGianGoc.trim()}" không đọc được — ghi dd/mm/yyyy - dd/mm/yyyy` : "Thiếu thời gian bắt đầu - kết thúc (ô trống, tiêu đề không ghi năm)");
    if (pbHienTai) pbHienTai.soHoSo++;
    if (mangHienTai) mangHienTai.soHoSo++;
    // Số tờ chỉ nhận số tự nhiên (anh Hiếu 5/10/2026): "01 quyển", "3,5", "12 tờ"… là lỗi chặn nhập, tô đỏ ở tệp đánh dấu.
    const soTo = lay(r, "soTo").trim();
    if (soTo && !/^\d+$/.test(soTo)) loi.push(`Số tờ "${soTo}" không hợp lệ — chỉ ghi số tự nhiên (vd. 12), bỏ chữ như "quyển", "tờ"`);
    else if (soTo && +soTo === 0) loi.push("Số tờ phải lớn hơn 0");
    out.rows.push({ dong, hopSo: hopHienTai, hoSoSo, tieuDe, thoiGianGoc, soTo, thbqGoc, thbq, vinhVien, ghiChu: lay(r, "ghiChu"), phongBan: pbHienTai, mang: mangHienTai, tg, loi });
  }

  // Hộp thuộc đúng một loại: hộp có cả hồ sơ vĩnh viễn lẫn có thời hạn → lỗi ở mọi dòng của hộp đó.
  const loaiHop = new Map<string, Set<boolean>>();
  for (const r of out.rows) if (r.hopSo && r.vinhVien !== null) (loaiHop.get(r.hopSo) ?? loaiHop.set(r.hopSo, new Set()).get(r.hopSo)!).add(r.vinhVien);
  for (const r of out.rows) {
    if (r.hopSo && (loaiHop.get(r.hopSo)?.size ?? 0) > 1) r.loi.push(`Hộp ${r.hopSo} có cả hồ sơ vĩnh viễn lẫn có thời hạn — mỗi hộp chỉ một loại`);
  }
  out.soHop = new Set(out.rows.filter((r) => r.hopSo).map((r) => `${r.vinhVien}|${r.hopSo}`)).size;

  // Phòng ban đã có dòng mảng "Vấn đề chung" trong tệp (kể cả đứng SAU hồ sơ, vd. "1. Vấn đề chung") → hồ sơ đứng ngay
  // dưới phòng ban vào luôn mảng đó, không tạo thêm mảng 00 (anh Hiếu 12/10/2026).
  for (const pb of out.phongBans) {
    const tu = pb.mangs.find((m) => m.tuDong);
    const co = pb.mangs.find((m) => !m.tuDong && laVanDeChung(m.ten));
    if (!tu || !co) continue;
    for (const r of out.rows) if (r.mang === tu) r.mang = co;
    co.soHoSo += tu.soHoSo;
    pb.mangs = pb.mangs.filter((m) => m !== tu);
  }

  if (out.rows.length === 0 && !vanDe.some((v) => v.muc === "loi")) vanDe.push({ dong: 0, muc: "loi", noiDung: "Tệp không có dòng hồ sơ nào (cột Hồ sơ số trống cả)" });
  let max = 0;
  for (const [k, v] of demThbq) {
    if (v > max) {
      max = v;
      out.thbqTep = k;
    }
  }
  out.soLoi = vanDe.filter((v) => v.muc === "loi").length + out.rows.filter((r) => r.loi.length > 0).length;
  out.soCanhBao = out.rows.filter((r) => r.tg.ketLuan === "canh_bao" || r.tg.ketLuan === "loi").length;
  out.soDaDien = out.rows.filter((r) => r.tg.ketLuan === "da_dien").length;
  return out;
}

// ------------------------------------------------------------------ đọc tệp

export async function docMucLucTuBuffer(buf: ArrayBuffer, tenTep: string, thbqDanhMuc: string[], thbqVinhVien?: string[]): Promise<PhanTichMucLuc> {
  const X = await import("xlsx");
  const wb = X.read(buf, { type: "array", cellDates: true });
  const name = wb.SheetNames[0] ?? "";
  const ws = wb.Sheets[name];
  if (!ws) return phanTichMucLuc([], { tenTep, sheet: name, thbqDanhMuc, thbqVinhVien });
  const ref = ws["!ref"] ? X.utils.decode_range(ws["!ref"]) : { s: { r: 0, c: 0 }, e: { r: 0, c: 0 } };
  const range = { s: { r: 0, c: 0 }, e: ref.e };
  const rows = X.utils.sheet_to_json<unknown[]>(ws, { header: 1, raw: true, defval: "", blankrows: true, range });
  const pt = phanTichMucLuc(rows, { tenTep, sheet: name, thbqDanhMuc, thbqVinhVien });
  pt.buf = buf;
  return pt;
}

export async function docMucLucHoSo(file: File, thbq: ClRow[]): Promise<PhanTichMucLuc> {
  // Loại vĩnh viễn = THBQ trang_thai 1 trong Danh mục.
  const vv = thbq.filter((t) => Number(t.trang_thai) === 1).map((t) => str(t, "ten"));
  return docMucLucTuBuffer(await file.arrayBuffer(), file.name, thbq.map((t) => str(t, "ten")), vv.length ? vv : undefined);
}

// ------------------------------------------------------------------ số tạm + dòng mẫu

/** Số tạm của từng dòng = Hồ sơ số trong tệp (như DigiText 2025: số tạm là số người lập ghi; mục lục đã có số thì dùng luôn). */
export function soTamCua(pt: PhanTichMucLuc): Map<number, string> {
  return new Map(pt.rows.map((r) => [r.dong, r.hoSoSo]));
}

/** Dòng gửi máy chủ / ghi vào mẫu, đúng khoá MAU_HS_KEYS + tên phòng ban / mảng để máy chủ tự tạo khi thiếu. */
export function dongMau(r: DongHoSo, soTam: string): Record<string, unknown> {
  const o: Record<string, unknown> = {
    ma_don_vi: r.phongBan?.ma ?? "",
    ma_mang: r.mang?.ma ?? "",
    ten_phong_ban: r.phongBan?.ten ?? "",
    ten_mang: r.mang?.ten ?? "",
    so_tam: soTam,
    ho_so_so: r.hoSoSo,
    hop_so: r.hopSo,
    tieu_de: r.tieuDe,
    ngay_bat_dau: r.tg.batDau,
    ngay_ket_thuc: r.tg.ketThuc,
    thbq: r.thbq ?? r.thbqGoc,
    so_to: r.soTo,
    ghi_chu: r.ghiChu,
    che_do_su_dung: "01",
    ngon_ngu: "01",
    tinh_trang_vat_ly: "02",
  };
  for (const k of Object.keys(o)) if (o[k] === "") delete o[k];
  return o;
}

export function coLoiChan(pt: PhanTichMucLuc): boolean {
  return pt.vanDe.some((v) => v.muc === "loi") || pt.rows.some((r) => r.loi.length > 0);
}

const COT_KIEM_TRA = ["Dòng gốc", "Loại", "Hộp số", "Hồ sơ số", "Số tạm", "Phòng ban", "Mảng", "Tiêu đề", "Thời gian gốc", "Bắt đầu", "Kết thúc", "THBQ", "Kết luận", "Ghi chú kiểm tra"];

function dongKiemTra(pt: PhanTichMucLuc, soTam: Map<number, string>): (string | number)[][] {
  const out: (string | number)[][] = pt.rows.map((r) => [
    r.dong,
    TEN_LOAI(r.vinhVien),
    r.hopSo,
    r.hoSoSo,
    soTam.get(r.dong) ?? "",
    r.phongBan ? `${r.phongBan.ma} · ${r.phongBan.ten}` : "",
    r.mang ? `${r.mang.ma} · ${r.mang.ten}${r.mang.tuDong ? " (tự tạo)" : ""}` : "",
    r.tieuDe,
    r.thoiGianGoc.replace(/\s*\n\s*/g, " "),
    r.tg.batDau,
    r.tg.ketThuc,
    r.thbq ?? r.thbqGoc,
    r.loi.length ? "Lỗi" : KET_LUAN_LABEL[r.tg.ketLuan],
    [...r.loi, ...r.tg.ghiChu].join("; "),
  ]);
  for (const v of pt.vanDe) out.push([v.dong || "", "", "", "", "", "", "", "", "", "", "", "", v.muc === "loi" ? "Lỗi" : "Cảnh báo", v.noiDung]);
  return out.sort((a, b) => Number(a[0] || 0) - Number(b[0] || 0));
}

/** Tệp đã chuyển: sheet 1 đúng mẫu lập hồ sơ 14 cột (đọc lại được bằng "Nhập hồ sơ từ Excel"), sheet 2 "Kiểm tra", sheet 3 danh mục THBQ. */
export async function xuatTepDaChuyen(pt: PhanTichMucLuc, maPhong: string, thbqs: ClRow[]): Promise<void> {
  const soTam = soTamCua(pt);
  const dau = mauLapHoSoDauTrang(maPhong);
  const aoa = [...dau.aoa];
  for (const r of pt.rows) {
    const o = dongMau(r, soTam.get(r.dong) ?? "");
    aoa.push(MAU_HS_KEYS.map((k) => (o[k] === undefined ? "" : String(o[k]))));
  }
  const kt: Sheet = { name: "Kiểm tra", aoa: [COT_KIEM_TRA, ...dongKiemTra(pt, soTam)], cols: [{ wch: 8 }, { wch: 12 }, { wch: 7 }, { wch: 8 }, { wch: 10 }, { wch: 22 }, { wch: 22 }, { wch: 50 }, { wch: 22 }, { wch: 11 }, { wch: 11 }, { wch: 11 }, { wch: 10 }, { wch: 60 }] };
  const goc = pt.tenTep.replace(/\.[^.]+$/, "").replace(/[^\p{L}\p{N}]+/gu, "-");
  await writeXlsx([{ name: "Lập hồ sơ điện tử", aoa, cols: dau.cols, merges: dau.merges }, kt, sheetDanhMucThbq(thbqs)], `mau-lap-ho-so-${maPhong}-${goc}.xlsx`);
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
 * Xuất lại CHÍNH tệp gốc (giữ nguyên bố cục, định dạng) với cột thời gian đã điền "dd/mm/yyyy dd/mm/yyyy",
 * ô được sửa tô màu + ghi chú (comment) — như `<tên>_DADIEN.xlsx` của DienThoiGian. Ô lỗi chặn nhập
 * (cột Tiêu đề / THBQ / Hồ sơ số) cũng tô + ghi chú để sửa thẳng trên tệp rồi chọn lại.
 */
export async function xuatTepDanhDau(pt: PhanTichMucLuc): Promise<ThongKeDanhDau> {
  const { bytes, tk } = await danhDauTepGoc(pt);
  await V4dz(bytes, `${pt.tenTep.replace(/\.[^.]+$/, "")}_DADIEN.xlsx`, "xlsx");
  return tk;
}

export async function danhDauTepGoc(pt: PhanTichMucLuc): Promise<{ bytes: Uint8Array; tk: ThongKeDanhDau }> {
  if (!pt.buf) throw new Error("Không còn nội dung tệp gốc — chọn lại tệp.");
  if (!/\.xlsx$|\.xlsm$/i.test(pt.tenTep)) throw new Error("Chỉ đánh dấu được tệp .xlsx. Mở tệp .xls bằng Excel, lưu lại dạng .xlsx rồi chọn lại.");
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
      danhDau(r.dong, cotTg, MAU_DANH_DAU.loi, `LỖI — không điền được thời gian\nGiá trị: ${goc || "(trống)"}\nLý do: ${lyDo || "không đọc được năm"}`);
    } else if (ketLuan === "canh_bao") {
      tk.canhBao++;
      danhDau(r.dong, cotTg, MAU_DANH_DAU.canhBao, `Cần xem lại\n${doi ? `Cũ : ${goc || "(trống)"}\nMới: ${moi}\n` : ""}Lý do: ${lyDo}`);
    } else if (doi) {
      tk.daDien++;
      danhDau(r.dong, cotTg, MAU_DANH_DAU.daDien, `Đã sửa\nCũ : ${goc || "(trống)"}\nMới: ${moi}${lyDo ? `\nLý do: ${lyDo}` : ""}`);
    }
    // Mọi hồ sơ có lỗi đều tô đỏ — kể cả cả đoạn hồ sơ không thuộc mảng / phòng ban nào (anh Hiếu 2/10).
    for (const loi of r.loi) {
      tk.loi++;
      const cot = /^Số tờ/.test(loi) ? pt.cot.soTo : /THBQ|bảo quản/i.test(loi) ? pt.cot.thbq : /Hồ sơ số/i.test(loi) ? pt.cot.hoSoSo : /Hộp/.test(loi) ? (pt.cot.hopSo ?? pt.cot.tieuDe) : pt.cot.tieuDe;
      danhDau(r.dong, cot, MAU_DANH_DAU.loi, `LỖI: ${loi}`);
    }
  }
  for (const v of pt.vanDe) {
    if (v.muc !== "loi" || !v.dong) continue;
    tk.loi++;
    danhDau(v.dong, pt.cot.tieuDe, MAU_DANH_DAU.loi, `LỖI: ${v.noiDung}`);
  }

  return { bytes: new Uint8Array(await wb.xlsx.writeBuffer()), tk };
}

export { MAU_HS_COLS };
