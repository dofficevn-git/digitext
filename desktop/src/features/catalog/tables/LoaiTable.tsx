// Copyright (c) 2026 DigiText. All rights reserved.

import { useState } from "react";
import V49a from "@/ui/DataTable";
import Modal, { V49j } from "@/ui/Modal";
import { isTauri } from "@/api/ipc";
import { V48v, V48u } from "@/api/catalog";
import type { V3yq } from "@/domain/types";
import { V49l, V49o, V49n, type V49k } from "./shared";

export function V49u({ data, setData: Vov }: V49k) {
  const [Vow, Vox] = useState<{
    mode: "add" | "edit";
    draft: V3yq;
    orig?: string;
  } | null>(null);
  const Voy = V49n("catalog.create", "catalog.update", "catalog.delete");
  const Voz = () => {
    if (!Vow || !Vow.draft.maLoai.trim() || !Vow.draft.tenLoai.trim()) return;
    const d = Vow.draft;
    Vov((s) =>
      Vow.mode === "add"
        ? { ...s, loaiVanBans: [...s.loaiVanBans, d] }
        : { ...s, loaiVanBans: s.loaiVanBans.map((x) => (x.maLoai === Vow.orig ? d : x)) },
    );
    if (isTauri()) V48u(d).catch(V49l);
    Vox(null);
  };
  const Vp0 = (Vp1: string) => {
    Vov((s) => ({ ...s, loaiVanBans: s.loaiVanBans.filter((x) => x.maLoai !== Vp1) }));
    if (isTauri()) V48v(Vp1).catch(V49l);
  };
  const Vp2 = (Vp3: (string | number)[]) => {
    Vov((s) => ({ ...s, loaiVanBans: s.loaiVanBans.filter((x) => !Vp3.includes(x.maLoai)) }));
    if (isTauri()) Vp3.forEach((Vp4) => V48v(String(Vp4)).catch(V49l));
  };

  return (
    <>
      <V49a<V3yq>
        addLabel="VGjDqm0gbG/huqFpIHTDoGkgbGnhu4d1"
        onAdd={
          !Voy.canCreate
            ? undefined
            : () => Vox({ mode: "add", draft: { maLoai: "", tenLoai: "" } })
        }
        rows={data.loaiVanBans}
        getId={(r) => r.maLoai}
        matches={(r, q) =>
          r.tenLoai.toLowerCase().includes(q) || r.maLoai.toLowerCase().includes(q)
        }
        columns={[
          { head: "TcOjIGxv4bqhaSB0w6BpIGxp4buHdQ==", cell: (r) => r.maLoai, cls: "tnum" },
          { head: "VMOqbiBsb+G6oWkgdMOgaSBsaeG7h3U=", cell: (r) => r.tenLoai },
        ]}
        rowActions={(r) =>
          V49o(
            () => Vox({ mode: "edit", draft: { ...r }, orig: r.maLoai }),
            () => Vp0(r.maLoai),
            undefined,
            Voy,
          )
        }
        onDeleteMany={Voy.canDelete ? Vp2 : undefined}
      />
      {Vow && (
        <Modal
          title={
            Vow.mode === "add"
              ? "VGjDqm0gbG/huqFpIHTDoGkgbGnhu4d1"
              : "Q+G6rXAgbmjhuq10IGxv4bqhaSB2xINuIGLhuqNu"
          }
          onClose={() => Vox(null)}
          onSave={Voz}
        >
          <V49j label="TcOjIGxv4bqhaSB0w6BpIGxp4buHdQ==">
            <input
              className="inp"
              value={Vow.draft.maLoai}
              disabled={Vow.mode === "edit"}
              onChange={(Vp5) => Vox({ ...Vow, draft: { ...Vow.draft, maLoai: Vp5.target.value } })}
            />
          </V49j>
          <V49j label="VMOqbiBsb+G6oWkgdMOgaSBsaeG7h3U=">
            <input
              className="inp"
              value={Vow.draft.tenLoai}
              onChange={(Vp6) =>
                Vox({ ...Vow, draft: { ...Vow.draft, tenLoai: Vp6.target.value } })
              }
            />
          </V49j>
        </Modal>
      )}
    </>
  );
}
