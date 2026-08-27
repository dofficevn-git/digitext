// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";

import "./khogom.css";
import { V3xq } from "@/api/ipc";
import {
  V4pj,
  V4pk,
  V4ph,
  V4pb,
  V4pf,
  type V4pi,
  type V4pe,
  type V4pg,
  type V4pa,
} from "@/api/uploadBatches";
import { Icon } from "@/components/Icon";

const V4pl = (V2bl: unknown) =>
  V2bl instanceof V3xq || V2bl instanceof Error ? V2bl.message : String(V2bl);

const V4pm: Record<V4pi, { cls: string; label: string }> = {
  ok: { cls: "ok", label: "SG/DoG4gdOG6pXQ=" },
  in_progress: { cls: "gray", label: "xJBhbmcgZ+G7rWk=" },
  attention: { cls: "warn", label: "Q+G6p24ga2nhu4NtIHRyYQ==" },
};

function V4pn(V2bm: string): string {
  const d = new Date(V2bm);
  return Number.isNaN(d.getTime()) ? V2bm : d.toLocaleString("vi-VN");
}

const V4po: V4pe = { admins: [], projects: [] };

export default function V4pp({ isSuper: V2bn }: { isSuper: boolean }) {
  const [V2bo, V2bp] = useState<V4pg>(() => V4ph(V4po));
  const [V2bq, V2br] = useState<V4pa[]>([]);
  const [V2bs, V2bt] = useState(true);
  const [V2bu, V2bv] = useState("");

  const V2bw = useCallback(async () => {
    V2bt(true);
    V2bv("");

    const [m, b] = await Promise.allSettled([V4pf(), V4pb()]);
    if (m.status === "fulfilled") V2bp(V4ph(m.value));
    else V2bv(V4pl(m.reason));
    if (b.status === "fulfilled") V2br(b.value);
    else if (m.status !== "rejected") V2bv(V4pl(b.reason));
    V2bt(false);
  }, []);

  V41p(V2bw, { topics: ["uploadBatches", "docs"] });

  return (
    <div>
      <div className="users-toolbar mb-2.5">
        <p className="muted m-0">
          {V2bn
            ? "TeG7l2kgbcOheSBhZG1pbiDEkeG6qXkgdMOgaSBsaeG7h3UgbMOqbiBtw6F5IGNo4bunIHRoZW8gdOG7q25nIHBow7RuZyDigJQgdGhlbyBkw7VpIGNow6lvIHThuqV0IGPhuqMu"
            : "VMOgaSBsaeG7h3UgYuG6oW4gKHbDoCDEkeG7k25nLWFkbWluKSDEkcOjIMSR4bqpeSBsw6puIG3DoXkgY2jhu6csIHRoZW8gdOG7q25nIHBow7RuZy4="}
        </p>
        <div className="spacer" />
        <button className="btn" onClick={() => void V2bw()} disabled={V2bs}>
          <Icon name="lamMoi" />
          Tải lại
        </button>
      </div>

      {V2bu && <div className="login-error">{V2bu}</div>}
      {V2bs && <p className="muted">Đang tải…</p>}

      {!V2bs && (
        <>
          <V4pq isSuper={V2bn} view={V2bo} />
          <V4pr batches={V2bq} isSuper={V2bn} />
        </>
      )}
    </div>
  );
}

