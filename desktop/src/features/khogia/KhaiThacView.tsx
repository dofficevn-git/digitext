// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useMemo, useState } from "react";
import { V41p } from "@/live/useLive";
import "@/features/dossier/laphoso.css";
import "./khaithac.css";
import { isTauri } from "@/api/ipc";
import { V40w, type V40v } from "@/api/archive";
import { V48j } from "@/api/catalog";

import { V4by, type V4bq } from "@/api/dossier";
import { bangTrongText, chuaChonPhong } from "@/features/dossier/phongScope";
import type { V3ym, V3yl, V3yo } from "@/domain/types";
import { V4lj, V4ln, V4lm, type V4li, type V4ld, type V4le, type V4lh } from "@/api/khaithac";
import { V4dr, V4cm, V4cl, V4dt, V4co } from "@/features/dossier/logic";
import { V4m4, V4m3, V4m8, V4m0, V4m1 } from "./logic";
import { V4n3, V4n4, V4n5, V4n6, V4na, V4n7, V4n8, V4n9 } from "./khaithacExport";
import { Icon } from "@/components/Icon";

const V4nb = [10, 25, 50, 100];

interface V4nc {
  kho: boolean;
  gia: boolean;
  bo: boolean;
  tap: boolean;
  tthua: boolean;
  vban: boolean;
}

interface V4nd {
  maDa: string;
  maDv: string;
  mangCvid: string;
  thbqSel: string[];
  maKho: string;
  maGia: string;
  vinhVien: boolean;
  giuLai: boolean;
  hetThbq: boolean;
  trungThua: boolean;
  search: string;
}

const V4ne: V4nd = {
  maDa: "",
  maDv: "",
  mangCvid: "",
  thbqSel: ["-1"],
  maKho: "",
  maGia: "",
  vinhVien: false,
  giuLai: false,
  hetThbq: false,
  trungThua: false,
  search: "",
};

function V4nf(f: V4nd): V4ld {
  return {
    maDa: f.maDa || undefined,
    maDv: f.maDv || undefined,
    mangCvid: f.mangCvid ? Number(f.mangCvid) : undefined,
    thbqs: f.thbqSel.filter((x) => x !== "").map(Number),
    maKho: f.maKho ? Number(f.maKho) : undefined,
    maGia: f.maGia ? Number(f.maGia) : undefined,
    vinhVien: f.vinhVien,
    giuLai: f.giuLai,
    hetThbq: f.hetThbq,
    trungThua: f.trungThua,
    search: f.search || undefined,
  };
}

