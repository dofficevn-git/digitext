// Copyright (c) 2026 DigiText. All rights reserved.

import type { V4bl, V4bm, V4cf, V4bp, V4bq } from "@/api/dossier";
import type { V3yr, V3ym, V3yl, V3yo } from "@/domain/types";
import type { V41b } from "@/state/navigation";

export function V4cl(Vwx: V3yl[], maDa?: string): V3yl[] {
  if (!maDa) return [];
  return Vwx.filter((Vwy) => Vwy.maPhong === maDa);
}

export function V4cm(Vwz: V3ym[], Vx0?: string): V3ym[] {
  if (!Vx0) return [];
  return Vwz.filter((m) => m.maDv === Vx0);
}

export function V4cn(Vx1: V3ym[], Vx2?: number): string {
  if (Vx2 == null) return "";
  return Vx1.find((m) => m.mangCvid === Vx2)?.tenMang ?? String(Vx2);
}

export function V4co(Vx3: V3yo[], Vx4?: number): string {
  if (Vx4 == null) return "—";
  return Vx3.find((t) => t.thbqid === Vx4)?.tenThbq ?? String(Vx4);
}

export function V4cp(): V4bl {
  return {
    maDa: "",
    maDv: "",
    soTam: "",
    tieuDe: "",
    trungThua: false,
  };
}

export function V4cq(r: V4bm): V4bl {
  return {
    maDa: r.maDa ?? "",
    maDv: r.maDv ?? "",
    mangCvid: r.mangCvid,
    thbq: r.thbq,
    soTam: r.soTam,
    maHoSo: r.maHoSo ?? "",
    tieuDe: r.tieuDe,
    ngayBatDau: r.ngayBatDau ?? "",
    ngayKetThuc: r.ngayKetThuc ?? "",
    soTo: r.soTo ?? "",
    trungThua: r.trungThua,
    tongSoVanBan: r.tongSoVanBan,
    soLuongTrang: r.soLuongTrang,
    cheDoSuDung: r.cheDoSuDung ?? "",
    ngonNgu: r.ngonNgu ?? "",
    tinhTrangVatLy: r.tinhTrangVatLy ?? "",
    ghiChu: r.ghiChu ?? "",
    hopSo: r.hopSo ?? "",
    boSo: r.boSo ?? "",
    tapSo: r.tapSo ?? "",
    trangThai: r.trangThai,
  };
}

export function V4cr(d: V4bl): V4bl {
  const s = (v?: string) => {
    const t = (v ?? "").trim();
    return t === "" ? undefined : t;
  };
  return {
    maDa: s(d.maDa),
    maDv: s(d.maDv),
    mangCvid: d.mangCvid,
    thbq: d.thbq,
    soTam: (d.soTam ?? "").trim(),
    maHoSo: s(d.maHoSo),
    tieuDe: (d.tieuDe ?? "").trim(),
    ngayBatDau: s(d.ngayBatDau),
    ngayKetThuc: s(d.ngayKetThuc),
    soTo: s(d.soTo),
    trungThua: !!d.trungThua,
    tongSoVanBan: d.tongSoVanBan,
    soLuongTrang: d.soLuongTrang,
    cheDoSuDung: s(d.cheDoSuDung),
    ngonNgu: s(d.ngonNgu),
    tinhTrangVatLy: s(d.tinhTrangVatLy),
    ghiChu: s(d.ghiChu),
    hopSo: s(d.hopSo),
    boSo: s(d.boSo),
    tapSo: s(d.tapSo),
    trangThai: s(d.trangThai),
  };
}

export function V4cs(d: V4bl): string | null {
  if (!(d.soTam ?? "").trim())
    return "Vui lòng nhập số tạm của hồ sơ.";
  if (d.ngayBatDau && d.ngayKetThuc && d.ngayBatDau > d.ngayKetThuc) {
    return "Ngày bắt đầu phải trước hoặc bằng ngày kết thúc.";
  }
  return null;
}

export function V4ct(r: V4bm, q: string): boolean {
  const Vx5 = [r.soTam, r.tieuDe, r.maHoSo, r.tenDa, r.hopSo]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();
  return Vx5.includes(q);
}

export function V4cu(r: V4bm): string {
  const Vx6 = [r.tenDa, `Số tạm ${r.soTam}`];
  if (r.tieuDe) Vx6.push(r.tieuDe);
  return Vx6.filter(Boolean).join(" · ");
}

