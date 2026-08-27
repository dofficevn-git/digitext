// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";

import {
  V4uk,
  V4ui,
  V4uh,
  V4ug,
  V4uc,
  V4uf,
  V4ue,
  V4ud,
  type V4u9,
  type V4ub,
  type V4u8,
} from "@/api/disposal";
import { V4ei, type V4eg } from "@/api/docs";
import { isTauri } from "@/api/ipc";
import { V460 } from "@/state";

import { V4un, V4uq, V4um, V4uo, V4ul, V4up } from "./disposalFlow";
import { Icon } from "@/components/Icon";

export default function V4ur() {
  const { session: V2yr } = V460();
  const V2ys = V2yr?.permissions ?? [];
  const V2yt = V2ys.includes("disposal.review");
  const V2yu = V2ys.includes("disposal.decide");
  const [V2yv, V2yw] = useState<V4u8[]>([]);
  const [V2yx, V2yy] = useState<V4u9[]>([]);
  const [V2yz, V2z0] = useState<V4ub | null>(null);
  const [V2z1, V2z2] = useState<string[]>([]);
  const [V2z3, V2z4] = useState<V4eg[]>([]);
  const [V2z5, V2z6] = useState(false);
  const [V2z7, V2z8] = useState(false);
  const [V2z9, V2za] = useState<string | null>(null);

  const reload = useCallback(async () => {
    if (!isTauri()) return;
    try {
      V2yw(await V4uc());
      V2yy(await V4ug());
    } catch (V2zb) {
      V2za(`S2jDtG5nIHThuqNpIMSRxrDhu6NjIGThu68gbGnhu4d1OiA=${String(V2zb)}`);
    }
  }, []);

  V41p(reload, { topics: ["disposal", "docs"] });

  const V2zc = async (V2zd: () => Promise<void>) => {
    V2z8(true);
    V2za(null);
    try {
      await V2zd();
    } catch (V2ze) {
      V2za(`VGhhbyB0w6FjIHRo4bqldCBi4bqhaTog${String(V2ze)}`);
    } finally {
      V2z8(false);
    }
  };

  const V2zf = () =>
    V2zc(async () => {
      const r = await V4ud();
      V2za(
        `UXXDqXQgeG9uZzog${r.trungLap}IHRyw7luZyBs4bq3cCAoNDguMiksIA==${r.hetThoiHan}IGjhur90IHRo4budaSBo4bqhbiAoNDguMyku`,
      );
      await reload();
    });

  const V2zg = () =>
    V2zc(async () => {
      if (V2z3.length === 0) V2z4(await V4ei());
      V2z6((v) => !v);
    });

  const V2zh = (doc: V4eg) =>
    V2zc(async () => {
      const V2zi = window
        .prompt(
          `SOG6oW4gbMawdSB0cuG7ryBjaG8gIg==${doc.filename}IiAoWVlZWS1NTS1ERCwgYuG7jyB0cuG7kW5nID0geG/DoSBo4bqhbik6`,
        )
        ?.trim();
      if (V2zi === undefined) return;
      await V4ue(doc.id, V2zi ? `${V2zi}T00:00:00+07:00` : null);
      V2za(
        `xJDDoyA=${V2zi ? `xJHhurd0IGjhuqFuIA==${V2zi}` : "YuG7jyBo4bqhbg=="} cho ${doc.filename}.`,
      );
    });

  const V2zj = (V2zk: V4u8) =>
    V2zc(async () => {
      const V2zl = window.prompt("TMO9IGRvIEdJ4buuIEzhuqBJIChsxrB1IHbhur90KTo=")?.trim();
      if (!V2zl) return;
      await V4uf(V2zk.id, "keep", V2zl);
      await reload();
    });

  const V2zm = (V2zn: V4u8) =>
    V2zc(async () => {
      const V2zo = window
        .prompt(
          "R2hpIGNow7ogKGLhuqNuIGdp4bqleSAvIHbhuq10IG1hbmcgdGluIGtow6FjIOKAlCDEkGnhu4F1IDQ4LjUpOg==",
          V2zn.ghiChu ?? "",
        )
        ?.trim();
      if (V2zo === undefined) return;
      await V4uf(V2zn.id, "note", V2zo);
      await reload();
    });

  const V2zp = () =>
    V2zc(async () => {
      const V2zq = Number(
        window.prompt(
          "TsSDbSBj4bunYSDEkeG7o3QgeMOpdCBo4buneTo=",
          String(new Date().getFullYear()),
        ) ?? "",
      );
      if (!Number.isFinite(V2zq) || V2zq < 2000) return;
      const b = await V4ui(V2zq, V2z1);
      V2z2([]);
      V2za(
        `xJDDoyBs4bqtcCDEkeG7o3QgeMOpdCBo4buneSBuxINtIA==${V2zq}IOKAlCBixrDhu5tjIEjhu5lpIMSR4buTbmcgKMSQaeG7gXUgNDkuMiku`,
      );
      await reload();
      V2z0(await V4uh(b.id));
    });

  const V2zr = (V2zs: string) =>
    V2zc(async () => {
      V2z0(await V4uh(V2zs));
    });

  const V2zt = (b: V4u9, V2zu: "council" | "appraise" | "decide" | "execute") =>
    V2zc(async () => {
      if (V2zu === "council" || V2zu === "appraise") {
        const V2zv =
          V2zu === "council"
            ? "S+G6v3QgcXXhuqMgbMOgbSB2aeG7h2MgY+G7p2EgSOG7mWkgxJHhu5NuZyAoxJBp4buBdSA0OS4yLmMpOg=="
            : "w50ga2nhur9uIHRo4bqpbSDEkeG7i25oIGPhu6dhIGPGoSBxdWFuIHF14bqjbiBsw70gKMSQaeG7gXUgNDkuNC81MC4yKTo=";
        const V2zw = window.prompt(V2zv)?.trim();
        if (!V2zw) return;
        await V4uk(b.id, V2zu, { note: V2zw });
      } else if (V2zu === "decide") {
        const V2zx = window
          .prompt("U+G7kSBRdXnhur90IMSR4buLbmggaOG7p3kgKMSQaeG7gXUgNDkuNik6")
          ?.trim();
        if (!V2zx) return;
        let V2zy = false;
        if (V4uq(b)) {
          V2zy = window.confirm(
            "Q2jGsGEgY8OzIMO9IGtp4bq/biB0aOG6qW0gxJHhu4tuaC4gWMOhYyBuaOG6rW4gVOG7sCBRVVnhur5UIHRoZW8gxJBp4buBdSA0OS44IA==" +
              "KGtow7RuZyB0aHXhu5ljIG5ndeG7k24gbuG7mXAgbMawdSB0cuG7ryBs4buLY2ggc+G7rSk/",
          );
          if (!V2zy) return;
        }
        await V4uk(b.id, "decide", { quyetDinhSo: V2zx, tuQuyet: V2zy });
      } else {
        if (
          !window.confirm(
            "VEhBTyBUw4FDIEjhu6ZZIHPhur0gWMOTQSBWxKhOSCBWSeG7hE4gb2JqZWN0IGPhu6dhIGPDoWMgdMOgaSBsaeG7h3UgdHJvbmcgxJHhu6N0IGto4buPaSBraG8g" +
              "KMSQaeG7gXUgNDkuNykuIFRp4bq/cCB04bulYz8=",
          )
        )
          return;
        const V2zz = await V4uk(b.id, "execute", {});
        V2za(
          `xJDDoyBo4buneSA=${V2zz?.disposed ?? 0}IHTDoGkgbGnhu4d1IOKAlCB24bq/dCDEkeG6p3kgxJHhu6cgdHJvbmcgbmjhuq10IGvDvSDEkeG7o3Qu`,
        );
      }
      await reload();
      V2z0(await V4uh(b.id));
    });

  const V300 = V2yv.filter((V301) => V301.status === "xem_xet" && !V301.batchId);

  return (
    <section className="view active">
      <div className="page-head">
        <h2>Hủy tài liệu hết giá trị (Điều 48–50 — TT 05/2025/TT-BNV)</h2>
        <div className="page-actions">
          <button className="btn" onClick={() => void reload()} disabled={V2z7}>
            <Icon name="lamMoi" />
            Làm mới
          </button>
          {V2yt && (
            <>
              <button className="btn" onClick={() => void V2zg()} disabled={V2z7}>
                <Icon name="hanLuuTru" />
                Hạn lưu trữ
              </button>
              <button className="btn primary" onClick={() => void V2zf()} disabled={V2z7}>
                <Icon name="tim" />
                Quét (48.2/48.3)
              </button>
            </>
          )}
        </div>
      </div>

      {V2z9 && <div className="card p-2.5 mb-2.5">{V2z9}</div>}

      {V2z5 && (
        <div className="card p-4 mb-4">
          <h3 className="mt-0">Đặt thời hạn lưu trữ (nguồn quét Điều 48.3)</h3>
          <div className="tbl-scroll max-h-65 overflow-y-auto">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Tên tệp</th>
                  <th>Mã hồ sơ</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {V2z3.map((d) => (
                  <tr key={d.id}>
                    <td>{d.filename}</td>
                    <td>{d.hoSoRef ?? "—"}</td>
                    <td>
                      <button className="btn sm" disabled={V2z7} onClick={() => void V2zh(d)}>
                        Đặt hạn
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      <div className="card p-4 mb-4">
        <div className="flex justify-between items-center">
          <h3 className="m-0">Danh mục tài liệu hết giá trị (Điều 48.4)</h3>
          {V2yt && (
            <button
              className="btn primary"
              disabled={V2z7 || V2z1.length === 0}
              onClick={() => void V2zp()}
            >
              <Icon name="bienBan" />
              Lập đợt xét hủy ({V2z1.length}) — Điều 49.1
            </button>
          )}
        </div>
        {V2yv.length === 0 ? (
          <p className="muted">Danh mục trống — hệ tự quét hằng ngày (48.2/48.3).</p>
        ) : (
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th className="w-7"></th>
                  <th>Tên tệp</th>
                  <th>Mã hồ sơ</th>
                  <th>Số trang</th>
                  <th>Lý do (48.2/48.3)</th>
                  <th>Ghi chú (48.5)</th>
                  <th>Trạng thái</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {V2yv.map((V302) => (
                  <tr key={V302.id}>
                    <td>
                      {V302.status === "xem_xet" && !V302.batchId && (
                        <input
                          type="checkbox"
                          checked={V2z1.includes(V302.id)}
                          onChange={(V303) =>
                            V2z2((V304) =>
                              V303.target.checked
                                ? [...V304, V302.id]
                                : V304.filter((x) => x !== V302.id),
                            )
                          }
                        />
                      )}
                    </td>
                    <td>{V302.filename}</td>
                    <td>{V302.hoSoRef ?? "—"}</td>
                    <td>{V302.pageCount ?? "—"}</td>
                    <td>{V4ul[V302.lyDo]}</td>
                    <td>{V302.ghiChu ?? "—"}</td>
                    <td>{V4um[V302.status]}</td>
                    <td>
                      {V2yt && V302.status === "xem_xet" && !V302.batchId && (
                        <>
                          <button
                            className="btn sm"
                            disabled={V2z7}
                            onClick={() => void V2zm(V302)}
                          >
                            Ghi chú
                          </button>{" "}
                          <button
                            className="btn sm"
                            disabled={V2z7}
                            onClick={() => void V2zj(V302)}
                          >
                            Giữ lại
                          </button>
                        </>
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
        {V300.length > 0 && (
          <p className="muted">{V300.length} dòng đang chờ xem xét, chưa vào đợt.</p>
        )}
      </div>

      <div className="card p-4 mb-4">
        <h3 className="mt-0">Các đợt xét hủy</h3>
        {V2yx.length === 0 ? (
          <p className="muted">Chưa có đợt nào.</p>
        ) : (
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Năm</th>
                  <th>Trạng thái</th>
                  <th>Quyết định</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {V2yx.map((b) => (
                  <tr key={b.id}>
                    <td>{b.nam}</td>
                    <td>{V4un[b.status]}</td>
                    <td>
                      {b.quyetDinhSo ?? "—"}
                      {b.tuQuyet ? "ICh04buxIHF1eeG6v3Qg4oCUIDQ5Ljgp" : ""}
                    </td>
                    <td>
                      <button className="btn sm" disabled={V2z7} onClick={() => V2zr(b.id)}>
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

      {V2yz && (
        <div className="card p-4">
          <h3 className="mt-0">
            Đợt xét hủy {V2yz.nam} — {V4un[V2yz.status]}
          </h3>
          <p className="muted">
            {V2yz.hoiDongNote
              ? `SOG7mWkgxJHhu5NuZzog${V2yz.hoiDongNote}`
              : "SOG7mWkgxJHhu5NuZyBjaMawYSBjaOG7kXQ="}
            {V2yz.thamDinhNote ? `IMK3IFRo4bqpbSDEkeG7i25oOiA=${V2yz.thamDinhNote}` : ""}
            {V2yz.quyetDinhSo ? `IMK3IFF1eeG6v3QgxJHhu4tuaDog${V2yz.quyetDinhSo}` : ""}
          </p>
          <div className="flex gap-2 flex-wrap mb-3">
            {V4up(V2yz, V2yt, V2yu).map((a) => (
              <button
                key={a.action}
                className={a.action === "execute" ? "btn danger" : "btn primary"}
                disabled={V2z7}
                onClick={() => void V2zt(V2yz, a.action)}
              >
                {a.label}
              </button>
            ))}
          </div>

          <h4>Tài liệu trong đợt ({V2yz.entries.length})</h4>
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th>Tên tệp</th>
                  <th>Lý do</th>
                  <th>Ghi chú</th>
                  <th>Trạng thái</th>
                </tr>
              </thead>
              <tbody>
                {V2yz.entries.map((V305) => (
                  <tr key={V305.id}>
                    <td>{V305.filename}</td>
                    <td>{V4ul[V305.lyDo]}</td>
                    <td>{V305.ghiChu ?? "—"}</td>
                    <td>{V4um[V305.status]}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          <h4>Vết toàn bộ quá trình (Điều 49.7.b)</h4>
          <ul className="pl-4.5">
            {V2yz.log.map((l, i) => (
              <li key={i}>
                {new Date(l.createdAt).toLocaleString("vi-VN")} — {V4uo[l.action] ?? l.action}
                {l.detail ? `: ${l.detail}` : ""}
              </li>
            ))}
          </ul>
        </div>
      )}
    </section>
  );
}
