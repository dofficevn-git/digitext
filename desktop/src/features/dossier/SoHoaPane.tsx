// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { V41p } from "@/live/useLive";
import "./laphoso.css";
import { isTauri, V3xq } from "@/api/ipc";
import type { V40v } from "@/api/archive";
import { V48j } from "@/api/catalog";
import type { V3ym, V3yl, V3yo } from "@/domain/types";
import { V4cg, V4br, type V4bm } from "@/api/dossier";
import { V460 } from "@/state";
import { bangTrongText, chuaChonPhong } from "./phongScope";
import V4dy from "./VanBanPanel";
import V4h8 from "./TaiLieuPanel";
import { V4e3, V4e8 } from "./excel";
import { type V4dk, V4d0, V4di, V4dr, V4cm, V4ct, V4cl, V4dt, V4dp, V4du, V4co } from "./logic";
import { Icon } from "@/components/Icon";
import { PackDialog } from "@/features/package/PackFromDossiers";

const V4h9 = [10, 25, 50, 100];

type V4hb = "import" | "inpack" | null;
type V4hc = { key: V4dk; dir: "asc" | "desc" };

/** "taolap" = Tạo lập cơ sở dữ liệu; "donggoi" = cùng bảng hồ sơ nhưng chỉ để tích chọn và đóng gói
 * (bỏ tìm kiếm / nhập / xuất / cột Mã hồ sơ, File đính kèm — theo đề nghị chỉnh sửa mục 7, 8). */
