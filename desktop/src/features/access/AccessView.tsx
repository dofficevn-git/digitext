// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";

import {
  V4tu,
  V4tr,
  V4tw,
  V4ty,
  V4tz,
  V4tp,
  V4to,
  V4tv,
  V4ts,
  type V4tn,
  type V4tl,
  type V4tx,
} from "@/api/access";
import { V4ei, type V4eg } from "@/api/docs";
import { isTauri } from "@/api/ipc";
import { V460 } from "@/state";

import { V4u6, V4u4, V4u0, V4u2, V4u1, V4u3, V4u5 } from "./accessFlow";
import { Icon } from "@/components/Icon";

export default function V4u7() {
  const { session: V2w1 } = V460();
  const V2w2 = V2w1?.permissions ?? [];
  const V2w3 = V2w2.includes("access.review");
  const [V2w4, V2w5] = useState<V4tl[]>([]);
  const [V2w6, V2w7] = useState<V4tn | null>(null);
  const [V2w8, V2w9] = useState<V4tx | null>(null);
  const [V2wa, V2wb] = useState(false);
  const [V2wc, V2wd] = useState(false);
  const [V2we, V2wf] = useState<string | null>(null);
  const [V2wg, V2wh] = useState<string[]>([]);

  const [V2wi, V2wj] = useState<string | null>(null);

  const [V2wk, V2wl] = useState<V4tl["kind"]>("doc");
  const [V2wm, V2wn] = useState("");
  const [V2wo, V2wp] = useState("tai_khoan");
  const [V2wq, V2wr] = useState("");
  const [V2ws, V2wt] = useState("");
  const [V2wu, V2wv] = useState<V4eg[]>([]);
  const [V2ww, V2wx] = useState<string[]>([]);

  const reload = useCallback(async () => {
    if (!isTauri()) return;
    try {
      V2w5(await V4to());
    } catch (V2wy) {
      V2wf(`Không tải được danh sách yêu cầu: ${String(V2wy)}`);
    }
    try {
      V2w9(await V4ty());
    } catch {
      V2w9(null);
    }
  }, []);

  V41p(reload, { topics: ["access", "docs"] });

  const V2wz = async () => {
    V2wb((v) => !v);
    if (V2wu.length === 0) {
      try {
        V2wv(await V4ei());
      } catch (V2x0) {
        V2wf(
          `Không tải được danh sách tài liệu server: ${String(V2x0)}`,
        );
      }
    }
  };

  const V2x1 = async (V2x2: string) => {
    V2wd(true);
    V2wj(null);
    try {
      V2w7(await V4tp(V2x2));
      V2wb(false);
    } catch (V2x3) {
      V2wf(`Không tải được chi tiết: ${String(V2x3)}`);
    } finally {
      V2wd(false);
    }
  };

  const V2x4 = async () => {
    const V2x5 = {
      projectId: V2w1?.projectId ?? null,
      kind: V2wk,
      mucDich: V2wm,
      phuongThuc: V2wo,
      nguoiNhan: V2wq || null,
      soDinhDanh: V2ws || null,
      documentIds: V2ww,
    };
    const V2x6 = V4u3(V2x5);
    V2wh(V2x6);
    if (V2x6.length > 0) return;
    V2wd(true);
    V2wf(null);
    try {
      const V2x7 = await V4tr(V2x5);
      V2wf(
        "Đã gửi yêu cầu khai thác — chờ xét duyệt (Điều 36).",
      );
      V2wb(false);
      V2wx([]);
      V2wn("");
      await reload();
      V2w7(V2x7);
    } catch (V2x8) {
      V2wf(`Gửi yêu cầu thất bại: ${String(V2x8)}`);
    } finally {
      V2wd(false);
    }
  };

  const V2x9 = async (V2xa: string, V2xb: "approve" | "refuse" | "pay" | "dip") => {
    let V2xc: string | undefined;
    if (V2xb === "refuse") {
      V2xc = window.prompt("Lý do từ chối (Điều 36.1.c):")?.trim() || undefined;
      if (!V2xc) return;
    }
    V2wd(true);
    try {
      const V2xd = await V4tu(V2xa, V2xb, V2xc);
      if (V2xb === "dip") {
        V2wf(
          `Gói DIP ${V2xd.maXacThuc ?? ""} đã xếp hàng đợi — theo dõi ở màn Đóng gói (Điều 42).`,
        );
      }
      await reload();
      await V2x1(V2xa);
    } catch (V2xe) {
      V2wf(`Thao tác thất bại: ${String(V2xe)}`);
    } finally {
      V2wd(false);
    }
  };

  const V2xf = async (V2xg: string, V2xh: string, V2xi: "duyet" | "tu_choi") => {
    let V2xj: string | undefined;
    let V2xk = false;
    if (V2xi === "tu_choi") {
      V2xj =
        window
          .prompt("Lý do không duyệt tài liệu (Điều 36.1.c):")
          ?.trim() || undefined;
      if (!V2xj) return;
      V2xk = window.confirm(
        "Tài liệu thuộc diện tiếp cận CÓ ĐIỀU KIỆN (Điều 36.1.d)?",
      );
    }
    V2wd(true);
    try {
      await V4ts(V2xg, V2xh, V2xi, V2xj, V2xk);
      await V2x1(V2xg);
    } catch (V2xl) {
      V2wf(`Chấm tài liệu thất bại: ${String(V2xl)}`);
    } finally {
      V2wd(false);
    }
  };

  const V2xm = async (V2xn: string, V2xo: string) => {
    V2wd(true);
    try {
      const V2xp = await V4tv(V2xn, V2xo);
      V2wj(`data:application/pdf;base64,${V2xp}`);
    } catch (V2xq) {
      V2wf(`Mở bản đọc thất bại: ${String(V2xq)}`);
    } finally {
      V2wd(false);
    }
  };

  const V2xr = async (V2xs: string, V2xt: string, name: string) => {
    V2wd(true);
    try {
      const path = await V4tw(V2xs, V2xt, name);
      V2wf(`Đã lưu bản sao: ${path}`);
    } catch (V2xu) {
      V2wf(`Tải bản sao thất bại: ${String(V2xu)}`);
    } finally {
      V2wd(false);
    }
  };

  const V2xv = async () => {
    if (!V2w8) return;
    V2wd(true);
    try {
      V2w9(await V4tz(V2w8.phiDocTaiLieu, V2w8.phiSaoTrang));
      V2wf("Đã lưu đơn giá phí.");
    } catch (V2xw) {
      V2wf(`Lưu đơn giá thất bại: ${String(V2xw)}`);
    } finally {
      V2wd(false);
    }
  };

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Khai thác lưu trữ số (Điều 35–42 — TT 05/2025/TT-BNV)</h2>
        <div className="page-actions">
          <button className="btn" onClick={() => void reload()} disabled={V2wc}>
            <Icon name="lamMoi" />
            Làm mới
          </button>
          {V2w2.includes("access.request") && (
            <button className="btn primary" onClick={() => void V2wz()}>
              + Đăng ký yêu cầu
            </button>
          )}
        </div>
      </div>

      {V2we && <div className="card p-2.5 mb-2.5">{V2we}</div>}

      {V2w3 && V2w8 && (
        <div className="card p-3 mb-3">
          <strong>Đơn giá phí</strong> — đọc {V4u4(V2w8.phiDocTaiLieu)}/tài liệu (Điều 36.1.c) · sao{" "}
          {V4u4(V2w8.phiSaoTrang)}/trang (Điều 39.1.c){" "}
          <input
            className="inp w-27.5"
            type="number"
            value={V2w8.phiDocTaiLieu}
            onChange={(V2xx) => V2w9({ ...V2w8, phiDocTaiLieu: Number(V2xx.target.value) })}
          />{" "}
          <input
            className="inp w-27.5"
            type="number"
            value={V2w8.phiSaoTrang}
            onChange={(V2xy) => V2w9({ ...V2w8, phiSaoTrang: Number(V2xy.target.value) })}
          />{" "}
          <button className="btn sm" onClick={() => void V2xv()} disabled={V2wc}>
            Lưu đơn giá
          </button>
        </div>
      )}

      {V2wa && (
        <div className="card p-4 mb-4">
          <h3 className="mt-0">Đăng ký yêu cầu khai thác (Điều 35 / 38)</h3>
          <div className="grid gap-2.5 [grid-template-columns:repeat(auto-fill,minmax(240px,1fr))]">
            <div className="field">
              <label>Hình thức</label>
              <select
                className="inp"
                value={V2wk}
                onChange={(V2xz) => V2wl(V2xz.target.value as V4tl["kind"])}
              >
                {Object.entries(V4u0).map(([k, V2y0]) => (
                  <option key={k} value={k}>
                    {V2y0}
                  </option>
                ))}
              </select>
            </div>
            <div className="field">
              <label>Mục đích sử dụng (Điều 35.4)</label>
              <input className="inp" value={V2wm} onChange={(V2y1) => V2wn(V2y1.target.value)} />
            </div>
            <div className="field">
              <label>Phương thức nhận (Điều 38.3)</label>
              <select className="inp" value={V2wo} onChange={(V2y2) => V2wp(V2y2.target.value)}>
                {Object.entries(V4u2).map(([k, V2y3]) => (
                  <option key={k} value={k}>
                    {V2y3}
                  </option>
                ))}
              </select>
            </div>
            {V2wk === "dip" && (
              <>
                <div className="field">
                  <label>Người nhận (Điều 42.2.c)</label>
                  <input
                    className="inp"
                    value={V2wq}
                    onChange={(V2y4) => V2wr(V2y4.target.value)}
                  />
                </div>
                <div className="field">
                  <label>Mã định danh / số giấy tờ tùy thân</label>
                  <input
                    className="inp"
                    value={V2ws}
                    onChange={(V2y5) => V2wt(V2y5.target.value)}
                  />
                </div>
              </>
            )}
          </div>

          <h4>Chọn tài liệu ({V2ww.length} đã chọn)</h4>
          <div className="tbl-scroll max-h-65 overflow-y-auto">
            <table className="tbl">
              <thead>
                <tr>
                  <th className="w-7"></th>
                  <th>Tên tệp</th>
                  <th>Mã hồ sơ</th>
                  <th>Số trang</th>
                </tr>
              </thead>
              <tbody>
                {V2wu.map((d) => (
                  <tr key={d.id}>
                    <td>
                      <input
                        type="checkbox"
                        checked={V2ww.includes(d.id)}
                        onChange={(V2y6) =>
                          V2wx((V2y7) =>
                            V2y6.target.checked ? [...V2y7, d.id] : V2y7.filter((x) => x !== d.id),
                          )
                        }
                      />
                    </td>
                    <td>{d.filename}</td>
                    <td>{d.hoSoRef ?? "—"}</td>
                    <td>{d.pageCount ?? "—"}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {V2wg.length > 0 && (
            <ul className="mt-2.5 text-danger-700">
              {V2wg.map((V2y8) => (
                <li key={V2y8}>{V2y8}</li>
              ))}
            </ul>
          )}
          <div className="mt-2.5">
            <button className="btn primary" onClick={() => void V2x4()} disabled={V2wc}>
              Gửi yêu cầu
            </button>
          </div>
        </div>
      )}

      <div className="card p-4 mb-4">
        <h3 className="mt-0">Danh sách yêu cầu</h3>
        {V2w4.length === 0 ? (
          <p className="muted">Chưa có yêu cầu khai thác nào.</p>
        ) : (
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Hình thức</th>
                  <th>Mục đích</th>
                  <th>Trạng thái</th>
                  <th>Phí</th>
                  <th>Hạn sử dụng</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {V2w4.map((r) => (
                  <tr key={r.id}>
                    <td>{V4u0[r.kind]}</td>
                    <td>{r.mucDich}</td>
                    <td title={r.reviewNote ?? undefined}>{V4u1[r.status]}</td>
                    <td>
                      {V4u4(r.phi)}
                      {r.phi != null &&
                        r.phi > 0 &&
                        (r.paidAt ? " (đã TT)" : " (chưa TT)")}
                    </td>
                    <td>{r.hanSuDung ? new Date(r.hanSuDung).toLocaleDateString("vi-VN") : "—"}</td>
                    <td>
                      <button className="btn sm" onClick={() => void V2x1(r.id)} disabled={V2wc}>
                        Chi tiết
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {V2w6 && (
        <div className="card p-4">
          <h3 className="mt-0">
            {V4u0[V2w6.kind]} — {V4u1[V2w6.status]}
          </h3>
          <p className="muted">
            Mục đích: {V2w6.mucDich} · phí {V4u4(V2w6.phi)}
            {V2w6.hanSuDung
              ? ` · hạn ${new Date(V2w6.hanSuDung).toLocaleDateString("vi-VN")}`
              : ""}
            {V2w6.reviewNote ? ` · ghi chú: ${V2w6.reviewNote}` : ""}
          </p>

          <div className="flex gap-2 flex-wrap mb-3">
            {V2w3 && V2w6.status === "submitted" && (
              <>
                <button
                  className="btn primary"
                  disabled={V2wc}
                  onClick={() => void V2x9(V2w6.id, "approve")}
                >
                  <Icon name="duyet" />
                  Duyệt (phí tự tính — Điều 36.1.c)
                </button>
                <button
                  className="btn"
                  disabled={V2wc}
                  onClick={() => void V2x9(V2w6.id, "refuse")}
                >
                  <Icon name="tuChoi" />
                  Từ chối
                </button>
              </>
            )}
            {V2w3 && V2w6.status === "approved" && (V2w6.phi ?? 0) > 0 && !V2w6.paidAt && (
              <button
                className="btn primary"
                disabled={V2wc}
                onClick={() => void V2x9(V2w6.id, "pay")}
              >
                <Icon name="thanhToan" />
                Xác nhận thanh toán (Điều 36.4)
              </button>
            )}
            {V2w3 && V2w6.kind === "dip" && V2w6.status === "approved" && (
              <button
                className="btn"
                disabled={V2wc || !V4u6(V2w6)}
                onClick={() => void V2x9(V2w6.id, "dip")}
              >
                <Icon name="goi" />
                Cấp gói DIP (Điều 42)
              </button>
            )}
          </div>

          <h4>Tài liệu trong yêu cầu ({V2w6.items.length})</h4>
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th>#</th>
                  <th>Tên tệp</th>
                  <th>Mã hồ sơ</th>
                  <th>Kết quả (Điều 36.1.a)</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {V2w6.items.map((V2y9) => {
                  const V2ya =
                    V2w6.status === "approved" &&
                    V2y9.verdict === "duyet" &&
                    !V2y9.deleted &&
                    V4u6(V2w6) &&
                    V4u5(V2w6);
                  return (
                    <tr key={V2y9.id}>
                      <td>{V2y9.stt}</td>
                      <td>{V2y9.filename ?? V2y9.documentId}</td>
                      <td>{V2y9.hoSoRef ?? "—"}</td>
                      <td title={V2y9.lyDo ?? undefined}>
                        {V2y9.verdict === "pending"
                          ? "—"
                          : V2y9.verdict === "duyet"
                            ? "Duyệt"
                            : `Không duyệt${V2y9.lyDo ? ` (${V2y9.lyDo})` : ""}${
                                V2y9.dieuKien
                                  ? " · tiếp cận có điều kiện"
                                  : ""
                              }`}
                        {V2y9.deleted ? " · đã xóa theo hạn (37.5/40.6)" : ""}
                      </td>
                      <td>
                        {V2w3 && V2w6.status === "submitted" && (
                          <>
                            <button
                              className="btn sm"
                              disabled={V2wc}
                              onClick={() => void V2xf(V2w6.id, V2y9.id, "duyet")}
                            >
                              Duyệt
                            </button>{" "}
                            <button
                              className="btn sm"
                              disabled={V2wc}
                              onClick={() => void V2xf(V2w6.id, V2y9.id, "tu_choi")}
                            >
                              Không
                            </button>
                          </>
                        )}
                        {V2ya && V2w6.kind === "doc" && (
                          <button
                            className="btn sm"
                            disabled={V2wc}
                            onClick={() => void V2xm(V2w6.id, V2y9.id)}
                          >
                            <Icon name="xem" />
                            Đọc (không tải — 37.4)
                          </button>
                        )}
                        {V2ya && (V2w6.kind === "ban_sao" || V2w6.kind === "ban_sao_ky") && (
                          <button
                            className="btn sm"
                            disabled={V2wc}
                            onClick={() =>
                              void V2xr(V2w6.id, V2y9.id, `ban-sao-${V2y9.hoSoRef ?? V2y9.stt}.pdf`)
                            }
                          >
                            <Icon name="taiVe" />
                            Tải bản sao (40.5)
                          </button>
                        )}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>

          {V2wi && (
            <div className="mt-3">
              <div className="flex justify-between">
                <h4 className="m-0">Bản đọc trên Hệ thống (Điều 37 — có watermark)</h4>
                <button className="btn sm" onClick={() => V2wj(null)}>
                  Đóng
                </button>
              </div>
              <embed src={V2wi} type="application/pdf" className="w-full h-140 mt-2" />
            </div>
          )}
        </div>
      )}
    </section>
  );
}
