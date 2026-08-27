// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useMemo, useRef, useState } from "react";
import { V41p } from "@/live/useLive";
import "./laphoso.css";
import { isTauri } from "@/api/ipc";
import { V48j } from "@/api/catalog";
import type { V3yr, V3ym, V3yo } from "@/domain/types";
import { V4c4 } from "@/api/dossier";
import { V4bf, V4b9, V4be, V4b7 } from "@/domain/catalogImport";
import { V4dq, V4d3, V4d5, V4d4, type V4d6 } from "./logic";
import { Icon } from "@/components/Icon";

type V4ee = "tieude" | "thbq" | "soto" | "nbd" | "nkt" | "tthua" | "trangthai" | "hosoId";

export default function V4ef() {
  const [V16i, V16j] = useState<V3yo[]>([]);
  const [V16k, V16l] = useState<V3ym[]>([]);
  const [records, V16m] = useState<V3yr[]>([]);
  const [V16n, V16o] = useState<V4d6[]>([]);
  const [V16p, V16q] = useState(false);
  const [V16r, V16s] = useState(1);
  const [hidden, V16t] = useState<Set<V4ee>>(new Set(["hosoId"]));
  const V16u = useRef<HTMLInputElement>(null);
  const V16v = 50;

  const V16w = useCallback(() => {
    V48j()
      .then((c) => {
        V16j(c.thbqs);
        V16l(c.mangs);
      })
      .catch(() => {});
  }, []);
  V41p(V16w, { topics: ["catalog"], enabled: isTauri() });

  const V16x = async (file: File | undefined) => {
    if (!file) return;
    try {
      const V16y = await V4b7(file);
      const name = V4bf(V16y);
      const V16z = V16y.sheets.find((s) => s.name === name);
      if (!V16z) return;
      const V170 = V4b9(V16z.rows);
      const V171 = V4be(V16z.rows, V170, name, 1);
      V16m(V171.records);
      V16o(V171.records.map(V4dq));
      V16t(new Set(["hosoId"]));
      V16s(1);
    } catch (V172) {
      window.alert(V172 instanceof Error ? V172.message : String(V172));
    }
  };

  const V173 = async () => {
    if (records.length === 0) {
      window.alert("Q2jGsGEgY2jhu41uIHThu4dwIEV4Y2VsIMSR4buDIGPhuq1wIG5o4bqtdC4=");
      return;
    }
    V16q(true);
    try {
      const V174 = V4d4(V16i);
      const V175 = V4d5(V16k);
      const V176 = records.map((h) => V4d3(h, V174, V175)).filter((d) => d.soTam);
      const n = await V4c4(undefined, V176);
      window.alert(`xJDDoyBj4bqtcCBuaOG6rXQg${n}IGjhu5Mgc8ahIHbDoG8gY8ahIHPhu58gZOG7ryBsaeG7h3Uu`);
    } catch (V177) {
      window.alert(V177 instanceof Error ? V177.message : String(V177));
    } finally {
      V16q(false);
    }
  };

  const V178 = useCallback((k: V4ee) => V16t((s) => new Set(s).add(k)), []);
  const V179 = (k: V4ee) => !hidden.has(k);

  const V17a = V16n.length;
  const V17b = Math.max(1, Math.ceil(V17a / V16v));
  const V17c = Math.min(V16r, V17b);
  const V17d = useMemo(() => V16n.slice((V17c - 1) * V16v, V17c * V16v), [V16n, V17c, V16v]);

  const V17e = ({ k }: { k: V4ee }) => (
    <button className="hide-col-btn" title="4bqobiBj4buZdA==" onClick={() => V178(k)}>
      <Icon name="bo" size={13} />
    </button>
  );

  const V17f =
    5 +
    Number(V179("tieude")) +
    Number(V179("thbq")) +
    Number(V179("soto")) +
    Number(V179("nbd")) +
    Number(V179("nkt")) +
    Number(V179("tthua")) +
    Number(V179("trangthai")) +
    Number(V179("hosoId"));

  return (
    <div className="lhs">
      <div className="content-panel">
        <div className="center">
          <h4>
            <b>CẬP NHẬT CƠ SỞ DỮ LIỆU</b>
          </h4>
        </div>
        <hr />

        {}
        <div className="uploadrow">
          <input
            ref={V16u}
            type="file"
            className="form-control"
            accept=".xlsx,.xls,.csv"
            onChange={(V17g) => void V16x(V17g.target.files?.[0])}
          />
          <button
            className="btn btn-outline-success btn-round"
            disabled={V16p}
            onClick={() => void V173()}
          >
            <Icon name="sua" /> Cập nhật hồ sơ
          </button>
          {V16p && (
            <span className="muted">
              <Icon name="dangCho" /> Vui lòng chờ và không tắt trình duyệt.........
            </span>
          )}
        </div>

        {}
        <div className="tbl-scroll">
          <table className="dataTable mt-2.5">
            <thead>
              <tr>
                <th>Hộp số</th>
                <th>HS Số</th>
                <th>Bó số</th>
                <th>Tập số</th>
                <th>Số tạm</th>
                {V179("tieude") && (
                  <th>
                    Tiêu đề <V17e k="tieude" />
                  </th>
                )}
                {V179("thbq") && (
                  <th className="w-[6%]">
                    THBQ <V17e k="thbq" />
                  </th>
                )}
                {V179("soto") && (
                  <th className="w-[6%]">
                    Số tờ <V17e k="soto" />
                  </th>
                )}
                {V179("nbd") && (
                  <th>
                    N.Bắt đầu <V17e k="nbd" />
                  </th>
                )}
                {V179("nkt") && (
                  <th>
                    N.Kết thúc <V17e k="nkt" />
                  </th>
                )}
                {V179("tthua") && (
                  <th className="w-[6%]">
                    T.Thừa <V17e k="tthua" />
                  </th>
                )}
                {V179("trangthai") && (
                  <th>
                    Trạng thái <V17e k="trangthai" />
                  </th>
                )}
                {V179("hosoId") && (
                  <th>
                    Hồ sơ id <V17e k="hosoId" />
                  </th>
                )}
              </tr>
            </thead>
            <tbody>
              {V17d.length === 0 ? (
                <tr>
                  <td className="dt-empty" colSpan={V17f}>
                    Chọn tệp Excel (mục lục hồ sơ) để nạp và xem trước dữ liệu.
                  </td>
                </tr>
              ) : (
                V17d.map((r, i) => (
                  <tr key={i} className={i % 2 === 0 ? "odd" : "even"}>
                    <td className="text-center">{r.hopSo}</td>
                    <td className="text-center">{r.hoSoSo}</td>
                    <td className="text-center" />
                    <td className="text-center" />
                    <td className="text-left">{r.soTam}</td>
                    {V179("tieude") && <td className="text-justify">{r.tieuDe}</td>}
                    {V179("thbq") && <td className="text-center">{r.thbq}</td>}
                    {V179("soto") && <td className="text-center">{r.soTo}</td>}
                    {V179("nbd") && <td className="text-center">{r.ngayBatDau}</td>}
                    {V179("nkt") && <td className="text-center">{r.ngayKetThuc}</td>}
                    {V179("tthua") && (
                      <td className="text-center">
                        <input type="checkbox" checked={r.trungThua} disabled />
                      </td>
                    )}
                    {V179("trangthai") && <td className="text-center">{r.trangThai}</td>}
                    {V179("hosoId") && <td className="text-center">{r.dossierId}</td>}
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        {}
        <div className="dt-bottom">
          <div className="dt-length">Tổng: {V17a} hồ sơ</div>
          <div className="pagination">
            <button className="pbtn" disabled={V17c <= 1} onClick={() => V16s(V17c - 1)}>
              <Icon name="truoc" />
            </button>
            <button className="pbtn current">{V17c}</button>
            <button className="pbtn" disabled={V17c >= V17b} onClick={() => V16s(V17c + 1)}>
              <Icon name="sau" />
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