export function V4cv(r: V4bm): number {
  return r.soVanBan > 0 ? r.soVanBan : (r.tongSoVanBan ?? 0);
}

export function V4cw(): V4bp {
  return { thuTu: 0, trichYeu: "" };
}

export function V4cx(v: V4bq): V4bp {
  return {
    thuTu: v.thuTu,
    loaiVanBan: v.loaiVanBan ?? "",
    soKyHieu: v.soKyHieu ?? "",
    ngayThang: v.ngayThang ?? "",
    coQuanBanHanh: v.coQuanBanHanh ?? "",
    trichYeu: v.trichYeu ?? "",
    toSo: v.toSo,
    soTrang: v.soTrang,
    trangSo: v.trangSo,
    ngonNgu: v.ngonNgu ?? "",
    cheDoSuDung: v.cheDoSuDung ?? "",
    doTinCay: v.doTinCay ?? "",
    butTich: v.butTich ?? "",
    tinhTrangVatLy: v.tinhTrangVatLy ?? "",
    taiLieuSo: v.taiLieuSo ?? "",
    ghiChu: v.ghiChu ?? "",
  };
}

export function V4cy(v: V4bp): V4bp {
  const s = (x?: string) => {
    const t = (x ?? "").trim();
    return t === "" ? undefined : t;
  };
  return {
    thuTu: Number.isFinite(v.thuTu) ? Number(v.thuTu) : 0,
    loaiVanBan: s(v.loaiVanBan),
    soKyHieu: s(v.soKyHieu),
    ngayThang: s(v.ngayThang),
    coQuanBanHanh: s(v.coQuanBanHanh),
    trichYeu: s(v.trichYeu),
    toSo: v.toSo,
    soTrang: v.soTrang,
    trangSo: v.trangSo,
    ngonNgu: s(v.ngonNgu),
    cheDoSuDung: s(v.cheDoSuDung),
    doTinCay: s(v.doTinCay),
    butTich: s(v.butTich),
    tinhTrangVatLy: s(v.tinhTrangVatLy),
    taiLieuSo: s(v.taiLieuSo),
    ghiChu: s(v.ghiChu),
  };
}

export function V4cz(v: V4bp): string | null {
  if (!(v.trichYeu ?? "").trim() && !(v.soKyHieu ?? "").trim()) {
    return "Nhập ít nhất Trích yếu hoặc Số/ký hiệu của văn bản.";
  }
  if (v.soTrang != null && v.soTrang < 0) return "Số trang không hợp lệ.";
  return null;
}

export function V4d0(r: V4bm): V41b | null {
  if (!r.maDa) return null;
  return {
    phong: { maDuAn: r.maDa, tenDuAn: r.tenDa ?? r.maDa },
    hoSo: {
      hopSo: r.hopSo ?? "",
      hoSoSo: r.soTam,
      tieuDe: r.tieuDe,
      soTo: r.soTo,
      ghiChu: r.ghiChu,
      thbq: r.thbq != null ? String(r.thbq) : undefined,
    },
    label: `${r.tenDa ?? r.maDa} · Số tạm ${r.soTam}`,
  };
}

export function V4d1(s?: string): { ngayBatDau?: string; ngayKetThuc?: string } {
  if (!s) return {};
  const ms = [...s.matchAll(/(\d{1,2})\/(\d{1,2})\/(\d{4})/g)];
  const Vx7 = (m: RegExpMatchArray) => `${m[3]}-${m[2].padStart(2, "0")}-${m[1].padStart(2, "0")}`;
  const Vx8: { ngayBatDau?: string; ngayKetThuc?: string } = {};
  if (ms[0]) Vx8.ngayBatDau = Vx7(ms[0]);
  if (ms[1]) Vx8.ngayKetThuc = Vx7(ms[1]);
  return Vx8;
}

function V4d2(s?: string): string | undefined {
  const t = (s ?? "").trim();
  if (!t) return undefined;
  if (/^\d{4}-\d{2}-\d{2}$/.test(t)) return t;
  const m = /^(\d{1,2})\/(\d{1,2})\/(\d{4})$/.exec(t);
  if (!m) return undefined;
  return `${m[3]}-${m[2].padStart(2, "0")}-${m[1].padStart(2, "0")}`;
}

