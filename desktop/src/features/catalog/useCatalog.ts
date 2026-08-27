// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import { V48g, V48h } from "@/mocks/catalogFixtures";
import type { V3ys, V3yj, V3yk } from "@/domain/types";
import { isTauri } from "@/api/ipc";
import { listHoSo, V40w } from "@/api/archive";
import { V48j } from "@/api/catalog";
import { V48w } from "./hoSoMapping";

export function V48x() {
  const [data, Vk9] = useState<V3ys>(isTauri() ? V48h : V48g);

  const Vka = useCallback(async () => {
    try {
      const [Vkb, Vkc, Vkd] = await Promise.all([V40w(), V48j(), listHoSo()]);
      const Vke: V3yk[] = Vkb.map((p) => ({
        maDuAn: p.maDuAn,
        tenDuAn: p.tenDuAn,
        maKhoi: (p.maKhoi === "DANG" ? "DANG" : "NN") as V3yj,
        phongSo: p.phongSo,
        mucLucSo: p.mucLucSo,
      }));
      Vk9((s) => ({
        ...s,
        phongs: Vke,
        phongBans: Vkc.phongBans,
        mangs: Vkc.mangs,
        thbqs: Vkc.thbqs,
        ngonNgus: Vkc.ngonNgus,
        loaiVanBans: Vkc.loaiVanBans,
        hoSos: Vkd.map(V48w),
      }));
    } catch {}
  }, []);

  V41p(Vka, { topics: ["catalog", "hoso", "tree"], enabled: isTauri() });

  return { data, setData: Vk9 };
}
