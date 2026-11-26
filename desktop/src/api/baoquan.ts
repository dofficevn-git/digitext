// Copyright (c) 2026 DigiText. All rights reserved.
// Bảo quản đủ Điều 29–31 TT 05/2025 (0032, 26/11/2026): kế hoạch kiểm tra có duyệt, danh mục lỗi và khắc phục, lịch sao lưu,
// phương tiện lưu trữ, sự cố. Tauri src-tauri/src/bao_quan.rs (bq_get / bq_call) → máy chủ routes/bao_quan.rs.
// Dòng dữ liệu là JSON thẳng từ bảng (snake_case) + vài trường tính thêm; tên trường camelCase ở `tep` và tình trạng lịch.

import { invoke } from "@/api/ipc";
import { qs } from "@/api/chinhly";
import type { NoiDungDien } from "@/features/records/RecordsView";

export const bqGet = <T>(path: string) => invoke<T>("bq_get", { path });
export const bqCall = <T>(method: "POST" | "PUT" | "DELETE", path: string, body?: unknown) => invoke<T>("bq_call", { method, path, body: body ?? null });

// ---------------------------------------------------------------- tổng quan

export interface TinhTrangLich {
  id: string;
  loai: "gia_tang" | "day_du" | "day_du_3_nam";
  ten: string;
  chuKyGio: number;
  doTreChoPhepGio: number;
  giuSoBan: number;
  bat: boolean;
  nguon: string | null;
  phuongTienId: string | null;
  phuongTienMa: string | null;
  ghiChu: string | null;
  lanDat: string | null;
  soBan: number;
  tinhTrang: "chua_co" | "dung_han" | "tre" | "tat";
  hanKeTiep: string | null;
  treGio: number;
  soBanCanXoa: number;
  banCuCanXoa: SaoLuuRun[];
}

export interface TongQuanBaoQuan {
  khChoDuyet: number;
  khDangChay: number;
  loiMo: number;
  suCoMo: number;
  lichTre: number;
  banCuCanXoa: number;
  phuongTienSapHet: number;
  lich: TinhTrangLich[];
}

export const bqTongQuan = () => bqGet<TongQuanBaoQuan>("/bao-quan/tong-quan");

// ---------------------------------------------------------------- kế hoạch kiểm tra (Điều 29)

export type TrangThaiKeHoach = "du_thao" | "cho_duyet" | "da_duyet" | "dang_thuc_hien" | "hoan_thanh" | "huy";

export interface PhamViKiemTra {
  phongIds?: string[];
  xacSuat?: number | null;
  lauChuaKiem?: boolean;
  goi?: boolean;
}

export interface KeHoach {
  id: string;
  project_id: string;
  nam: number;
  ma: string;
  ten: string;
  pham_vi: PhamViKiemTra;
  phuong_phap: string | null;
  trang_thai: TrangThaiKeHoach;
  nguoi_lap: string | null;
  trinh_luc: string | null;
  duyet_boi: string | null;
  duyet_luc: string | null;
  ly_do_tu_choi: string | null;
  bat_dau_luc: string | null;
  ket_thuc_luc: string | null;
  so_tep: number;
  so_dat: number;
  so_loi: number;
  record_id: string | null;
  ghi_chu: string | null;
  created_at: string;
  updated_at: string;
  nguoi_lap_ten: string | null;
  duyet_ten: string | null;
  so_cho: number;
  so_loi_mo: number;
}

export interface TepKeHoach {
  id: string;
  documentId: string | null;
  packageId: string | null;
  trangThai: "cho" | "dat" | "loi" | "bo_qua";
  chiTiet: string | null;
  kiemLuc: string | null;
  tenTep: string | null;
  maHoSo: string | null;
  objid: string | null;
  projectId: string | null;
}

export interface KeHoachBody {
  projectId?: string;
  nam?: number;
  ten: string;
  phamVi: PhamViKiemTra;
  phuongPhap?: string | null;
  ghiChu?: string | null;
}

