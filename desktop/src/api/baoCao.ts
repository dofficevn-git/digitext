// Copyright (c) 2026 DigiText. All rights reserved.
// Báo cáo thống kê TT05 Điều 44.3b (27/11/2026, đợt 3 nhóm 3A): danh mục báo cáo do máy chủ định nghĩa (routes/bao_cao.rs),
// desktop dựng bộ lọc và bảng từ đó; Excel / PDF máy chủ dựng, tải về qua Tauri bc_download (hộp thoại Lưu).

import { invoke } from "@/api/ipc";
import { qs } from "@/api/chinhly";

export type KieuCot = "chu" | "so" | "ma" | "ngay" | "bool";
export type BoLocBaoCao = "phong" | "nam" | "thoi_gian";

export interface CotBaoCao {
  k: string;
  ten: string;
  kieu: KieuCot;
  rong: number;
}

export interface DinhNghiaBaoCao {
  ma: string;
  ten: string;
  canCu: string;
  nhom: string;
  ban: "ca_hai" | "thi_cong" | "luu_tru";
  boLoc: BoLocBaoCao[];
  cot: CotBaoCao[];
  tong: boolean;
}

export interface DanhMucBaoCao {
  nhom: { ma: string; ten: string }[];
  baoCao: DinhNghiaBaoCao[];
}

export type DongBaoCao = Record<string, unknown>;

export interface KetQuaBaoCao {
  def: DinhNghiaBaoCao;
  rows: DongBaoCao[];
  tong: string[] | null;
}

export interface ThamSoBaoCao {
  projectId: string;
  phongId?: string;
  namTu?: string | number;
  namDen?: string | number;
  /** yyyy-MM-dd (máy chủ nhận cả dd/MM/yyyy). */
  from?: string;
  to?: string;
}

const bcGet = <T>(path: string) => invoke<T>("bc_get", { path });

export const bcDanhMuc = () => bcGet<DanhMucBaoCao>("/bao-cao");
export const bcChay = (ma: string, p: ThamSoBaoCao) => bcGet<KetQuaBaoCao>(`/bao-cao/${ma}${qs({ ...p })}`);

/** Tải tệp máy chủ dựng (báo cáo, phiếu mượn, hồ sơ theo kho) — trả đường dẫn đã lưu, null nếu người dùng huỷ. */
export const bcDownload = (path: string, defaultName: string) => invoke<string | null>("bc_download", { path, defaultName });

export const bcTai = (ma: string, p: ThamSoBaoCao, format: "xlsx" | "pdf") =>
  bcDownload(`/bao-cao/${ma}${qs({ ...p, format })}`, `bao-cao-${ma.replace(/_/g, "-")}-${new Date().toISOString().slice(0, 10)}.${format}`);

/** Giá trị ô theo kiểu cột, cho bảng trên màn. */
export function oChu(c: CotBaoCao, v: unknown): string {
  if (v === null || v === undefined) return "";
  switch (c.kieu) {
    case "so": {
      const n = typeof v === "number" ? v : Number(v);
      if (Number.isNaN(n)) return String(v);
      return Number.isInteger(n) ? n.toLocaleString("vi-VN") : n.toLocaleString("vi-VN", { maximumFractionDigits: 1 });
    }
    case "ngay": {
      const s = String(v);
      const d = s.slice(0, 10);
      const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(d);
      return m ? `${m[3]}/${m[2]}/${m[1]}` : s;
    }
    case "bool":
      return v === true ? "Có" : "";
    default:
      return String(v);
  }
}