function V4pq({ isSuper: V2bx, view }: { isSuper: boolean; view: V4pg }) {
  return (
    <div className="card kg-card p-0 overflow-hidden mb-4">
      <div className="kg-bar">
        <h3 className="m-0">Tài liệu đã nộp lên máy chủ</h3>
        <span className="muted text-sm">
          {view.grandTotal.toLocaleString("vi-VN")} tài liệu · {view.admins.length} máy admin ·{" "}
          {view.rows.length} phông
        </span>
      </div>

      {view.isEmpty ? (
        <p className="kg-note">
          Chưa có máy admin nào đẩy tài liệu lên máy chủ qua luồng “Đợt nộp”. Ở màn Đồng bộ dữ liệu,
          bấm “② Đẩy lên máy chủ” để bắt đầu.
        </p>
      ) : (
        <div className="tbl-scroll kg-matrix">
          <table className="tbl kg-table">
            <thead>
              <tr>
                <th className="kg-corner">Kho lưu trữ</th>
                {view.admins.map((a) => (
                  <th key={a.id} className="kg-colhead">
                    <div className="kg-colname">
                      <Icon name="mayTram" />
                      <b>{a.name ?? "—"}</b>
                      {}
                      {a.attention && (
                        <span
                          className="text-warn"
                          title="Q8OzIMSR4bujdCBtw6F5IGNo4bunIG5o4bqtbiB0aGnhur91IHNvIHbhu5tpIGLDoW8gY8OhbyDigJQgeGVtIGRhbmggc8OhY2ggxJHhu6N0IGLDqm4gZMaw4bubaQ=="
                        >
                          {" "}
                          <Icon name="canhBao" size={13} />
                        </span>
                      )}
                    </div>
                    <div className="muted">{view.columnTotals[a.id] ?? 0} tài liệu</div>
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {view.rows.map(({ project: V2by, total: V2bz }) => (
                <tr key={V2by.id}>
                  <th className="kg-rowhead kg-hoso">
                    <div>
                      <Icon name="thuMuc" /> {V2by.name}
                    </div>
                    <div className="muted">
                      {V2by.code} · {V2bz} tài liệu
                    </div>
                  </th>
                  {view.admins.map((a) => {
                    const n = V2by.cells[a.id] ?? 0;
                    return (
                      <td key={a.id} className="kg-cell text-center">
                        {n > 0 ? <b>{n}</b> : <span className="muted">—</span>}
                      </td>
                    );
                  })}
                </tr>
              ))}
            </tbody>
            <tfoot>
              <tr className="kg-grouprow">
                <th className="kg-rowhead">Tổng</th>
                {view.admins.map((a) => (
                  <td key={a.id} className="text-center">
                    <b>{view.columnTotals[a.id] ?? 0}</b>
                  </td>
                ))}
              </tr>
            </tfoot>
          </table>
        </div>
      )}

      <p className="kg-note">
        {V2bx
          ? "Q2jhu4kgxJHhur9tIHTDoGkgbGnhu4d1IMSR4bqpeSBxdWEg4oCcxJDhu6N0IG7hu5lw4oCdLiBVcGxvYWQgbOG6uyAoa2jDtG5nIHF1YSBH4butaSBsw7QpIGtow7RuZyBjw7MgbcOheS1n4butaSBuw6puIGtow7RuZyBu4bqxbSB0cm9uZyBi4bqjbmcgbsOgeS4="
          : "Q+G7mXQgZ+G7k20gY+G6oyDEkeG7k25nLWFkbWluIGPDuW5nIHBo4bulIHRyw6FjaCBwaMO0bmcgduG7m2kgYuG6oW4u"}
      </p>
    </div>
  );
}

function V4pr({ batches: V2c0, isSuper: V2c1 }: { batches: V4pa[]; isSuper: boolean }) {
  return (
    <div className="card p-0 overflow-hidden">
      <div className="flex items-center py-3 px-4 gap-3">
        <h3 className="m-0">Các đợt nộp</h3>
        <span className="muted text-sm">{V2c0.length} đợt</span>
      </div>

      {V2c0.length === 0 ? (
        <p className="muted pt-0 px-4 pb-4">
          Chưa có đợt nộp nào. Mỗi lần bấm “② Đẩy lên máy chủ” ở màn Đồng bộ dữ liệu là một đợt.
        </p>
      ) : (
        <div className="tbl-scroll">
          <table className="tbl">
            <thead>
              <tr>
                <th>Thời điểm</th>
                {V2c1 && <th>Máy admin</th>}
                <th>Tình trạng</th>
                <th className="num">Nhận / Gửi</th>
                <th className="num">Phông</th>
                <th>Chi tiết</th>
              </tr>
            </thead>
            <tbody>
              {V2c0.map((b) => {
                const V2c2 = V4pm[V4pj(b)];
                return (
                  <tr key={b.id}>
                    <td>{V4pn(b.createdAt)}</td>
                    {V2c1 && <td>{b.forwardedByName ?? "—"}</td>}
                    <td>
                      <span className={`badge ${V2c2.cls}`}>{V2c2.label}</span>
                    </td>
                    <td className="num">
                      <b className={b.received < b.forwarded ? "text-warn" : undefined}>
                        {b.received}
                      </b>
                      {" / "}
                      {b.forwarded}
                    </td>
                    <td className="num">{b.projectCount}</td>
                    <td className="muted">
                      {V4pk(b)}
                      {b.note ? ` · ${b.note}` : ""}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}
