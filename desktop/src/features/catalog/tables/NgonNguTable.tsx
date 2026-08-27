// Copyright (c) 2026 DigiText. All rights reserved.

import { useState } from "react";
import V49a from "@/ui/DataTable";
import Modal, { V49j } from "@/ui/Modal";
import { isTauri } from "@/api/ipc";
import { V48t, V48s } from "@/api/catalog";
import type { V3yp } from "@/domain/types";
import { V49l, V49o, V49n, type V49k } from "./shared";

export function V49t({ data, setData: Voj }: V49k) {
  const [Vok, Vol] = useState<{ mode: "add" | "edit"; draft: V3yp; orig?: string } | null>(null);
  const Vom = V49n("catalog.create", "catalog.update", "catalog.delete");
  const Von = () => {
    if (!Vok || !Vok.draft.maNgonNgu.trim() || !Vok.draft.tenNgonNgu.trim()) return;
    const d = Vok.draft;
    Voj((s) =>
      Vok.mode === "add"
        ? { ...s, ngonNgus: [...s.ngonNgus, d] }
        : { ...s, ngonNgus: s.ngonNgus.map((x) => (x.maNgonNgu === Vok.orig ? d : x)) },
    );
    if (isTauri()) V48s(d).catch(V49l);
    Vol(null);
  };
  const Voo = (Vop: string) => {
    Voj((s) => ({ ...s, ngonNgus: s.ngonNgus.filter((x) => x.maNgonNgu !== Vop) }));
    if (isTauri()) V48t(Vop).catch(V49l);
  };
  const Voq = (Vor: (string | number)[]) => {
    Voj((s) => ({ ...s, ngonNgus: s.ngonNgus.filter((x) => !Vor.includes(x.maNgonNgu)) }));
    if (isTauri()) Vor.forEach((Vos) => V48t(String(Vos)).catch(V49l));
  };

  return (
    <>
      <V49a<V3yp>
        addLabel="Thêm ngôn ngữ"
        onAdd={
          !Vom.canCreate
            ? undefined
            : () => Vol({ mode: "add", draft: { maNgonNgu: "", tenNgonNgu: "" } })
        }
        rows={data.ngonNgus}
        getId={(r) => r.maNgonNgu}
        matches={(r, q) =>
          r.tenNgonNgu.toLowerCase().includes(q) || r.maNgonNgu.toLowerCase().includes(q)
        }
        columns={[
          { head: "Mã ngôn ngữ", cell: (r) => r.maNgonNgu, cls: "tnum" },
          { head: "Tên ngôn ngữ", cell: (r) => r.tenNgonNgu },
        ]}
        rowActions={(r) =>
          V49o(
            () => Vol({ mode: "edit", draft: { ...r }, orig: r.maNgonNgu }),
            () => Voo(r.maNgonNgu),
            undefined,
            Vom,
          )
        }
        onDeleteMany={Vom.canDelete ? Voq : undefined}
      />
      {Vok && (
        <Modal
          title={
            Vok.mode === "add" ? "Thêm ngôn ngữ" : "Cập nhật ngôn ngữ"
          }
          onClose={() => Vol(null)}
          onSave={Von}
        >
          <V49j label="Mã ngôn ngữ">
            <input
              className="inp"
              value={Vok.draft.maNgonNgu}
              disabled={Vok.mode === "edit"}
              onChange={(Vot) =>
                Vol({ ...Vok, draft: { ...Vok.draft, maNgonNgu: Vot.target.value } })
              }
            />
          </V49j>
          <V49j label="Tên ngôn ngữ">
            <input
              className="inp"
              value={Vok.draft.tenNgonNgu}
              onChange={(Vou) =>
                Vol({ ...Vok, draft: { ...Vok.draft, tenNgonNgu: Vou.target.value } })
              }
            />
          </V49j>
        </Modal>
      )}
    </>
  );
}
