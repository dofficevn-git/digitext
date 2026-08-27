// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useMemo, useState } from "react";
import { V41p } from "@/live/useLive";
import "./laphoso.css";
import { isTauri } from "@/api/ipc";
import { V40w, type V40v } from "@/api/archive";
import { V48j } from "@/api/catalog";
import type { V3ym, V3yl, V3yo } from "@/domain/types";
import { V4br, type V4bm } from "@/api/dossier";
import { bangTrongText, chuaChonPhong } from "./phongScope";
import V4dy from "./VanBanPanel";
import { V4dr, V4cm, V4ct, V4cl, V4dt, V4dw, V4du, V4co, type V4dv } from "./logic";
import { Icon } from "@/components/Icon";

const V4ed = [10, 25, 50, 100];

export default function HeThongHoaPane() {
  const [V14l, V14m] = useState<V40v[]>([]);
  const [V14n, V14o] = useState<V3yl[]>([]);
  const [V14p, V14q] = useState<V3ym[]>([]);
  const [V14r, V14s] = useState<V3yo[]>([]);

  const [maDa, V14t] = useState("");
  const [V14u, V14v] = useState("");
  const [V14w, V14x] = useState("");
  const [V14y, V14z] = useState<string[]>([]);
  const [V150, V151] = useState("");
  const [V152, V153] = useState(false);
  const [V154, V155] = useState(false);
  const [V156, V157] = useState(false);
  const [V158, V159] = useState(false);

  const [V15a, V15b] = useState<V4bm[]>([]);
  const [V15c, V15d] = useState<V4bm | null>(null);
  const [V15e, V15f] = useState<"tam" | "thbq" | null>(null);

  const [cols, V15g] = useState({ bo: false, tap: false, sotam: false, tthua: false });

  const [V15h, V15i] = useState(50);
  const [V15j, V15k] = useState(1);

  const V15l = useCallback(() => {
    V40w()
      .then(V14m)
      .catch(() => {});
    V48j()
      .then((c) => {
        V14o(c.phongBans);
        V14q(c.mangs);
        V14s(c.thbqs);
      })
      .catch(() => {});
  }, []);
  V41p(V15l, { topics: ["catalog", "hoso"], enabled: isTauri() });

  const V15m = useCallback(() => {
    if (!isTauri()) return;

    if (chuaChonPhong(maDa)) {
      V15b([]);
      return;
    }
    V4br({
      maDa,
      maDv: V14u || undefined,
      mangCvid: V14w ? Number(V14w) : undefined,
    })
      .then(V15b)
      .catch(() => V15b([]));
  }, [maDa, V14u, V14w]);

  V41p(V15m, { topics: ["dossier", "khogia"] });

  const V15n = useMemo(() => {
    let V15o = V15a;
    const q = V150.trim().toLowerCase();
    if (q) V15o = V15o.filter((r) => V4ct(r, q));
    if (V14y.length && !V14y.includes("-1")) {
      const set = new Set(V14y.map(Number));
      V15o = V15o.filter((r) => r.thbq != null && set.has(r.thbq));
    }
    if (V152)
      V15o = V15o.filter((r) => V4co(V14r, r.thbq).toLowerCase().includes("dsSpbmggdmnhu4Vu"));
    if (V154) V15o = V15o.filter((r) => (r.trangThai || "").toUpperCase().includes("GL"));
    if (V156) V15o = V15o.filter((r) => (r.trangThai || "").toLowerCase().includes("het"));
    if (V158) V15o = V15o.filter((r) => r.trungThua);
    return V15o;
  }, [V15a, V150, V14y, V152, V154, V156, V158, V14r]);

  useEffect(() => V15k(1), [V15n.length]);

  const V15p = V4cl(V14n, maDa);
  const V15q = V4cm(V14p, V14u);

  const V15r = (v: string) => {
    V14t(v);
    V14v("");
    V14x("");
  };
  const V15s = (v: string) => {
    V14v(v);
    V14x("");
  };

  const V15t = async (V15u: V4dv) => {
    V15f(null);
    try {
      const V15v = await import("xlsx");
      const V15w = V4dw(V15n, V14r, V15u);
      const data = V15w.map((r) => ({
        TcOjIEjhu5MgU8ah: r.maHoSo ?? "",
        "SOG7mXAgU+G7kQ==": r.hopSo ?? "",
        "SFMgU+G7kQ==": r.thuTuSapXep ?? "",
        "QsOzIFPhu5E=": r.boSo ?? "",
        "VOG6rXAgU+G7kQ==": r.tapSo ?? "",
        "U+G7kSBU4bqhbQ==": r.soTam,
        "VGnDqnUgxJDhu4E=": r.tieuDe,
        THBQ: V4co(V14r, r.thbq),
        "U+G7kSBU4bud": V4dt(r.soTo),
        "Ti5C4bqvdCDEkOG6p3U=": V4dr(r.ngayBatDau),
        "Ti5L4bq/dCBUaMO6Yw==": V4dr(r.ngayKetThuc),
        "VHLhuqFuZyBUaMOhaQ==": V4du(r, V14r).label,
      }));
      const V15x = V15v.utils.json_to_sheet(data);
      const V15y = V15v.utils.book_new();
      V15v.utils.book_append_sheet(V15y, V15x, "MucLuc");
      V15v.writeFile(V15y, `muc-luc-ho-so-${maDa || "tat-ca"}.xlsx`);
    } catch (V15z) {
      window.alert(V15z instanceof Error ? V15z.message : String(V15z));
    }
  };

  const V160 = V15n.length;
  const V161 = Math.max(1, Math.ceil(V160 / V15h));
  const V162 = Math.min(V15j, V161);
  const V163 = useMemo(() => V15n.slice((V162 - 1) * V15h, V162 * V15h), [V15n, V162, V15h]);

  if (V15c) {
    return (
      <V4dy
        dossier={V15c}
        onBack={() => {
          V15d(null);
          V15m();
        }}
      />
    );
  }

  const V164 = 10 + Number(cols.bo) + Number(cols.tap) + Number(cols.sotam) + Number(cols.tthua);

  return (
    <div className="lhs">
      <div className="content-panel">
        <div className="center">
          <h4>
            <b>HỆ THỐNG HÓA HỒ SƠ</b>
          </h4>
        </div>
        <hr />

        {}
        <div className="row">
          <div className="col-4">
            <div className="input-group">
              <label className="input-group-addon w-1/4">Phông</label>
              <select
                className="form-control"
                value={maDa}
                onChange={(V165) => V15r(V165.target.value)}
              >
                <option value="">-- CHỌN PHÔNG LƯU TRỮ --</option>
                {V14l.map((p) => (
                  <option key={p.maDuAn} value={p.maDuAn}>
                    {p.tenDuAn}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="col-4">
            <div className="input-group">
              <label className="input-group-addon w-1/4">Phòng ban</label>
              <select
                className="form-control"
                value={V14u}
                disabled={!maDa}
                onChange={(V166) => V15s(V166.target.value)}
              >
                <option value="">-- CHỌN PHÒNG BAN --</option>
                {V15p.map((V167) => (
                  <option key={V167.maDv} value={V167.maDv}>
                    {V167.tenDonVi}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="col-4">
            <div className="input-group">
              <label className="input-group-addon w-1/4">Mảng CV</label>
              <select
                className="form-control"
                value={V14w}
                disabled={!V14u}
                onChange={(V168) => V14x(V168.target.value)}
              >
                <option value="">-- CHỌN MẢNG CÔNG VIỆC --</option>
                {V15q.map((m) => (
                  <option key={m.mangCvid} value={m.mangCvid}>
                    {m.tenMang}
                  </option>
                ))}
              </select>
            </div>
          </div>
        </div>

        {}
        <div className="row items-center mt-1.25">
          <div className="col-4">
            <div className="input-group">
              <label className="input-group-addon w-1/4 text-sm">THBQ</label>
              <select
                className="form-control h-8.5"
                multiple
                value={V14y}
                onChange={(V169) => V14z([...V169.target.selectedOptions].map((o) => o.value))}
              >
                <option value="-1">-- Tất cả THBQ--</option>
                {V14r.map((t) => (
                  <option key={t.thbqid} value={t.thbqid}>
                    {t.tenThbq}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="col-2">
            <label className="labelEx">
              <input
                type="checkbox"
                className="inputEx"
                checked={V152}
                onChange={(V16a) => V153(V16a.target.checked)}
              />{" "}
              Vĩnh viễn
            </label>
          </div>
          <div className="col-2">
            <label className="labelEx">
              <input
                type="checkbox"
                className="inputEx"
                checked={V154}
                onChange={(V16b) => V155(V16b.target.checked)}
              />{" "}
              Giữ lại
            </label>
          </div>
          <div className="col-2">
            <label className="labelEx">
              <input
                type="checkbox"
                className="inputEx"
                checked={V156}
                onChange={(V16c) => V157(V16c.target.checked)}
              />{" "}
              Hết THBQ
            </label>
          </div>
          <div className="col-2">
            <label className="labelEx">
              <input
                type="checkbox"
                className="inputEx"
                checked={V158}
                onChange={(V16d) => V159(V16d.target.checked)}
              />{" "}
              Trùng thừa
            </label>
          </div>
        </div>

        {}
        <div className="row items-center mt-1.25">
          <div className="col-10">
            <input
              className="form-control solo"
              value={V150}
              placeholder="Tmjhuq1wIGvDvSB04buxIGLhuqV0IGvhu7MgdsOgIG5o4bqlbiBwaMOtbSBFbnRlciDEkeG7gyB0w6xtIGtp4bq/bQ=="
              onChange={(V16e) => V151(V16e.target.value)}
              onKeyDown={(V16f) => V16f.key === "Enter" && V15m()}
            />
          </div>
          <div className="col-2 text-right">
            <button className="btn btn-outline-success btn-round" onClick={() => V15m()}>
              <Icon name="tim" /> Tìm kiếm
            </button>
          </div>
        </div>

        {}
        <div className="actionbar left relative">
          {V15e && <div className="dd-backdrop" onClick={() => V15f(null)} />}
          <div className="dropdown">
            <button
              className="btn btn-outline-success btn-round dropdown-toggle"
              onClick={() => V15f(V15e === "tam" ? null : "tam")}
            >
              <Icon name="xuatTep" /> Xuất mục lục hồ sơ tạm
            </button>
            {V15e === "tam" && (
              <div className="dropdown-menu w-75">
                <a className="dropdown-item" onClick={() => void V15t("timeAsc")}>
                  <Icon name="moNhanh" /> Mặt hoạt động-Thời gian
                </a>
                <a className="dropdown-item" onClick={() => void V15t("timeDesc")}>
                  <Icon name="moNhanh" /> Thời gian-Mặt hoạt động
                </a>
              </div>
            )}
          </div>
          <span className="w-2" />
          <div className="dropdown">
            <button
              className="btn btn-outline-success btn-round dropdown-toggle"
              onClick={() => V15f(V15e === "thbq" ? null : "thbq")}
            >
              <Icon name="xuatTep" /> Xuất mục lục hồ sơ theo THBQ
            </button>
            {V15e === "thbq" && (
              <div className="dropdown-menu w-125">
                <a className="dropdown-item" onClick={() => void V15t("thbqAsc")}>
                  <Icon name="moNhanh" /> Xuất theo mẫu Thời hạn bảo quản tăng dần - Mặt hoạt động -
                  Thời gian
                </a>
                <a className="dropdown-item" onClick={() => void V15t("thbqDesc")}>
                  <Icon name="moNhanh" /> Xuất theo mẫu Thời hạn bảo quản giảm dần - Mặt hoạt động -
                  Thời gian
                </a>
                <a className="dropdown-item" onClick={() => void V15t("thbqAsc")}>
                  <Icon name="moNhanh" /> Xuất theo mẫu Thời hạn bảo quản tăng dần - Thời gian - Mặt
                  hoạt động
                </a>
                <a className="dropdown-item" onClick={() => void V15t("thbqDesc")}>
                  <Icon name="moNhanh" /> Xuất theo mẫu Thời hạn bảo quản giảm dần - Thời gian - Mặt
                  hoạt động
                </a>
              </div>
            )}
          </div>
        </div>

        <hr />

        {}
        <div className="tbl-scroll">
          <table className="dataTable">
            <thead>
              <tr>
                <th>Mã Hồ Sơ</th>
                <th>Hộp Số</th>
                <th>HS Số</th>
                {cols.bo && <th>Bó Số</th>}
                {cols.tap && <th>Tập Số</th>}
                {cols.sotam && <th>Số Tạm</th>}
                <th>Tiêu Đề</th>
                <th>THBQ</th>
                <th>Số Tờ</th>
                <th>N.Bắt Đầu</th>
                <th>N.Kết Thúc</th>
                {cols.tthua && <th>Trùng Thừa</th>}
                <th>Trạng Thái</th>
                <th>Văn Bản</th>
              </tr>
            </thead>
            <tbody>
              {V163.length === 0 ? (
                <tr>
                  <td className="dt-empty" colSpan={V164}>
                    {bangTrongText(maDa)}
                  </td>
                </tr>
              ) : (
                V163.map((r, i) => {
                  const V16g = V4du(r, V14r);
                  return (
                    <tr key={r.id} className={i % 2 === 0 ? "odd" : "even"}>
                      <td className="text-center">{r.maHoSo}</td>
                      <td className="text-center">{r.hopSo}</td>
                      <td className="text-center">{r.thuTuSapXep ?? ""}</td>
                      {cols.bo && <td className="text-center">{r.boSo}</td>}
                      {cols.tap && <td className="text-center">{r.tapSo}</td>}
                      {cols.sotam && <td className="text-left">{r.soTam}</td>}
                      <td className="text-justify">{r.tieuDe}</td>
                      <td className="text-center">{V4co(V14r, r.thbq)}</td>
                      <td className="text-left">{V4dt(r.soTo)}</td>
                      <td className="text-center">{V4dr(r.ngayBatDau)}</td>
                      <td className="text-center">{V4dr(r.ngayKetThuc)}</td>
                      {cols.tthua && (
                        <td className="text-center">
                          <input type="checkbox" checked={r.trungThua} disabled />
                        </td>
                      )}
                      <td className="text-center">
                        <span className={`badge-thbq ${V16g.cls}`}>{V16g.label}</span>
                      </td>
                      <td className="text-center">
                        <a
                          className="ic-word"
                          onClick={() => V15d(r)}
                          title="VGjDqm0gdGjDtG5nIHRpbiB2xINuIGLhuqNu"
                        >
                          <Icon name="tepWord" />
                        </a>
                        <b className="cnt-red"> ({r.soVanBan}) </b>
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
            <tfoot>
              <tr>
                <td className="footer-restore" colSpan={V164}>
                  <a
                    className="toggle-vis"
                    title="SGnhu4NuIHRo4buLIHRvw6BuIGLhu5kgY+G7mXQ="
                    onClick={() => V15g({ bo: true, tap: true, sotam: true, tthua: true })}
                  >
                    <Icon name="luoi" />
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.bo ? "on" : "off"}`}
                    onClick={() => V15g((c) => ({ ...c, bo: !c.bo }))}
                  >
                    Bó số
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.tap ? "on" : "off"}`}
                    onClick={() => V15g((c) => ({ ...c, tap: !c.tap }))}
                  >
                    Tập số
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.sotam ? "on" : "off"}`}
                    onClick={() => V15g((c) => ({ ...c, sotam: !c.sotam }))}
                  >
                    Số tạm
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.tthua ? "on" : "off"}`}
                    onClick={() => V15g((c) => ({ ...c, tthua: !c.tthua }))}
                  >
                    T.Thừa
                  </a>
                </td>
              </tr>
            </tfoot>
          </table>
        </div>

        <div className="dt-bottom">
          <div className="dt-length">
            <label>
              Hiển thị{" "}
              <select
                value={V15h}
                onChange={(V16h) => {
                  V15i(Number(V16h.target.value));
                  V15k(1);
                }}
              >
                {V4ed.map((n) => (
                  <option key={n} value={n}>
                    {n}
                  </option>
                ))}
              </select>{" "}
              bản ghi.
            </label>
          </div>
          <div className="pagination">
            <button className="pbtn" disabled={V162 <= 1} onClick={() => V15k(V162 - 1)}>
              <Icon name="truoc" />
            </button>
            <button className="pbtn current">{V162}</button>
            <button className="pbtn" disabled={V162 >= V161} onClick={() => V15k(V162 + 1)}>
              <Icon name="sau" />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
