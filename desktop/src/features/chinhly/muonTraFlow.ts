// Copyright (c) 2026 DigiText. All rights reserved.
// Nhãn, màu, bước và hành động theo trạng thái của mượn – trả có duyệt (0034). Bảng chuyển chép đúng
// digitext_core::chinh_ly::muon_tra: cho_duyet → da_duyet | tu_choi | huy; da_duyet → dang_muon | huy; dang_muon → da_tra (+ gia hạn).

import type { V3y0 } from "@/domain/types";
import type { DongMuon, TrangThaiMuon } from "@/api/muonTra";
import type { StepItem } from "@/ui";
import type { V41v as IconName } from "@/components/Icon";

export const TT_MUON: Record<TrangThaiMuon, { label: string; tone: V3y0; step: number }> = {
  cho_duyet: { label: "Chờ duyệt", tone: "warn", step: 1 },
  da_duyet: { label: "Chờ giao", tone: "info", step: 2 },
  tu_choi: { label: "Từ chối", tone: "err", step: 1 },
  dang_muon: { label: "Đang mượn", tone: "run", step: 3 },
  da_tra: { label: "Đã trả", tone: "ok", step: 4 },
  huy: { label: "Đã huỷ", tone: "gray", step: 0 },
};

export const BUOC_MUON: StepItem[] = [
  { key: "lap", label: "Lập phiếu" },
  { key: "duyet", label: "Duyệt" },
  { key: "giao", label: "Giao hồ sơ" },
  { key: "tra", label: "Trả" },
];

export const GIA_HAN_TOI_DA = 2;

export interface HanhDongMuon {
  key: "duyet" | "tu-choi" | "giao" | "gia-han" | "tra" | "huy" | "duyet-phieu" | "tu-choi-phieu";
  label: string;
  icon: IconName;
  danger?: boolean;
}

/** Hành động trên MỘT dòng theo trạng thái và quyền; `nhieuDongCho` = phiếu còn ≥ 2 dòng chờ duyệt → thêm duyệt / từ chối cả phiếu. */
export function hanhDongMuon(r: DongMuon, q: { manage: boolean; approve: boolean }, nhieuDongCho: boolean): HanhDongMuon[] {
  const out: HanhDongMuon[] = [];
  switch (r.trang_thai) {
    case "cho_duyet":
      if (q.approve) {
        out.push({ key: "duyet", label: "Duyệt", icon: "duyet" }, { key: "tu-choi", label: "Từ chối", icon: "tuChoi", danger: true });
        if (nhieuDongCho) out.push({ key: "duyet-phieu", label: "Duyệt cả phiếu", icon: "duyet" }, { key: "tu-choi-phieu", label: "Từ chối cả phiếu", icon: "tuChoi", danger: true });
      }
      if (q.manage) out.push({ key: "huy", label: "Huỷ phiếu", icon: "daHuy", danger: true });
      break;
    case "da_duyet":
      if (q.manage) out.push({ key: "giao", label: "Giao hồ sơ", icon: "xuatTep" }, { key: "huy", label: "Huỷ phiếu", icon: "daHuy", danger: true });
      break;
    case "dang_muon":
      if (q.manage) {
        out.push({ key: "tra", label: "Ghi trả", icon: "nhapKho" });
        if ((r.so_lan_gia_han ?? 0) < GIA_HAN_TOI_DA) out.push({ key: "gia-han", label: "Gia hạn", icon: "nhacHan" });
      }
      break;
    default:
      break;
  }
  return out;
}

export const homNay = () => new Date().toISOString().slice(0, 10);
export const congNgay = (iso: string, n: number) => {
  const d = new Date(iso + "T00:00:00");
  d.setDate(d.getDate() + n);
  return d.toISOString().slice(0, 10);
};