export default function KhaiThacView() {
  const [V1xd, V1xe] = useState<V40v[]>([]);
  const [V1xf, V1xg] = useState<V3yl[]>([]);
  const [V1xh, V1xi] = useState<V3ym[]>([]);
  const [V1xj, V1xk] = useState<V3yo[]>([]);
  const [V1xl, V1xm] = useState<V4lh[]>([]);
  const [V1xn, V1xo] = useState<V4li[]>([]);

  const [f, V1xp] = useState<V4nd>(V4ne);
  const [V1xq, V1xr] = useState<V4ld>({});

  const [V1xs, V1xt] = useState<V4le[]>([]);
  const [V1xu, V1xv] = useState(0);
  const [V1xw, V1xx] = useState(1);
  const [V1xy, V1xz] = useState(50);
  const [V1y0, V1y1] = useState("");
  const [V1y2, V1y3] = useState(false);
  const [V1y4, V1y5] = useState(false);
  const [V1y6, V1y7] = useState<string | null>(null);

  const [cols, V1y8] = useState<V4nc>({
    kho: false,
    gia: false,
    bo: false,
    tap: false,
    tthua: false,
    vban: false,
  });
  const [V1y9, V1ya] = useState<V4le | null>(null);
  const [V1yb, V1yc] = useState<V4bq[]>([]);

  const V1yd = useCallback(() => {
    V40w()
      .then(V1xe)
      .catch(() => {});
    V48j()
      .then((c) => {
        V1xg(c.phongBans);
        V1xi(c.mangs);
        V1xk(c.thbqs);
      })
      .catch(() => {});
    V4lm()
      .then(V1xm)
      .catch(() => {});
  }, []);
  V41p(V1yd, { topics: ["catalog", "hoso", "khogia"], enabled: isTauri() });

  useEffect(() => {
    if (!isTauri()) return;
    const V1ye = V4m4();
    if (Object.keys(V1ye).length) {
      V1xp((V1yf) => ({
        ...V1yf,
        maDa: V1ye.maDa ?? "",
        maDv: V1ye.maDv ?? "",
        mangCvid: V1ye.mangCvid != null ? String(V1ye.mangCvid) : "",
        thbqSel: V1ye.thbqs?.length ? V1ye.thbqs.map(String) : ["-1"],
        maKho: V1ye.maKho != null ? String(V1ye.maKho) : "",
        maGia: V1ye.maGia != null ? String(V1ye.maGia) : "",
        vinhVien: !!V1ye.vinhVien,
        giuLai: !!V1ye.giuLai,
        hetThbq: !!V1ye.hetThbq,
        trungThua: !!V1ye.trungThua,
        search: V1ye.search ?? "",
      }));

      V1xr({
        maDa: V1ye.maDa || undefined,
        maDv: V1ye.maDv || undefined,
        mangCvid: V1ye.mangCvid,
        thbqs: V1ye.thbqs ?? [],
        maKho: V1ye.maKho,
        maGia: V1ye.maGia,
        vinhVien: !!V1ye.vinhVien,
        giuLai: !!V1ye.giuLai,
        hetThbq: !!V1ye.hetThbq,
        trungThua: !!V1ye.trungThua,
        search: V1ye.search || undefined,
      });
    }
  }, []);

  const V1yg = useCallback(() => {
    V4ln(f.maKho ? Number(f.maKho) : undefined)
      .then(V1xo)
      .catch(() => V1xo([]));
  }, [f.maKho]);
  V41p(V1yg, { topics: ["khogia"], enabled: isTauri() });

  const V1yh = useCallback(() => {
    if (chuaChonPhong(V1xq.maDa)) {
      V1xt([]);
      V1xv(0);
      V1y7(null);
      V1y3(false);
      return Promise.resolve();
    }
    V1y3(true);
    V1y7(null);
    return V4lj({ ...V1xq, page: V1xw, pageSize: V1xy })
      .then((p) => {
        V1xt(p.rows);
        V1xv(p.total);
      })
      .catch((V1yi) => {
        V1xt([]);
        V1xv(0);
        V1y7(V1yi instanceof Error ? V1yi.message : String(V1yi));
      })
      .finally(() => V1y3(false));
  }, [V1xq, V1xw, V1xy]);
  V41p(V1yh, {
    topics: ["dossier", "khogia", "catalog", "hoso"],
    enabled: isTauri(),
  });

  const V1yj = V4cl(V1xf, f.maDa);
  const V1yk = V4cm(V1xh, f.maDv);

  const set = (V1yl: Partial<V4nd>) => V1xp((V1ym) => ({ ...V1ym, ...V1yl }));

  const V1yn = (v: string) => set({ maDa: v, maDv: "", mangCvid: "" });
  const V1yo = (v: string) => set({ maDv: v, mangCvid: "" });
  const V1yp = (v: string) => set({ maKho: v, maGia: "" });

  const V1yq = useCallback(() => {
    V1xx(1);
    V1xr(V4nf(f));
    V4m3(V4nf(f));
  }, [f]);

  const V1yr = (V1ys: "vinhVien" | "giuLai" | "hetThbq" | "trungThua") => {
    const V1yt = { ...f, [V1ys]: !f[V1ys] };
    V1xp(V1yt);
    V1xx(1);
    V1xr(V4nf(V1yt));
    V4m3(V4nf(V1yt));
  };

  const V1yu = async (r: V4le) => {
    V1ya(r);
    V1yc([]);
    try {
      V1yc(await V4by(r.hoSoId));
    } catch {
      V1yc([]);
    }
  };

  const V1yv = async (V1yw: () => Promise<number>, V1yx = "Đã kết xuất.") => {
    V1y5(true);
    try {
      const n = await V1yw();
      if (V1yx) window.alert(`${V1yx} (${n} dòng)`);
    } catch (V1yy) {
      window.alert(V1yy instanceof Error ? V1yy.message : String(V1yy));
    } finally {
      V1y5(false);
    }
  };

  const V1yz = useMemo(() => {
    if (!V1xq.maDa) return "Tất cả phông lưu trữ";
    return V1xd.find((p) => p.maDuAn === V1xq.maDa)?.tenDuAn ?? V1xq.maDa;
  }, [V1xq.maDa, V1xd]);

  const V1z0 = Math.max(1, Math.ceil(V1xu / V1xy));
  const V1z1 = Math.min(V1xw, V1z0);

  const V1z2 =
    10 +
    Number(cols.kho) +
    Number(cols.gia) +
    Number(cols.bo) +
    Number(cols.tap) +
    Number(cols.tthua) +
    Number(cols.vban);

  const V1z3 = (V1z4?: string) => V1xf.find((p) => p.maDv === V1z4)?.tenDonVi ?? "";
  const V1z5 = (V1z6?: number) => V1xh.find((m) => m.mangCvid === V1z6)?.tenMang ?? "";

  const V1z7 = (r: V4le) =>
    V4m8(r.tieuDeHs, f.search).map((V1z8, i) =>
      V1z8.mark ? <mark key={i}>{V1z8.text}</mark> : <span key={i}>{V1z8.text}</span>,
    );

  return (
    <div className="lhs khaithac">
      {V1y4 && (
        <div className="kt-busy">
          <span>
            <Icon name="dangCho" /> Đang xử lý…
          </span>
        </div>
      )}

      <div className="content-panel">
        <div className="center">
          <h4>
            <b>TÌM KIẾM &amp; KHAI THÁC TÀI LIỆU</b>
          </h4>
        </div>
        <hr />

        {}
        <div className="row">
          <div className="col-3">
            <div className="input-group">
              <label className="input-group-addon">Phông</label>
              <select
                className="form-control"
                value={f.maDa}
                onChange={(V1z9) => V1yn(V1z9.target.value)}
              >
                <option value="">--CHỌN PHÔNG LƯU TRỮ--</option>
                {V1xd.map((p) => (
                  <option key={p.maDuAn} value={p.maDuAn}>
                    {p.tenDuAn}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="col-3">
            <div className="input-group">
              <label className="input-group-addon">Phòng ban</label>
              <select
                className="form-control"
                value={f.maDv}
                disabled={!f.maDa}
                onChange={(V1za) => V1yo(V1za.target.value)}
              >
                <option value="">-- CHỌN PHÒNG BAN --</option>
                {V1yj.map((V1zb) => (
                  <option key={V1zb.maDv} value={V1zb.maDv}>
                    {V1zb.tenDonVi}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="col-3">
            <div className="input-group">
              <label className="input-group-addon">Mảng CV</label>
              <select
                className="form-control"
                value={f.mangCvid}
                disabled={!f.maDv}
                onChange={(V1zc) => set({ mangCvid: V1zc.target.value })}
              >
                <option value="">-- CHỌN MẢNG CÔNG VIỆC --</option>
                {V1yk.map((m) => (
                  <option key={m.mangCvid} value={m.mangCvid}>
                    {m.tenMang}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="col-3">
            <div className="input-group">
              <label className="input-group-addon">THBQ</label>
              <select
                className="form-control h-8.5"
                multiple
                value={f.thbqSel}
                onChange={(V1zd) =>
                  set({ thbqSel: [...V1zd.target.selectedOptions].map((o) => o.value) })
                }
              >
                <option value="-1">--Tất cả THBQ--</option>
                {V1xj.map((t) => (
                  <option key={t.thbqid} value={t.thbqid}>
                    {t.tenThbq}
                  </option>
                ))}
              </select>
            </div>
          </div>
        </div>

        {}
        <div className="row items-center">
          <div className="col-3">
            <div className="input-group">
              <label className="input-group-addon">Kho</label>
              <select
                className="form-control"
                value={f.maKho}
                onChange={(V1ze) => V1yp(V1ze.target.value)}
              >
                <option value="">--TẤT CẢ KHO--</option>
                {V1xl.map((k) => (
                  <option key={k.maKho} value={k.maKho}>
                    {k.tenKho}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="col-3">
            <div className="input-group">
              <label className="input-group-addon">Giá/kệ</label>
              <select
                className="form-control"
                value={f.maGia}
                onChange={(V1zf) => set({ maGia: V1zf.target.value })}
              >
                <option value="">--TẤT CẢ GIÁ--</option>
                {V1xn.map((g) => (
                  <option key={g.maGia} value={g.maGia}>
                    {g.tenGia}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="col-6 kt-checks">
            <label className="labelEx">
              <input
                type="checkbox"
                className="inputEx"
                checked={f.vinhVien}
                onChange={() => V1yr("vinhVien")}
              />{" "}
              Vĩnh viễn
            </label>
            <label className="labelEx">
              <input
                type="checkbox"
                className="inputEx"
                checked={f.giuLai}
                onChange={() => V1yr("giuLai")}
              />{" "}
              Giữ lại
            </label>
            <label className="labelEx">
              <input
                type="checkbox"
                className="inputEx"
                checked={f.hetThbq}
                onChange={() => V1yr("hetThbq")}
              />{" "}
              Hết THBQ
            </label>
            <label className="labelEx">
              <input
                type="checkbox"
                className="inputEx"
                checked={f.trungThua}
                onChange={() => V1yr("trungThua")}
              />{" "}
              Trùng thừa
            </label>
          </div>
        </div>

        {}
        <div className="row items-center">
          <div className="col-6">
            <input
              className="form-control solo searchInput"
              value={f.search}
              placeholder="Nhập ký tự bất kỳ và nhấn phím Enter để tìm kiếm"
              onChange={(V1zg) => set({ search: V1zg.target.value })}
              onKeyDown={(V1zh) => V1zh.key === "Enter" && V1yq()}
            />
          </div>
          <div className="col-6 text-right">
            <button className="btn btn-success btn-round" onClick={V1yq}>
              <Icon name="tim" /> Tìm kiếm
            </button>{" "}
            {}
            <button
              className="btn btn-outline-secondary btn-round"
              disabled={V1y4}
              onClick={() => void V1yv(() => V4n3({ ...V1xq }), "Đã xuất Metadata.")}
            >
              <Icon name="tepExcel" /> Xuất toàn bộ Metadata
            </button>
          </div>
        </div>

        {}
        <div className="row">
          <div className="col-12 kt-exportbar">
            <button
              className="btn btn-outline-success btn-round"
              disabled={V1y4}
              onClick={() => void V1yv(() => V4n4({ ...V1xq }), "Đã xuất MLHS.")}
            >
              <Icon name="xuatTep" /> Xuất MLHS
            </button>
            <button
              className="btn btn-outline-success btn-round"
              disabled={V1y4}
              onClick={() => void V1yv(() => V4n7({ ...V1xq }, V1yz), "")}
            >
              <Icon name="inRa" /> In MLHS
            </button>
            <button
              className="btn btn-outline-success btn-round"
              disabled={V1y4}
              onClick={() => void V1yv(() => V4n5({ ...V1xq }), "Đã xuất MLVB.")}
            >
              <Icon name="xuatTep" /> Xuất MLVB
            </button>
            <button
              className="btn btn-outline-success btn-round"
              disabled={V1y4}
              onClick={() => void V1yv(() => V4n8({ ...V1xq }), "")}
            >
              <Icon name="inRa" /> In MLVB
            </button>
            <button
              className="btn btn-outline-success btn-round"
              disabled={V1y4}
              onClick={() => void V1yv(() => V4na({ ...V1xq }, V1yz), "")}
            >
              <Icon name="inRa" /> In nhãn hộp
            </button>
            <button
              className="btn btn-outline-success btn-round"
              disabled={V1y4}
              onClick={() => void V1yv(() => V4n9({ ...V1xq }, V1z5), "")}
            >
              <Icon name="inRa" /> In phụ lục
            </button>
            <button
              className="btn btn-outline-success btn-round"
              disabled={V1y4}
              onClick={() =>
                void V1yv(() => V4n6({ ...V1xq }, V1z5), "Đã xuất phụ lục.")
              }
            >
              <Icon name="tepExcel" /> Xuất phụ lục
            </button>
          </div>
        </div>
        <hr />
      </div>

      {}
      <div className="content-panel pt-3">
        {V1y6 && <div className="kt-err">{V1y6}</div>}
        <div className="tbl-scroll">
          <table className="dataTable">
            <thead>
              <tr>
                <th className="w-15">ID</th>
                {cols.kho && <th className="w-22.5">Kho</th>}
                {cols.gia && <th className="w-22.5">Giá/Kệ</th>}
                <th className="w-25">Mã Hồ Sơ</th>
                <th className="w-15">Hộp Số</th>
                {cols.bo && <th className="w-15">Số bó</th>}
                {cols.tap && <th className="w-15">Số tập</th>}
                <th className="w-15">HS.Số</th>
                <th>Tiêu Đề</th>
                <th className="w-25">THBQ</th>
                <th className="w-13.75">Số Tờ</th>
                <th className="w-18.75">B.Đầu</th>
                <th className="w-18.75">K.Thúc</th>
                {cols.tthua && <th className="w-13.75">T.Thừa</th>}
                {cols.vban && <th className="w-17.5">V.Bản</th>}
                <th className="w-27.5">Trạng thái</th>
              </tr>
            </thead>
            <tbody>
              {V1y2 ? (
                <tr>
                  <td className="dt-empty" colSpan={V1z2}>
                    <Icon name="dangCho" /> Đang tải dữ liệu…
                  </td>
                </tr>
              ) : V1xs.length === 0 ? (
                <tr>
                  <td className="dt-empty" colSpan={V1z2}>
                    {bangTrongText(V1xq.maDa)}
                  </td>
                </tr>
              ) : (
                V1xs.map((r, i) => {
                  const V1zi = V4m0(r.status);
                  return (
                    <tr key={r.hoSoId} className={i % 2 === 0 ? "odd" : "even"}>
                      <td className="text-center">{r.hoSoId}</td>
                      {cols.kho && <td className="text-left">{r.tenKho ?? ""}</td>}
                      {cols.gia && <td className="text-left">{r.tenGia ?? ""}</td>}
                      <td className="text-center">{r.maHoSo ?? ""}</td>
                      <td className="text-center">{r.hopTaiLieuSo ?? ""}</td>
                      {cols.bo && <td className="text-center">{r.soBo ?? ""}</td>}
                      {cols.tap && <td className="text-center">{r.soTap ?? ""}</td>}
                      <td className="text-center">{r.hososo}</td>
                      <td className="text-justify">
                        <a
                          className="tbllink"
                          onClick={() => void V1yu(r)}
                          title="Chi tiết văn bản"
                        >
                          {V1z7(r)}
                        </a>
                      </td>
                      <td className="text-center">{r.thoiHanBaoQuanStr ?? V4co(V1xj, r.thbq)}</td>
                      <td className="text-center">{V4dt(r.soTo)}</td>
                      <td className="text-center">{V4dr(r.thoiGianBatDau)}</td>
                      <td className="text-center">{V4dr(r.thoiGianKetThuc)}</td>
                      {cols.tthua && (
                        <td className="text-center">
                          <input type="checkbox" checked={r.trungThua} disabled />
                        </td>
                      )}
                      {cols.vban && (
                        <td className="text-center">
                          <a
                            className="ic-word"
                            onClick={() => void V1yu(r)}
                            title="Xem văn bản"
                          >
                            <Icon name="tepWord" />
                          </a>
                          <b className="cnt-red"> ({r.soVanBan}) </b>
                        </td>
                      )}
                      <td className="text-center">
                        <span className={`badge-thbq ${V1zi.cls}`}>{V1zi.label}</span>
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
            <tfoot>
              <tr>
                <td className="footer-restore" colSpan={V1z2}>
                  <a
                    className="toggle-vis"
                    title="Hiển thị toàn bộ cột"
                    onClick={() =>
                      V1y8({
                        kho: true,
                        gia: true,
                        bo: true,
                        tap: true,
                        tthua: true,
                        vban: true,
                      })
                    }
                  >
                    <Icon name="luoi" />
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.kho ? "on" : "off"}`}
                    onClick={() => V1y8((c) => ({ ...c, kho: !c.kho }))}
                  >
                    Kho
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.gia ? "on" : "off"}`}
                    onClick={() => V1y8((c) => ({ ...c, gia: !c.gia }))}
                  >
                    Giá/Kệ
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.bo ? "on" : "off"}`}
                    onClick={() => V1y8((c) => ({ ...c, bo: !c.bo }))}
                  >
                    Số bó
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.tap ? "on" : "off"}`}
                    onClick={() => V1y8((c) => ({ ...c, tap: !c.tap }))}
                  >
                    Số tập
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.tthua ? "on" : "off"}`}
                    onClick={() => V1y8((c) => ({ ...c, tthua: !c.tthua }))}
                  >
                    T.Thừa
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.vban ? "on" : "off"}`}
                    onClick={() => V1y8((c) => ({ ...c, vban: !c.vban }))}
                  >
                    V.Bản
                  </a>
                </td>
              </tr>
            </tfoot>
          </table>
        </div>

        {}
        <div className="dt-bottom">
          <div className="dt-length">
            <label>
              Hiển thị{" "}
              <select
                value={V1xy}
                onChange={(V1zj) => {
                  V1xz(Number(V1zj.target.value));
                  V1xx(1);
                }}
              >
                {V4nb.map((n) => (
                  <option key={n} value={n}>
                    {n}
                  </option>
                ))}
              </select>{" "}
              bản ghi. &nbsp;<span className="muted">Tổng {V1xu} hồ sơ</span>
            </label>
          </div>
          <div className="pagination">
            <button className="pbtn" disabled={V1z1 <= 1} onClick={() => V1xx(V1z1 - 1)}>
              <Icon name="truoc" />
            </button>
            <button className="pbtn current">{V1z1}</button>
            <button className="pbtn" disabled={V1z1 >= V1z0} onClick={() => V1xx(V1z1 + 1)}>
              <Icon name="sau" />
            </button>
          </div>
          <div className="dt-goto">
            <label>Tới trang</label>
            <input
              type="number"
              min={1}
              max={V1z0}
              value={V1y0}
              onChange={(V1zk) => V1y1(V1zk.target.value)}
            />
            <button
              className="pbtn"
              onClick={() => {
                const p = Number(V1y0);
                if (p >= 1 && p <= V1z0) V1xx(p);
              }}
            >
              Đi
            </button>
          </div>
        </div>
      </div>

      {}
      {V1y9 && (
        <div className="lhs-overlay" onClick={() => V1ya(null)}>
          <div className="lhs-modal" onClick={(V1zl) => V1zl.stopPropagation()}>
            <div className="m-head">
              <h4>THÔNG TIN VĂN BẢN</h4>
              <button className="m-close" onClick={() => V1ya(null)}>
                ×
              </button>
            </div>
            <div className="m-body">
              <div className="kt-detail-head">
                <div>
                  <b>Phông:</b> <span className="kt-red">{V1y9.tenDa ?? V1y9.maDa ?? ""}</span>
                </div>
                <div>
                  <b>Phòng ban:</b> <span className="kt-red">{V1z3(V1y9.maDv)}</span>
                </div>
                <div>
                  <b>Mảng:</b> <span className="kt-red">{V1z5(V1y9.mangCvid)}</span>
                </div>
                <div>
                  <b>Tiêu đề:</b> <span className="kt-red">{V1y9.tieuDeHs}</span>
                </div>
                <div>
                  <b>Thời gian:</b> <span className="kt-red">{V4m1(V1y9)}</span>
                </div>
              </div>
              <div className="overflow-x-auto">
                <div className="tbl-scroll">
                  <table className="dataTable kt-vbtable">
                    <thead>
                      <tr>
                        <th>Số ký hiệu</th>
                        <th>Loại văn bản</th>
                        <th>Cơ quan ban hành</th>
                        <th>Trích yếu</th>
                        <th>Bút tích</th>
                        <th>Số trang</th>
                        <th>Tờ số</th>
                        <th>Trang số</th>
                        <th>Ngày tháng</th>
                        <th>Ghi chú</th>
                      </tr>
                    </thead>
                    <tbody>
                      {V1yb.length === 0 ? (
                        <tr>
                          <td className="dt-empty" colSpan={10}>
                            Hồ sơ chưa có văn bản.
                          </td>
                        </tr>
                      ) : (
                        V1yb.map((v) => (
                          <tr key={v.id}>
                            <td className="text-center">{v.soKyHieu ?? "---"}</td>
                            <td className="text-center">{v.loaiVanBan ?? "---"}</td>
                            <td className="text-justify">{v.coQuanBanHanh ?? "---"}</td>
                            <td className="text-justify">{v.trichYeu ?? "---"}</td>
                            <td className="text-center">{v.butTich ?? "---"}</td>
                            <td className="text-center">{v.soTrang ?? 0}</td>
                            <td className="text-center">{v.toSo ?? 0}</td>
                            <td className="text-center">{v.trangSo ?? 0}</td>
                            <td className="text-center">
                              {v.ngayThang ? V4dr(v.ngayThang) : "---"}
                            </td>
                            <td className="text-center">{v.ghiChu ?? "---"}</td>
                          </tr>
                        ))
                      )}
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
