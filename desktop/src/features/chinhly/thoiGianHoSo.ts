// Copyright (c) 2026 DigiText. All rights reserved.
// Kiểm tra + tự điền cột "Thời gian BĐ - KT" của mục lục hồ sơ. Chuyển từ hai công cụ ngoài
// (check_thoigian.py / dien_thoigian.py) để không phải chạy tay trước khi nhập Excel.
//
// Quy tắc điền:  2020 → 01/01/2020 – 31/12/2020 · 2015-2019 → 01/01/2015 – 31/12/2019 · 04/2020 → 01/04/2020 – 30/04/2020
//                04/05/2020 → lặp lại 2 mốc · 2 mốc → chuẩn hoá dd/mm/yyyy · đầu > cuối → đảo
//                mốc sai nhưng còn mốc đúng → giữ mốc đúng, mốc thiếu suy từ năm trong ô
//                ô trống / không có năm trong ô → năm trong tiêu đề · vẫn không ra → không điền
// Quy tắc kiểm:  ô trống · mốc sai định dạng · đầu sau cuối · năm của ngày phải nằm trong dải năm ghi ở tiêu đề
//                (cụm năm ngay sau chữ "năm", không có thì cụm năm cuối cùng; bỏ "nhiệm kỳ yyyy - yyyy").

export type KetLuanThoiGian = "dung" | "da_dien" | "canh_bao" | "loi";

export interface KetQuaThoiGian {
  /** dd/mm/yyyy hoặc "" khi không điền được. */
  batDau: string;
  ketThuc: string;
  ketLuan: KetLuanThoiGian;
  /** Ghi chú kiểm tra (mỗi dòng một ý), rỗng khi đúng. */
  ghiChu: string[];
}

export interface Moc {
  kind: "ngay" | "thang" | "nam" | "sai";
  raw: string;
  y: number;
  m: number;
  d: number;
}

interface YMD {
  y: number;
  m: number;
  d: number;
}

const MIN_YEAR = 1800;
const MAX_YEAR = 2100;

