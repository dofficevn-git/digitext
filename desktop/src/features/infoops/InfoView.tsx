// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";

import { V4ux, V4uv, V4uy, V4ut, type V4us } from "@/api/infoops";
import { isTauri } from "@/api/ipc";
import { V460 } from "@/state";

import { V4v6, V4v7, V4vb, V4v9 } from "./infoFlow";
import { Icon } from "@/components/Icon";

export default function V4vc() {
  const { session: V30f } = V460();
  const V30g = V30f?.permissions ?? [];
  const V30h = V30g.includes("access.review");
  const [V30i, V30j] = useState<V4us[]>([]);
  const [V30k, V30l] = useState(false);
  const [V30m, V30n] = useState(false);
  const [V30o, V30p] = useState<string | null>(null);
  const [V30q, V30r] = useState<string[]>([]);

  const [V30s, V30t] = useState<V4us["hinhThuc"]>("danh_muc");
  const [V30u, V30v] = useState("");
  const [V30w, V30x] = useState("");

  const reload = useCallback(async () => {
    if (!isTauri()) return;
    try {
      V30j(await V4ut());
    } catch (V30y) {
      V30p(`S2jDtG5nIHThuqNpIMSRxrDhu6NjIGRhbmggc8OhY2g6IA==${String(V30y)}`);
    }
  }, []);

  V41p(reload, { topics: ["info", "notifications"] });

  const V30z = async () => {
    const V310 = {
      projectId: V30f?.projectId ?? null,
      hinhThuc: V30s,
      chuDe: V30u,
      mucDich: V30w,
      phuongThuc: "tai_khoan",
    };
    const V311 = V4v9(V310);
    V30r(V311);
    if (V311.length > 0) return;
    V30n(true);
    V30p(null);
    try {
      await V4uv(V310);
      V30p(
        "xJDDoyBn4butaSB5w6p1IGPhuqd1IOKAlCBTTEEgdGnhur9wIG5o4bqtbiArIHjDqXQgZHV54buHdCA2IG5nw6B5IGzDoG0gdmnhu4djICjEkGnhu4F1IDQ2LjUpLg==",
      );
      V30l(false);
      V30v("");
      V30x("");
      await reload();
    } catch (V312) {
      V30p(`R+G7rWkgecOqdSBj4bqndSB0aOG6pXQgYuG6oWk6IA==${String(V312)}`);
    } finally {
      V30n(false);
    }
  };

  const V313 = async (row: V4us, V314: "approve" | "refuse" | "pay" | "generate" | "fulfill") => {
    let V315: string | undefined;
    let V316: string | undefined;
    if (V314 === "refuse") {
      V315 = window.prompt("TMO9IGRvIHThu6sgY2jhu5FpICjEkGnhu4F1IDQ2LjEuYyk6")?.trim() || undefined;
      if (!V315) return;
    }
    if (V314 === "generate") {
      V316 =
        window
          .prompt(
            "TuG7mWkgZHVuZyByw6Agc2/DoXQsIGJpw6puIHThuq1wICjEkGnhu4F1IDQ3LjIg4oCUIHR14buzIGNo4buNbik6",
          )
          ?.trim() || undefined;
    }
    V30n(true);
    try {
      await V4ux(row.id, V314, { note: V315, bienTap: V316 });
      await reload();
    } catch (V317) {
      V30p(`VGhhbyB0w6FjIHRo4bqldCBi4bqhaTog${String(V317)}`);
    } finally {
      V30n(false);
    }
  };

  const V318 = async (row: V4us) => {
    V30n(true);
    try {
      const path = await V4uy(row.id, `cung-cap-thong-tin-${row.chuDe.slice(0, 30)}.pdf`);
      V30p(`xJDDoyBsxrB1IGvhur90IHF14bqjOiA=${path}`);
    } catch (V319) {
      V30p(`VOG6o2kga+G6v3QgcXXhuqMgdGjhuqV0IGLhuqFpOiA=${String(V319)}`);
    } finally {
      V30n(false);
    }
  };

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Cung cấp thông tin (Mục 4, Điều 44–47 — TT 05/2025/TT-BNV)</h2>
        <div className="page-actions">
          <button className="btn" onClick={() => void reload()} disabled={V30m}>
            <Icon name="lamMoi" />
            Làm mới
          </button>
          {V30g.includes("access.request") && (
            <button className="btn primary" onClick={() => V30l((v) => !v)}>
              + Đăng ký yêu cầu
            </button>
          )}
        </div>
      </div>

      {V30o && <div className="card p-2.5 mb-2.5">{V30o}</div>}

      {V30k && (
        <div className="card p-4 mb-4">
          <h3 className="mt-0">Đăng ký yêu cầu (Điều 45)</h3>
          <div className="grid gap-2.5 [grid-template-columns:repeat(auto-fill,minmax(260px,1fr))]">
            <div className="field">
              <label>Hình thức (Điều 44)</label>
              <select
                className="inp"
                value={V30s}
                onChange={(V31a) => V30t(V31a.target.value as V4us["hinhThuc"])}
              >
                {Object.entries(V4v6).map(([k, V31b]) => (
                  <option key={k} value={k}>
                    {V31b}
                  </option>
                ))}
              </select>
            </div>
            <div className="field">
              <label>Chủ đề cụ thể (44.1.a)</label>
              <input className="inp" value={V30u} onChange={(V31c) => V30v(V31c.target.value)} />
            </div>
            <div className="field">
              <label>Mục đích sử dụng (45.3)</label>
              <input className="inp" value={V30w} onChange={(V31d) => V30x(V31d.target.value)} />
            </div>
          </div>
          {V30q.length > 0 && (
            <ul className="mt-2.5 text-danger-700">
              {V30q.map((V31e) => (
                <li key={V31e}>{V31e}</li>
              ))}
            </ul>
          )}
          <div className="mt-2.5">
            <button className="btn primary" onClick={() => void V30z()} disabled={V30m}>
              Gửi yêu cầu
            </button>
          </div>
        </div>
      )}

      <div className="card p-4">
        <h3 className="mt-0">Danh sách yêu cầu</h3>
        {V30i.length === 0 ? (
          <p className="muted">Chưa có yêu cầu nào.</p>
        ) : (
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Hình thức</th>
                  <th>Chủ đề</th>
                  <th>Trạng thái</th>
                  <th>Phí</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {V30i.map((r) => (
                  <tr key={r.id}>
                    <td>{V4v6[r.hinhThuc]}</td>
                    <td title={r.mucDich}>{r.chuDe}</td>
                    <td title={r.reviewNote ?? r.bienTap ?? undefined}>
                      {V4v7[r.status]}
                      {r.coKetQua ? "IMK3IGPDsyBr4bq/dCBxdeG6ow==" : ""}
                    </td>
                    <td>
                      {r.phi == null ? "—" : r.phi === 0 ? "TWnhu4VuIHBow60=" : `${r.phi}IMSR`}
                      {(r.phi ?? 0) > 0 && (r.paidAt ? "ICjEkcOjIFRUKQ==" : "IChjaMawYSBUVCk=")}
                    </td>
                    <td className="whitespace-nowrap">
                      {V30h &&
                        V4vb(r).map((a) => (
                          <button
                            key={a.action}
                            className="btn sm mr-1"
                            disabled={V30m}
                            onClick={() => void V313(r, a.action)}
                          >
                            <Icon name={a.icon} />
                            {a.label}
                          </button>
                        ))}
                      {(r.status === "fulfilled" || (V30h && r.coKetQua)) && (
                        <button className="btn sm" disabled={V30m} onClick={() => void V318(r)}>
                          <Icon name="taiVe" />
                          Tải kết quả
                        </button>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </section>
  );
}
