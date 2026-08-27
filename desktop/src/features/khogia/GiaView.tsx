// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import V49a from "@/ui/DataTable";
import Modal, { V49j } from "@/ui/Modal";
import { isTauri, V3xq } from "@/api/ipc";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { V48j } from "@/api/catalog";
import { V40w, type V40v } from "@/api/archive";
import type { V3yl } from "@/domain/types";
import { V4lz, V4lx, V4lv, V4ls, V4lw, type V4lp, type V4lr, type V4lo } from "@/api/khaithac";
import { V4nj, V4nl, V4nh } from "./khoGiaLogic";
import { Icon } from "@/components/Icon";

interface V4np {
  mode: "add" | "edit";
  draft: V4lr;
}

function V4nq(V20c: unknown): void {
  const V20d = V20c instanceof V3xq || V20c instanceof Error ? V20c.message : String(V20c);
  window.alert(`TMawdSBnacOhIGzGsHUgdHLhu68gdGjhuqV0IGLhuqFpOiA=${V20d}`);
}

export default function V4nr() {
  const { session: V20e } = V460();
  const p = V20e?.permissions;
  const V20f = V3z9(p, "catalog.create");
  const V20g = V3z9(p, "catalog.update");
  const V20h = V3z9(p, "catalog.delete");

  const [V20i, V20j] = useState<V4lp[]>([]);
  const [V20k, V20l] = useState<V4lo[]>([]);
  const [V20m, V20n] = useState<V40v[]>([]);
  const [V20o, V20p] = useState<V3yl[]>([]);
  const [V20q, V20r] = useState<number | undefined>(undefined);
  const [V20s, V20t] = useState<V4np | null>(null);
  const [V20u, V20v] = useState<string | null>(null);

  const reload = useCallback(() => {
    if (!isTauri()) return;
    V4lv(V20q)
      .then(V20j)
      .catch((V20w) => V20v(V20w instanceof Error ? V20w.message : String(V20w)));
  }, [V20q]);
  V41p(reload, { topics: ["khogia"] });

  const V20x = useCallback(() => {
    V4ls()
      .then(V20l)
      .catch(() => {});
    V40w()
      .then(V20n)
      .catch(() => {});
    V48j()
      .then((c) => V20p(c.phongBans))
      .catch(() => {});
  }, []);
  V41p(V20x, { topics: ["khogia", "catalog", "hoso"], enabled: isTauri() });

  const V20y = () =>
    V20t({
      mode: "add",
      draft: { tenGia: "", maKho: V20q ?? V20k[0]?.maKho ?? 0 },
    });
  const V20z = (g: V4lp) =>
    V20t({
      mode: "edit",
      draft: {
        maGia: g.maGia,
        tenGia: g.tenGia,
        maKho: g.maKho,
        maDa: g.maDa,
        maDv: g.maDv,
        tuHop: g.tuHop,
        denHop: g.denHop,
        ghiChu: g.ghiChu ?? "",
        soTt: g.soTt,
      },
    });

  const V210 = async () => {
    if (!V20s) return;
    const V211 = V4nh(V20s.draft);
    if (V211) {
      window.alert(V211);
      return;
    }
    const V212: V4lr = {
      ...V20s.draft,
      maDa: V20s.draft.maDa || undefined,
      maDv: V20s.draft.maDv || undefined,
      ghiChu: V20s.draft.ghiChu?.trim() || undefined,
    };
    try {
      if (isTauri()) {
        await V4lw(V212);
        reload();
        V4ls()
          .then(V20l)
          .catch(() => {});
      } else {
        const V213 = V20k.find((k) => k.maKho === V212.maKho)?.tenKho;
        const V214 = V20m.find((V215) => V215.maDuAn === V212.maDa)?.tenDuAn;
        const V216 = V20o.find((b) => b.maDv === V212.maDv)?.tenDonVi;
        V20j((s) =>
          V20s.mode === "add"
            ? [
                ...s,
                {
                  ...V212,
                  maGia: Date.now(),
                  tenKho: V213,
                  tenDuAn: V214,
                  tenDonVi: V216,
                  soHoSo: 0,
                },
              ]
            : s.map((x) =>
                x.maGia === V212.maGia
                  ? { ...x, ...V212, tenKho: V213, tenDuAn: V214, tenDonVi: V216 }
                  : x,
              ),
        );
      }
      V20t(null);
    } catch (V217) {
      V4nq(V217);
    }
  };

  const V218 = V3z9(p, "dossier.update");
  const V219 = async (g: V4lp) => {
    if (g.tuHop == null || g.denHop == null) {
      window.alert(
        "R2nDoSBuw6B5IGNoxrBhIG5o4bqtcCBk4bqjaSBo4buZcCAoVOG7qyBo4buZcCAvIMSQ4bq/biBo4buZcCkgbsOqbiBjaMawYSB0aOG7gyB04buxIHjhur9wIGjhu5Mgc8ahLg==",
      );
      return;
    }
    if (
      !window.confirm(
        `VOG7sSB44bq/cCBjw6FjIGjhu5Mgc8ahIGPDsyBo4buZcCBz4buRIHRyb25nIFs=${g.tuHop} – ${g.denHop}XSB2w6BvIGdpw6EgIg==${g.tenGia}"?`,
      )
    )
      return;
    try {
      if (isTauri()) {
        const n = await V4lz(g.maGia);
        reload();
        window.alert(`xJDDoyB44bq/cCA=${n}IGjhu5Mgc8ahIHbDoG8gZ2nDoSAi${g.tenGia}".`);
      }
    } catch (V21a) {
      V4nq(V21a);
    }
  };

  const V21b = async (g: V4lp) => {
    const V21c =
      g.soHoSo > 0
        ? ` ${g.soHoSo}IGjhu5Mgc8ahIMSRYW5nIHjhur9wIOG7nyBnacOhIG7DoHkgc+G6vSDEkcaw4bujYyBn4buhIHbhu4EgImNoxrBhIHjhur9wIi4=`
        : "";
    if (!window.confirm(`WG/DoSBnacOhICI=${g.tenGia}"?${V21c}IEtow7RuZyB0aOG7gyBob8OgbiB0w6FjLg==`))
      return;
    try {
      if (isTauri()) {
        await V4lx(g.maGia);
        reload();
      } else {
        V20j((s) => s.filter((x) => x.maGia !== g.maGia));
      }
    } catch (V21d) {
      V4nq(V21d);
    }
  };

  const V21e = V20k.map((k) => (
    <option key={k.maKho} value={k.maKho}>
      {k.tenKho}
    </option>
  ));

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Giá lưu trữ</h2>
        <p className="muted">
          Danh mục giá/kệ trong kho — mỗi giá thuộc một phông + phòng ban và giữ một dải hộp.
        </p>
      </div>
      {V20u && (
        <div className="err-banner" role="alert">
          Không tải được danh sách giá: {V20u}
        </div>
      )}
      <V49a<V4lp>
        addLabel="VGjDqm0gZ2nDoQ=="
        onAdd={V20f ? V20y : undefined}
        rows={V20i}
        getId={(r) => r.maGia}
        matches={V4nj}
        extraFilters={
          <select
            className="inp max-w-55"
            value={V20q ?? ""}
            onChange={(V21f) => V20r(V21f.target.value ? Number(V21f.target.value) : undefined)}
          >
            <option value="">— Tất cả kho —</option>
            {V21e}
          </select>
        }
        columns={[
          { head: "UGjDtG5n", cell: (r) => r.tenDuAn ?? r.maDa ?? "" },
          { head: "UGjDsm5nIGJhbg==", cell: (r) => r.tenDonVi ?? "" },
          { head: "VMOqbiBraG8=", cell: (r) => r.tenKho ?? "" },
          { head: "VMOqbiBnacOh", cell: (r) => r.tenGia },
          { head: "VOG7qyBo4buZcA==", cell: (r) => r.tuHop ?? "", cls: "tnum" },
          { head: "xJDhur9uIGjhu5lw", cell: (r) => r.denHop ?? "", cls: "tnum" },
          { head: "U+G7kSBo4buTIHPGoQ==", cell: (r) => r.soHoSo, cls: "tnum" },
        ]}
        rowActions={(r) => (
          <>
            {V218 && (
              <button
                className="linkbtn"
                onClick={() => V219(r)}
                title="VOG7sSB44bq/cCBo4buTIHPGoSB0aGVvIGThuqNpIGjhu5lw"
              >
                <Icon name="nhapKho" /> Xếp hồ sơ
              </button>
            )}
            {V20g && (
              <button className="linkbtn edit" onClick={() => V20z(r)}>
                <Icon name="sua" /> Sửa
              </button>
            )}
            {V20h && (
              <button className="linkbtn del" onClick={() => V21b(r)}>
                <Icon name="xoa" /> Xóa
              </button>
            )}
          </>
        )}
      />
      {V20s && (
        <Modal
          title={
            V20s.mode === "add"
              ? "VGjDqm0gZ2nDoSBsxrB1IHRy4buv"
              : "Q+G6rXAgbmjhuq10IGdpw6EgbMawdSB0cuG7rw=="
          }
          onClose={() => V20t(null)}
          onSave={V210}
        >
          <V49j label="S2hvIGzGsHUgdHLhu68=">
            <select
              className="inp"
              value={V20s.draft.maKho || ""}
              onChange={(V21g) =>
                V20t({ ...V20s, draft: { ...V20s.draft, maKho: Number(V21g.target.value) } })
              }
            >
              <option value="">— Chọn kho —</option>
              {V21e}
            </select>
          </V49j>
          <V49j label="UGjDtG5nIGzGsHUgdHLhu68=">
            <select
              className="inp"
              value={V20s.draft.maDa ?? ""}
              onChange={(V21h) =>
                V20t({
                  ...V20s,
                  draft: { ...V20s.draft, maDa: V21h.target.value || undefined, maDv: undefined },
                })
              }
            >
              <option value="">— Không gán —</option>
              {V20m.map((V21i) => (
                <option key={V21i.maDuAn} value={V21i.maDuAn}>
                  {V21i.tenDuAn}
                </option>
              ))}
            </select>
          </V49j>
          <V49j label="UGjDsm5nIGJhbg==">
            <select
              className="inp"
              value={V20s.draft.maDv ?? ""}
              onChange={(V21j) =>
                V20t({ ...V20s, draft: { ...V20s.draft, maDv: V21j.target.value || undefined } })
              }
            >
              <option value="">— Không gán —</option>
              {V20o.filter((b) => !V20s.draft.maDa || b.maPhong === V20s.draft.maDa).map((b) => (
                <option key={b.maDv} value={b.maDv}>
                  {b.tenDonVi}
                </option>
              ))}
            </select>
          </V49j>
          <V49j label="VMOqbiBnacOh">
            <input
              className="inp"
              value={V20s.draft.tenGia}
              onChange={(V21k) =>
                V20t({ ...V20s, draft: { ...V20s.draft, tenGia: V21k.target.value } })
              }
            />
          </V49j>
          <V49j label="VOG7qyBo4buZcA==">
            <input
              className="inp"
              type="number"
              value={V20s.draft.tuHop ?? ""}
              onChange={(V21l) =>
                V20t({ ...V20s, draft: { ...V20s.draft, tuHop: V4nl(V21l.target.value) } })
              }
            />
          </V49j>
          <V49j label="xJDhur9uIGjhu5lw">
            <input
              className="inp"
              type="number"
              value={V20s.draft.denHop ?? ""}
              onChange={(V21m) =>
                V20t({ ...V20s, draft: { ...V20s.draft, denHop: V4nl(V21m.target.value) } })
              }
            />
          </V49j>
          <V49j label="R2hpIGNow7o=">
            <input
              className="inp"
              value={V20s.draft.ghiChu ?? ""}
              onChange={(V21n) =>
                V20t({ ...V20s, draft: { ...V20s.draft, ghiChu: V21n.target.value } })
              }
            />
          </V49j>
          <V49j label="U+G7kSB0aOG7qSB04bux">
            <input
              className="inp"
              type="number"
              value={V20s.draft.soTt ?? ""}
              onChange={(V21o) =>
                V20t({ ...V20s, draft: { ...V20s.draft, soTt: V4nl(V21o.target.value) } })
              }
            />
          </V49j>
        </Modal>
      )}
    </section>
  );
}