export function V4d3(h: V3yr, Vx9?: Record<string, number>, Vxa?: Record<string, number>): V4bl {
  const Vxb = V4d1(h.thoiGian);
  const Vxc = (h.thbq ?? "").trim().toLowerCase();
  const Vxd = Vx9 && Vxc ? Vx9[Vxc] : undefined;
  const Vxe = (h.mang ?? "").trim().toLowerCase();
  const Vxf = Vxa && Vxe ? Vxa[Vxe] : undefined;
  return {
    mangCvid: Vxf,
    soTam: (h.soTam ?? h.hoSoSo ?? "").trim(),
    maHoSo: h.maHoSo?.trim() || undefined,
    tieuDe: h.tieuDe,
    hopSo: h.hopSo?.trim() || undefined,
    soTo: h.soTo?.trim() || undefined,
    ghiChu: h.ghiChu?.trim() || undefined,
    ngayBatDau: V4d2(h.ngayBatDau) ?? Vxb.ngayBatDau,
    ngayKetThuc: V4d2(h.ngayKetThuc) ?? Vxb.ngayKetThuc,
    thbq: Vxd,
    trungThua: h.trungThua ?? false,
    tongSoVanBan: h.tongSoVanBan,
    trangThai: h.trangThai?.trim() || undefined,
  };
}

export function V4d4(Vxg: V3yo[]): Record<string, number> {
  const m: Record<string, number> = {};
  for (const t of Vxg) m[t.tenThbq.trim().toLowerCase()] = t.thbqid;
  return m;
}

export function V4d5(Vxh: V3ym[]): Record<string, number> {
  const m: Record<string, number> = {};
  for (const x of Vxh) {
    m[x.tenMang.trim().toLowerCase()] = x.mangCvid;
    m[x.maMang.trim().toLowerCase()] = x.mangCvid;
  }
  return m;
}

export interface V4d6 {
  hopSo: string;
  hoSoSo: string;
  soTam: string;
  tieuDe: string;
  thbq: string;
  soTo: string;
  ngayBatDau: string;
  ngayKetThuc: string;
  trungThua: boolean;
  trangThai: string;
  dossierId: string;
}

export type V4d7 =
  | "invalid_header"
  | "missing_dossier"
  | "dossier_not_found"
  | "dossier_ambiguous"
  | "missing_content";

export interface V4d8 {
  line: number;
  reason: V4d7;
}

export function V4d9(Vxi: string): string {
  switch (Vxi) {
    case "create":
      return "Thêm";
    case "update":
      return "Sửa";
    case "delete":
      return "Xoá";
    default:
      return Vxi;
  }
}

export function V4da(s?: string): { ngay: string; gio: string } {
  const m = /^(\d{4})-(\d{1,2})-(\d{1,2})[ T](\d{2}:\d{2}(?::\d{2})?)/.exec((s ?? "").trim());
  if (!m) return { ngay: s ?? "", gio: "" };
  return { ngay: `${Number(m[3])}/${Number(m[2])}/${m[1]}`, gio: m[4] };
}

export const V4db = [
  "Hồ sơ id",
  "Số thứ tự",
  "Loại văn bản",
  "Số ký hiệu",
  "Ngày tháng",
  "Cơ quan ban hành",
  "Trích yếu",
  "Tờ số",
  "Số trang",
  "Trang số",
  "Ngôn ngữ",
  "Ghi chú",
];

function V4dc(s: string): string {
  return s
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/đ/g, "d")
    .replace(/\s+/g, " ")
    .trim();
}

const V4dd: { key: string; aliases: string[] }[] = [
  { key: "dossierId", aliases: ["ho so id", "hoso id", "id ho so"] },
  { key: "maHoSo", aliases: ["ma ho so", "ma h.s", "ma hs"] },
  { key: "soTam", aliases: ["ho so so", "so ho so", "so tam", "so hs"] },
  { key: "hopSo", aliases: ["hop so", "so hop", "hop"] },
  { key: "tieuDeHoSo", aliases: ["tieu de ho so", "tieu de", "noi dung"] },
  { key: "thuTu", aliases: ["so thu tu", "stt", "thu tu"] },
  { key: "loaiVanBan", aliases: ["loai van ban", "loai vb", "loai"] },
  { key: "soKyHieu", aliases: ["so ky hieu", "so/ky hieu", "so hieu"] },
  { key: "ngayThang", aliases: ["ngay thang", "ngay"] },
  { key: "coQuanBanHanh", aliases: ["co quan ban hanh", "co quan"] },
  { key: "trichYeu", aliases: ["trich yeu"] },
  { key: "toSo", aliases: ["to so"] },
  { key: "soTrang", aliases: ["so trang"] },
  { key: "trangSo", aliases: ["trang so"] },
  { key: "ngonNgu", aliases: ["ngon ngu"] },
  { key: "ghiChu", aliases: ["ghi chu"] },
];

