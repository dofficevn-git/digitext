// Copyright (c) 2026 DigiText. All rights reserved.

import type { V3yr } from "@/domain/types";

export interface V4b1 {
  name: string;
  rows: string[][];
}
export interface V4b2 {
  sheets: V4b1[];
}

function V4b3(s: string): string {
  return s
    .toLowerCase()
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/đ/g, "d")
    .replace(/\s+/g, " ")
    .trim();
}

interface V4b4 {
  key: keyof V3yr;
  label: string;
  aliases: string[];
}

export const V4b5: V4b4[] = [
  { key: "mang", label: "Mảng", aliases: ["mang", "mang cong viec", "ten mang"] },
  { key: "dossierId", label: "ID Hồ sơ", aliases: ["id ho so", "ho so id", "dossier id"] },
  { key: "hopSo", label: "Hộp số", aliases: ["hop so", "so hop", "hop"] },
  { key: "hoSoSo", label: "Hồ sơ số", aliases: ["ho so so", "so ho so", "so hs"] },
  { key: "soTam", label: "Số tạm", aliases: ["so tam"] },
  {
    key: "maHoSo",
    label: "Mã hồ sơ",
    aliases: ["ma ho so", "ma hs", "ma h.s", "ma hoso", "ma so ho so"],
  },
  {
    key: "tieuDe",
    label: "Tiêu đề hồ sơ",
    aliases: ["tieu de ho so", "tieu de", "noi dung"],
  },
  {
    key: "thoiGian",
    label: "Thời gian (BĐ–KT)",
    aliases: ["ngay thang", "thoi gian", "ngay thang bd"],
  },
  {
    key: "ngayBatDau",
    label: "Ngày BĐ",
    aliases: ["ngay bd", "ngay bat dau", "n.bd", "n bat dau", "ngaybd", "tu ngay"],
  },
  {
    key: "ngayKetThuc",
    label: "Ngày KT",
    aliases: ["ngay kt", "ngay ket thuc", "n.kt", "n ket thuc", "ngaykt", "den ngay"],
  },
  { key: "soTo", label: "Số tờ", aliases: ["so to", "so trang", "so luong to"] },
  {
    key: "thbq",
    label: "Thời hạn bảo quản",
    aliases: ["thbq", "thoi han bao quan", "bao quan"],
  },
  { key: "trungThua", label: "T.Thừa", aliases: ["t.thua", "t thua", "trung thua", "thua"] },
  {
    key: "trangThai",
    label: "T.Thái",
    aliases: ["t.thai", "t thai", "trang thai", "trangthai", "tt"],
  },
  {
    key: "tongSoVanBan",
    label: "Tài Liệu",
    aliases: ["tai lieu", "so van ban", "tong so van ban", "so vb", "tong so vb", "sl vb"],
  },
  { key: "ghiChu", label: "Ghi chú", aliases: ["ghi chu", "note", "ghichu"] },
];

const V4b6 = V4b5.map((f) => ({ ...f, aliases: f.aliases.map(V4b3) }));

export async function V4b7(file: File): Promise<V4b2> {
  const Vqo = await import("xlsx");
  const Vqp = await file.arrayBuffer();
  const Vqq = Vqo.read(Vqp, { type: "array" });
  const Vqr: V4b1[] = Vqq.SheetNames.map((name) => {
    const Vqs = Vqq.Sheets[name];
    const Vqt = Vqo.utils.sheet_to_json<unknown[]>(Vqs, {
      header: 1,
      blankrows: false,
      defval: "",
      raw: false,
    });
    const Vqu = Vqt.map((r) => (r as unknown[]).map((c) => (c == null ? "" : String(c).trim())));
    return { name, rows: Vqu };
  });
  return { sheets: Vqr };
}

export interface V4b8 {
  headerRow: number;
  map: Record<string, number>;
  score: number;
}

