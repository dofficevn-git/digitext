// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useState } from "react";
import { V41p } from "@/live/useLive";

import { isTauri } from "@/api/ipc";
import { V4t0, V4t1, V4sz, type V4sy } from "@/api/records";
import { V460 } from "@/state";

import { V4ti, V4tf, V4tg, type V4ta } from "./recordForms";
import { Icon } from "@/components/Icon";

const V4tj: Record<string, string> = Object.fromEntries(V4tf.map((t) => [t.kind, t.label]));
V4tj["thu_nop"] = "Biên bản thu nộp (Điều 25.2)";
V4tj["giao_nhan"] =
  "Biên bản giao nhận vào lưu trữ lịch sử (Điều 25.5.a)";
V4tj["tra_lai"] =
  "Biên bản hồ sơ, tài liệu trả lại (Điều 25.5.a)";

export default function V4tk() {
  const { session: V2uf } = V460();
  const V2ug = V2uf?.permissions ?? [];
  const [V2uh, V2ui] = useState<V4sy[]>([]);
  const [V2uj, V2uk] = useState<V4ta>("kiem_tra");
  const [V2ul, V2um] = useState<Record<string, string>>({});
  const [V2un, V2uo] = useState<Record<string, Record<string, string>[]>>({});
  const [V2up, V2uq] = useState(false);
  const [V2ur, V2us] = useState<string | null>(null);
  const [V2ut, V2uu] = useState<string[]>([]);

  const template = V4tg(V2uj);

  const reload = useCallback(async () => {
    if (!isTauri()) return;
    try {
      V2ui(await V4sz());
    } catch (V2uv) {
      V2us(`Không tải được danh sách biên bản: ${String(V2uv)}`);
    }
  }, []);

  V41p(reload, { topics: ["records"] });

  const V2uw = (k: V4ta) => {
    V2uk(k);
    V2um({});
    const V2ux: Record<string, Record<string, string>[]> = {};
    for (const l of V4tg(k).lists) V2ux[l.key] = l.minRows > 0 ? [{}] : [];
    V2uo(V2ux);
    V2uu([]);
  };

  useEffect(() => {
    V2uw("kiem_tra");
  }, []);

  const V2uy = (V2uz: string, row: number, V2v0: string, v: string) =>
    V2uo((V2v1) => {
      const V2v2 = [...(V2v1[V2uz] ?? [])];
      V2v2[row] = { ...V2v2[row], [V2v0]: v };
      return { ...V2v1, [V2uz]: V2v2 };
    });

  const V2v3 = async () => {
    const { payload: V2v4, errors: V2v5 } = V4ti(template, V2ul, V2un);
    V2uu(V2v5);
    if (!V2v4) return;
    V2uq(true);
    V2us(null);
    try {
      const row = await V4t0({
        projectId: V2uf?.projectId ?? null,
        kind: V2uj,
        payload: V2v4,
      });
      V2us(`Đã lập biên bản số ${row.soBienBan}.`);
      V2uw(V2uj);
      await reload();
    } catch (V2v6) {
      V2us(`Lập biên bản thất bại: ${String(V2v6)}`);
    } finally {
      V2uq(false);
    }
  };

  const V2v7 = async (row: V4sy) => {
    V2uq(true);
    try {
      const path = await V4t1(row.id, `${row.soBienBan.split("/").join("-")}.pdf`);
      V2us(`Đã lưu biên bản: ${path}`);
    } catch (V2v8) {
      V2us(`Tải biên bản thất bại: ${String(V2v8)}`);
    } finally {
      V2uq(false);
    }
  };

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Biên bản (Phụ lục VII — TT 05/2025/TT-BNV)</h2>
        <div className="page-actions">
          <button className="btn" onClick={() => void reload()} disabled={V2up}>
            <Icon name="lamMoi" />
            Làm mới
          </button>
        </div>
      </div>

      {V2ur && <div className="card p-2.5 mb-2.5">{V2ur}</div>}

      {V2ug.includes("record.create") && (
        <div className="card p-4 mb-4">
          <div className="field max-w-105">
            <label>Mẫu biên bản</label>
            <select
              className="inp"
              value={V2uj}
              onChange={(V2v9) => V2uw(V2v9.target.value as V4ta)}
            >
              {V4tf.map((t) => (
                <option key={t.kind} value={t.kind}>
                  {t.label} — {t.legal}
                </option>
              ))}
            </select>
          </div>

          <div className="grid gap-2.5 mt-2 [grid-template-columns:repeat(auto-fill,minmax(240px,1fr))]">
            {template.fields.map((f) => (
              <div className="field" key={f.key}>
                <label>
                  {f.label}
                  {f.optional ? " (tuỳ chọn)" : ""}
                </label>
                <input
                  className="inp"
                  value={V2ul[f.key] ?? ""}
                  onChange={(V2va) => V2um((v) => ({ ...v, [f.key]: V2va.target.value }))}
                />
              </div>
            ))}
          </div>

          {template.lists.map((l) => (
            <div key={l.key} className="mt-3">
              <h4 className="mb-1.5">{l.label}</h4>
              <div className="tbl-scroll">
                <table className="tbl">
                  <thead>
                    <tr>
                      <th className="w-7">#</th>
                      {l.simple ? (
                        <th>Nội dung</th>
                      ) : (
                        l.columns.map((c) => (
                          <th key={c.key}>
                            {c.label}
                            {c.optional ? " (tuỳ chọn)" : ""}
                          </th>
                        ))
                      )}
                      <th className="w-9"></th>
                    </tr>
                  </thead>
                  <tbody>
                    {(V2un[l.key] ?? []).map((row, i) => (
                      <tr key={i}>
                        <td>{i + 1}</td>
                        {l.simple ? (
                          <td>
                            <input
                              className="inp"
                              value={row.value ?? ""}
                              onChange={(V2vb) => V2uy(l.key, i, "value", V2vb.target.value)}
                            />
                          </td>
                        ) : (
                          l.columns.map((c) => (
                            <td key={c.key}>
                              <input
                                className="inp"
                                value={row[c.key] ?? ""}
                                onChange={(V2vc) => V2uy(l.key, i, c.key, V2vc.target.value)}
                              />
                            </td>
                          ))
                        )}
                        <td>
                          <button
                            className="btn sm"
                            onClick={() =>
                              V2uo((V2vd) => ({
                                ...V2vd,
                                [l.key]: (V2vd[l.key] ?? []).filter((_V2ve, V2vf) => V2vf !== i),
                              }))
                            }
                          >
                            <Icon name="xoa" size={14} label="Xoá dòng" />
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              <button
                className="btn sm mt-1.5"
                onClick={() => V2uo((V2vg) => ({ ...V2vg, [l.key]: [...(V2vg[l.key] ?? []), {}] }))}
              >
                + Thêm dòng
              </button>
            </div>
          ))}

          {V2ut.length > 0 && (
            <ul className="mt-2.5 text-danger-700">
              {V2ut.map((V2vh) => (
                <li key={V2vh}>{V2vh}</li>
              ))}
            </ul>
          )}

          <div className="mt-3">
            <button className="btn primary" onClick={() => void V2v3()} disabled={V2up}>
              <Icon name="taiLieu" />
              Lập biên bản
            </button>
          </div>
        </div>
      )}

      <div className="card p-4">
        <h3 className="mt-0">Sổ biên bản</h3>
        {V2uh.length === 0 ? (
          <p className="muted">Chưa có biên bản nào.</p>
        ) : (
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Số biên bản</th>
                  <th>Loại</th>
                  <th>Tiêu đề</th>
                  <th>Ngày lập</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {V2uh.map((r) => (
                  <tr key={r.id}>
                    <td>{r.soBienBan}</td>
                    <td>{V4tj[r.kind] ?? r.kind}</td>
                    <td>{r.title}</td>
                    <td>{new Date(r.createdAt).toLocaleDateString("vi-VN")}</td>
                    <td>
                      <button className="btn sm" onClick={() => void V2v7(r)} disabled={V2up}>
                        <Icon name="taiVe" />
                        Tải PDF
                      </button>
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