function V4de(s?: string): string {
  return (s ?? "")
    .trim()
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/đ/g, "d")
    .replace(/\s+/g, " ");
}

function V4df(
  r: string[],
  Vxj: Record<string, number>,
  Vxk: V4bm[],
): { dossierId?: number; reason?: V4d7 } {
  const Vxl = (k: string) => {
    const c = Vxj[k];
    return c == null ? "" : (r[c] ?? "").trim();
  };
  const Vxm = (s: string) => {
    const n = Number.parseInt(s, 10);
    return Number.isNaN(n) ? undefined : n;
  };

  const Vxn = Vxm(Vxl("dossierId"));
  if (Vxn) {
    if (Vxk.length === 0) return { dossierId: Vxn };
    return Vxk.some((d) => d.id === Vxn) ? { dossierId: Vxn } : { reason: "dossier_not_found" };
  }

  const Vxo = V4de(Vxl("maHoSo"));
  const Vxp = V4de(Vxl("soTam"));
  const Vxq = V4de(Vxl("hopSo"));
  const Vxr = V4de(Vxl("tieuDeHoSo"));

  if (!Vxo && !Vxp && !Vxq && !Vxr) {
    return { reason: "missing_dossier" };
  }

  let Vxs = Vxk;
  if (Vxo) Vxs = Vxs.filter((d) => V4de(d.maHoSo) === Vxo);
  if (Vxp) Vxs = Vxs.filter((d) => V4de(d.soTam) === Vxp);
  if (Vxq) Vxs = Vxs.filter((d) => V4de(d.hopSo) === Vxq);
  if (Vxs.length > 1 && Vxr) {
    Vxs = Vxs.filter((d) => {
      const Vxt = V4de(d.tieuDe);
      return Vxt === Vxr || Vxt.includes(Vxr) || Vxr.includes(Vxt);
    });
  }
  if (Vxs.length === 1) return { dossierId: Vxs[0].id };
  if (Vxs.length === 0) return { reason: "dossier_not_found" };
  return { reason: "dossier_ambiguous" };
}

export function V4dg(
  Vxu: string[][],
  Vxv: V4bm[] = [],
): { items: V4cf[]; skipped: number; issues: V4d8[] } {
  let Vxw = -1;
  let Vxx: Record<string, number> = {};
  const Vxy = Math.min(Vxu.length, 15);
  for (let i = 0; i < Vxy; i++) {
    const Vxz = Vxu[i].map(V4dc);
    const m: Record<string, number> = {};
    let Vy0 = 0;
    for (const f of V4dd) {
      const Vy1 = Vxz.findIndex((c) => c.length > 0 && f.aliases.some((a) => c.includes(a)));
      if (Vy1 >= 0 && !Object.values(m).includes(Vy1)) {
        m[f.key] = Vy1;
        Vy0++;
      }
    }
    if (Vy0 > Object.keys(Vxx).length) {
      Vxx = m;
      Vxw = i;
    }
  }

  const Vy2 = (r: string[], k: string) => {
    const c = Vxx[k];
    return c == null ? "" : (r[c] ?? "").trim();
  };
  const Vy3 = (s: string) => {
    const n = Number.parseInt(s, 10);
    return Number.isNaN(n) ? undefined : n;
  };

  const Vy4: V4cf[] = [];
  const Vy5: V4d8[] = [];
  let Vy6 = 0;
  if (Vxw < 0 || Object.keys(Vxx).length === 0) {
    return {
      items: Vy4,
      skipped: Vxu.length > 0 ? 1 : 0,
      issues: Vxu.length > 0 ? [{ line: 1, reason: "invalid_header" }] : [],
    };
  }
  for (let i = Vxw + 1; i < Vxu.length; i++) {
    const r = Vxu[i];
    const Vy7 = V4df(r, Vxx, Vxv);
    const Vy8 = Vy2(r, "tieuDeHoSo");
    const Vy9 = Vy2(r, "trichYeu") || Vy8;
    const Vya = Vy2(r, "soKyHieu");
    const line = i + 1;
    if (!Vy7.dossierId) {
      Vy6++;
      Vy5.push({ line, reason: Vy7.reason ?? "dossier_not_found" });
      continue;
    }
    if (!Vy9 && !Vya) {
      Vy6++;
      Vy5.push({ line, reason: "missing_content" });
      continue;
    }
    Vy4.push({
      dossierId: Vy7.dossierId,
      vanBan: {
        thuTu: Vy3(Vy2(r, "thuTu")) ?? 0,
        loaiVanBan: Vy2(r, "loaiVanBan") || undefined,
        soKyHieu: Vya || undefined,
        ngayThang: Vy2(r, "ngayThang") || undefined,
        coQuanBanHanh: Vy2(r, "coQuanBanHanh") || undefined,
        trichYeu: Vy9 || undefined,
        toSo: Vy3(Vy2(r, "toSo")),
        soTrang: Vy3(Vy2(r, "soTrang")),
        trangSo: Vy3(Vy2(r, "trangSo")),
        ngonNgu: Vy2(r, "ngonNgu") || undefined,
        ghiChu: Vy2(r, "ghiChu") || undefined,
      },
    });
  }
  return { items: Vy4, skipped: Vy6, issues: Vy5 };
}

