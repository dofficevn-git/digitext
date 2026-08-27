// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";

import { V4v5, V4v4, V4v1, V4v2, type V4v3, type V4v0 } from "@/api/infoops";
import { isTauri } from "@/api/ipc";
import { V460 } from "@/state";

import { V4v8 } from "./infoFlow";
import { Icon } from "@/components/Icon";

export default function V4vd() {
  const { session: V31f } = V460();
  const V31g = V31f?.permissions ?? [];
  const V31h = V31g.includes("bk.manage");
  const [report, V31i] = useState<V4v0 | null>(null);
  const [V31j, V31k] = useState<V4v3[]>([]);
  const [V31l, V31m] = useState(false);
  const [V31n, V31o] = useState<string | null>(null);

  const [loai, V31p] = useState<V4v3["loai"]>("gia_tang");
  const [nguon, V31q] = useState(
    "Q1NETCB0w6BpIGxp4buHdSBsxrB1IHRy4buvICsgbcOjIG5ndeG7k24gcGjhuqduIG3hu4Ft",
  );
  const [V31r, V31s] = useState("");
  const [V31t, V31u] = useState<V4v3["ketQua"]>("dat");
  const [V31v, V31w] = useState("");

  const reload = useCallback(async () => {
    if (!isTauri()) return;
    try {
      V31i(await V4v1());
      V31k(await V4v4());
    } catch (V31x) {
      V31o(`S2jDtG5nIHThuqNpIMSRxrDhu6NjIGThu68gbGnhu4d1OiA=${String(V31x)}`);
    }
  }, []);

  V41p(reload, { topics: ["ops"] });

  const V31y = async () => {
    V31m(true);
    try {
      const r = await V4v2();
      V31o(`xJDDoyBraeG7g20g${r.daKiem}IHTDoGkgbGnhu4d1IOKAlCA=${r.loi}IGzhu5dpLg==`);
      await reload();
    } catch (V31z) {
      V31o(`Q2jhuqF5IGtp4buDbSB0cmEgdGjhuqV0IGLhuqFpOiA=${String(V31z)}`);
    } finally {
      V31m(false);
    }
  };

  const V320 = async () => {
    if (!nguon.trim()) {
      V31o("VGhp4bq/dSBuZ3Xhu5NuIHNhbyBsxrB1ICjEkGnhu4F1IDMwLjMuYSku");
      return;
    }
    V31m(true);
    try {
      await V4v5({
        loai,
        nguon: nguon.trim(),
        phuongTien: V31r.trim() || null,
        ketQua: V31t,
        chiTiet: V31v.trim() || null,
      });
      V31o(
        "xJDDoyBnaGkgbmjhuq1uIGzhuqduIHNhbyBsxrB1IOKAlCBuaOG7myBs4bqtcCBuaOG6rXQga8O9L2Jpw6puIGLhuqNuIChN4bulYyAyKzMgUGjhu6UgbOG7pWMgVklJKS4=",
      );
      V31w("");
      await reload();
    } catch (V321) {
      V31o(`R2hpIG5o4bqtbiB0aOG6pXQgYuG6oWk6IA==${String(V321)}`);
    } finally {
      V31m(false);
    }
  };

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Bảo quản &amp; sao lưu (Điều 29–30 — TT 05/2025/TT-BNV)</h2>
        <div className="page-actions">
          <button className="btn" onClick={() => void reload()} disabled={V31l}>
            <Icon name="lamMoi" />
            Làm mới
          </button>
          {V31h && (
            <button className="btn primary" onClick={() => void V31y()} disabled={V31l}>
              <Icon name="tim" />
              Chạy kiểm tra một lô (Điều 29)
            </button>
          )}
        </div>
      </div>

      {V31n && <div className="card p-2.5 mb-2.5">{V31n}</div>}

      <div className="card p-4 mb-4">
        <h3 className="mt-0">Kiểm tra tài liệu — mục tiêu toàn bộ trong 03 năm (29.2)</h3>
        {report ? (
          <>
            <p>
              Đã kiểm {report.daKiem3Nam}/{report.tongTaiLieu} tài liệu trong 3 năm · {report.soLoi}{" "}
              lỗi. Worker tự kiểm một lô mỗi ngày.
            </p>
            {report.tepLoi.length > 0 && (
              <>
                <h4>Danh mục tệp lỗi (29.4.đ) — dùng lập biên bản kiểm tra (Mục 1 Phụ lục VII)</h4>
                <div className="tbl-scroll">
                  <table className="tbl">
                    <thead>
                      <tr>
                        <th>Tên tệp</th>
                        <th>Mã hồ sơ</th>
                        <th>Chi tiết lỗi</th>
                        <th>Thời điểm kiểm</th>
                      </tr>
                    </thead>
                    <tbody>
                      {report.tepLoi.map((t) => (
                        <tr key={t.documentId}>
                          <td>{t.filename}</td>
                          <td>{t.hoSoRef ?? "—"}</td>
                          <td>{t.chiTiet ?? "—"}</td>
                          <td>{new Date(t.checkedAt).toLocaleString("vi-VN")}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </>
            )}
          </>
        ) : (
          <p className="muted">Chưa có dữ liệu.</p>
        )}
      </div>

      <div className="card p-4">
        <h3 className="mt-0">Sổ sao lưu (Điều 30)</h3>
        {V31h && (
          <div className="grid gap-2.5 mb-3 [grid-template-columns:repeat(auto-fill,minmax(220px,1fr))]">
            <div className="field">
              <label>Loại (30.1)</label>
              <select
                className="inp"
                value={loai}
                onChange={(V322) => V31p(V322.target.value as V4v3["loai"])}
              >
                {Object.entries(V4v8).map(([k, V323]) => (
                  <option key={k} value={k}>
                    {V323}
                  </option>
                ))}
              </select>
            </div>
            <div className="field">
              <label>Nguồn (30.3.a/c)</label>
              <input className="inp" value={nguon} onChange={(V324) => V31q(V324.target.value)} />
            </div>
            <div className="field">
              <label>Phương tiện</label>
              <input className="inp" value={V31r} onChange={(V325) => V31s(V325.target.value)} />
            </div>
            <div className="field">
              <label>Kết quả kiểm tra sau sao lưu (30.3.d)</label>
              <select
                className="inp"
                value={V31t}
                onChange={(V326) => V31u(V326.target.value as V4v3["ketQua"])}
              >
                <option value="dat">Đạt</option>
                <option value="khong_dat">Không đạt</option>
              </select>
            </div>
            <div className="field">
              <label>Chi tiết / biện pháp khắc phục</label>
              <input className="inp" value={V31v} onChange={(V327) => V31w(V327.target.value)} />
            </div>
            <div className="field self-end">
              <button className="btn primary" onClick={() => void V320()} disabled={V31l}>
                + Ghi nhận lần sao lưu
              </button>
            </div>
          </div>
        )}
        {V31j.length === 0 ? (
          <p className="muted">
            Chưa có lần sao lưu nào — quá 26 giờ không có sao lưu đạt, hệ sẽ cảnh báo admin.
          </p>
        ) : (
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Thời điểm</th>
                  <th>Loại</th>
                  <th>Nguồn</th>
                  <th>Phương tiện</th>
                  <th>Kết quả</th>
                </tr>
              </thead>
              <tbody>
                {V31j.map((b) => (
                  <tr key={b.id}>
                    <td>{new Date(b.createdAt).toLocaleString("vi-VN")}</td>
                    <td>{V4v8[b.loai] ?? b.loai}</td>
                    <td>{b.nguon}</td>
                    <td>{b.phuongTien ?? "—"}</td>
                    <td title={b.chiTiet ?? undefined}>
                      {b.ketQua === "dat" ? "xJDhuqF0" : "S2jDtG5nIMSR4bqhdA=="}
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
