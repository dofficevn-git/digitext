// Copyright (c) 2026 DigiText. All rights reserved.

import type { V40d } from "@/api/archive";
import type { V3yr } from "@/domain/types";

export function V48w(r: V40d): V3yr {
  return {
    id: r.id,
    hopSo: r.hopSo,
    hoSoSo: r.hoSoSo,
    tieuDe: r.tieuDe,
    thoiGian: r.thoiGian ?? "",
    soTo: r.soTo ?? "",
    thbq: r.thbq ?? "",
    ghiChu: r.ghiChu || undefined,
    nhom: r.nhom || undefined,
    nam: r.nam || undefined,
    pdfPath: r.pdfPath || undefined,
    pageCount: r.pageCount ?? undefined,
    status: r.status === "done" ? "done" : "pending",
    ngayBatDau: r.ngayBatDau || undefined,
    ngayKetThuc: r.ngayKetThuc || undefined,
  };
}
