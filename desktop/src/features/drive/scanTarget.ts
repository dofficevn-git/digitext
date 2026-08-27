// Copyright (c) 2026 DigiText. All rights reserved.

import type { V418 } from "@/api/archive";
import type { ScanTarget } from "@/state";

export function V47u(Vfs: V418): ScanTarget {
  return {
    phong: {
      maDuAn: Vfs.maDuAn,
      tenDuAn: Vfs.tenDuAn,
      phongSo: Vfs.phongSo,
      mucLucSo: Vfs.mucLucSo,
    },
    hoSo: {
      hopSo: Vfs.hopSo,
      hoSoSo: Vfs.hoSoSo,
      tieuDe: Vfs.tieuDe,
      nam: Vfs.nam,
      thoiGian: Vfs.thoiGian,
      ngayBatDau: Vfs.ngayBatDau,
      ngayKetThuc: Vfs.ngayKetThuc,
    },
    label: `${Vfs.tenDuAn}IMK3IEjhu5lwIA==${Vfs.hopSo} · HS ${Vfs.hoSoSo}`,
  };
}