function V4dh(Vyb: V4d7): string {
  switch (Vyb) {
    case "invalid_header":
      return "không nhận diện được cột Excel";
    case "missing_dossier":
      return "thiếu thông tin hồ sơ để đối chiếu";
    case "dossier_not_found":
      return "không tìm thấy hồ sơ tương ứng";
    case "dossier_ambiguous":
      return "khớp nhiều hồ sơ, cần lọc hẹp hơn";
    case "missing_content":
      return "thiếu cả trích yếu lẫn số ký hiệu";
  }
}

export function V4di(Vyc: V4d8[], Vyd = 8): string {
  if (Vyc.length === 0) return "";
  const Vye = Vyc.slice(0, Vyd).map((x) => `- Dòng ${x.line}: ${V4dh(x.reason)}`);
  if (Vyc.length > Vyd) Vye.push(`- ... và ${Vyc.length - Vyd} dòng khác`);
  return Vye.join("\n");
}

export function V4dj(r: V4bm, Vyf: V3yo[], Vyg: V3ym[]): Record<string, string | number> {
  return {
    Mảng: V4cn(Vyg, r.mangCvid),
    "ID Hồ Sơ": r.id,
    "Mã Hồ Sơ": r.maHoSo ?? "",
    "Số Tạm": r.soTam,
    "Tiêu Đề": r.tieuDe,
    THBQ: V4co(Vyf, r.thbq),
    "Số Tờ": V4dt(r.soTo),
    "Ngày BĐ": V4dr(r.ngayBatDau),
    "Ngày KT": V4dr(r.ngayKetThuc),
    "T.Thừa": r.trungThua ? "x" : "",
    "T.Thái": V4du(r, Vyf).label,
    "Tài Liệu": V4cv(r),
    "Chức Năng": "",
  };
}

export type V4dk =
  | "mang"
  | "id"
  | "maHoSo"
  | "hopSo"
  | "soTam"
  | "tieuDe"
  | "thbq"
  | "soTo"
  | "ngayBatDau"
  | "ngayKetThuc"
  | "trungThua"
  | "trangThai"
  | "taiLieu"
  | "soTaiLieuScan"
  | "soFileScan";

export type V4dl = "asc" | "desc";

function V4dm(a?: string, b?: string): number {
  return (a ?? "").localeCompare(b ?? "", "vi", {
    sensitivity: "base",
    numeric: true,
    ignorePunctuation: true,
  });
}

function V4dn(a?: number, b?: number): number {
  return (a ?? Number.NEGATIVE_INFINITY) - (b ?? Number.NEGATIVE_INFINITY);
}

function V4do(r: V4bm, Vyh: V4dk, Vyi: V3yo[], Vyj: V3ym[]): string | number {
  switch (Vyh) {
    case "mang":
      return V4cn(Vyj, r.mangCvid);
    case "id":
      return r.id;
    case "maHoSo":
      return r.maHoSo ?? "";
    case "hopSo":
      return r.hopSo ?? "";
    case "soTam":
      return r.soTam;
    case "tieuDe":
      return r.tieuDe;
    case "thbq": {
      const t = Vyi.find((x) => x.thbqid === r.thbq);
      return t?.sort ?? Number.MAX_SAFE_INTEGER;
    }
    case "soTo":
      return Number.parseInt(V4dt(r.soTo), 10) || 0;
    case "ngayBatDau":
      return r.ngayBatDau ?? "";
    case "ngayKetThuc":
      return r.ngayKetThuc ?? "";
    case "trungThua":
      return r.trungThua ? 1 : 0;
    case "trangThai":
      return V4du(r, Vyi).label;
    case "taiLieu":
      return V4cv(r);
    case "soTaiLieuScan":
      return r.soTaiLieuScan;
    case "soFileScan":
      return r.soFileScan;
  }
}

