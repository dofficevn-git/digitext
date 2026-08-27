// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import "./laphoso.css";
import { V3xq, isTauri } from "@/api/ipc";
import { V4cj, type V4ch } from "@/api/dossier";
import { V4e3 } from "./excel";
import { V4d9, V4da } from "./logic";
import { Icon } from "@/components/Icon";

export default function V4e9({
  entity: V10b,
  maDa,
  onClose: V10c,
}: {
  entity: "hoso" | "vanban";
  maDa?: string;
  onClose: () => void;
}) {
  const [V10d, V10e] = useState("");
  const [V10f, V10g] = useState("");
  const [V10h, V10i] = useState("");
  const [V10j, V10k] = useState<V4ch[]>([]);
  const [V10l, V10m] = useState<string | null>(null);

  const V10n =
    V10b === "hoso"
      ? "TOG7ikNIIFPhu6wgQ+G6rFAgTkjhuqxUIEjhu5IgU8ag"
      : "TOG7ikNIIFPhu6wgQ+G6rFAgTkjhuqxUIFbEgk4gQuG6ok4=";

  const reload = useCallback(() => {
    if (!isTauri()) return;
    V4cj({
      entity: V10b,
      maDa: maDa || undefined,
      tuNgay: V10d || undefined,
      denNgay: V10f || undefined,
      search: V10h || undefined,
    })
      .then((rs) => {
        V10k(rs);
        V10m(null);
      })
      .catch((V10o) => V10m(V10o instanceof V3xq ? V10o.message : String(V10o)));
  }, [V10b, maDa, V10d, V10f, V10h]);

  V41p(reload, { topics: ["dossier"] });

  const V10p = async () => {
    const data = V10j.map((r) => {
      const { ngay, gio: V10q } = V4da(r.createdAt);
      return {
        VGhhbyB0w6Fj: V4d9(r.action),
        Ngày: ngay,
        Giờ: V10q,
        "TmfGsOG7nWkgdOG6oW8=": r.nguoi ?? "",
        "VMOzbSB04bqvdA==": r.summary ?? "",
        "TuG7mWkgZHVuZw==": r.content ?? "",
      };
    });
    await V4e3(data.length ? data : [{ VGhhbyB0w6Fj: "" }], "LichSu", `lich-su-${V10b}.xlsx`);
  };

  return (
    <div className="lhs-overlay" onClick={V10c}>
      <div className="lhs" onClick={(V10r) => V10r.stopPropagation()}>
        <div className="lhs-modal">
          <div className="m-head">
            <h4>{V10n}</h4>
            <button className="m-close" onClick={V10c}>
              ×
            </button>
          </div>
          <div className="m-body">
            {V10l && (
              <div className="banner err">
                <span className="bi">
                  <Icon name="canhBao" size={18} />
                </span>
                <span className="bx">{V10l}</span>
              </div>
            )}

            <div className="row items-center">
              <div className="col-4">
                <div className="input-group">
                  <label className="input-group-addon w-[35%]">Từ ngày</label>
                  <input
                    className="form-control"
                    type="date"
                    value={V10d}
                    onChange={(V10s) => V10e(V10s.target.value)}
                  />
                </div>
              </div>
              <div className="col-4">
                <div className="input-group">
                  <label className="input-group-addon w-[35%]">Đến ngày</label>
                  <input
                    className="form-control"
                    type="date"
                    value={V10f}
                    onChange={(V10t) => V10g(V10t.target.value)}
                  />
                </div>
              </div>
              <div className="col-2">
                <input
                  className="form-control solo"
                  placeholder="VMOsbSBraeG6v23igKY="
                  value={V10h}
                  onChange={(V10u) => V10i(V10u.target.value)}
                  onKeyDown={(V10v) => V10v.key === "Enter" && reload()}
                />
              </div>
              <div className="col-2 text-right flex gap-1.5 justify-end">
                <button className="btn btn-outline-primary btn-round" onClick={() => reload()}>
                  <Icon name="tim" /> Tìm
                </button>
                <button
                  className="btn btn-outline-secondary btn-round"
                  disabled={V10j.length === 0}
                  onClick={() => void V10p()}
                >
                  <Icon name="tepExcel" /> Tải lịch sử
                </button>
              </div>
            </div>

            <hr />

            <div className="tbl-scroll">
              <table className="dataTable">
                <thead>
                  <tr>
                    <th className="w-17.5">Thao tác</th>
                    <th className="w-22.5">Ngày</th>
                    <th className="w-20">Giờ</th>
                    <th className="w-30">Người tạo</th>
                    <th>Tóm tắt</th>
                    <th>Nội dung</th>
                  </tr>
                </thead>
                <tbody>
                  {V10j.length === 0 ? (
                    <tr>
                      <td className="dt-empty" colSpan={6}>
                        Không tìm thấy dữ liệu.
                      </td>
                    </tr>
                  ) : (
                    V10j.map((r, i) => {
                      const { ngay, gio: V10w } = V4da(r.createdAt);
                      return (
                        <tr key={r.id} className={i % 2 === 0 ? "odd" : "even"}>
                          <td className="text-center">{V4d9(r.action)}</td>
                          <td className="text-center">{ngay}</td>
                          <td className="text-center">{V10w}</td>
                          <td className="text-center">{r.nguoi}</td>
                          <td className="text-justify">{r.summary}</td>
                          <td className="text-left">{r.content}</td>
                        </tr>
                      );
                    })
                  )}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
