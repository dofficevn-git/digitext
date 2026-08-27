// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { V41p } from "@/live/useLive";
import "./laphoso.css";
import { isTauri } from "@/api/ipc";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { V40w, type V40v } from "@/api/archive";
import { V48j } from "@/api/catalog";
import type { V3ym, V3yl, V3yo } from "@/domain/types";
import { V4cg, V4c4, V4bt, V4bx, V4br, V4bw, type V4bl, type V4bm } from "@/api/dossier";
import { V4bf, V4b9, V4be, V4b7 } from "@/domain/catalogImport";
import { bangTrongText, chuaChonPhong } from "./phongScope";
import V4dy from "./VanBanPanel";
import V4e9 from "./HistoryModal";
import { V4e6, V4e3, V4e7, V4e8 } from "./excel";
import {
  V4cr,
  V4ds,
  V4cv,
  V4dj,
  V4cq,
  type V4dk,
  V4cp,
  V4di,
  V4dr,
  V4d3,
  V4cn,
  V4cm,
  V4ct,
  V4cl,
  V4dt,
  V4dp,
  V4du,
  V4co,
  V4d5,
  V4d4,
  V4cs,
} from "./logic";
import { Icon } from "@/components/Icon";

const V4ea = [10, 25, 50, 100];

type V4eb = {
  mode: "add" | "edit";
  id?: number;
  draft: V4bl;
  nbd: string;
  nkt: string;
} | null;
type V4ec = { key: V4dk; dir: "asc" | "desc" };

