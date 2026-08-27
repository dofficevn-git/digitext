// Copyright (c) 2026 DigiText. All rights reserved.

import { useState } from "react";
import { V460 } from "@/state";
import type { V3xv } from "@/domain/types";
import { V48x } from "./useCatalog";
import { V46k } from "./catalogNav";
import { V49p } from "./tables/PhongTable";
import { V49q } from "./tables/PhongBanTable";
import { V49r } from "./tables/MangTable";
import { V49s } from "./tables/ThbqTable";
import { V49t } from "./tables/NgonNguTable";
import { V49u } from "./tables/LoaiTable";
import { HoSoTable } from "./tables/HoSoTable";

export default function V4bk() {
  const { data, setData: Vvc } = V48x();
  const { catalogTab: Vvd, gotoCatalog: Vve } = V460();

  const [Vvf, Vvg] = useState<string | null>(null);
  const [Vvh, Vvi] = useState<string | null>(null);

  const Vvj: Record<V3xv, number> = {
    phong: data.phongs.length,
    phongban: data.phongBans.length,
    mang: data.mangs.length,
    thbq: data.thbqs.length,
    ngonngu: data.ngonNgus.length,
    loai: data.loaiVanBans.length,
    hoso: data.hoSos.length,
  };

  const Vvk = (Vvl: string) => {
    Vvg(Vvl);
    Vve("phongban");
  };
  const Vvm = (Vvn: string) => {
    Vvi(Vvn);
    Vve("mang");
  };

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Danh mục: {V46k(Vvd)}</h2>
        <span className="sub">{Vvj[Vvd]} bản ghi · chọn danh mục ở thanh bên trái</span>
      </div>

      <div className="settings-grid solo">
        <div className="set-body">
          {Vvd === "phong" && <V49p data={data} setData={Vvc} onAddPhongBan={Vvk} />}
          {Vvd === "phongban" && (
            <V49q
              data={data}
              setData={Vvc}
              onAddMang={Vvm}
              seedPhong={Vvf}
              onConsumeSeed={() => Vvg(null)}
            />
          )}
          {Vvd === "mang" && (
            <V49r data={data} setData={Vvc} seedPB={Vvh} onConsumeSeed={() => Vvi(null)} />
          )}
          {Vvd === "thbq" && <V49s data={data} setData={Vvc} />}
          {Vvd === "ngonngu" && <V49t data={data} setData={Vvc} />}
          {Vvd === "loai" && <V49u data={data} setData={Vvc} />}
          {Vvd === "hoso" && <HoSoTable data={data} setData={Vvc} />}
        </div>
      </div>
    </section>
  );
}
