// Copyright (c) 2026 DigiText. All rights reserved.
// Lưu trữ hiện hành — Điều 19 TT 05/2025 (26/11/2026): nộp hồ sơ vào lưu trữ cơ quan và danh mục hồ sơ dự kiến.
// Tauri src-tauri/src/luu_tru.rs (lt_get / lt_call) → máy chủ routes/nop_luu.rs, routes/danh_muc_ho_so.rs.

import { invoke } from "@/api/ipc";
import { qs } from "@/api/chinhly";

export const ltGet = <T>(path: string) => invoke<T>("lt_get", { path });
export const ltCall = <T>(method: "POST" | "PUT" | "DELETE", path: string, body?: unknown) => invoke<T>("lt_call", { method, path, body: body ?? null });

// ---------------------------------------------------------------- nộp lưu

export interface TongQuanNopLuu {
  denHan: number;
  quaHan: number;
  chuaNop: number;
  phieuChoTiepNhan: number;
  quyenSapHet: number;
}

export type TrangThaiPhieu = "cho_tiep_nhan" | "da_tiep_nhan" | "tra_lai" | "rut";

/** Phiếu nộp lưu (JSON thẳng từ bảng lt_nop_luu + tên phòng ban / người). */
export interface PhieuNopLuu {
  id: string;
  project_id: string;
  phong_id: string;
  phong_ban_id: string | null;
  ma_phieu: string;
  nguoi_nop: string | null;
  ghi_chu: string | null;
  trang_thai: TrangThaiPhieu;
  tiep_nhan_boi: string | null;
  tiep_nhan_luc: string | null;
  ly_do_tra: string | null;
  so_ho_so: number;
  so_van_ban: number | null;
  so_trang: number | null;
  bien_ban_id: string | null;
  created_at: string;
  updated_at: string;
  ten_phong_ban: string | null;
  ma_don_vi: string | null;
  ma_phong: string;
  ten_phong: string;
  nguoi_nop_ten: string | null;
  tiep_nhan_ten: string | null;
  so_nhan: number;
  so_tra_lai: number;
  so_bien_ban: string | null;
}

export interface HoSoTrongPhieu {
  id: string;
  hoSoId: string;
  stt: number | null;
  ketQua: "cho" | "nhan" | "tra_lai";
  lyDo: string | null;
  soTam: string;
  hoSoSo: string | null;
  maHoSo: string | null;
  tieuDe: string;
  ngayBatDau: string | null;
  ngayKetThuc: string | null;
  soTo: string | null;
  tenThbq: string | null;
  soVanBan: number;
  tenPhongBan: string | null;
  nopLuuTrangThai: string | null;
  khoaLuc: string | null;
}

/** Dòng view cl_ho_so_v + hạn nộp (yyyy-mm-dd) + số ngày còn (âm = quá hạn). */
export type HoSoCanNop = Record<string, unknown> & {
  id: string;
  so_tam: string;
  tieu_de: string;
  ngay_bat_dau: string | null;
  ngay_ket_thuc: string | null;
  ten_phong_ban: string | null;
  ten_thbq: string | null;
  so_van_ban: number;
  han_nop: string | null;
  con_ngay: number | null;
  nop_luu_trang_thai: string | null;
};

export const nopLuuTongQuan = (projectId: string, phongBanId?: string | null) => ltGet<TongQuanNopLuu>(`/nop-luu/tong-quan${qs({ projectId, phongBanId })}`);
export const nopLuuCanNop = (p: { projectId: string; phongBanId?: string | null; loc?: "den_han" | "qua_han" | ""; limit?: number; offset?: number }) =>
  ltGet<{ rows: HoSoCanNop[]; total: number }>(`/nop-luu/ho-so-can-nop${qs(p)}`);
