// Copyright (c) 2026 DigiText. All rights reserved.
// Mượn – trả hồ sơ giấy có duyệt (0034, 27/11/2026, đợt 3 nhóm 4A): routes/chinh_ly/muon_tra.rs qua Tauri cl_remote.
// Dòng = một hồ sơ trên phiếu (snake_case từ cl_muon_tra + hồ sơ + tên người); nhiều dòng cùng nhom_id / so_phieu = một phiếu.

import { clRemote, qs, type ClRow } from "@/api/chinhly";
import { bcDownload } from "@/api/baoCao";

export type TrangThaiMuon = "cho_duyet" | "da_duyet" | "tu_choi" | "dang_muon" | "da_tra" | "huy";
export type LocMuon = TrangThaiMuon | "qua_han" | "mo" | "";

export type DongMuon = ClRow & {
  nhom_id?: string | null;
  so_phieu?: string | null;
  trang_thai: TrangThaiMuon;
  qua_han?: boolean;
  so_lan_gia_han?: number;
  han_tra_goc?: string | null;
  gia_han_log?: { tu?: string | null; den?: string; lyDo?: string | null; boiTen?: string; luc?: string }[];
};

export interface TongQuanMuon {
  choDuyet: number;
  phieuChoDuyet: number;
  choGiao: number;
  dangMuon: number;
  quaHan: number;
  sapHan: number;
  daTra30Ngay: number;
}

export interface LapPhieuBody {
  projectId: string;
  hoSoIds: string[];
  nguoiMuon: string;
  donVi?: string;
  dienThoai?: string;
  email?: string;
  noiDung?: string;
  ngayMuon?: string;
  ngayDuKienTra?: string;
  ghiChu?: string;
  giaoNgay?: boolean;
}

export const mtList = (projectId: string, trangThai: LocMuon, q: string, limit = 500) =>
  clRemote<{ rows: DongMuon[]; total: number }>("GET", `/cl/muon-tra${qs({ projectId, trangThai, q, limit })}`);
export const mtTongQuan = (projectId: string) => clRemote<TongQuanMuon>("GET", `/cl/muon-tra/tong-quan${qs({ projectId })}`);
export const mtLap = (b: LapPhieuBody) => clRemote<{ nhomId: string; soPhieu: string; rows: DongMuon[] }>("POST", "/cl/muon-tra", b);
export const mtNhom = (nhomId: string) => clRemote<{ rows: DongMuon[] }>("GET", `/cl/muon-tra/nhom/${nhomId}`);

export type BuocMuon = "duyet" | "tu-choi" | "giao" | "tra" | "huy";
export const mtBuoc = (id: string, buoc: BuocMuon, body?: { lyDo?: string; ngay?: string }) => clRemote<DongMuon>("POST", `/cl/muon-tra/${id}/${buoc}`, body ?? {});
export const mtGiaHan = (id: string, hanMoi: string, lyDo?: string) => clRemote<DongMuon>("POST", `/cl/muon-tra/${id}/gia-han`, { hanMoi, lyDo });
export const mtNhomBuoc = (nhomId: string, buoc: "duyet" | "tu-choi" | "huy", lyDo?: string) =>
  clRemote<{ daChuyen: number; rows: DongMuon[] }>("POST", `/cl/muon-tra/nhom/${nhomId}/${buoc}`, { lyDo });

export const mtPhieuPdf = (id: string, soPhieu: string | null | undefined) =>
  bcDownload(`/cl/muon-tra/${id}/phieu.pdf`, `phieu-muon-${(soPhieu ?? id).replace(/\//g, "-")}.pdf`);
