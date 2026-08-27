// Copyright (c) 2026 DigiText. All rights reserved.

import { useState } from "react";
import V49a from "@/ui/DataTable";
import Modal, { V49j } from "@/ui/Modal";
import { isTauri } from "@/api/ipc";
import { V48r, V48q } from "@/api/catalog";
import type { V3yj, V3yk } from "@/domain/types";
import { V49l, V49o, V49n, type V49k } from "./shared";
import { Icon } from "@/components/Icon";

export function V49p({
  data,
  setData: Vm5,
  onAddPhongBan: Vm6,
}: V49k & { onAddPhongBan: (Vm7: string) => void }) {
  const [Vm8, Vm9] = useState<{ mode: "add" | "edit"; draft: V3yk; orig?: string } | null>(null);
  const Vma = V49n("catalog.create", "catalog.update", "catalog.delete");

  const Vmb = () => {
    if (!Vm8) return;
    const d = Vm8.draft;
    if (!d.maDuAn.trim() || !d.tenDuAn.trim()) return;
    Vm5((s) =>
      Vm8.mode === "add"
        ? { ...s, phongs: [...s.phongs, d] }
        : { ...s, phongs: s.phongs.map((p) => (p.maDuAn === Vm8.orig ? d : p)) },
    );
    if (isTauri()) V48q(d).catch(V49l);
    Vm9(null);
  };
  const Vmc = (Vmd: string) => {
    Vm5((s) => {
      const Vme = s.phongBans.filter((x) => x.maPhong === Vmd).map((x) => x.maDv);
      return {
        ...s,
        phongs: s.phongs.filter((p) => p.maDuAn !== Vmd),
        phongBans: s.phongBans.filter((x) => x.maPhong !== Vmd),
        mangs: s.mangs.filter((x) => !Vme.includes(x.maDv)),
      };
    });
    if (isTauri()) V48r(Vmd).catch(V49l);
  };
  const Vmf = (Vmg: (string | number)[]) => {
    Vm5((s) => ({ ...s, phongs: s.phongs.filter((p) => !Vmg.includes(p.maDuAn)) }));
    if (isTauri()) Vmg.forEach((Vmh) => V48r(String(Vmh)).catch(V49l));
  };

  return (
    <>
      <V49a<V3yk>
        addLabel="VGjDqm0gcGjDtG5n"
        onAdd={
          !Vma.canCreate
            ? undefined
            : () => Vm9({ mode: "add", draft: { maDuAn: "", tenDuAn: "", maKhoi: "NN" } })
        }
        rows={data.phongs}
        getId={(p) => p.maDuAn}
        matches={(p, q) =>
          p.tenDuAn.toLowerCase().includes(q) || p.maDuAn.toLowerCase().includes(q)
        }
        columns={[
          { head: "SUQgcGjDtG5nIGzGsHUgdHLhu68=", cell: (p) => p.maDuAn, cls: "tnum" },
          { head: "TcOjIHBow7RuZyBsxrB1IHRy4buv", cell: (p) => p.maDuAn, cls: "tnum" },
          {
            head: "VMOqbiBwaMO0bmcgbMawdSB0cuG7rw==",
            cell: (p) => (
              <>
                {p.tenDuAn} {p.isTest && <span className="badge warn">test</span>}
              </>
            ),
          },
          { head: "UGjDtG5nIHPhu5E=", cell: (p) => p.phongSo || "—" },
          { head: "TeG7pWMgbOG7pWMgc+G7kQ==", cell: (p) => p.mucLucSo || "—" },
        ]}
        rowActions={(p) =>
          V49o(
            () => Vm9({ mode: "edit", draft: { ...p }, orig: p.maDuAn }),
            () => Vmc(p.maDuAn),
            Vma.canCreate ? (
              <button className="linkbtn add2" onClick={() => Vm6(p.maDuAn)}>
                <Icon name="them" /> Thêm phòng
              </button>
            ) : undefined,
            Vma,
          )
        }
        onDeleteMany={Vma.canDelete ? Vmf : undefined}
      />
      {Vm8 && (
        <Modal
          title={
            Vm8.mode === "add"
              ? "VGjDqm0gcGjDtG5nIGzGsHUgdHLhu68="
              : "VGhheSDEkeG7lWkgZOG7ryBsaeG7h3UgcGjDtG5nIGzGsHUgdHLhu68="
          }
          onClose={() => Vm9(null)}
          onSave={Vmb}
        >
          <V49j label="TcOjIHBow7RuZyBsxrB1IHRy4buv">
            <input
              className="inp"
              value={Vm8.draft.maDuAn}
              disabled={Vm8.mode === "edit"}
              onChange={(Vmi) => Vm9({ ...Vm8, draft: { ...Vm8.draft, maDuAn: Vmi.target.value } })}
            />
          </V49j>
          <V49j label="VMOqbiBwaMO0bmcgbMawdSB0cuG7rw==">
            <input
              className="inp"
              value={Vm8.draft.tenDuAn}
              onChange={(Vmj) =>
                Vm9({ ...Vm8, draft: { ...Vm8.draft, tenDuAn: Vmj.target.value } })
              }
            />
          </V49j>
          <V49j label="S2jhu5Fp">
            <select
              className="inp"
              value={Vm8.draft.maKhoi}
              onChange={(Vmk) =>
                Vm9({ ...Vm8, draft: { ...Vm8.draft, maKhoi: Vmk.target.value as V3yj } })
              }
            >
              <option value="DANG">Khối Đảng</option>
              <option value="NN">Khối Nhà nước</option>
            </select>
          </V49j>
          <V49j label="UGjDtG5nIHPhu5E=">
            <input
              className="inp"
              value={Vm8.draft.phongSo ?? ""}
              onChange={(Vml) =>
                Vm9({ ...Vm8, draft: { ...Vm8.draft, phongSo: Vml.target.value } })
              }
            />
          </V49j>
          <V49j label="TeG7pWMgbOG7pWMgc+G7kQ==">
            <input
              className="inp"
              value={Vm8.draft.mucLucSo ?? ""}
              onChange={(Vmm) =>
                Vm9({ ...Vm8, draft: { ...Vm8.draft, mucLucSo: Vmm.target.value } })
              }
            />
          </V49j>
        </Modal>
      )}
    </>
  );
}