export const nopLuuList = (p: { projectId: string; trangThai?: TrangThaiPhieu | ""; phongBanId?: string | null; nam?: number; limit?: number; offset?: number }) =>
  ltGet<{ rows: PhieuNopLuu[]; total: number }>(`/nop-luu${qs(p)}`);
export const nopLuuGet = (id: string) => ltGet<{ phieu: PhieuNopLuu; hoSo: HoSoTrongPhieu[] }>(`/nop-luu/${id}`);
export const nopLuuLap = (b: { projectId: string; phongBanId?: string | null; hoSoIds: string[]; ghiChu?: string | null }) => ltCall<PhieuNopLuu>("POST", "/nop-luu", b);
export const nopLuuRut = (id: string) => ltCall<PhieuNopLuu>("POST", `/nop-luu/${id}/rut`);
export const nopLuuCham = (id: string, hoSoId: string, ketQua: "nhan" | "tra_lai" | "cho", lyDo?: string | null) =>
  ltCall<{ hoSo: HoSoTrongPhieu[] }>("POST", `/nop-luu/${id}/ho-so/${hoSoId}/ket-qua`, { ketQua, lyDo });
export const nopLuuTiepNhan = (id: string, ghiChu?: string | null) => ltCall<{ phieu: PhieuNopLuu; hoSo: HoSoTrongPhieu[] }>("POST", `/nop-luu/${id}/tiep-nhan`, { ghiChu });
export const nopLuuTraLai = (id: string, lyDo: string) => ltCall<{ phieu: PhieuNopLuu; hoSo: HoSoTrongPhieu[] }>("POST", `/nop-luu/${id}/tra-lai`, { lyDo });
export const nopLuuBienBan = (id: string) => ltCall<{ id: string; soBienBan: string }>("POST", `/nop-luu/${id}/bien-ban`);
export const nopLuuMoKhoa = (id: string, hoSoId: string, lyDo: string) => ltCall<{ hoSo: HoSoTrongPhieu[] }>("POST", `/nop-luu/${id}/mo-khoa`, { hoSoId, lyDo });
export const nopLuuSo = (projectId: string, nam?: number, phongBanId?: string | null) => ltGet<{ nam: number; rows: PhieuNopLuu[] }>(`/nop-luu/so-nop-luu${qs({ projectId, nam, phongBanId })}`);
export const nopLuuSoBienBan = (projectId: string, nam: number, phongBanId?: string | null) => ltCall<{ id: string; soBienBan: string }>("POST", "/nop-luu/so-nop-luu/bien-ban", { projectId, nam, phongBanId });

// ---------------------------------------------------------------- danh mục hồ sơ (Điều 19.4)

export type TrangThaiDanhMuc = "du_thao" | "cho_duyet" | "da_duyet" | "ban_hanh" | "het_hieu_luc";

export interface DanhMuc {
  id: string;
  project_id: string;
  phong_id: string;
  nam: number;
  phien_ban: number;
  trang_thai: TrangThaiDanhMuc;
  so_quyet_dinh: string | null;
  ngay_ban_hanh: string | null;
  trinh_boi: string | null;
  trinh_luc: string | null;
  duyet_boi: string | null;
  duyet_luc: string | null;
  ban_hanh_boi: string | null;
  ban_hanh_luc: string | null;
  ly_do_tra: string | null;
  ghi_chu: string | null;
  created_by: string | null;
  created_at: string;
  updated_at: string;
  ma_phong: string;
  ten_phong: string;
  so_dong: number;
  so_phong_ban: number;
  trinh_ten: string | null;
  duyet_ten: string | null;
  ban_hanh_ten: string | null;
}

export interface DongDanhMuc {
  id: string;
  danh_muc_id: string;
  phong_ban_id: string;
  mang_id: string | null;
  ma_dong: string;
  tieu_de: string;
  thbq_id: string | null;
  nguoi_lap: string | null;
  ghi_chu: string | null;
  so_tt: number | null;
  tien_than_id: string | null;
  ten_phong_ban: string | null;
  ma_don_vi: string | null;
  ten_mang: string | null;
  ten_thbq: string | null;
  so_ho_so: number;
}

