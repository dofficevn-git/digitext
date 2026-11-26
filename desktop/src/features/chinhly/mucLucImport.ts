// Copyright (c) 2026 DigiText. All rights reserved.
//
// ⚠️ TỆP NÀY KHÔNG CÒN CHẠY TRONG APP (từ 10/11/2026).
//
// Bộ đọc mục lục hồ sơ đã chuyển sang máy chủ: crate `digitext-mucluc`, cửa `POST /cl/muc-luc/doc`,
// màn hình gọi qua `mucLucMayChu.ts`. Lý do: cổng nhà cung cấp của hệ thống ngoài cũng cần đọc mục
// lục, hai bản luật cùng chạy là đúng căn bệnh "luật hai nơi" (bộ quyền ở `permission.rs` và
// `permissions.ts`) — luật MLHS đã sửa 4 lần trong 5 tuần, bản nào cũ đi là bản đó báo "nhập được"
// cho tệp sắp bị từ chối.
//
// Tệp ở lại làm **BẢN ĐỐI CHỨNG** cho bản Rust, và chỉ `scripts/kiem-muc-luc.ts` dùng:
//
//     cd desktop && npm run kiem:mucluc -- D:/DigiText/test/MLHS      # bản TS (chuẩn)
//     cargo run -p digitext-mucluc --bin mucluc-do -- D:/DigiText/test/MLHS
//
// Hai bên phải ra y hệt trên 23 mục lục thật của Bắc Giang (`test/MLHS`). Mốc 10/11/2026:
// 23 tệp · 10171 hồ sơ · 10 tệp bị chặn · 126 lỗi · 1725 cảnh báo, và 10171/10171 dòng khớp từng dòng.
//
// ⇒ **Đừng import tệp này từ mã app**, và **đừng sửa nó để "theo" bản Rust**: nó là cái thước. Luật
// đổi thì sửa bản Rust trước, chạy đối chứng, rồi mới sửa thước ở đây cho cùng luật mới.
// Phần kết xuất Excel (tệp đã chuyển, tệp đánh dấu) đã dời sang `mucLucXuat.ts`.
//
// Nhập mục lục hồ sơ (Excel 7 cột: Hộp số · Hồ sơ số · Tiêu đề · Thời gian BĐ - KT · Số tờ · THBQ · Ghi chú) có
// dòng nhóm trong cột Tiêu đề: "I. …" = phòng ban, "1. …" = mảng công việc. Phân tích thuần trên máy → dựng dòng
// theo mẫu lập hồ sơ 14 cột (nhập qua /cl/ho-so/import, tải về được) + sheet "Kiểm tra".
// Quyết định 1/10/2026: chỉ nhận I./1. làm phòng ban / mảng (5/10: dòng nhóm khác = mục nhỏ trong mảng, bỏ qua);
// mã phòng ban / mảng = số thứ tự 2 chữ số. 5/10/2026 (theo DigiText 2025): số tạm = Hồ sơ số trong tệp; Hồ sơ số và
// Hộp số ghi luôn vào hồ sơ (Hộp số chỉ ghi ở dòng đầu hộp → mang xuống). Đánh số riêng cho 2 loại: vĩnh viễn / có
// thời hạn (migration 0015) — số tạm, hộp duy nhất trong từng loại. 12/10/2026 (anh Hiếu): phòng ban có hồ sơ đứng ngay
// dưới mà chưa có dòng mảng "1. …" → tự tạo mảng "Vấn đề chung" (mã 00) nhét các hồ sơ đó vào, không báo lỗi nữa.
// 2/11/2026 (23 mục lục thật của Bắc Giang, test/MLHS): tên cột so sau khi bỏ dấu + bỏ mọi khoảng trắng ("TH\nBQ",
// "BĐ"/"KT" tách cột, "SỐ" = số tờ, "Hồ sơ mới", "Cặp số", "Trích yếu"); thời gian không bắt buộc có cột (suy từ
// tiêu đề); tự chọn sheet có dòng tiêu đề (sheet đầu thường là bìa); tệp không có dòng "I." → CHẶN (26/11, trước đó tự tạo "Chung");
// số tờ không phải số / thiếu số tờ / hộp lẫn hai loại chỉ là CẢNH BÁO (vẫn nhập); THBQ trống → THBQ mặc định chọn
// cho cả tệp. Vẫn chặn: hồ sơ số trùng cùng loại, thiếu tiêu đề, thời gian không suy được (12/10).

