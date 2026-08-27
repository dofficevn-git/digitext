// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import V49a from "@/ui/DataTable";
import { V49o, V49n } from "@/features/catalog/tables/shared";
import { V4bz, V4c1, V4by, V4c0, type V4bm, type V4bp, type V4bq } from "@/api/dossier";
import { V3xq } from "@/api/ipc";
import Banner from "./Banner";
import V4ck from "./VanBanForm";
import { V4cy, V4cu, V4cw, V4cz, V4cx } from "./logic";

type V4dx = { mode: "add" | "edit"; draft: V4bp; id?: number } | null;

export default function V4dy({ dossier, onBack: Vz1 }: { dossier: V4bm; onBack: () => void }) {
  const Vz2 = V49n("dossier.update", "dossier.update", "dossier.update");
  const [Vz3, Vz4] = useState<V4bq[]>([]);
  const [Vz5, Vz6] = useState<string | null>(null);
  const [Vz7, Vz8] = useState<V4dx>(null);

  const reload = useCallback(() => {
    V4by(dossier.id)
      .then((rs) => {
        Vz4(rs);
        Vz6(null);
      })
      .catch((Vz9) => Vz6(Vz9 instanceof V3xq ? Vz9.message : String(Vz9)));
  }, [dossier.id]);

  V41p(reload, { topics: ["dossier"] });

  const Vza = (p: Partial<V4bp>) => Vz7 && Vz8({ ...Vz7, draft: { ...Vz7.draft, ...p } });

  const Vzb = async () => {
    if (!Vz7) return;
    const Vzc = V4cz(Vz7.draft);
    if (Vzc) {
      Vz6(Vzc);
      return;
    }
    try {
      const body = V4cy(Vz7.draft);
      if (Vz7.mode === "add") await V4bz(dossier.id, body);
      else if (Vz7.id != null) await V4c0(Vz7.id, body);
      Vz8(null);
      reload();
    } catch (Vzd) {
      Vz6(Vzd instanceof V3xq ? Vzd.message : String(Vzd));
    }
  };

  const Vze = async (Vzf: number) => {
    try {
      await V4c1(Vzf);
      reload();
    } catch (Vzg) {
      Vz6(Vzg instanceof V3xq ? Vzg.message : String(Vzg));
    }
  };

  const Vzh = async (Vzi: (string | number)[]) => {
    try {
      await Promise.all(Vzi.map((Vzj) => V4c1(Number(Vzj))));
      reload();
    } catch (Vzk) {
      Vz6(Vzk instanceof V3xq ? Vzk.message : String(Vzk));
    }
  };

  return (
    <>
      <div className="page-head mt-0">
        <h2>
          <button className="linkbtn" onClick={Vz1}>
            ← Danh sách hồ sơ
          </button>{" "}
          / Văn bản
        </h2>
        <span className="sub">{V4cu(dossier)}</span>
      </div>

      {Vz5 && <Banner tone="err" text={Vz5} onClose={() => Vz6(null)} />}

      <V49a<V4bq>
        addLabel="Thêm văn bản"
        onAdd={!Vz2.canCreate ? undefined : () => Vz8({ mode: "add", draft: V4cw() })}
        rows={Vz3}
        getId={(v) => v.id}
        matches={(v, q) =>
          [v.trichYeu, v.soKyHieu, v.coQuanBanHanh, v.loaiVanBan]
            .filter(Boolean)
            .join(" ")
            .toLowerCase()
            .includes(q)
        }
        columns={[
          { head: "STT", cell: (v) => v.thuTu, cls: "num" },
          { head: "Loại VB", cell: (v) => v.loaiVanBan },
          { head: "Số/ký hiệu", cell: (v) => v.soKyHieu, cls: "tnum" },
          { head: "Ngày tháng", cell: (v) => v.ngayThang },
          { head: "Cơ quan ban hành", cell: (v) => v.coQuanBanHanh },
          { head: "Trích yếu", cell: (v) => v.trichYeu },
          { head: "Số tờ", cell: (v) => v.toSo, cls: "num" },
          { head: "Số trang", cell: (v) => v.soTrang, cls: "num" },
        ]}
        rowActions={(v) =>
          V49o(
            () => Vz8({ mode: "edit", draft: V4cx(v), id: v.id }),
            () => void Vze(v.id),
            undefined,
            Vz2,
          )
        }
        onDeleteMany={Vz2.canDelete ? Vzh : undefined}
      />

      {Vz7 && (
        <V4ck
          mode={Vz7.mode}
          draft={Vz7.draft}
          onChange={Vza}
          onClose={() => Vz8(null)}
          onSave={() => void Vzb()}
        />
      )}
    </>
  );
}