export function V4dp(Vyk: V4bm[], Vyl: V4dk, Vym: V4dl, Vyn: V3yo[], Vyo: V3ym[]): V4bm[] {
  const sign = Vym === "asc" ? 1 : -1;
  return [...Vyk].sort((a, b) => {
    const Vyp = V4do(a, Vyl, Vyn, Vyo);
    const Vyq = V4do(b, Vyl, Vyn, Vyo);
    const base =
      typeof Vyp === "number" && typeof Vyq === "number"
        ? V4dn(Vyp, Vyq)
        : V4dm(String(Vyp), String(Vyq));
    if (base !== 0) return base * sign;
    return a.id - b.id;
  });
}

export function V4dq(h: V3yr): V4d6 {
  const Vyr = V4d1(h.thoiGian);
  return {
    hopSo: h.hopSo ?? "",
    hoSoSo: h.hoSoSo ?? "",
    soTam: h.soTam ?? h.hoSoSo ?? "",
    tieuDe: h.tieuDe ?? "",
    thbq: h.thbq ?? "",
    soTo: h.soTo ?? "",
    ngayBatDau: V4dr(V4d2(h.ngayBatDau) ?? Vyr.ngayBatDau),
    ngayKetThuc: V4dr(V4d2(h.ngayKetThuc) ?? Vyr.ngayKetThuc),
    trungThua: !!h.trungThua,
    trangThai: h.trangThai ?? "",
    dossierId: h.dossierId != null ? String(h.dossierId) : "",
  };
}

export function V4dr(Vys?: string): string {
  if (!Vys) return "";
  const m = /^(\d{4})-(\d{1,2})-(\d{1,2})/.exec(Vys.trim());
  if (!m) return Vys;
  return `${Number(m[3])}/${Number(m[2])}/${m[1]}`;
}

export function V4ds(s?: string): string | undefined {
  const m = /^(\d{1,2})\/(\d{1,2})\/(\d{4})$/.exec((s ?? "").trim());
  if (!m) return undefined;
  return `${m[3]}-${m[2].padStart(2, "0")}-${m[1].padStart(2, "0")}`;
}

export function V4dt(Vyt?: string): string {
  const t = (Vyt ?? "").trim();
  return t === "" || t === "0" ? "" : t;
}

export function V4du(r: V4bm, Vyu: V3yo[]): { label: string; cls: string } {
  const ten = V4co(Vyu, r.thbq).toLowerCase();
  if (ten.includes("vĩnh viễn")) return { label: "Vĩnh viễn", cls: "badge-vv" };
  if (r.trangThai && r.trangThai.toLowerCase().includes("het"))
    return { label: "Hết THBQ", cls: "badge-het" };
  return { label: "Giữ lại", cls: "badge-gl" };
}

export type V4dv = "thbqAsc" | "thbqDesc" | "timeAsc" | "timeDesc";

export function V4dw(Vyv: V4bm[], Vyw: V3yo[], Vyx: V4dv): V4bm[] {
  const Vyy = (Vyz?: number): number => {
    if (Vyz == null) return Number.MAX_SAFE_INTEGER;
    const t = Vyw.find((x) => x.thbqid === Vyz);
    return t ? t.sort : Number.MAX_SAFE_INTEGER;
  };
  const Vz0 = (a: V4bm, b: V4bm): number => {
    if (Vyx === "timeAsc") return (a.ngayBatDau ?? "").localeCompare(b.ngayBatDau ?? "");
    if (Vyx === "timeDesc") return (b.ngayKetThuc ?? "").localeCompare(a.ngayKetThuc ?? "");
    const d = Vyy(a.thbq) - Vyy(b.thbq);
    const base = Vyx === "thbqDesc" ? -d : d;
    if (base !== 0) return base;
    return (a.ngayBatDau ?? "").localeCompare(b.ngayBatDau ?? "");
  };
  return [...Vyv].sort(Vz0);
}