import { str, type ClRow } from "@/api/chinhly";
import { MAU_HS_COLS } from "./excel";
import { cellText, kiemVaDienThoiGian, type KetQuaThoiGian } from "./thoiGianHoSo";

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
/** THBQ (tên trong Danh mục) từ 20 năm trở lên: "20 năm", "50 năm", "70 năm"… Không có số năm ("Đánh giá lại",
 *  "Theo tuổi thọ công trình") → false; vĩnh viễn xét riêng theo `vinhVien`. */
export function tuHaiMuoiNam(thbq: string): boolean {
  const m = /(\d+)\s*năm/i.exec(thbq.normalize("NFC"));
  return !!m && Number(m[1]) >= 20;
}
/** Tên mảng là "Vấn đề chung" (không phân biệt hoa thường, khoảng trắng, dấu chấm cuối). */
export function laVanDeChung(ten: string): boolean {
  return ten.normalize("NFC").toLowerCase().replace(/\s+/g, " ").replace(/[.:;]+$/, "").trim() === MANG_VAN_DE_CHUNG.toLowerCase();
}
/** Phòng ban tự tạo khi tệp không có dòng "I. …" nào (mục lục không chia phòng ban, 2/11/2026). */
export const MA_PHONG_BAN_CHUNG = "00";
export const PHONG_BAN_CHUNG = "Chung";
export interface NhomPhongBan {
  dong: number;
  stt: number;
  ma: string;
  ten: string;
  mangs: NhomMang[];
  soHoSo: number;
  /** Phòng ban "Chung" do hệ thống tự tạo vì tệp không chia phòng ban. */
  tuDong?: boolean;
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
  /** Cảnh báo của dòng (vẫn nhập): số tờ không phải số, thiếu số tờ, hộp lẫn hai loại, THBQ lấy mặc định. */
  canhBao: string[];
}
export type CotMap = Partial<Record<"hopSo" | "hoSoSo" | "tieuDe" | "thoiGian" | "ngayBatDau" | "ngayKetThuc" | "soTo" | "thbq" | "ghiChu", number>>;