export const khList = (projectId: string, nam?: number | null) => bqGet<{ rows: KeHoach[] }>(`/bao-quan/ke-hoach${qs({ projectId, nam })}`);
export const khGet = (id: string) => bqGet<KeHoach>(`/bao-quan/ke-hoach/${id}`);
export const khTao = (b: KeHoachBody) => bqCall<KeHoach>("POST", "/bao-quan/ke-hoach", b);
export const khSua = (id: string, b: KeHoachBody) => bqCall<KeHoach>("PUT", `/bao-quan/ke-hoach/${id}`, b);
export const khXemTruoc = (projectId: string, phamVi: PhamViKiemTra) => bqCall<{ taiLieu: number; goi: number; uocLuong: number; moTa: string }>("POST", "/bao-quan/ke-hoach/xem-truoc", { projectId, phamVi });
export const khTrinh = (id: string) => bqCall<KeHoach>("POST", `/bao-quan/ke-hoach/${id}/trinh`);
export const khDuyet = (id: string) => bqCall<KeHoach>("POST", `/bao-quan/ke-hoach/${id}/duyet`);
export const khTuChoi = (id: string, lyDo: string) => bqCall<KeHoach>("POST", `/bao-quan/ke-hoach/${id}/tu-choi`, { lyDo });
export const khHuy = (id: string) => bqCall<KeHoach>("POST", `/bao-quan/ke-hoach/${id}/huy`);
export const khChay = (id: string, limit = 100) => bqCall<{ daKiem: number; loi: number; hoanThanh: boolean }>("POST", `/bao-quan/ke-hoach/${id}/chay${qs({ limit })}`);
export const khGanBienBan = (id: string, recordId: string) => bqCall<KeHoach>("POST", `/bao-quan/ke-hoach/${id}/bien-ban`, { recordId });
export const khTep = (id: string, trangThai?: string, limit = 200, offset = 0) => bqGet<{ rows: TepKeHoach[]; total: number }>(`/bao-quan/ke-hoach/${id}/tep${qs({ trangThai, limit, offset })}`);
export const khKetLuan = (id: string) => bqGet<NoiDungDien & { keHoachId: string }>(`/bao-quan/ke-hoach/${id}/ket-luan`);

// ---------------------------------------------------------------- danh mục lỗi

export type TrangThaiLoi = "moi" | "da_khac_phuc" | "kiem_lai_dat" | "kiem_lai_loi" | "bo_qua";

export interface LoiBaoQuan {
  id: string;
  project_id: string;
  ke_hoach_id: string | null;
  document_id: string | null;
  package_id: string | null;
  chi_tiet: string | null;
  phat_hien_luc: string;
  trang_thai: TrangThaiLoi;
  khac_phuc: string | null;
  khac_phuc_boi: string | null;
  khac_phuc_luc: string | null;
  kiem_lai_luc: string | null;
  ten_tep: string | null;
  ma_ho_so: string | null;
  objid: string | null;
  khac_phuc_ten: string | null;
  ke_hoach_ma: string | null;
}

export const loiList = (p: { projectId?: string | null; keHoachId?: string | null; mo?: boolean; limit?: number; offset?: number }) => bqGet<{ rows: LoiBaoQuan[] }>(`/bao-quan/loi${qs(p)}`);
export const loiKhacPhuc = (id: string, khacPhuc: string) => bqCall<{ ok: true }>("POST", `/bao-quan/loi/${id}/khac-phuc`, { khacPhuc });
export const loiKiemLai = (id: string) => bqCall<{ ketQua: "dat" | "loi"; chiTiet: string | null }>("POST", `/bao-quan/loi/${id}/kiem-lai`);
export const loiBoQua = (id: string, lyDo: string) => bqCall<{ ok: true }>("POST", `/bao-quan/loi/${id}/bo-qua`, { lyDo });

// ---------------------------------------------------------------- sao lưu (Điều 30)

export interface SaoLuuRun {
  id: string;
  loai: "gia_tang" | "day_du" | "day_du_3_nam";
  nguon: string;
  phuong_tien: string | null;
  ket_qua: "dat" | "khong_dat";
  chi_tiet: string | null;
  reported_by: string | null;
  created_at: string;
  lich_id: string | null;
  phuong_tien_id: string | null;
  duong_dan: string | null;
  dung_luong_bytes: number | null;
  ma_bam: string | null;
  bat_dau_at: string | null;
  ket_thuc_at: string | null;
  nguoi_thuc_hien: string | null;
  da_xoa_at: string | null;
  lich_ten: string | null;
  phuong_tien_ma: string | null;
  reported_ten: string | null;
}

export interface LichBody {
  loai?: SaoLuuRun["loai"];
  ten: string;
  chuKyGio: number;
  doTreChoPhepGio: number;
  giuSoBan: number;
  bat: boolean;
  nguon?: string | null;
  phuongTienId?: string | null;
  ghiChu?: string | null;
}

