// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import { V460 } from "@/state";
import { isTauri } from "@/api/ipc";
import { V40w, type V40v } from "@/api/archive";
import LapHoSoView from "@/features/chinhly/LapHoSoView";
import DongBoView from "@/features/chinhly/DongBoView";
import DanhMucView from "@/features/chinhly/DanhMucView";
import HeThongHoaPane from "@/features/chinhly/HeThongHoaView";
import SoHoaPane from "./SoHoaPane";
import OcrPane from "./OcrPane";

export default function V4lc() {
  const { hoSoPane: V1ug } = V460();
  const [V1uh, V1ui] = useState<V40v[]>([]);

  const V1uj = useCallback(() => {
    V40w()
      .then(V1ui)
      .catch(() => {});
  }, []);
  V41p(V1uj, { topics: ["catalog", "hoso"], enabled: isTauri() });

  return (
    <section className="view active">
      {V1ug === "danhmuccl" && <DanhMucView />}
      {V1ug === "laphoso" && <LapHoSoView />}
      {V1ug === "hethonghoa" && <HeThongHoaPane />}
      {V1ug === "dongbo" && <DongBoView />}
      {V1ug === "sohoa" && <SoHoaPane phongs={V1uh} />}
      {V1ug === "ocr" && <OcrPane />}
    </section>
  );
}