export interface PhanTichMucLuc {
  tenTep: string;
  sheet: string;
  /** Mọi sheet trong tệp — hộp thoại cho đổi sheet rồi phân tích lại từ `buf`. */
  sheets: string[];
  /** Tệp không có dòng "I. …" → hồ sơ gom vào phòng ban "Chung" tự tạo. */
  khongChiaPhongBan: boolean;
  /** THBQ mặc định đã dùng cho các ô THBQ trống (tên trong Danh mục), rỗng = không dùng. */
  thbqMacDinh: string;
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

/** Khớp tên THBQ trong tệp với Danh mục: bỏ dấu cách thừa, không phân biệt hoa thường; "20" → "20 năm"; "vv"/"vv." → Vĩnh viễn. */
export function khopThbq(ten: string, danhMuc: string[]): string | null {
  const t = ten.trim().replace(/[.:;]+$/, "").trim();
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

/** Tên cột để so: bỏ dấu, bỏ MỌI khoảng trắng / dấu câu ("TH\nBQ" → "thbq", "Số\n hộp" → "sohop", "N.BĐ" → "nbd"). */
export function chuanTenCot(s: string): string {
  return boDau(s).replace(/[^a-z0-9]+/g, "");
}

/**
 * Bí danh tên cột (đã nén như `chuanTenCot`). `chua` = cột chứa chuỗi; `bang` = cột phải đúng bằng (tên ngắn như
 * "BĐ", "KT", "SỐ", "Năm" — nếu so "chứa" thì "SỐ" bắt nhầm "Số hồ sơ"). Thứ tự quan trọng: Hộp / Hồ sơ số nhận trước
 * để "Số hộp", "Số hồ sơ" không rơi vào Số tờ. "STT" / "TT" không bao giờ gán.
 */
const NHAN_COT: { k: keyof CotMap; chua: string[]; bang?: string[] }[] = [
  { k: "hopSo", chua: ["hopso", "sohop", "capso", "hopmoi"] },
  { k: "hoSoSo", chua: ["hososo", "sohoso", "hsso", "sohs", "dvbq", "sochinhthuc", "sotam", "mahoso", "hosomoi", "hsmoi"] },
  { k: "tieuDe", chua: ["tieude", "tenhoso", "noidung", "trichyeu", "nhomhoso"] },
  { k: "ngayBatDau", chua: ["ngaybatdau", "batdau", "ngaybd", "tungay", "nbd"], bang: ["bd"] },
  { k: "ngayKetThuc", chua: ["ngayketthuc", "ketthuc", "ngaykt", "denngay", "nkt"], bang: ["kt"] },
  // "Ngày hồ sơ (BĐ-KT)" (mục lục Đảng uỷ Kim Lập, 26/11/2026): trước đây không nhận → cả cột bị bỏ, năm chỉ suy từ tiêu đề.
  { k: "thoiGian", chua: ["thoigian", "ngaythang", "ngayhoso", "bdkt"], bang: ["nam"] },
  { k: "soTo", chua: ["soto", "sotrang", "soluongto"], bang: ["so"] },
  { k: "thbq", chua: ["thbq", "thoihan", "baoquan"] },
  { k: "ghiChu", chua: ["ghichu"] },
];

/** Tìm dòng tiêu đề cột trong 15 dòng đầu: phải có Hồ sơ số + Tiêu đề. Thời gian không bắt buộc (suy từ tiêu đề). */
export function timTieuDeCot(rows: unknown[][]): { dong: number; cot: CotMap } | null {
  for (let i = 0; i < Math.min(rows.length, 15); i++) {
    const cells = (rows[i] ?? []).map((c) => chuanTenCot(cellText(c)));
    const cot: CotMap = {};
    const dung = new Set<number>();
    for (const { k, chua, bang } of NHAN_COT) {
      const j = cells.findIndex((c, idx) => !dung.has(idx) && c.length > 0 && (chua.some((a) => c.includes(a)) || (bang?.includes(c) ?? false)));
      if (j >= 0) {
        cot[k] = j;
        dung.add(j);
      }
    }
    if (cot.hoSoSo !== undefined && cot.tieuDe !== undefined) return { dong: i + 1, cot };
  }
  return null;
}

const RE_PHONG_BAN = /^\s*([IVXLCDM]+)\s*[.\-)]\s*(.+)$/;
const RE_MANG = /^\s*(\d+)\s*[.\-)]\s*(.+)$/;
/** Mục nhỏ trong mảng: "1.1 Văn bản đi", "2.3.1 …" — anh Hiếu 26/10/2026: coi là một phần của mảng lớn phía trên,
 *  không phải mảng mới (trước đây RE_MANG bắt "1.1 …" thành mảng 1 → báo "Mảng 1. lặp lại"). */
const RE_MANG_CON = /^\s*\d+(?:\s*\.\s*\d+)+\s*[.\-)]?\s*\S/;
const RE_DONG_DANH_SO = /^\(?\s*-?\d{1,2}\s*\)?$/;

// ------------------------------------------------------------------ phân tích (thuần)

