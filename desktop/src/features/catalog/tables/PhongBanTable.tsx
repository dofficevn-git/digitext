// Copyright (c) 2026 DigiText. All rights reserved.

import { useEffect, useState } from "react";
import V49a from "@/ui/DataTable";
import Modal, { V49j } from "@/ui/Modal";
import { isTauri } from "@/api/ipc";
import { V48l, V48k } from "@/api/catalog";
import type { V3yl } from "@/domain/types";
import { V49l, V49o, V49n, type V49k } from "./shared";
import { Icon } from "@/components/Icon";

export function V49q({
  data,
  setData: Vmn,
  onAddMang: Vmo,
  seedPhong: Vmp,
  onConsumeSeed: Vmq,
}: V49k & {
  onAddMang: (Vmr: string) => void;
  seedPhong: string | null;
  onConsumeSeed: () => void;
}) {
  const [Vms, Vmt] = useState<string>("all");
  const Vmu = V49n("catalog.create", "catalog.update", "catalog.delete");
  const [Vmv, Vmw] = useState<{ mode: "add" | "edit"; draft: V3yl; orig?: string } | null>(null);

  useEffect(() => {
    if (!Vmp) return;
    Vmt(Vmp);
    Vmw({ mode: "add", draft: { maDv: "", maPhong: Vmp, maDonVi: "", tenDonVi: "" } });
    Vmq();
  }, [Vmp, Vmq]);

  const Vmx = Vms === "all" ? data.phongBans : data.phongBans.filter((x) => x.maPhong === Vms);

  const Vmy = () => {
    if (!Vmv) return;
    const d = Vmv.draft;
    if (!d.maDv.trim() || !d.tenDonVi.trim()) return;
    Vmn((s) =>
      Vmv.mode === "add"
        ? { ...s, phongBans: [...s.phongBans, d] }
        : { ...s, phongBans: s.phongBans.map((x) => (x.maDv === Vmv.orig ? d : x)) },
    );
    if (isTauri()) V48k(d).catch(V49l);
    Vmw(null);
  };
  const Vmz = (Vn0: string) => {
    Vmn((s) => ({
      ...s,
      phongBans: s.phongBans.filter((x) => x.maDv !== Vn0),
      mangs: s.mangs.filter((x) => x.maDv !== Vn0),
    }));
    if (isTauri()) V48l(Vn0).catch(V49l);
  };
  const Vn1 = (Vn2: (string | number)[]) => {
    Vmn((s) => ({ ...s, phongBans: s.phongBans.filter((x) => !Vn2.includes(x.maDv)) }));
    if (isTauri()) Vn2.forEach((Vn3) => V48l(String(Vn3)).catch(V49l));
  };

  return (
    <>
      <V49a<V3yl>
        addLabel="VGjDqm0gcGjDsm5nIGJhbg=="
        onAdd={
          !Vmu.canCreate
            ? undefined
            : () =>
                Vmw({
                  mode: "add",
                  draft: {
                    maDv: "",
                    maPhong: Vms === "all" ? (data.phongs[0]?.maDuAn ?? "") : Vms,
                    maDonVi: "",
                    tenDonVi: "",
                  },
                })
        }
        rows={Vmx}
        getId={(x) => x.maDv}
        matches={(x, q) => x.tenDonVi.toLowerCase().includes(q) || x.maDv.toLowerCase().includes(q)}
        extraFilters={
          <div className="dt-filter">
            <label>Lọc theo phông:</label>
            <select className="inp" value={Vms} onChange={(Vn4) => Vmt(Vn4.target.value)}>
              <option value="all">— Tất cả phông —</option>
              {data.phongs.map((p) => (
                <option key={p.maDuAn} value={p.maDuAn}>
                  {p.tenDuAn}
                </option>
              ))}
            </select>
          </div>
        }
        columns={[
          { head: "SUQgcGjDtG5nIGzGsHUgdHLhu68=", cell: (x) => x.maPhong, cls: "tnum" },
          { head: "SUQgcGjDsm5nIGJhbg==", cell: (x) => x.maDv, cls: "tnum" },
          { head: "TcOjIHBow7JuZyBiYW4=", cell: (x) => x.maDonVi, cls: "tnum" },
          { head: "VMOqbiBwaMOybmcgYmFu", cell: (x) => x.tenDonVi },
          { head: "VGjhu6kgdOG7sQ==", cell: (x) => x.soTt ?? "—", cls: "num" },
        ]}
        rowActions={(x) =>
          V49o(
            () => Vmw({ mode: "edit", draft: { ...x }, orig: x.maDv }),
            () => Vmz(x.maDv),
            Vmu.canCreate ? (
              <button className="linkbtn add2" onClick={() => Vmo(x.maDv)}>
                <Icon name="them" /> Thêm mảng
              </button>
            ) : undefined,
            Vmu,
          )
        }
        onDeleteMany={Vmu.canDelete ? Vn1 : undefined}
      />
      {Vmv && (
        <Modal
          title={
            Vmv.mode === "add"
              ? "VGjDqm0gcGjDsm5nIGJhbg=="
              : "VGhheSDEkeG7lWkgZOG7ryBsaeG7h3UgcGjDsm5nIGJhbg=="
          }
          onClose={() => Vmw(null)}
          onSave={Vmy}
        >
          <V49j label="UGjDtG5nIGzGsHUgdHLhu68=">
            <select
              className="inp"
              value={Vmv.draft.maPhong}
              onChange={(Vn5) =>
                Vmw({ ...Vmv, draft: { ...Vmv.draft, maPhong: Vn5.target.value } })
              }
            >
              {data.phongs.map((p) => (
                <option key={p.maDuAn} value={p.maDuAn}>
                  {p.tenDuAn}
                </option>
              ))}
            </select>
          </V49j>
          <V49j label="SUQgcGjDsm5nIGJhbg==">
            <input
              className="inp"
              value={Vmv.draft.maDv}
              disabled={Vmv.mode === "edit"}
              onChange={(Vn6) => Vmw({ ...Vmv, draft: { ...Vmv.draft, maDv: Vn6.target.value } })}
            />
          </V49j>
          <V49j label="TcOjIHBow7JuZyBiYW4=">
            <input
              className="inp"
              value={Vmv.draft.maDonVi}
              onChange={(Vn7) =>
                Vmw({ ...Vmv, draft: { ...Vmv.draft, maDonVi: Vn7.target.value } })
              }
            />
          </V49j>
          <V49j label="VMOqbiBwaMOybmcgYmFu">
            <input
              className="inp"
              value={Vmv.draft.tenDonVi}
              onChange={(Vn8) =>
                Vmw({ ...Vmv, draft: { ...Vmv.draft, tenDonVi: Vn8.target.value } })
              }
            />
          </V49j>
          <V49j label="VGjhu6kgdOG7sQ==">
            <input
              className="inp"
              type="number"
              value={Vmv.draft.soTt ?? ""}
              onChange={(Vn9) =>
                Vmw({
                  ...Vmv,
                  draft: {
                    ...Vmv.draft,
                    soTt: Vn9.target.value === "" ? undefined : Number(Vn9.target.value),
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