export default function LapHoSoView() {
  const { session: V10x } = V460();
  const V10y = V3z9(V10x?.permissions, "dossier.create");
  const V10z = V3z9(V10x?.permissions, "dossier.update");
  const V110 = V3z9(V10x?.permissions, "dossier.delete");

  const [V111, V112] = useState<V40v[]>([]);
  const [V113, V114] = useState<V3yl[]>([]);
  const [V115, V116] = useState<V3ym[]>([]);
  const [V117, V118] = useState<V3yo[]>([]);

  const [maDa, V119] = useState("");
  const [V11a, V11b] = useState("");
  const [V11c, V11d] = useState<string>("");
  const [V11e, V11f] = useState<string[]>([]);
  const [V11g, V11h] = useState("");
  const [V11i, V11j] = useState(false);
  const [V11k, V11l] = useState(false);

  const [V11m, V11n] = useState<V4bm[]>([]);
  const [V11o, V11p] = useState<V4eb>(null);
  const [V11q, V11r] = useState<Set<number>>(new Set());
  const [V11s, V11t] = useState<V4bm | null>(null);
  const [V11u, V11v] = useState<"hoso" | "vanban" | null>(null);
  const V11w = useRef<HTMLInputElement>(null);
  const V11x = useRef<HTMLInputElement>(null);

  const [cols, V11y] = useState({ mang: false, id: false, tthua: false, tthai: false });

  const [V11z, V120] = useState(50);
  const [V121, V122] = useState(1);
  const [V123, V124] = useState("");
  const [V125, V126] = useState<V4ec>({ key: "id", dir: "asc" });

  const V127 = useCallback(() => {
    V40w()
      .then(V112)
      .catch(() => {});
    V48j()
      .then((c) => {
        V114(c.phongBans);
        V116(c.mangs);
        V118(c.thbqs);
      })
      .catch(() => {});
  }, []);
  V41p(V127, { topics: ["catalog", "hoso"], enabled: isTauri() });

  const V128 = useCallback(() => {
    if (!isTauri()) return;

    if (chuaChonPhong(maDa)) {
      V11n([]);
      return;
    }
    V4br({
      maDa,
      maDv: V11a || undefined,
      mangCvid: V11c ? Number(V11c) : undefined,
    })
      .then(V11n)
      .catch(() => V11n([]));
  }, [maDa, V11a, V11c]);

  V41p(V128, { topics: ["dossier", "khogia"] });

  const V129 = useMemo(() => {
    let V12a = V11m;
    const q = V11g.trim().toLowerCase();
    if (q) V12a = V12a.filter((r) => V4ct(r, q));
    if (V11e.length && !V11e.includes("-1")) {
      const set = new Set(V11e.map(Number));
      V12a = V12a.filter((r) => r.thbq != null && set.has(r.thbq));
    }
    if (V11k) V12a = V12a.filter((r) => (r.trangThai || "").toUpperCase().includes("GL"));
    return V4dp(V12a, V125.key, V125.dir, V117, V115);
  }, [V11m, V11g, V11e, V11k, V125, V117, V115]);

  useEffect(() => V122(1), [V129.length, V125]);

  const V12b = V4cl(V113, maDa);
  const V12c = V4cm(V115, V11a);

  const V12d = (v: string) => {
    V119(v);
    V11b("");
    V11d("");
  };
  const V12e = (v: string) => {
    V11b(v);
    V11d("");
  };

  const V12f = () => {
    const t = V11e.find((x) => x !== "-1");
    return t ? Number(t) : undefined;
  };
  const V12g = () => {
    V11p({
      mode: "add",
      draft: {
        ...V4cp(),
        maDa,
        maDv: V11a,
        mangCvid: V11c ? Number(V11c) : undefined,
        thbq: V12f(),
        soTo: "0",
      },
      nbd: "",
      nkt: "",
    });
  };

  const V12h = (r: V4bm) => {
    V11p({
      mode: "edit",
      id: r.id,
      draft: V4cq(r),
      nbd: V4dr(r.ngayBatDau),
      nkt: V4dr(r.ngayKetThuc),
    });
  };

  const V12i = (p: Partial<V4bl>) => V11o && V11p({ ...V11o, draft: { ...V11o.draft, ...p } });

  const V12j = async () => {
    if (!V11o) return;
    const V12k: V4bl = {
      ...V11o.draft,
      ngayBatDau: V4ds(V11o.nbd),
      ngayKetThuc: V4ds(V11o.nkt),
    };
    const V12l = V4cs(V12k);
    if (V12l) {
      window.alert(V12l);
      return;
    }
    try {
      const body = V4cr(V12k);
      if (V11o.mode === "add") {
        await V4bt(body);

        V11p({
          ...V11o,
          draft: {
            ...V4cp(),
            maDa,
            maDv: V11a,
            mangCvid: V11c ? Number(V11c) : undefined,
            thbq: V12f(),
            soTo: "0",
          },
          nbd: "",
          nkt: "",
        });
      } else if (V11o.id != null) {
        await V4bw(V11o.id, body);
        V11p(null);
      }
      V128();
    } catch (V12m) {
      window.alert(V12m instanceof Error ? V12m.message : String(V12m));
    }
  };

  const V12n = async (r: V4bm) => {
    if (!window.confirm(`Xóa hồ sơ "${r.tieuDe || r.soTam}"?`)) return;
    try {
      await V4bx(r.id);
      V128();
    } catch (V12o) {
      window.alert(V12o instanceof Error ? V12o.message : String(V12o));
    }
  };

  const V12p = async () => {
    if (V11q.size === 0) {
      window.alert("Chưa chọn hồ sơ nào để xóa.");
      return;
    }
    if (!window.confirm(`WMOzYSA=${V11q.size} hồ sơ đã chọn?`)) return;
    try {
      await Promise.all([...V11q].map((V12q) => V4bx(V12q)));
      V11r(new Set());
      V128();
    } catch (V12r) {
      window.alert(V12r instanceof Error ? V12r.message : String(V12r));
    }
  };

  const V12s = async (file: File | undefined) => {
    if (!file) return;
    try {
      const V12t = await V4b7(file);
      const name = V4bf(V12t);
      const V12u = V12t.sheets.find((s) => s.name === name);
      if (!V12u) return;
      const V12v = V4be(V12u.rows, V4b9(V12u.rows), name, 1);
      const V12w = V4d4(V117);
      const V12x = V4d5(V115);
      const V12y = V12v.records.map((h) => V4d3(h, V12w, V12x)).filter((d) => d.soTam);
      if (V12y.length === 0) {
        window.alert("Không đọc được hồ sơ nào từ tệp.");
        return;
      }
      const n = await V4c4(maDa || undefined, V12y);
      window.alert(`Đã cập nhật ${n} hồ sơ từ tệp.`);
      V128();
    } catch (V12z) {
      window.alert(V12z instanceof Error ? V12z.message : String(V12z));
    }
  };

  const V130 = async (file: File | undefined) => {
    if (!file) return;
    try {
      const { items: V131, skipped: V132, issues: V133 } = await V4e8(file, V11m);
      if (V131.length === 0) {
        const V134 = await V4b7(file);
        const name = V4bf(V134);
        const V135 = V134.sheets.find((s) => s.name === name);
        if (V135) {
          const V136 = V4be(V135.rows, V4b9(V135.rows), name, 1);
          const V137 = V4d4(V117);
          const V138 = V4d5(V115);
          const V139 = V136.records.map((h) => V4d3(h, V137, V138)).filter((d) => d.soTam);
          if (V139.length > 0) {
            const n = await V4c4(maDa || undefined, V139);
            window.alert(
              `Tệp này có dạng mục lục hồ sơ. Đã cập nhật ${n} hồ sơ vào Lập hồ sơ điện tử.`,
            );
            V128();
            return;
          }
        }

        const V13a = V4di(V133);
        window.alert(
          V13a
            ? `Không đọc được văn bản hợp lệ nào.\n\n${V13a}`
            : "Không đọc được văn bản hợp lệ nào (cần xác định được hồ sơ từ 'Hồ sơ id' hoặc 'Mã/Hồ sơ số', và có trích yếu/số ký hiệu hoặc tiêu đề hồ sơ).",
        );
        return;
      }
      const n = await V4cg(V131);
      window.alert(
        `Đã nhập ${n} văn bản${V132 ? `IChi4buPIHF1YSA=${V132}IGTDsm5nIHRoaeG6v3UgZOG7ryBsaeG7h3Up` : ""}.`,
      );
      V128();
    } catch (V13b) {
      window.alert(V13b instanceof Error ? V13b.message : String(V13b));
    }
  };

  const V13c = async () => {
    if (V129.length === 0) {
      window.alert("Không có hồ sơ nào để chuyển đổi/xuất.");
      return;
    }
    try {
      await V4e3(
        V129.map((r) => V4dj(r, V117, V115)),
        "LapHoSoFrontend",
        "lap-ho-so-frontend.xlsx",
      );
    } catch (V13d) {
      window.alert(V13d instanceof Error ? V13d.message : String(V13d));
    }
  };

  const V13e = V129.length;
  const V13f = Math.max(1, Math.ceil(V13e / V11z));
  const V13g = Math.min(V121, V13f);
  const V13h = useMemo(() => V129.slice((V13g - 1) * V11z, V13g * V11z), [V129, V13g, V11z]);

  const V13i = (V13j: number) =>
    V11r((s) => {
      const n = new Set(s);
      n.has(V13j) ? n.delete(V13j) : n.add(V13j);
      return n;
    });
  const V13k = V13h.length > 0 && V13h.every((r) => V11q.has(r.id));
  const V13l = () =>
    V11r((s) => {
      const n = new Set(s);
      if (V13k) V13h.forEach((r) => n.delete(r.id));
      else V13h.forEach((r) => n.add(r.id));
      return n;
    });

  const V13m = (V13n: V4dk) => {
    V126((V13o) =>
      V13o.key === V13n
        ? { key: V13n, dir: V13o.dir === "asc" ? "desc" : "asc" }
        : { key: V13n, dir: "asc" },
    );
  };

  const V13p = (V13q: V4dk) => {
    if (V125.key !== V13q) return "sapXep";
    return V125.dir === "asc" ? "tangDan" : "giamDan";
  };

  const V13r = ({
    label: V13s,
    sortKey: V13t,
    className,
  }: {
    label: string;
    sortKey: V4dk;
    className?: string;
  }) => (
    <th className={`sortable-th${className ? ` ${className}` : ""}`} onClick={() => V13m(V13t)}>
      <span className="sort-label">{V13s}</span>
      <Icon name={V13p(V13t)} size={12} />
    </th>
  );

  if (V11s) {
    return (
      <V4dy
        dossier={V11s}
        onBack={() => {
          V11t(null);
          V128();
        }}
      />
    );
  }

  const V13u = 10 + Number(cols.mang) + Number(cols.id) + Number(cols.tthua) + Number(cols.tthai);

  return (
    <div className="lhs">
      <div className="content-panel">
        <div className="center">
          <h4>
            <b>LẬP HỒ SƠ ĐIỆN TỬ</b>
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
                onChange={(V13v) => V12d(V13v.target.value)}
              >
                <option value="">Lọc theo phông lưu trữ</option>
                {V111.map((p) => (
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
                value={V11a}
                disabled={!maDa}
                onChange={(V13w) => V12e(V13w.target.value)}
              >
                <option value="">--Chọn phòng ban--</option>
                {V12b.map((V13x) => (
                  <option key={V13x.maDv} value={V13x.maDv}>
                    {V13x.tenDonVi}
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
                value={V11c}
                disabled={!V11a}
                onChange={(V13y) => V11d(V13y.target.value)}
              >
                <option value="">--Chọn mảng công việc--</option>
                {V12c.map((m) => (
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
                value={V11e}
                onChange={(V13z) => V11f([...V13z.target.selectedOptions].map((o) => o.value))}
              >
                <option value="-1">Tất cả THBQ</option>
                {V117.map((t) => (
                  <option key={t.thbqid} value={t.thbqid}>
                    {t.tenThbq}
                  </option>
                ))}
              </select>
            </div>
          </div>
          <div className="col-4">
            <input
              className="form-control solo searchInput"
              value={V11g}
              placeholder="Nhập ký tự bất kỳ và nhấn phím Enter để tìm kiếm"
              onChange={(V140) => V11h(V140.target.value)}
              onKeyDown={(V141) => V141.key === "Enter" && V128()}
            />
          </div>
          <div className="col-2">
            <label className="labelEx">
              <input
                type="checkbox"
                className="inputEx"
                checked={V11i}
                onChange={(V142) => V11j(V142.target.checked)}
              />{" "}
              Hết THBQ
            </label>
            <label className="labelEx ml-3">
              <input
                type="checkbox"
                className="inputEx"
                checked={V11k}
                onChange={(V143) => V11l(V143.target.checked)}
              />{" "}
              Giữ lại
            </label>
          </div>
          <div className="col-2 text-right">
            <button className="btn btn-outline-success btn-round" onClick={() => V128()}>
              <Icon name="tim" /> Tìm kiếm
            </button>
          </div>
        </div>

        <hr />

        {}
        {V11o && (
          <div className="newrow">
            <div className="row">
              <div className="col-4">
                <div className="input-group">
                  <label className="input-group-addon">Số tạm</label>
                  <input
                    className="form-control"
                    value={V11o.draft.soTam}
                    placeholder="Số tạm"
                    onChange={(V144) => V12i({ soTam: V144.target.value })}
                  />
                </div>
              </div>
              <div className="col-6">
                <div className="input-group">
                  <label className="input-group-addon">Tiêu đề</label>
                  <input
                    className="form-control"
                    value={V11o.draft.tieuDe ?? ""}
                    placeholder="Tiêu đề"
                    onChange={(V145) => V12i({ tieuDe: V145.target.value })}
                  />
                </div>
              </div>
              <div className="col-2">
                <div className="input-group">
                  <label className="input-group-addon">Số tờ</label>
                  <input
                    className="form-control"
                    value={V11o.draft.soTo ?? ""}
                    onChange={(V146) => V12i({ soTo: V146.target.value })}
                  />
                </div>
              </div>
            </div>
            <div className="row">
              <div className="col-4">
                <div className="input-group">
                  <label className="input-group-addon">Ngày bắt đầu</label>
                  <input
                    className="form-control"
                    placeholder="dd/mm/yyyy"
                    value={V11o.nbd}
                    onChange={(V147) => V11p({ ...V11o, nbd: V147.target.value })}
                  />
                </div>
              </div>
              <div className="col-4">
                <div className="input-group">
                  <label className="input-group-addon">Ngày kết thúc</label>
                  <input
                    className="form-control"
                    placeholder="dd/mm/yyyy"
                    value={V11o.nkt}
                    onChange={(V148) => V11p({ ...V11o, nkt: V148.target.value })}
                  />
                </div>
              </div>
              <div className="col-4">
                <label className="labelEx">
                  <input
                    type="checkbox"
                    className="inputEx"
                    checked={!!V11o.draft.trungThua}
                    onChange={(V149) => V12i({ trungThua: V149.target.checked })}
                  />{" "}
                  Trùng thừa
                </label>
              </div>
            </div>
            <div className="row">
              <div className="col-4">
                <div className="input-group">
                  <label className="input-group-addon">Tổng số văn bản</label>
                  <input
                    className="form-control"
                    type="number"
                    value={V11o.draft.tongSoVanBan ?? ""}
                    onChange={(V14a) =>
                      V12i({
                        tongSoVanBan:
                          V14a.target.value === "" ? undefined : Number(V14a.target.value),
                      })
                    }
                  />
                </div>
              </div>
              <div className="col-4">
                <div className="input-group">
                  <label className="input-group-addon">Số lượng trang</label>
                  <input
                    className="form-control"
                    type="number"
                    value={V11o.draft.soLuongTrang ?? ""}
                    onChange={(V14b) =>
                      V12i({
                        soLuongTrang:
                          V14b.target.value === "" ? undefined : Number(V14b.target.value),
                      })
                    }
                  />
                </div>
              </div>
              <div className="col-4">
                <div className="input-group">
                  <label className="input-group-addon">Chế độ sử dụng</label>
                  <input
                    className="form-control"
                    value={V11o.draft.cheDoSuDung ?? ""}
                    onChange={(V14c) => V12i({ cheDoSuDung: V14c.target.value })}
                  />
                </div>
              </div>
            </div>
            <div className="row">
              <div className="col-4">
                <div className="input-group">
                  <label className="input-group-addon">Ngôn ngữ</label>
                  <input
                    className="form-control"
                    placeholder="Ngôn ngữ"
                    value={V11o.draft.ngonNgu ?? ""}
                    onChange={(V14d) => V12i({ ngonNgu: V14d.target.value })}
                  />
                </div>
              </div>
              <div className="col-4">
                <div className="input-group">
                  <label className="input-group-addon">Tình trạng vật lý</label>
                  <input
                    className="form-control"
                    value={V11o.draft.tinhTrangVatLy ?? ""}
                    onChange={(V14e) => V12i({ tinhTrangVatLy: V14e.target.value })}
                  />
                </div>
              </div>
              <div className="col-4">
                <div className="input-group">
                  <label className="input-group-addon">Ghi chú</label>
                  <textarea
                    className="form-control"
                    value={V11o.draft.ghiChu ?? ""}
                    onChange={(V14f) => V12i({ ghiChu: V14f.target.value })}
                  />
                </div>
              </div>
            </div>
            <div className="row">
              <div className="col-12 text-center mb-1.75">
                {V11o.mode === "add" && (
                  <button className="btn btn-outline-success btn-round" onClick={() => void V12j()}>
                    <Icon name="luu" /> Lưu &amp; Tiếp
                  </button>
                )}{" "}
                {V11o.mode === "edit" && (
                  <button className="btn btn-outline-success btn-round" onClick={() => void V12j()}>
                    <Icon name="luu" /> Lưu
                  </button>
                )}{" "}
                <button className="btn btn-outline-danger btn-round" onClick={() => V11p(null)}>
                  <Icon name="dong" /> Đóng
                </button>
              </div>
            </div>
          </div>
        )}
      </div>

      {}
      <div className="actionbar">
        <div className="grp">
          {V10y && (
            <button className="btn btn-outline-info btn-round" onClick={V12g}>
              <Icon name="them" /> Thêm mới
            </button>
          )}
          {V110 && (
            <button className="btn btn-outline-danger btn-round" onClick={() => void V12p()}>
              <Icon name="xoa" /> Xóa
            </button>
          )}
          <button className="btn btn-outline-success btn-round" onClick={() => V11v("hoso")}>
            <Icon name="lichSu" /> L.Sử (Hồ sơ)
          </button>
          <button className="btn btn-outline-success btn-round" onClick={() => V11v("vanban")}>
            <Icon name="lichSu" /> L. Sử (Văn bản)
          </button>
        </div>
        <div className="grp">
          <button className="btn btn-outline-secondary btn-round" onClick={() => void V4e6()}>
            <Icon name="tepExcel" /> Mẫu H.Sơ
          </button>
          <button className="btn btn-outline-info btn-round" onClick={() => void V13c()}>
            <Icon name="chuyenDoi" /> Chuyển đổi Excel
          </button>
          {V10z && (
            <button
              className="btn btn-outline-success btn-round"
              onClick={() => V11w.current?.click()}
            >
              <Icon name="tepWord" /> Nhập V.Bản
            </button>
          )}
          <input
            ref={V11w}
            type="file"
            accept=".xlsx,.xls,.csv"
            className="hidden"
            onChange={(V14g) => {
              void V130(V14g.target.files?.[0]);
              V14g.target.value = "";
            }}
          />
        </div>
      </div>

      {}
      <div className="content-panel pt-3">
        <div className="uploadrow">
          <input
            ref={V11x}
            type="file"
            className="form-control"
            accept=".xlsx,.xls,.csv"
            onChange={(V14h) => {
              void V12s(V14h.target.files?.[0]);
              V14h.target.value = "";
            }}
          />
          <button
            className="btn btn-outline-success btn-round"
            onClick={() => V11x.current?.click()}
          >
            <Icon name="sua" /> Cập nhật hồ sơ
          </button>
        </div>

        {}
        <div className="tbl-scroll">
          <table className="dataTable">
            <thead>
              <tr>
                <th className="w-2.5">
                  <input type="checkbox" checked={V13k} onChange={V13l} />
                </th>
                {cols.mang && <V13r label="Mảng" sortKey="mang" />}
                {cols.id && <V13r label="ID Hồ sơ" sortKey="id" />}
                <V13r label="Mã Hồ Sơ" sortKey="maHoSo" className="w-22.5" />
                <V13r label="Số Tạm" sortKey="soTam" className="w-17.5" />
                <V13r label="Tiêu Đề" sortKey="tieuDe" />
                <V13r label="THBQ" sortKey="thbq" className="w-25" />
                <V13r label="Số Tờ" sortKey="soTo" className="w-12.5" />
                <V13r label="Ngày BĐ" sortKey="ngayBatDau" className="w-20" />
                <V13r label="Ngày KT" sortKey="ngayKetThuc" className="w-20" />
                {cols.tthua && <V13r label="T.Thừa" sortKey="trungThua" className="w-15" />}
                {cols.tthai && <V13r label="T.Thái" sortKey="trangThai" className="w-27.5" />}
                <V13r label="Tài Liệu" sortKey="taiLieu" className="w-25" />
                <th className="w-20">Chức Năng</th>
              </tr>
            </thead>
            <tbody>
              {V13h.length === 0 ? (
                <tr>
                  <td className="dt-empty" colSpan={V13u}>
                    {bangTrongText(maDa)}
                  </td>
                </tr>
              ) : (
                V13h.map((r, i) => {
                  const V14i = V4du(r, V117);
                  return (
                    <tr key={r.id} className={i % 2 === 0 ? "odd" : "even"}>
                      <td className="text-center">
                        <input
                          type="checkbox"
                          checked={V11q.has(r.id)}
                          onChange={() => V13i(r.id)}
                        />
                      </td>
                      {cols.mang && <td className="text-left">{V4cn(V115, r.mangCvid)}</td>}
                      {cols.id && <td className="text-center">{r.id}</td>}
                      <td className="text-center">{r.maHoSo}</td>
                      <td className="text-left">{r.soTam}</td>
                      <td className="text-justify">
                        <a
                          className="tbllink"
                          onClick={() => V12h(r)}
                          title="Sửa thông tin hồ sơ"
                        >
                          {r.tieuDe}
                        </a>
                      </td>
                      <td className="text-center">{V4co(V117, r.thbq)}</td>
                      <td className="text-center">{V4dt(r.soTo)}</td>
                      <td className="text-center">{V4dr(r.ngayBatDau)}</td>
                      <td className="text-center">{V4dr(r.ngayKetThuc)}</td>
                      {cols.tthua && (
                        <td className="text-center">
                          <input type="checkbox" checked={r.trungThua} disabled />
                        </td>
                      )}
                      {cols.tthai && (
                        <td className="text-center">
                          <span className={`badge-thbq ${V14i.cls}`}>{V14i.label}</span>
                        </td>
                      )}
                      <td className="text-center">
                        <a
                          className="ic-word"
                          onClick={() => V11t(r)}
                          title="Thêm thông tin văn bản"
                        >
                          <Icon name="tepWord" />
                        </a>
                        <b className="cnt-red"> ({V4cv(r)}) </b>
                        <button
                          type="button"
                          className="ic-btn"
                          title="Tải mẫu lập văn bản"
                          onClick={() => void V4e7()}
                        >
                          <Icon name="taiVe" size={14} />
                        </button>{" "}
                        <button
                          type="button"
                          className="ic-btn"
                          title="Tải mẫu nhập file đính kèm"
                          onClick={() => void V4e7()}
                        >
                          <Icon name="taiVe" size={14} />
                        </button>
                      </td>
                      <td className="text-center">
                        {V10z && (
                          <a
                            className="ic-edit"
                            onClick={() => V12h(r)}
                            title="Sửa thông tin hồ sơ"
                          >
                            <Icon name="sua" />
                          </a>
                        )}{" "}
                        {V110 && (
                          <a
                            className="ic-del"
                            onClick={() => void V12n(r)}
                            title="Xóa thông tin hồ sơ"
                          >
                            <Icon name="xoa" />
                          </a>
                        )}
                      </td>
                    </tr>
                  );
                })
              )}
            </tbody>
            <tfoot>
              <tr>
                <td className="footer-restore" colSpan={V13u}>
                  <a
                    className="toggle-vis"
                    title="Hiển thị toàn bộ cột"
                    onClick={() => V11y({ mang: true, id: true, tthua: true, tthai: true })}
                  >
                    <Icon name="luoi" />
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.mang ? "on" : "off"}`}
                    onClick={() => V11y((c) => ({ ...c, mang: !c.mang }))}
                  >
                    Mảng
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.id ? "on" : "off"}`}
                    onClick={() => V11y((c) => ({ ...c, id: !c.id }))}
                  >
                    ID Hồ sơ
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.tthua ? "on" : "off"}`}
                    onClick={() => V11y((c) => ({ ...c, tthua: !c.tthua }))}
                  >
                    T.Thừa
                  </a>{" "}
                  <a
                    className={`toggle-vis ${cols.tthai ? "on" : "off"}`}
                    onClick={() => V11y((c) => ({ ...c, tthai: !c.tthai }))}
                  >
                    T.Thái
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
                value={V11z}
                onChange={(V14j) => {
                  V120(Number(V14j.target.value));
                  V122(1);
                }}
              >
                {V4ea.map((n) => (
                  <option key={n} value={n}>
                    {n}
                  </option>
                ))}
              </select>{" "}
              bản ghi.
            </label>
          </div>
          <div className="pagination">
            <button className="pbtn" disabled={V13g <= 1} onClick={() => V122(V13g - 1)}>
              <Icon name="truoc" />
            </button>
            <button className="pbtn current">{V13g}</button>
            <button className="pbtn" disabled={V13g >= V13f} onClick={() => V122(V13g + 1)}>
              <Icon name="sau" />
            </button>
          </div>
          <div className="dt-goto">
            <label>Tới trang</label>
            <input
              type="number"
              min={1}
              value={V123}
              onChange={(V14k) => V124(V14k.target.value)}
            />
            <button
              className="pbtn"
              onClick={() => {
                const p = Number(V123);
                if (p >= 1 && p <= V13f) V122(p);
              }}
            >
              Đi
            </button>
          </div>
        </div>
      </div>

      {V11u && <V4e9 entity={V11u} maDa={maDa || undefined} onClose={() => V11v(null)} />}
    </div>
  );
}