/** `rows[i]` = dòng Excel i+1 (giữ cả dòng trống để số dòng khớp tệp). */
export interface TuyChonMucLuc {
  tenTep: string;
  sheet: string;
  sheets?: string[];
  thbqDanhMuc: string[];
  thbqVinhVien?: string[];
  /** Tên THBQ (trong Danh mục) dùng cho ô THBQ trống; rỗng/không có → ô trống là lỗi dòng. */
  thbqMacDinh?: string;
}

export function phanTichMucLuc(rows: unknown[][], opts: TuyChonMucLuc): PhanTichMucLuc {
  const vanDe: VanDe[] = [];
  const thbqMacDinh = opts.thbqMacDinh ? khopThbq(opts.thbqMacDinh, opts.thbqDanhMuc) : null;
  const out: PhanTichMucLuc = {
    tenTep: opts.tenTep,
    sheet: opts.sheet,
    sheets: opts.sheets ?? [opts.sheet],
    khongChiaPhongBan: false,
    thbqMacDinh: thbqMacDinh ?? "",
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
    vanDe.push({ dong: 0, muc: "loi", noiDung: "Không tìm thấy dòng tiêu đề cột (cần có cột Hồ sơ số và Tiêu đề hồ sơ trong 15 dòng đầu của sheet — thử chọn sheet khác)" });
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

  // 26/11/2026 (anh Hiếu): KHÔNG tự tạo phòng ban "Chung" nữa — thiếu mảng thì tạo "Vấn đề chung" được, thiếu phòng
  // ban thì không. Cờ chỉ còn để hộp thoại giải thích; từng hồ sơ nhận lỗi chặn "Chưa có dòng phòng ban" ở dưới.
  const coDongPhongBan = rows.slice(hd.dong).some((r) => {
    const rr = r ?? [];
    return !soChuan(hd.cot.hoSoSo === undefined ? "" : cellText(rr[hd.cot.hoSoSo])) && RE_PHONG_BAN.test(cellText(rr[hd.cot.tieuDe ?? -1]).replace(/\s+/g, " ").trim());
  });
  if (!coDongPhongBan) {
    out.khongChiaPhongBan = true;
  }

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
      // "1.1 …" là mục nhỏ trong mảng hiện tại → bỏ qua dòng, hồ sơ phía dưới vẫn thuộc mảng đó.
      if (RE_MANG_CON.test(tieuDe)) continue;
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
    const canhBao: string[] = [];
    const thbqGoc = lay(r, "thbq");
    let thbq = khopThbq(thbqGoc, opts.thbqDanhMuc);
    if (!thbqGoc) {
      if (thbqMacDinh) {
        thbq = thbqMacDinh;
        canhBao.push(`THBQ trống — dùng mặc định "${thbqMacDinh}"`);
      } else loi.push("Thiếu thời hạn bảo quản (chọn THBQ mặc định cho tệp hoặc điền vào ô)");
    } else if (!thbq) loi.push(`Thời hạn bảo quản "${thbqGoc}" không có trong Danh mục`);
    if (thbq) demThbq.set(thbq, (demThbq.get(thbq) ?? 0) + 1);
    const vinhVien = thbq ? tenVv.has(thbq.toLowerCase()) : null;
    // Hồ sơ số (= số tạm) duy nhất trong từng loại: mục lục vĩnh viễn và có thời hạn đều đánh từ 1.
    const khoa = `${vinhVien}|${hoSoSo.toLowerCase()}`;
    const truoc = daCoHoSoSo.get(khoa);
    if (truoc !== undefined) loi.push(`Hồ sơ số ${hoSoSo} trùng với dòng ${truoc} (cùng loại ${TEN_LOAI(vinhVien)})`);
    else daCoHoSoSo.set(khoa, dong);
    // 2/11/2026: mục lục chưa đánh hộp (cột trống) vẫn nhập — hồ sơ không xếp hộp, xếp sau ở Hệ thống hoá.
    if (hd.cot.hopSo !== undefined && !hopHienTai) canhBao.push("Chưa có Hộp số — nhập không xếp hộp (xếp sau ở Hệ thống hoá)");

    const thoiGianGoc = hd.cot.thoiGian !== undefined ? lay(r, "thoiGian", true) : [lay(r, "ngayBatDau", true), lay(r, "ngayKetThuc", true)].filter(Boolean).join(" - ");
    const tg = kiemVaDienThoiGian(thoiGianGoc, tieuDe);
    // Không xác định được thời gian (ô trống + tiêu đề không ghi năm, hoặc sai định dạng không đọc được) → lỗi chặn
    // nhập; trước 12/10/2026 chỉ là cảnh báo nên hồ sơ vào hệ thống với thời gian trống (anh Hiếu hỏi).
    if (!tg.batDau || !tg.ketThuc) loi.push(thoiGianGoc.trim() ? `Thời gian "${thoiGianGoc.trim()}" không đọc được — ghi dd/mm/yyyy - dd/mm/yyyy` : "Thiếu thời gian bắt đầu - kết thúc (ô trống, tiêu đề không ghi năm)");
    if (pbHienTai) pbHienTai.soHoSo++;
    if (mangHienTai) mangHienTai.soHoSo++;
    // Số tờ chỉ nhận số tự nhiên (anh Hiếu 5/10/2026); "47 tờ" / "47tờ" / "47 TỜ" / "47 to" nhận là 47 (16/10/2026).
    // 2/11/2026: "18 quyển", "3,5" → CẢNH BÁO (vẫn nhập, số tờ để trống, giá trị gốc
    // giữ ở Ghi chú) — mục lục cũ ghi "quyển" hàng trăm dòng, chặn thì không nhập được tệp nào.
    const soToGoc = lay(r, "soTo").trim();
    let soTo = soToGoc.normalize("NFC").replace(/^(\d+)\s*(tờ|to)\.?$/iu, "$1");
    let ghiChu = lay(r, "ghiChu");
    const phaiDanhSoTo = vinhVien === true || (thbq !== null && tuHaiMuoiNam(thbq));
    if (soTo && !/^\d+$/.test(soTo)) {
      canhBao.push(`Số tờ "${soToGoc}" không phải số — để trống, ghi vào Ghi chú${phaiDanhSoTo ? " (tài liệu bảo quản từ 20 năm phải đánh số tờ)" : ""}`);
      ghiChu = [ghiChu, `Số tờ gốc: ${soToGoc}`].filter(Boolean).join("; ");
      soTo = "";
    } else if (soTo && +soTo === 0) {
      // Số tờ 0 ở tài liệu phải đánh số tờ cũng là thiếu số tờ → chặn như ô trống.
      if (phaiDanhSoTo) loi.push(`Thiếu số tờ — tài liệu bảo quản ${thbq} phải đánh số tờ (ô ghi 0)`);
      else canhBao.push("Số tờ 0 — để trống");
      soTo = "";
      // 26/11/2026 (anh Hiếu): thiếu số tờ ở tài liệu vĩnh viễn / từ 20 năm là LỖI CHẶN, tô đỏ — trước đó chỉ cảnh báo.
    } else if (!soTo && phaiDanhSoTo) loi.push(`Thiếu số tờ — tài liệu bảo quản ${thbq} phải đánh số tờ`);
    out.rows.push({ dong, hopSo: hopHienTai, hoSoSo, tieuDe, thoiGianGoc, soTo, thbqGoc, thbq, vinhVien, ghiChu, phongBan: pbHienTai, mang: mangHienTai, tg, loi, canhBao });
  }

  // Hộp thuộc đúng một loại: hộp có cả hồ sơ vĩnh viễn lẫn có thời hạn → cảnh báo (máy chủ tự tách thành hai hộp
  // cùng số, mỗi loại một hộp — khoá (phòng ban, loại, số hộp) ở he_thong_hoa::boxing).
  const loaiHop = new Map<string, Set<boolean>>();
  for (const r of out.rows) if (r.hopSo && r.vinhVien !== null) (loaiHop.get(r.hopSo) ?? loaiHop.set(r.hopSo, new Set()).get(r.hopSo)!).add(r.vinhVien);
  for (const r of out.rows) {
    if (r.hopSo && (loaiHop.get(r.hopSo)?.size ?? 0) > 1) r.canhBao.push(`Hộp ${r.hopSo} có cả hồ sơ vĩnh viễn lẫn có thời hạn — máy chủ sẽ tách thành hai hộp cùng số`);
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
  out.soCanhBao = out.rows.filter((r) => r.tg.ketLuan === "canh_bao" || r.tg.ketLuan === "loi" || r.canhBao.length > 0).length;
  out.soDaDien = out.rows.filter((r) => r.tg.ketLuan === "da_dien").length;
  return out;
}

// ------------------------------------------------------------------ đọc tệp

export interface TuyChonDoc {
  /** Sheet muốn phân tích; không có → sheet đầu tiên có dòng tiêu đề cột (sheet 1 thường là bìa). */
  sheet?: string;
  thbqMacDinh?: string;
}

export async function docMucLucTuBuffer(buf: ArrayBuffer, tenTep: string, thbqDanhMuc: string[], thbqVinhVien?: string[], tuyChon: TuyChonDoc = {}): Promise<PhanTichMucLuc> {
  const X = await import("xlsx");
  const wb = X.read(buf, { type: "array", cellDates: true });
  const sheets = wb.SheetNames;
  const luoi = (name: string): unknown[][] => {
    const ws = wb.Sheets[name];
    if (!ws) return [];
    const ref = ws["!ref"] ? X.utils.decode_range(ws["!ref"]) : { s: { r: 0, c: 0 }, e: { r: 0, c: 0 } };
    return X.utils.sheet_to_json<unknown[]>(ws, { header: 1, raw: true, defval: "", blankrows: true, range: { s: { r: 0, c: 0 }, e: ref.e } });
  };
  let name = tuyChon.sheet && sheets.includes(tuyChon.sheet) ? tuyChon.sheet : "";
  let rows: unknown[][] = [];
  if (name) rows = luoi(name);
  else {
    for (const s of sheets) {
      const r = luoi(s);
      if (timTieuDeCot(r)) {
        name = s;
        rows = r;
        break;
      }
    }
    if (!name) {
      name = sheets[0] ?? "";
      rows = luoi(name);
    }
  }
  const pt = phanTichMucLuc(rows, { tenTep, sheet: name, sheets, thbqDanhMuc, thbqVinhVien, thbqMacDinh: tuyChon.thbqMacDinh });
  pt.buf = buf;
  return pt;
}

export async function docMucLucHoSo(file: File, thbq: ClRow[], tuyChon?: TuyChonDoc): Promise<PhanTichMucLuc> {
  return phanTichLaiMucLuc(await file.arrayBuffer(), file.name, thbq, tuyChon);
}

/** Phân tích lại từ nội dung đã đọc (đổi sheet / THBQ mặc định trong hộp thoại, không chọn lại tệp). */
export function phanTichLaiMucLuc(buf: ArrayBuffer, tenTep: string, thbq: ClRow[], tuyChon?: TuyChonDoc): Promise<PhanTichMucLuc> {
  // Loại vĩnh viễn = THBQ trang_thai 1 trong Danh mục.
  const vv = thbq.filter((t) => Number(t.trang_thai) === 1).map((t) => str(t, "ten"));
  return docMucLucTuBuffer(buf, tenTep, thbq.map((t) => str(t, "ten")), vv.length ? vv : undefined, tuyChon);
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

export { MAU_HS_COLS };
