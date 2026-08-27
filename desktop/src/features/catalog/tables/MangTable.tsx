// Copyright (c) 2026 DigiText. All rights reserved.

import { useEffect, useState } from "react";
import V49a from "@/ui/DataTable";
import Modal, { V49j } from "@/ui/Modal";
import { isTauri } from "@/api/ipc";
import { V48n, V48m } from "@/api/catalog";
import type { V3ym } from "@/domain/types";
import { V49l, V49o, V49n, type V49k } from "./shared";

export function V49r({
  data,
  setData: Vna,
  seedPB: Vnb,
  onConsumeSeed: Vnc,
}: V49k & { seedPB: string | null; onConsumeSeed: () => void }) {
  const [Vnd, Vne] = useState<string>("all");
  const Vnf = V49n("catalog.create", "catalog.update", "catalog.delete");
  const [Vng, Vnh] = useState<string>("all");
  const [Vni, Vnj] = useState<{
    mode: "add" | "edit";
    draft: V3ym;
    orig?: number;
  } | null>(null);

  useEffect(() => {
    if (!Vnb) return;
    const Vnk = data.phongBans.find((x) => x.maDv === Vnb);
    if (Vnk) {
      Vne(Vnk.maPhong);
      Vnh(Vnb);
    }
    Vnj({
      mode: "add",
      draft: { mangCvid: Date.now(), maMang: "", tenMang: "", maDv: Vnb },
    });
    Vnc();
  }, [Vnb, data.phongBans, Vnc]);

  const Vnl = (Vnm: string) => data.phongBans.find((x) => x.maDv === Vnm)?.tenDonVi ?? Vnm;
  const Vnn = Vnd === "all" ? data.phongBans : data.phongBans.filter((x) => x.maPhong === Vnd);
  const Vno = data.mangs.filter((m) => {
    if (Vng !== "all") return m.maDv === Vng;
    if (Vnd !== "all") return Vnn.some((Vnp) => Vnp.maDv === m.maDv);
    return true;
  });

  const Vnq = () => {
    if (!Vni) return;
    const d = Vni.draft;
    if (!d.tenMang.trim() || !d.maDv) return;
    Vna((s) =>
      Vni.mode === "add"
        ? { ...s, mangs: [...s.mangs, d] }
        : { ...s, mangs: s.mangs.map((x) => (x.mangCvid === Vni.orig ? d : x)) },
    );
    if (isTauri()) V48m(d).catch(V49l);
    Vnj(null);
  };
  const Vnr = (Vns: number) => {
    Vna((s) => ({ ...s, mangs: s.mangs.filter((x) => x.mangCvid !== Vns) }));
    if (isTauri()) V48n(Vns).catch(V49l);
  };
  const Vnt = (Vnu: (string | number)[]) => {
    Vna((s) => ({ ...s, mangs: s.mangs.filter((x) => !Vnu.includes(x.mangCvid)) }));
    if (isTauri()) Vnu.forEach((Vnv) => V48n(Number(Vnv)).catch(V49l));
  };

  return (
    <>
      <V49a<V3ym>
        addLabel="Thêm mảng"
        onAdd={
          !Vnf.canCreate
            ? undefined
            : () =>
                Vnj({
                  mode: "add",
                  draft: {
                    mangCvid: Date.now(),
                    maMang: "",
                    tenMang: "",
                    maDv: Vng !== "all" ? Vng : (Vnn[0]?.maDv ?? ""),
                  },
                })
        }
        rows={Vno}
        getId={(m) => m.mangCvid}
        matches={(m, q) =>
          m.tenMang.toLowerCase().includes(q) || m.maMang.toLowerCase().includes(q)
        }
        extraFilters={
          <div className="dt-filter">
            <label>Phông:</label>
            <select
              className="inp"
              value={Vnd}
              onChange={(Vnw) => {
                Vne(Vnw.target.value);
                Vnh("all");
              }}
            >
              <option value="all">— Tất cả —</option>
              {data.phongs.map((p) => (
                <option key={p.maDuAn} value={p.maDuAn}>
                  {p.tenDuAn}
                </option>
              ))}
            </select>
            <label>Phòng ban:</label>
            <select className="inp" value={Vng} onChange={(Vnx) => Vnh(Vnx.target.value)}>
              <option value="all">— Tất cả —</option>
              {Vnn.map((Vny) => (
                <option key={Vny.maDv} value={Vny.maDv}>
                  {Vny.tenDonVi}
                </option>
              ))}
            </select>
          </div>
        }
        columns={[
          { head: "ID Mảng", cell: (m) => m.mangCvid, cls: "tnum" },
          { head: "Mã mảng", cell: (m) => m.maMang, cls: "tnum" },
          { head: "Tên mảng công việc", cell: (m) => m.tenMang },
          { head: "Tên phòng ban", cell: (m) => Vnl(m.maDv) },
          { head: "Thứ tự", cell: (m) => m.soTt ?? "—", cls: "num" },
        ]}
        rowActions={(m) =>
          V49o(
            () => Vnj({ mode: "edit", draft: { ...m }, orig: m.mangCvid }),
            () => Vnr(m.mangCvid),
            undefined,
            Vnf,
          )
        }
        onDeleteMany={Vnf.canDelete ? Vnt : undefined}
      />
      {Vni && (
        <Modal
          title={
            Vni.mode === "add"
              ? "Thêm mảng công việc"
              : "Thay đổi dữ liệu mảng công việc"
          }
          onClose={() => Vnj(null)}
          onSave={Vnq}
        >
          <V49j label="Phòng ban">
            <select
              className="inp"
              value={Vni.draft.maDv}
              onChange={(Vnz) => Vnj({ ...Vni, draft: { ...Vni.draft, maDv: Vnz.target.value } })}
            >
              {data.phongBans.map((Vo0) => (
                <option key={Vo0.maDv} value={Vo0.maDv}>
                  {Vo0.tenDonVi} ({Vo0.maPhong})
                </option>
              ))}
            </select>
          </V49j>
          <V49j label="Mã mảng">
            <input
              className="inp"
              value={Vni.draft.maMang}
              onChange={(Vo1) => Vnj({ ...Vni, draft: { ...Vni.draft, maMang: Vo1.target.value } })}
            />
          </V49j>
          <V49j label="Tên mảng công việc">
            <input
              className="inp"
              value={Vni.draft.tenMang}
              onChange={(Vo2) =>
                Vnj({ ...Vni, draft: { ...Vni.draft, tenMang: Vo2.target.value } })
              }
            />
          </V49j>
          <V49j label="Thứ tự">
            <input
              className="inp"
              type="number"
              value={Vni.draft.soTt ?? ""}
              onChange={(Vo3) =>
                Vnj({
                  ...Vni,
                  draft: {
                    ...Vni.draft,
                    soTt: Vo3.target.value === "" ? undefined : Number(Vo3.target.value),
                  },
                })
              }
            />
          </V49j>
        </Modal>
      )}
    </>
  );
}