export const lichList = () => bqGet<{ rows: TinhTrangLich[] }>("/bao-quan/lich");
export const lichTao = (b: LichBody) => bqCall<{ id: string }>("POST", "/bao-quan/lich", b);
export const lichSua = (id: string, b: LichBody) => bqCall<{ ok: true }>("PUT", `/bao-quan/lich/${id}`, b);
export const saoLuuRuns = (lichId?: string | null, limit = 200, offset = 0) => bqGet<{ rows: SaoLuuRun[] }>(`/bao-quan/sao-luu${qs({ lichId, limit, offset })}`);
export const saoLuuGhi = (b: {
  loai: SaoLuuRun["loai"];
  nguon: string;
  phuongTien?: string | null;
  ketQua: "dat" | "khong_dat";
  chiTiet?: string | null;
  lichId?: string | null;
  phuongTienId?: string | null;
  duongDan?: string | null;
  dungLuongBytes?: number | null;
  maBam?: string | null;
  batDauAt?: string | null;
  ketThucAt?: string | null;
  nguoiThucHien?: string | null;
}) => bqCall<SaoLuuRun>("POST", "/bao-quan/sao-luu", b);
export const saoLuuDaXoa = (runId: string) => bqCall<{ ok: true }>("POST", `/bao-quan/sao-luu/${runId}/da-xoa`);

// ---------------------------------------------------------------- phương tiện (Điều 30.4)

export interface PhuongTien {
  id: string;
  ma: string;
  loai: string;
  mo_ta: string | null;
  dung_luong_bytes: number | null;
  noi_bao_quan: string | null;
  ngay_dua_vao: string | null;
  han_do_ben: string | null;
  trang_thai: "dang_dung" | "da_thay" | "hong" | "loai_bo";
  thay_boi_id: string | null;
  nhac_luc: string | null;
  ghi_chu: string | null;
  created_at: string;
  so_lan_dung: number;
  thay_boi_ma: string | null;
  con_ngay: number | null;
  sap_het: boolean;
}

export interface PhuongTienBody {
  ma: string;
  loai: string;
  moTa?: string | null;
  dungLuongBytes?: number | null;
  noiBaoQuan?: string | null;
  ngayDuaVao?: string | null;
  hanDoBen?: string | null;
  trangThai?: PhuongTien["trang_thai"];
  thayBoiId?: string | null;
  ghiChu?: string | null;
}

export const ptList = () => bqGet<{ rows: PhuongTien[] }>("/bao-quan/phuong-tien");
export const ptTao = (b: PhuongTienBody) => bqCall<PhuongTien>("POST", "/bao-quan/phuong-tien", b);
export const ptSua = (id: string, b: PhuongTienBody) => bqCall<PhuongTien>("PUT", `/bao-quan/phuong-tien/${id}`, b);

// ---------------------------------------------------------------- sự cố (Điều 31)

export type MucDoSuCo = "thap" | "trung_binh" | "cao" | "nghiem_trong";
export type TrangThaiSuCo = "moi" | "dang_xu_ly" | "da_phuc_hoi" | "dong";

export interface SuCo {
  id: string;
  ma: string;
  project_id: string | null;
  thoi_diem: string;
  mo_ta: string;
  nguyen_nhan: string | null;
  muc_do: MucDoSuCo;
  pham_vi: string | null;
  bien_phap: string | null;
  backup_run_id: string | null;
  phuong_phap_phuc_hoi: string | null;
  phuc_hoi_luc: string | null;
  ket_qua_kiem_tra: string | null;
  du_lieu_mat: string | null;
  trang_thai: TrangThaiSuCo;
  nguoi_xu_ly: string | null;
  record_id: string | null;
  created_at: string;
  nguoi_xu_ly_ten: string | null;
  project_code: string | null;
  backup_run: { id: string; loai: string; nguon: string; created_at: string; duong_dan: string | null } | null;
  so_bien_ban: string | null;
}

export interface SuCoBody {
  projectId?: string | null;
  thoiDiem?: string | null;
  moTa: string;
  nguyenNhan?: string | null;
  mucDo?: MucDoSuCo;
  phamVi?: string | null;
  bienPhap?: string | null;
  backupRunId?: string | null;
  phuongPhapPhucHoi?: string | null;
  phucHoiLuc?: string | null;
  ketQuaKiemTra?: string | null;
  duLieuMat?: string | null;
  trangThai?: TrangThaiSuCo;
  nguoiXuLy?: string | null;
}

export const scList = (mo = false, limit = 200) => bqGet<{ rows: SuCo[] }>(`/bao-quan/su-co${qs({ mo, limit })}`);
export const scGet = (id: string) => bqGet<SuCo>(`/bao-quan/su-co/${id}`);
export const scTao = (b: SuCoBody) => bqCall<SuCo>("POST", "/bao-quan/su-co", b);
export const scSua = (id: string, b: SuCoBody) => bqCall<SuCo>("PUT", `/bao-quan/su-co/${id}`, b);
export const scPrefill = (id: string) => bqGet<NoiDungDien & { suCoId: string }>(`/bao-quan/su-co/${id}/prefill`);
export const scGanBienBan = (id: string, recordId: string) => bqCall<SuCo>("POST", `/bao-quan/su-co/${id}/bien-ban`, { recordId });