export default function SoHoaPane({
  phongs: V1e7,
  mode = "taolap",
}: {
  phongs: V40v[];
  mode?: "taolap" | "donggoi";
}) {
  const dg = mode === "donggoi";
  const { setScanTarget: V1e8, goto } = V460();

  const [V1e9, V1ea] = useState<V3yl[]>([]);
  const [V1eb, V1ec] = useState<V3ym[]>([]);
  const [V1ed, V1ee] = useState<V3yo[]>([]);

  const [maDa, V1ef] = useState("");
  const [V1eg, V1eh] = useState("");
  const [V1ei, V1ej] = useState<string>("");
  const [V1ek, V1el] = useState<string[]>([]);
  const [V1em, V1en] = useState("");
  const [V1eo, V1ep] = useState(false);
  const [V1eq, V1er] = useState(false);

  const [V1es, V1et] = useState<V4bm[]>([]);
  const [V1eu, V1ev] = useState<Set<number>>(new Set());
  const [V1ew, V1ex] = useState<V4bm | null>(null);
  const [V1ey, V1ez] = useState<V4bm | null>(null);

  const [cols, V1f0] = useState({ soTo: false, nbd: false, nkt: false });

  const [V1f1, V1f2] = useState(25);
  const [V1f3, V1f4] = useState(1);
  const [V1f5, V1f6] = useState("");
  const [V1f7, V1f8] = useState<V4hc>({ key: "id", dir: "asc" });

  const [V1f9, V1fa] = useState<V4hb>(null);
  const [V1fl, V1fm] = useState(false);
  const V1fn = useRef<HTMLInputElement>(null);
  const V1fo = useRef<HTMLInputElement>(null);

  const V1fp = useCallback(() => {
    V48j()
      .then((c) => {
        V1ea(c.phongBans);
        V1ec(c.mangs);
        V1ee(c.thbqs);
      })
      .catch(() => {});
  }, []);
  V41p(V1fp, { topics: ["catalog"], enabled: isTauri() });


  const V1fr = useCallback(() => {
    if (!isTauri()) return;

    if (chuaChonPhong(maDa)) {
      V1et([]);
      return;
    }
    V4br({
      maDa,
      maDv: V1eg || undefined,
      mangCvid: V1ei ? Number(V1ei) : undefined,
    })
      .then(V1et)
      .catch(() => V1et([]));
  }, [maDa, V1eg, V1ei]);

  V41p(V1fr, { topics: ["dossier", "khogia"] });

  const V1fs = useMemo(() => {
    let V1ft = V1es;
    const q = V1em.trim().toLowerCase();
    if (q) V1ft = V1ft.filter((r) => V4ct(r, q));
    if (V1ek.length && !V1ek.includes("-1")) {
      const set = new Set(V1ek.map(Number));
      V1ft = V1ft.filter((r) => r.thbq != null && set.has(r.thbq));
    }
    if (V1eq) V1ft = V1ft.filter((r) => V4du(r, V1ed).cls === "badge-gl");
    if (V1eo) V1ft = V1ft.filter((r) => V4du(r, V1ed).cls === "badge-het");
    return V4dp(V1ft, V1f7.key, V1f7.dir, V1ed, V1eb);
  }, [V1es, V1em, V1ek, V1eq, V1eo, V1f7, V1ed, V1eb]);

  useEffect(() => V1f4(1), [V1fs.length, V1f7]);

  const V1fu = V4cl(V1e9, maDa);
  const V1fv = V4cm(V1eb, V1eg);

  const V1fw = (v: string) => {
    V1ef(v);
    V1eh("");
    V1ej("");
  };
  const V1fx = (v: string) => {
    V1eh(v);
    V1ej("");
  };

  const V1fy = V1fs.length;
  const V1fz = Math.max(1, Math.ceil(V1fy / V1f1));
  const V1g0 = Math.min(V1f3, V1fz);
  const V1g1 = useMemo(() => V1fs.slice((V1g0 - 1) * V1f1, V1g0 * V1f1), [V1fs, V1g0, V1f1]);

  const V1g2 = (V1g3: number) =>
    V1ev((s) => {
      const n = new Set(s);
      n.has(V1g3) ? n.delete(V1g3) : n.add(V1g3);
      return n;
    });
  const V1g4 = V1g1.length > 0 && V1g1.every((r) => V1eu.has(r.id));
  const V1g5 = () =>
    V1ev((s) => {
      const n = new Set(s);
      if (V1g4) V1g1.forEach((r) => n.delete(r.id));
      else V1g1.forEach((r) => n.add(r.id));
      return n;
    });

  const V1g6 = (V1g7: unknown) =>
    V1g7 instanceof V3xq || V1g7 instanceof Error ? V1g7.message : String(V1g7);

  const V1g8 = (r: V4bm) => {
    const target = V4d0(r);
    if (!target) {
      window.alert(
        "Hồ sơ chưa gắn phông nên không tạo được cây thư mục để quét. Hãy chọn phông cho hồ sơ trước.",
      );
      return;
    }
    V1e8(target);
    goto("scan");
  };

  const V1g9 = async (file: File | undefined) => {
    if (!file) return;
    V1fm(true);
    try {
      const { items: V1ga, skipped: V1gb, issues: V1gc } = await V4e8(file, V1es);
      if (V1ga.length === 0) {
        const V1gd = V4di(V1gc);
        window.alert(
          V1gd
            ? `Không đọc được văn bản hợp lệ nào.\n\n${V1gd}`
            : "Không đọc được văn bản hợp lệ nào (cần xác định được hồ sơ từ 'Hồ sơ id' hoặc 'Mã/Hồ sơ số', và có trích yếu/số ký hiệu hoặc tiêu đề hồ sơ).",
        );
        return;
      }
      const n = await V4cg(V1ga);
      window.alert(`Đã nhập ${n} văn bản${V1gb ? ` (bỏ qua ${V1gb} dòng thiếu dữ liệu)` : ""}.`);
      V1fa(null);
      V1fr();
    } catch (V1ge) {
      window.alert(V1g6(V1ge));
    } finally {
      V1fm(false);
    }
  };

  const V1gf = async () => {
    if (V1fs.length === 0) {
      window.alert("Không có hồ sơ nào để xuất.");
      return;
    }
    const data = V1fs.map((r) => ({
      "Mã hồ sơ": r.maHoSo ?? "",
      "Hộp số": r.hopSo ?? "",
      "HS số": r.soTam,
      "Tiêu đề": r.tieuDe,
      THBQ: V4co(V1ed, r.thbq),
      "Số tờ": V4dt(r.soTo),
      "Ngày bắt đầu": V4dr(r.ngayBatDau),
      "Ngày kết thúc": V4dr(r.ngayKetThuc),
      "Trạng thái": V4du(r, V1ed).label,
      "Số văn bản": r.soVanBan,
      "File đính kèm": r.soFileScan,
    }));
    try {
      await V4e3(data, "SoHoa", "xuat-csdl-so-hoa.xlsx");
    } catch (V1gg) {
      window.alert(V1g6(V1gg));
    }
  };

  const V1gu = () => {
    if (V1eu.size === 0) {
      window.alert("Vui lòng chọn ít nhất một hồ sơ để đóng gói!");
      return;
    }
    V1fa("inpack");
  };

  const V1gx = (V1gy: V4dk) => {
    V1f8((V1gz) =>
      V1gz.key === V1gy
        ? { key: V1gy, dir: V1gz.dir === "asc" ? "desc" : "asc" }
        : { key: V1gy, dir: "asc" },
    );
  };

  const V1h0 = (V1h1: V4dk) => {
    if (V1f7.key !== V1h1) return "sapXep";
    return V1f7.dir === "asc" ? "tangDan" : "giamDan";
  };

  const V1h2 = ({
    label: V1h3,
    sortKey: V1h4,
    className,
  }: {
    label: string;
    sortKey: V4dk;
    className?: string;
  }) => (
    <th className={`sortable-th${className ? ` ${className}` : ""}`} onClick={() => V1gx(V1h4)}>
      <span className="sort-label">{V1h3}</span>
      <Icon name={V1h0(V1h4)} size={12} />
    </th>
  );

  if (V1ew) {
    return (
      <V4dy
        dossier={V1ew}
        onBack={() => {
          V1ex(null);
          V1fr();
        }}
      />
    );
  }

  if (V1ey) {
    return <V4h8 dossier={V1ey} onBack={() => V1ez(null)} />;
  }

  const V1h5 = (dg ? 6 : 8) + Number(cols.soTo) + Number(cols.nbd) + Number(cols.nkt);

  return (
    <div className="lhs">
      <div className="content-panel">
        <div className="center">
          <h4>
            <b>{dg ? "ĐÓNG GÓI DỮ LIỆU" : "SỐ HÓA TÀI LIỆU"}</b>
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
                onChange={(V1h6) => V1fw(V1h6.target.value)}
              >
                <option value="">-- CHỌN PHÔNG LƯU TRỮ --</option>
                {V1e7.map((p) => (
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
                value={V1eg}
                disabled={!maDa}
                onChange={(V1h7) => V1fx(V1h7.target.value)}
              >
                <option value="">-- CHỌN PHÒNG BAN --</option>
                {V1fu.map((V1h8) => (
                  <option key={V1h8.maDv} value={V1h8.maDv}>
                    {V1h8.tenDonVi}
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
                value={V1ei}
                disabled={!V1eg}
                onChange={(V1h9) => V1ej(V1h9.target.value)}
              >
                <option value="">-- CHỌN MẢNG CÔNG VIỆC --</option>
                {V1fv.map((m) => (
                  <option key={m.mangCvid} value={m.mangCvid}>
                    {m.tenMang}
                  </option>
                ))}
              </select>
            </div>
          </div>
        </div>

        {}
        <div className="row items-center">
          <div className="col-4">
            <div className="input-group">
              <label className="input-group-addon w-1/4">THBQ</label>
              <select
                className="form-control h-8.5"
                multiple
                value={V1ek}
                onChange={(V1ha) => V1el([...V1ha.target.selectedOptions].map((o) => o.value))}
              >
                <option value="-1">Tất cả THBQ</option>
                {V1ed.map((t) => (
                  <option key={t.thbqid} value={t.thbqid}>
                    {t.tenThbq}
                  </option>
                ))}
              </select>
            </div>
          </div>
          {!dg && (
            <>
              <div className="col-2">
                <input
                  className="form-control solo searchInput"
                  value={V1em}
                  placeholder="Nhập ký tự tìm kiếm"
                  onChange={(V1hb) => V1en(V1hb.target.value)}
                  onKeyDown={(V1hc) => V1hc.key === "Enter" && V1fr()}
                />
              </div>
              <div className="col-2">
                <label className="labelEx">
                  <input
                    type="checkbox"
                    className="inputEx"
                    checked={V1eo}
                    onChange={(V1hd) => V1ep(V1hd.target.checked)}
                  />{" "}
                  Hết THBQ
                </label>
                <label className="labelEx ml-3">
                  <input
                    type="checkbox"
                    className="inputEx"
                    checked={V1eq}
                    onChange={(V1he) => V1er(V1he.target.checked)}
                  />{" "}
                  Giữ lại
                </label>
              </div>
            </>
          )}
          <div className={dg ? "col-8 text-right" : "col-4 text-right"}>
            <button className="btn btn-outline-success btn-round" onClick={() => V1fr()}>
              <Icon name="tim" /> Tìm kiếm
            </button>{" "}
            {!dg && (
              <>
                <button
                  className="btn btn-outline-success btn-round"
                  onClick={() => V1fa("import")}
                >
                  <Icon name="tepPdf" /> Nhập tài liệu
                </button>{" "}
                <button className="btn btn-outline-secondary btn-round" onClick={() => void V1gf()}>
                  <Icon name="tepExcel" /> Xuất CSDL
                </button>
              </>
            )}
            {dg && (
              <button className="btn btn-success btn-round" onClick={V1gu}>
                <Icon name="goi" /> Đóng gói
              </button>
            )}
          </div>
        </div>
      </div>

      {}
      {!dg && (
        <div className="actionbar">
          <div className="grp">
            <input
              ref={V1fo}
              type="file"
              className="form-control max-w-65"
              accept=".xlsx,.xls,.csv"
              onChange={(V1hf) => {
                void V1g9(V1hf.target.files?.[0]);
                V1hf.target.value = "";
              }}
            />
            <button
              className="btn btn-outline-primary btn-round"
              onClick={() => V1fo.current?.click()}
            >
              <Icon name="taiLen" /> Import file PDF
            </button>
          </div>
        </div>
      )}

      {}
      <div className="content-panel pt-3">
        <div className="tbl-scroll">
          <table className="dataTable">
            <thead>
              <tr>
                <th className="w-2.5">
                  <input type="checkbox" checked={V1g4} onChange={V1g5} />
                </th>
                {!dg && <V1h2 label="Mã hồ sơ" sortKey="maHoSo" className="w-32.5" />}
                <V1h2 label="Hộp số" sortKey="hopSo" className="w-15" />
                <V1h2 label="HS số" sortKey="soTam" className="w-15" />
                <V1h2 label="Tiêu đề" sortKey="tieuDe" />
                {cols.soTo && <V1h2 label="Số tờ" sortKey="soTo" className="w-12.5" />}
                {cols.nbd && <V1h2 label="N. Bắt đầu" sortKey="ngayBatDau" className="w-20" />}
                {cols.nkt && <V1h2 label="N. Kết thúc" sortKey="ngayKetThuc" className="w-20" />}
                <V1h2 label="THBQ" sortKey="trangThai" className="w-30" />
                <V1h2 label="Tài liệu" sortKey="soTaiLieuScan" className="w-22.5" />
                {!dg && <V1h2 label="File Đính kèm" sortKey="soFileScan" className="w-22.5" />}
              </tr>
            </thead>
            <tbody>
              {V1g1.length === 0 ? (
                <tr>
                  <td className="dt-empty" colSpan={V1h5}>
                    {bangTrongText(maDa)}
                  </td>
                </tr>
              ) : (
                V1g1.map((r, i) => {
                  const V1hg = V4du(r, V1ed);
                  return (
                    <tr key={r.id} className={i % 2 === 0 ? "odd" : "even"}>
                      <td className="text-center">
                        <input
                          type="checkbox"
                          checked={V1eu.has(r.id)}
                          onChange={() => V1g2(r.id)}
                        />
                      </td>
                      {!dg && <td className="text-left">{r.maHoSo}</td>}
                      <td className="text-center">{r.hopSo}</td>
                      <td className="text-center">{r.soTam}</td>
                      <td className="text-justify">{r.tieuDe}</td>
                      {cols.soTo && <td className="text-center">{V4dt(r.soTo)}</td>}
                      {cols.nbd && <td className="text-center">{V4dr(r.ngayBatDau)}</td>}
                      {cols.nkt && <td className="text-center">{V4dr(r.ngayKetThuc)}</td>}
                      <td className="text-center">
                        <span className={`badge-thbq ${V1hg.cls}`} title={V4co(V1ed, r.thbq)}>
                          {V1hg.label}
                        </span>
                      </td>
                      <td className="text-center">
                        <a
                          className="ic-word"
                          onClick={() => V1ez(r)}
                          title="Xem danh sách file PDF trong hồ sơ"
                        >
                          <Icon name="tepPdf" />
                        </a>
                        <b className="cnt-red" title="Số file PDF đã quét trong hồ sơ">
                          {" "}
                          ({r.soTaiLieuScan})
                        </b>
                      </td>
                      {!dg && (
                        <td className="text-center">
                          {r.soFileScan}{" "}
                          <a className="ic-word" onClick={() => V1g8(r)} title="Quét vào hồ sơ">
                            <Icon name="quet" />
                          </a>
                        </td>
                      )}
                    </tr>
                  );
                })
              )}
            </tbody>
            <tfoot>
              <tr>
                <td className="footer-restore" colSpan={V1h5}>
                  <a
                    className="toggle-vis"
                    title="Hiển thị toàn bộ cột"
                    onClick={() => V1f0({ soTo: true, nbd: true, nkt: true })}
                  >
                    <Icon name="luoi" />
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.soTo ? "on" : "off"}`}
                    onClick={() => V1f0((c) => ({ ...c, soTo: !c.soTo }))}
                  >
                    Số tờ
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.nbd ? "on" : "off"}`}
                    onClick={() => V1f0((c) => ({ ...c, nbd: !c.nbd }))}
                  >
                    N. Bắt đầu
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.nkt ? "on" : "off"}`}
                    onClick={() => V1f0((c) => ({ ...c, nkt: !c.nkt }))}
                  >
                    N. Kết thúc
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
                value={V1f1}
                onChange={(V1hh) => {
                  V1f2(Number(V1hh.target.value));
                  V1f4(1);
                }}
              >
                {V4h9.map((n) => (
                  <option key={n} value={n}>
                    {n}
                  </option>
                ))}
              </select>{" "}
              bản ghi.
            </label>
          </div>
          <div className="pagination">
            <button className="pbtn" disabled={V1g0 <= 1} onClick={() => V1f4(V1g0 - 1)}>
              <Icon name="truoc" />
            </button>
            <button className="pbtn current">{V1g0}</button>
            <button className="pbtn" disabled={V1g0 >= V1fz} onClick={() => V1f4(V1g0 + 1)}>
              <Icon name="sau" />
            </button>
          </div>
          <div className="dt-goto">
            <label>Tới trang</label>
            <input
              type="number"
              min={1}
              value={V1f5}
              onChange={(V1hi) => V1f6(V1hi.target.value)}
            />
            <button
              className="pbtn"
              onClick={() => {
                const p = Number(V1f5);
                if (p >= 1 && p <= V1fz) V1f4(p);
              }}
            >
              Đi
            </button>
          </div>
        </div>
      </div>

      {}
      {V1f9 === "import" && (
        <Modal title="Nhập Tài Liệu vào Văn Bản" onClose={() => V1fa(null)}>
          <p className="muted">
            Chọn tệp Excel văn bản (có cột "Hồ sơ id") để nhập hàng loạt vào các hồ sơ.
          </p>
          <input ref={V1fn} type="file" className="form-control" accept=".xlsx,.xls,.csv" />
          <div className="text-right mt-3">
            <button
              className="btn btn-outline-primary btn-round"
              disabled={V1fl}
              onClick={() => void V1g9(V1fn.current?.files?.[0])}
            >
              <Icon name="taiLen" /> Nhập
            </button>{" "}
            <button className="btn btn-outline-danger btn-round" onClick={() => V1fa(null)}>
              Đóng
            </button>
          </div>
        </Modal>
      )}

      {}
      {V1f9 === "inpack" && (
        <PackDialog dossiers={V1es.filter((r) => V1eu.has(r.id))} thbqs={V1ed} onClose={() => V1fa(null)} />
      )}
    </div>
  );
}

function Modal({
  title: V1hm,
  onClose: V1hn,
  children: V1ho,
}: {
  title: string;
  onClose: () => void;
  children: ReactNode;
}) {
  return (
    <div className="lhs-overlay" onClick={V1hn}>
      <div className="lhs-modal" onClick={(V1hp) => V1hp.stopPropagation()}>
        <div className="m-head">
          <h4>{V1hm}</h4>
          <button className="m-close" onClick={V1hn}>
            ×
          </button>
        </div>
        <div className="m-body">{V1ho}</div>
      </div>
    </div>
  );
}
