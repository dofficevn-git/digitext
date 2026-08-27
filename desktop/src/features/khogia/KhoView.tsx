// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import V49a from "@/ui/DataTable";
import Modal, { V49j } from "@/ui/Modal";
import { isTauri, V3xq } from "@/api/ipc";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { V4lu, V4ls, V4lt, type V4lo, type V4lq } from "@/api/khaithac";
import { V4ni, V4nl, V4ng } from "./khoGiaLogic";
import { Icon } from "@/components/Icon";

interface V4nm {
  mode: "add" | "edit";
  draft: V4lq;
}

function V4nn(V1zo: unknown): void {
  const V1zp = V1zo instanceof V3xq || V1zo instanceof Error ? V1zo.message : String(V1zo);
  window.alert(`TMawdSBraG8gbMawdSB0cuG7ryB0aOG6pXQgYuG6oWk6IA==${V1zp}`);
}

export default function V4no() {
  const { session: V1zq } = V460();
  const p = V1zq?.permissions;
  const V1zr = V3z9(p, "catalog.create");
  const V1zs = V3z9(p, "catalog.update");
  const V1zt = V3z9(p, "catalog.delete");

  const [V1zu, V1zv] = useState<V4lo[]>([]);
  const [V1zw, V1zx] = useState<V4nm | null>(null);
  const [V1zy, V1zz] = useState<string | null>(null);

  const reload = useCallback(() => {
    if (!isTauri()) return;
    V4ls()
      .then(V1zv)
      .catch((V200) => V1zz(V200 instanceof Error ? V200.message : String(V200)));
  }, []);

  V41p(reload, { topics: ["khogia"] });

  const V201 = () => V1zx({ mode: "add", draft: { tenKho: "", ghiChu: "", soTt: undefined } });
  const V202 = (k: V4lo) =>
    V1zx({
      mode: "edit",
      draft: { maKho: k.maKho, tenKho: k.tenKho, ghiChu: k.ghiChu ?? "", soTt: k.soTt },
    });

  const V203 = async () => {
    if (!V1zw) return;
    const V204 = V4ng(V1zw.draft);
    if (V204) {
      window.alert(V204);
      return;
    }
    try {
      if (isTauri()) {
        await V4lt({
          ...V1zw.draft,
          ghiChu: V1zw.draft.ghiChu?.trim() || undefined,
        });
        reload();
      } else {
        V1zv((s) =>
          V1zw.mode === "add"
            ? [
                ...s,
                {
                  maKho: Date.now(),
                  tenKho: V1zw.draft.tenKho,
                  ghiChu: V1zw.draft.ghiChu,
                  soTt: V1zw.draft.soTt,
                  soGia: 0,
                },
              ]
            : s.map((x) => (x.maKho === V1zw.draft.maKho ? { ...x, ...V1zw.draft } : x)),
        );
      }
      V1zx(null);
    } catch (V205) {
      V4nn(V205);
    }
  };

  const V206 = async (k: V4lo) => {
    const V207 =
      k.soGia > 0
        ? `IEtobyBuw6B5IGPDsyA=${k.soGia}IGdpw6Eg4oCUIHhvw6Egc+G6vSB4b8OhIGPhuqMgY8OhYyBnacOhIMSRw7Mu`
        : "";
    if (!window.confirm(`WG/DoSBraG8gIg==${k.tenKho}"?${V207}IEtow7RuZyB0aOG7gyBob8OgbiB0w6FjLg==`))
      return;
    try {
      if (isTauri()) {
        await V4lu(k.maKho);
        reload();
      } else {
        V1zv((s) => s.filter((x) => x.maKho !== k.maKho));
      }
    } catch (V208) {
      V4nn(V208);
    }
  };

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Kho lưu trữ</h2>
        <p className="muted">Danh mục kho — mỗi kho chứa nhiều giá/kệ; hồ sơ được xếp vào giá.</p>
      </div>
      {V1zy && (
        <div className="err-banner" role="alert">
          Không tải được danh sách kho: {V1zy}
        </div>
      )}
      <V49a<V4lo>
        addLabel="VGjDqm0ga2hv"
        onAdd={V1zr ? V201 : undefined}
        rows={V1zu}
        getId={(r) => r.maKho}
        matches={V4ni}
        columns={[
          { head: "VMOqbiBraG8=", cell: (r) => r.tenKho },
          { head: "R2hpIGNow7o=", cell: (r) => r.ghiChu ?? "" },
          { head: "U+G7kSBnacOh", cell: (r) => r.soGia, cls: "tnum" },
        ]}
        rowActions={(r) => (
          <>
            {V1zs && (
              <button className="linkbtn edit" onClick={() => V202(r)}>
                <Icon name="sua" /> Sửa
              </button>
            )}
            {V1zt && (
              <button className="linkbtn del" onClick={() => V206(r)}>
                <Icon name="xoa" /> Xóa
              </button>
            )}
          </>
        )}
      />
      {V1zw && (
        <Modal
          title={
            V1zw.mode === "add"
              ? "VGjDqm0ga2hvIGzGsHUgdHLhu68="
              : "Q+G6rXAgbmjhuq10IGtobyBsxrB1IHRy4buv"
          }
          onClose={() => V1zx(null)}
          onSave={V203}
        >
          <V49j label="VMOqbiBraG8=">
            <input
              className="inp"
              value={V1zw.draft.tenKho}
              onChange={(V209) =>
                V1zx({ ...V1zw, draft: { ...V1zw.draft, tenKho: V209.target.value } })
              }
            />
          </V49j>
          <V49j label="R2hpIGNow7o=">
            <input
              className="inp"
              value={V1zw.draft.ghiChu ?? ""}
              onChange={(V20a) =>
                V1zx({ ...V1zw, draft: { ...V1zw.draft, ghiChu: V20a.target.value } })
              }
            />
          </V49j>
          <V49j label="U+G7kSB0aOG7qSB04bux">
            <input
              className="inp"
              type="number"
              value={V1zw.draft.soTt ?? ""}
              onChange={(V20b) =>
                V1zx({ ...V1zw, draft: { ...V1zw.draft, soTt: V4nl(V20b.target.value) } })
              }
            />
          </V49j>
        </Modal>
      )}
    </section>
  );
}