export interface DongMoi {
  phongBanId: string;
  mangId?: string | null;
  maDong: string;
  tieuDe: string;
  thbqId?: string | null;
  nguoiLap?: string | null;
  ghiChu?: string | null;
  soTt?: number | null;
}

export interface DongHieuLuc {
  id: string;
  maDong: string;
  tieuDe: string;
  phongBanId: string;
  mangId: string | null;
  thbqId: string | null;
}

export interface DoiChieu {
  theoDanhMuc: { dongId: string; maDong: string; tieuDe: string; phongBanId: string; tenPhongBan: string | null; soHoSo: number }[];
  ngoaiDanhMuc: { id: string; soTam: string; tieuDe: string; phongBanId: string | null; tenPhongBan: string | null; ngayKetThuc: string | null; nopLuuTrangThai: string | null }[];
}

export const dmList = (projectId: string, phongId?: string | null, nam?: number | null) => ltGet<{ rows: DanhMuc[] }>(`/danh-muc-ho-so${qs({ projectId, phongId, nam })}`);
export const dmGet = (id: string) => ltGet<{ danhMuc: DanhMuc; dong: DongDanhMuc[]; phongBanCuaToi: string | null }>(`/danh-muc-ho-so/${id}`);
export const dmHieuLuc = (phongId: string, nam: number, phongBanId?: string | null) => ltGet<{ rows: DongHieuLuc[] }>(`/danh-muc-ho-so/hieu-luc${qs({ phongId, nam, phongBanId })}`);
export const dmTao = (b: { projectId: string; phongId: string; nam: number; saoChepTu?: string | null }) => ltCall<DanhMuc>("POST", "/danh-muc-ho-so", b);
export const dmThemDong = (id: string, d: DongMoi) => ltCall<{ id: string; dong: DongDanhMuc[] }>("POST", `/danh-muc-ho-so/${id}/dong`, d);
export const dmSuaDong = (id: string, dongId: string, d: DongMoi) => ltCall<{ dong: DongDanhMuc[] }>("PUT", `/danh-muc-ho-so/${id}/dong/${dongId}`, d);
export const dmXoaDong = (id: string, dongId: string) => ltCall<{ dong: DongDanhMuc[] }>("DELETE", `/danh-muc-ho-so/${id}/dong/${dongId}`);
export const dmNhapDong = (id: string, rows: DongMoi[]) => ltCall<{ them: number; sua: number; loi: { dong: number; loi: string }[]; dong: DongDanhMuc[] }>("POST", `/danh-muc-ho-so/${id}/dong/nhap`, { rows });
export const dmTrinh = (id: string) => ltCall<DanhMuc>("POST", `/danh-muc-ho-so/${id}/trinh`);
export const dmDuyet = (id: string) => ltCall<DanhMuc>("POST", `/danh-muc-ho-so/${id}/duyet`);
export const dmTraLai = (id: string, lyDo: string) => ltCall<DanhMuc>("POST", `/danh-muc-ho-so/${id}/tra-lai`, { lyDo });
export const dmBanHanh = (id: string, soQuyetDinh: string, ngayBanHanh: string) => ltCall<DanhMuc>("POST", `/danh-muc-ho-so/${id}/ban-hanh`, { soQuyetDinh, ngayBanHanh });
export const dmPhienBanMoi = (id: string) => ltCall<DanhMuc>("POST", `/danh-muc-ho-so/${id}/phien-ban-moi`);
export const dmDoiChieu = (id: string) => ltGet<DoiChieu>(`/danh-muc-ho-so/${id}/doi-chieu`);
export const dmBienBan = (id: string) => ltCall<{ id: string; soBienBan: string }>("POST", `/danh-muc-ho-so/${id}/bien-ban`);
