// Copyright (c) 2026 DigiText. All rights reserved.
// Kiểm chữ ký số của tệp PDF trên ổ cứng — src-tauri/src/chuky.rs.
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke, isTauri } from "@/api/ipc";

/** Một chữ ký trong tệp. Tệp ký nhiều lần (VGCA: phê duyệt, số, ngày, dấu tổ chức) có nhiều phần tử. */
export interface ChuKyRa {
  /** 1, 2, 3… theo thứ tự ký: chữ ký phủ ít byte nhất là chữ ký ký trước. */
  thuTu: number;
  /** Nhãn gọi tên trong báo cáo, vd `Chữ ký #2 (Dấu tổ chức)`. */
  nhan: string;
  /** `Sig` hoặc `DocTimeStamp`. */
  loai: string;
  khuonDang: string | null;
  tenKhai: string | null;
  lyDo: string | null;
  noi: string | null;
  /** CN của chứng thư — người ký thật, khác `tenKhai` do công cụ ký ghi vào tệp. */
  nguoiKy: string | null;
  toChuc: string | null;
  nhaCap: string | null;
  hieuLucTu: string | null;
  hieuLucDen: string | null;
  /** Thời điểm máy người ký khai. KHÔNG chứng minh được. */
  thoiDiemKhai: string | null;
  /** Thời điểm theo dấu thời gian — thứ chứng minh được. */
  thoiDiemTsa: string | null;
  coTsa: boolean;
  ganChungThu: boolean;
  toanVen: boolean;
  /** Phủ tới hết tệp, tức là chữ ký sau cùng. Chữ ký cũ KHÔNG phủ hết là chuyện bình thường. */
  phuTronTep: boolean;
  dat: boolean;
  loi: string | null;
}

export type KetLuanTep = "Đạt" | "Đạt, có cảnh báo" | "Không đạt" | "Chưa ký" | "Bỏ qua";

export interface TepDaKy {
  duongDan: string;
  ten: string;
  thuMuc: string;
  co: number;
  ketLuan: KetLuanTep;
  soChuKy: number;
  /** Byte nối thêm sau `%%EOF` cuối — khác 0 là dấu hiệu bị can thiệp thật. */
  byteThuaCuoi: number;
  /** Có dữ liệu xác minh dài hạn (LTV/DSS) nhúng kèm — mức PAdES-LT. */
  coDss: boolean;
  /** Có dấu thời gian cho cả tài liệu — mức PAdES-LTA, mức cao nhất. */
  coDauThoiGianTaiLieu: boolean;
  loi: string[];
  canhBao: string[];
  chuKy: ChuKyRa[];
}

export interface KetQuaQuet {
  tep: TepDaKy[];
  soPdf: number;
  /** Mục không phải .pdf — đã bỏ. Hiện ra để người dùng khỏi hoang mang vì số không khớp. */
  boQuaKhongPdf: number;
  boQuaQuaLon: number;
  biCat: boolean;
  soDat: number;
  soCanhBao: number;
  soKhongDat: number;
  soChuaKy: number;
}

export interface ChuKyProgress {
  done: number;
  total: number;
  name: string;
}

export function chukyQuet(paths: string[]): Promise<KetQuaQuet> {
  return invoke<KetQuaQuet>("chuky_quet", { args: { paths } });
}

export function chukyHuy(): Promise<void> {
  return invoke<void>("chuky_huy");
}

export async function onChuKyProgress(cb: (p: ChuKyProgress) => void): Promise<UnlistenFn> {
  if (!isTauri()) return () => {};
  return listen<ChuKyProgress>("chuky://progress", (e) => cb(e.payload));
}