export function V4b9(Vqv: string[][]): V4b8 {
  let Vqw: V4b8 = { headerRow: -1, map: {}, score: 0 };
  const Vqx = Math.min(Vqv.length, 15);
  for (let i = 0; i < Vqx; i++) {
    const Vqy = Vqv[i].map(V4b3);
    const Vqz: Record<string, number> = {};
    let Vr0 = 0;
    for (const f of V4b6) {
      const Vr1 = Vqy.findIndex((c) => c.length > 0 && f.aliases.some((a) => c.includes(a)));
      if (Vr1 >= 0 && !Object.values(Vqz).includes(Vr1)) {
        Vqz[f.key as string] = Vr1;
        Vr0++;
      }
    }
    if (Vr0 > Vqw.score) Vqw = { headerRow: i, map: Vqz, score: Vr0 };
  }
  return Vqw;
}

export interface V4ba {
  records: V3yr[];
  skipped: number;
  groups: number;
}

const V4bb = /n[aă]m\s+\d{4}|^\s*\d+\.\s*n[aă]m/i;

function V4bc(s: string): boolean | undefined {
  const v = V4b3(s);
  if (!v) return undefined;
  if (["1", "x", "true", "yes", "co", "c", "checked"].includes(v)) return true;
  if (["0", "false", "no", "khong", "k"].includes(v)) return false;
  return undefined;
}

function V4bd(s: string): number | undefined {
  const n = Number.parseInt(s.trim(), 10);
  return Number.isNaN(n) ? undefined : n;
}

export function V4be(Vr2: string[][], m: V4b8, Vr3: string, Vr4: number): V4ba {
  const Vr5 = (r: string[], Vr6: string) => {
    const c = m.map[Vr6];
    return c == null ? "" : (r[c] ?? "").trim();
  };
  let Vr7 = "";
  let Vr8 = "";
  let Vr9 = Vr4;
  let Vra = 0;
  let Vrb = 0;
  const records: V3yr[] = [];
  const from = m.headerRow >= 0 ? m.headerRow + 1 : 0;

  for (let i = from; i < Vr2.length; i++) {
    const r = Vr2[i];
    const Vrc = Vr5(r, "hopSo");
    const Vrd = Vr5(r, "hoSoSo");
    const Vre = Vr5(r, "soTam");
    const Vrf = Vr5(r, "tieuDe");

    if (!Vrc && !Vrd && !Vre) {
      if (Vrf) {
        if (V4bb.test(Vrf)) Vr8 = Vrf;
        else Vr7 = Vrf;
        Vrb++;
      } else {
        Vra++;
      }
      continue;
    }

    records.push({
      id: Vr9++,
      mang: Vr5(r, "mang") || undefined,
      dossierId: V4bd(Vr5(r, "dossierId")),
      hopSo: Vrc,
      hoSoSo: Vrd,
      soTam: Vre || undefined,
      maHoSo: Vr5(r, "maHoSo") || undefined,
      tieuDe: Vrf,
      thoiGian: Vr5(r, "thoiGian"),
      ngayBatDau: Vr5(r, "ngayBatDau") || undefined,
      ngayKetThuc: Vr5(r, "ngayKetThuc") || undefined,
      soTo: Vr5(r, "soTo"),
      thbq: Vr5(r, "thbq"),
      trungThua: V4bc(Vr5(r, "trungThua")),
      trangThai: Vr5(r, "trangThai") || undefined,
      tongSoVanBan: V4bd(Vr5(r, "tongSoVanBan")),
      ghiChu: Vr5(r, "ghiChu") || undefined,
      nhom: Vr7 || undefined,
      nam: Vr8 || undefined,
      nguon: Vr3,
    });
  }
  return { records, skipped: Vra, groups: Vrb };
}

export function V4bf(Vrg: V4b2): string {
  let name = Vrg.sheets[0]?.name ?? "";
  let Vrh = -1;
  for (const s of Vrg.sheets) {
    const Vri = V4b9(s.rows).score;
    if (Vri > Vrh) {
      Vrh = Vri;
      name = s.name;
    }
  }
  return name;
}
