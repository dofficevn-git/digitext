// Copyright (c) 2026 DigiText. All rights reserved.

import { V4dr } from "@/features/dossier/logic";
import type { V4nu, V4nv, V4nw, V4nt } from "@/api/report";

export interface V4o1<T> {
  head: string;
  value: (r: T) => string | number;
  num?: boolean;
}

function V4o2(ten?: string, ma?: string): string {
  return ten || ma || "KENoxrBhIGfDoW4gcGjDtG5nKQ==";
}

export const V4o3: V4o1<V4nt>[] = [
  { head: "UGjDtG5n", value: (r) => V4o2(r.tenDuAn, r.maDuAn) },
  { head: "U+G7kSBo4buTIHPGoQ==", value: (r) => r.soHoSo, num: true },
  { head: "U+G7kSBQREY=", value: (r) => r.soPdf, num: true },
  { head: "A0", value: (r) => r.a0, num: true },
  { head: "A1", value: (r) => r.a1, num: true },
  { head: "A2", value: (r) => r.a2, num: true },
  { head: "A3", value: (r) => r.a3, num: true },
  { head: "A4", value: (r) => r.a4, num: true },
  { head: "A5", value: (r) => r.a5, num: true },
  { head: "S2jDoWM=", value: (r) => r.khac, num: true },
  { head: "VOG7lW5nIHRyYW5n", value: (r) => r.tongTrang, num: true },
  { head: "UXV5IMSR4buVaSBBNA==", value: (r) => Math.round(r.quyDoi * 10) / 10, num: true },
];

export const V4o4: V4o1<V4nu>[] = [
  {
    head: "UGjDsm5nIGJhbg==",
    value: (r) => r.tenDonVi || r.maDv || "KENoxrBhIGfDoW4gcGjDsm5nIGJhbik=",
  },
  { head: "SOG7mXAgdsSpbmggdmnhu4Vu", value: (r) => r.hopVinhVien, num: true },
  { head: "SOG7kyBzxqEgdsSpbmggdmnhu4Vu", value: (r) => r.hoSoVinhVien, num: true },
  { head: "SOG7mXAgY8OzIHRo4budaSBo4bqhbg==", value: (r) => r.hopCoThoiHan, num: true },
  { head: "SOG7kyBzxqEgY8OzIHRo4budaSBo4bqhbg==", value: (r) => r.hoSoCoThoiHan, num: true },
  { head: "QsOzIGjhu5Mgc8ahIGxv4bqhaQ==", value: (r) => r.boLoai, num: true },
  { head: "VOG6rXAgaOG7kyBzxqEgbG/huqFp", value: (r) => r.tapLoai, num: true },
];

export const V4o5: V4o1<V4nv>[] = [
  { head: "TmjDom4gdmnDqm4=", value: (r) => r.ten },
  { head: "U+G7kSBo4buTIHPGoQ==", value: (r) => r.soHoSo, num: true },
  { head: "U+G7kSB2xINuIGLhuqNu", value: (r) => r.soVanBan, num: true },
  { head: "U+G7kSB0cmFuZyBxdcOpdA==", value: (r) => r.soTrangQuet, num: true },
  { head: "VGhhbyB0w6FjIHRow6pt", value: (r) => r.thaoTacThem, num: true },
  { head: "VGhhbyB0w6FjIHPhu61h", value: (r) => r.thaoTacSua, num: true },
];

export const V4o6: V4o1<V4nw>[] = [
  { head: "UGjDtG5n", value: (r) => V4o2(r.tenDuAn, r.maDuAn) },
  { head: "VGjhu51pIGdpYW4ga2jhu59pIHThuqFv", value: (r) => V4dr(r.thoiGianKhoiTao) },
  { head: "SOG7mXAgdsSpbmggdmnhu4Vu", value: (r) => r.hopVinhVien, num: true },
  { head: "SOG7kyBzxqEgdsSpbmggdmnhu4Vu", value: (r) => r.hoSoVinhVien, num: true },
  { head: "SOG7mXAgY8OzIHRo4budaSBo4bqhbg==", value: (r) => r.hopCoThoiHan, num: true },
  { head: "SOG7kyBzxqEgY8OzIHRo4budaSBo4bqhbg==", value: (r) => r.hoSoCoThoiHan, num: true },
  { head: "QsOzIGjhu5Mgc8ahIGxv4bqhaQ==", value: (r) => r.boLoai, num: true },
  { head: "VOG6rXAgaOG7kyBzxqEgbG/huqFp", value: (r) => r.tapLoai, num: true },
];

export function V4o7<T>(cols: V4o1<T>[], V21s: T[]): Record<string, unknown>[] {
  return V21s.map((r) => {
    const o: Record<string, unknown> = {};
    for (const c of cols) o[c.head] = c.value(r);
    return o;
  });
}