/** Bỏ dấu nháy đơn Excel ở mọi vị trí ("'05/12/2025", "2016-'2018"). */
export function boNhay(s: string): string {
  return s.replace(/['’‘`]/g, "").trim();
}

export function soNgayTrongThang(y: number, m: number): number {
  if (m === 2) return (y % 4 === 0 && y % 100 !== 0) || y % 400 === 0 ? 29 : 28;
  return [4, 6, 9, 11].includes(m) ? 30 : 31;
}

function ngayHopLe(y: number, m: number, d: number): boolean {
  return y >= MIN_YEAR && y <= MAX_YEAR && m >= 1 && m <= 12 && d >= 1 && d <= soNgayTrongThang(y, m);
}

function fmt(x: YMD): string {
  return `${String(x.d).padStart(2, "0")}/${String(x.m).padStart(2, "0")}/${x.y}`;
}

function cmp(a: YMD, b: YMD): number {
  return a.y - b.y || a.m - b.m || a.d - b.d;
}

/** Ô Excel → chuỗi: Date → dd/mm/yyyy; số serial (cột ngày) → dd/mm/yyyy; số khác → nguyên; còn lại trim. */
export function cellText(v: unknown, laCotNgay = false): string {
  if (v === null || v === undefined) return "";
  if (v instanceof Date) {
    if (Number.isNaN(v.getTime())) return "";
    return fmt({ y: v.getFullYear(), m: v.getMonth() + 1, d: v.getDate() });
  }
  if (typeof v === "number") {
    if (laCotNgay && Number.isInteger(v) && v > 20000 && v < 80000) {
      // serial Excel: 1899-12-30 + n
      const ms = (v - 25569) * 86400000;
      const dt = new Date(ms);
      return fmt({ y: dt.getUTCFullYear(), m: dt.getUTCMonth() + 1, d: dt.getUTCDate() });
    }
    return Number.isInteger(v) ? String(v) : String(v);
  }
  return String(v).trim();
}

/** Tách ô thành các mốc theo dấu gạch / xuống dòng / khoảng trắng; bỏ từ nối ("đến", "từ"). */
export function tachMoc(raw: string): Moc[] {
  const s = boNhay(raw);
  if (!s) return [];
  return s
    .split(/[-–—\s;,]+/)
    .map((t) => t.trim())
    .filter((t) => t && !/^(từ|đến|tới|tu|den|toi|to|from)$/i.test(t))
    .map((t): Moc => {
      const tok = /^\d{1,2}\.\d{1,2}\.\d{4}$/.test(t) ? t.replace(/\./g, "/") : t;
      let m = /^(\d{1,2})\s*\/\s*(\d{1,2})\s*\/\s*(\d{4})$/.exec(tok);
      if (m) {
        const y = +m[3], mo = +m[2], d = +m[1];
        return ngayHopLe(y, mo, d) ? { kind: "ngay", raw: t, y, m: mo, d } : { kind: "sai", raw: t, y: y >= MIN_YEAR && y <= MAX_YEAR ? y : 0, m: 0, d: 0 };
      }
      m = /^(\d{1,2})\s*\/\s*(\d{4})$/.exec(tok);
      if (m) {
        const y = +m[2], mo = +m[1];
        return mo >= 1 && mo <= 12 && y >= MIN_YEAR && y <= MAX_YEAR ? { kind: "thang", raw: t, y, m: mo, d: 0 } : { kind: "sai", raw: t, y: 0, m: 0, d: 0 };
      }
      m = /^(\d{4})$/.exec(tok);
      if (m) {
        const y = +m[1];
        return y >= MIN_YEAR && y <= MAX_YEAR ? { kind: "nam", raw: t, y, m: 0, d: 0 } : { kind: "sai", raw: t, y: 0, m: 0, d: 0 };
      }
      return { kind: "sai", raw: t, y: 0, m: 0, d: 0 };
    });
}

/** Mốc → ngày bắt đầu của nó: 06/2013 → 01/06/2013, 2013 → 01/01/2013. */
function mocDau(m: Moc): YMD {
  if (m.kind === "ngay") return { y: m.y, m: m.m, d: m.d };
  if (m.kind === "thang") return { y: m.y, m: m.m, d: 1 };
  return { y: m.y, m: 1, d: 1 };
}
/** Mốc → ngày kết thúc của nó: 06/2013 → 30/06/2013, 2013 → 31/12/2013. */
function mocCuoi(m: Moc): YMD {
  if (m.kind === "ngay") return { y: m.y, m: m.m, d: m.d };
  if (m.kind === "thang") return { y: m.y, m: m.m, d: soNgayTrongThang(m.y, m.m) };
  return { y: m.y, m: 12, d: 31 };
}

/** Các năm (1800–2100) xuất hiện trong chuỗi, theo thứ tự. */
export function namTrongChuoi(s: string): number[] {
  const out: number[] = [];
  for (const m of s.matchAll(/(?<!\d)(\d{4})(?!\d)/g)) {
    const y = +m[1];
    if (y >= MIN_YEAR && y <= MAX_YEAR) out.push(y);
  }
  return out;
}

export interface NamTieuDe {
  y1: number;
  y2: number;
  raw: string;
}

/**
 * Dải năm tài liệu ghi ở tiêu đề: cụm năm ngay sau chữ "năm" ("… Năm 2014 - 2015"), không có thì cụm năm
 * xuất hiện cuối cùng. Bỏ "nhiệm kỳ 2020 - 2025" (tên đại hội, không phải thời gian tài liệu).
 */
export function namTrongTieuDe(tieuDe: string): NamTieuDe | null {
  const s = (tieuDe ?? "").replace(/nhi[ệe]m\s*k[ỳy]\s*\d{4}(?:\s*[-–—]\s*\d{4})?/gi, " ");
  const re = /n[ăa]m\s*(\d{4})(?:\s*[-–—]\s*(\d{4}))?/gi;
  let last: NamTieuDe | null = null;
  for (const m of s.matchAll(re)) {
    const y1 = +m[1];
    const y2 = m[2] ? +m[2] : y1;
    if (y1 >= MIN_YEAR && y1 <= MAX_YEAR) last = { y1, y2: y2 >= y1 && y2 <= MAX_YEAR ? y2 : y1, raw: m[0].trim() };
  }
  if (last) return last;
  let cuoi: NamTieuDe | null = null;
  for (const m of s.matchAll(/(?<!\d)(\d{4})(?:\s*[-–—]\s*(\d{4}))?(?!\d)/g)) {
    const y1 = +m[1];
    if (y1 < MIN_YEAR || y1 > MAX_YEAR) continue;
    const y2 = m[2] ? +m[2] : y1;
    cuoi = { y1, y2: y2 >= y1 && y2 <= MAX_YEAR ? y2 : y1, raw: m[0].trim() };
  }
  return cuoi;
}

const THU_TU: Record<KetLuanThoiGian, number> = { dung: 0, da_dien: 1, canh_bao: 2, loi: 3 };

/** Kiểm và điền một ô thời gian. `tieuDe` dùng để suy năm khi ô thiếu và để đối chiếu. */
export function kiemVaDienThoiGian(raw: string, tieuDe: string): KetQuaThoiGian {
  const ghiChu: string[] = [];
  let ketLuan: KetLuanThoiGian = "dung";
  const nang = (k: KetLuanThoiGian) => {
    if (THU_TU[k] > THU_TU[ketLuan]) ketLuan = k;
  };
  const goc = boNhay(raw ?? "");
  const mocs = tachMoc(goc);
  const sai = mocs.filter((m) => m.kind === "sai");
  const dung = mocs.filter((m) => m.kind !== "sai");
  const tdNam = namTrongTieuDe(tieuDe);

  let bd: YMD | null = null;
  let kt: YMD | null = null;

  if (!goc) {
    if (tdNam) {
      bd = { y: tdNam.y1, m: 1, d: 1 };
      kt = { y: tdNam.y2, m: 12, d: 31 };
      ghiChu.push(`Ô trống — điền theo năm trong tiêu đề (${tdNam.raw})`);
      nang("da_dien");
    } else {
      ghiChu.push("Ô thời gian trống và tiêu đề không ghi năm — không điền được");
      nang("loi");
    }
  } else if (sai.length === 0 && dung.length > 0) {
    bd = mocDau(dung[0]);
    kt = mocCuoi(dung[dung.length - 1]);
    if (dung.length === 1) {
      const k = dung[0].kind;
      ghiChu.push(k === "ngay" ? "Chỉ ghi 1 ngày — lặp lại thành 2 mốc" : k === "thang" ? "Chỉ ghi tháng/năm — trải cả tháng" : "Chỉ ghi năm — trải cả năm");
      nang("da_dien");
    } else {
      const thieu = dung.filter((m) => m.kind !== "ngay");
      if (thieu.length) {
        ghiChu.push(`Mốc ${thieu.map((m) => `'${m.raw}'`).join(", ")} chỉ ghi ${thieu[0].kind === "nam" ? "năm" : "tháng/năm"} — trải cả ${thieu[0].kind === "nam" ? "năm" : "tháng"}`);
        nang("da_dien");
      }
    }
    if (cmp(bd, kt) > 0) {
      [bd, kt] = [kt, bd];
      ghiChu.push("Ngày đầu sau ngày cuối — đã đảo lại thứ tự");
      nang("canh_bao");
    }
  } else {
    for (const m of sai) ghiChu.push(`Sai định dạng: '${m.raw}' không khớp dd/mm/yyyy`);
    nang("canh_bao");
    const nam = namTrongChuoi(goc);
    const yMin = nam.length ? Math.min(...nam) : null;
    const yMax = nam.length ? Math.max(...nam) : null;
    const dauDung = mocs[0]?.kind !== "sai" ? mocs[0] : null;
    const cuoiDung = mocs[mocs.length - 1]?.kind !== "sai" ? mocs[mocs.length - 1] : null;
    if (yMin !== null && yMax !== null) {
      bd = dauDung ? mocDau(dauDung) : { y: yMin, m: 1, d: 1 };
      kt = cuoiDung ? mocCuoi(cuoiDung) : { y: yMax, m: 12, d: 31 };
      ghiChu.push(dauDung || cuoiDung ? "Giữ mốc đúng, mốc thiếu suy từ năm trong ô" : "Giá trị sai — điền theo năm trong ô");
    } else if (tdNam) {
      bd = { y: tdNam.y1, m: 1, d: 1 };
      kt = { y: tdNam.y2, m: 12, d: 31 };
      ghiChu.push(`Không đọc được năm trong ô — điền theo năm trong tiêu đề (${tdNam.raw})`);
    } else {
      ghiChu.push("Không đọc được năm trong ô lẫn tiêu đề — không điền được, cần sửa tay");
      nang("loi");
    }
    if (bd && kt && cmp(bd, kt) > 0) {
      [bd, kt] = [kt, bd];
      ghiChu.push("Ngày đầu sau ngày cuối — đã đảo lại thứ tự");
    }
  }

  if (bd && kt) {
    if (tdNam) {
      const ngoai = [bd.y, kt.y].filter((y) => y < tdNam.y1 || y > tdNam.y2);
      if (ngoai.length) {
        ghiChu.push(`Năm ${[...new Set(ngoai)].join(", ")} không nằm trong năm tài liệu ở tiêu đề (${tdNam.raw})`);
        nang("canh_bao");
      }
    } else if (goc) {
      ghiChu.push("Tiêu đề không ghi năm tài liệu để đối chiếu");
      nang("canh_bao");
    }
  }

  return { batDau: bd ? fmt(bd) : "", ketThuc: kt ? fmt(kt) : "", ketLuan, ghiChu };
}

export const KET_LUAN_LABEL: Record<KetLuanThoiGian, string> = { dung: "Đúng", da_dien: "Đã điền", canh_bao: "Cảnh báo", loi: "Lỗi" };
