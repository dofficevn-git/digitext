// Copyright (c) 2026 DigiText. All rights reserved.

import { useState } from "react";
import V49a from "@/ui/DataTable";
import Modal, { V49j } from "@/ui/Modal";
import { isTauri } from "@/api/ipc";
import { V48p, V48o } from "@/api/catalog";
import { V48f } from "@/mocks/catalogFixtures";
import type { V3yn, V3yo } from "@/domain/types";
import { V49l, V49o, V49n, type V49k } from "./shared";

export function V49s({ data, setData: Vo4 }: V49k) {
  const [Vo5, Vo6] = useState<{
    mode: "add" | "edit";
    draft: V3yo;
    orig?: number;
  } | null>(null);
  const Vo7 = V49n("catalog.create", "catalog.update", "catalog.delete");
  const Vo8 = () => Math.max(0, ...data.thbqs.map((r) => r.thbqid)) + 1;

  const Vo9 = () => {
    if (!Vo5) return;
    if (!Vo5.draft.tenThbq.trim()) return;
    Vo4((s) =>
      Vo5.mode === "add"
        ? { ...s, thbqs: [...s.thbqs, Vo5.draft] }
        : { ...s, thbqs: s.thbqs.map((x) => (x.thbqid === Vo5.orig ? Vo5.draft : x)) },
    );
    if (isTauri()) V48o(Vo5.draft).catch(V49l);
    Vo6(null);
  };
  const Voa = (Vob: number) => {
    Vo4((s) => ({ ...s, thbqs: s.thbqs.filter((x) => x.thbqid !== Vob) }));
    if (isTauri()) V48p(Vob).catch(V49l);
  };
  const Voc = (Vod: (string | number)[]) => {
    Vo4((s) => ({ ...s, thbqs: s.thbqs.filter((x) => !Vod.includes(x.thbqid)) }));
    if (isTauri()) Vod.forEach((Voe) => V48p(Number(Voe)).catch(V49l));
  };

  return (
    <>
      <V49a<V3yo>
        addLabel="Thêm thời hạn"
        onAdd={
          !Vo7.canCreate
            ? undefined
            : () =>
                Vo6({
                  mode: "add",
                  draft: {
                    thbqid: Vo8(),
                    tenThbq: "",
                    trangThai: 2,
                    giaTri: 0,
                    sort: data.thbqs.length + 1,
                  },
                })
        }
        rows={data.thbqs}
        getId={(r) => r.thbqid}
        matches={(r, q) => r.tenThbq.toLowerCase().includes(q)}
        columns={[
          { head: "Mã THBQ (nhập HS tạm)", cell: (r) => r.thbqid, cls: "tnum" },
          { head: "Tên thời hạn bảo quản", cell: (r) => r.tenThbq },
          { head: "Giá trị (năm)", cell: (r) => r.giaTri, cls: "num" },
          {
            head: "Trạng thái",
            cell: (r) => <span className="badge gray">{V48f[r.trangThai]}</span>,
          },
          { head: "Thứ tự", cell: (r) => r.sort, cls: "num" },
        ]}
        rowActions={(r) =>
          V49o(
            () => Vo6({ mode: "edit", draft: { ...r }, orig: r.thbqid }),
            () => Voa(r.thbqid),
            undefined,
            Vo7,
          )
        }
        onDeleteMany={Vo7.canDelete ? Voc : undefined}
      />
      {Vo5 && (
        <Modal
          title={
            Vo5.mode === "add"
              ? "Thêm thời hạn bảo quản"
              : "Cập nhật thời hạn bảo quản"
          }
          onClose={() => Vo6(null)}
          onSave={Vo9}
        >
          <V49j label="Tên thời hạn bảo quản">
            <input
              className="inp"
              value={Vo5.draft.tenThbq}
              onChange={(Vof) =>
                Vo6({ ...Vo5, draft: { ...Vo5.draft, tenThbq: Vof.target.value } })
              }
            />
          </V49j>
          <V49j label="Giá trị (số năm)">
            <input
              className="inp"
              type="number"
              value={Vo5.draft.giaTri}
              onChange={(Vog) =>
                Vo6({ ...Vo5, draft: { ...Vo5.draft, giaTri: Number(Vog.target.value) } })
              }
            />
          </V49j>
          <V49j label="Trạng thái">
            <select
              className="inp"
              value={Vo5.draft.trangThai}
              onChange={(Voh) =>
                Vo6({
                  ...Vo5,
                  draft: { ...Vo5.draft, trangThai: Number(Voh.target.value) as V3yn },
                })
              }
            >
              <option value={1}>{V48f[1]}</option>
              <option value={2}>{V48f[2]}</option>
              <option value={3}>{V48f[3]}</option>
            </select>
          </V49j>
          <V49j label="Thứ tự">
            <input
              className="inp"
              type="number"
              value={Vo5.draft.sort}
              onChange={(Voi) =>
                Vo6({ ...Vo5, draft: { ...Vo5.draft, sort: Number(Voi.target.value) } })
              }
            />
          </V49j>
        </Modal>
      )}
    </>
  );
}
