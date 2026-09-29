// Copyright (c) 2026 DigiText. All rights reserved.

import { useMemo, useRef, useState } from "react";
import { V4at } from "@/domain/hoSoTime";
import type { V3yr } from "@/domain/types";
import { V4bf, V4b9, V4b5, V4be, V4b7, type V4b8, type V4b2 } from "@/domain/catalogImport";
import { Icon } from "@/components/Icon";

export default function V4bg({
  startId: Vrj,
  onClose: Vrk,
  onImport: Vrl,
}: {
  startId: number;
  onClose: () => void;
  onImport: (records: V3yr[]) => void;
}) {
  const [Vrm, Vrn] = useState<V4b2 | null>(null);
  const [Vro, Vrp] = useState("");
  const [Vrq, Vrr] = useState("");
  const [Vrs, Vrt] = useState(-1);
  const [Vru, Vrv] = useState<Record<string, number>>({});
  const [Vrw, Vrx] = useState("");
  const [Vry, Vrz] = useState(false);
  const Vs0 = useRef<HTMLInputElement>(null);

  const Vs1 = (w: V4b2, name: string) => {
    const s = w.sheets.find((x) => x.name === name);
    const d = s ? V4b9(s.rows) : { headerRow: -1, map: {}, score: 0 };
    Vrr(name);
    Vrt(d.headerRow);
    Vrv(d.map);
  };

  const Vs2 = async (f: File) => {
    Vrz(true);
    Vrx("");
    try {
      const w = await V4b7(f);
      Vrn(w);
      Vrp(f.name);
      Vs1(w, V4bf(w));
    } catch (Vs3) {
      Vrx("Không đọc được file: " + String(Vs3));
    } finally {
      Vrz(false);
    }
  };

  const Vs4 = Vrm?.sheets.find((s) => s.name === Vrq);
  const Vs5 = Vs4 ? ((Vrs >= 0 ? Vs4.rows[Vrs] : Vs4.rows[0]) ?? []) : [];
  const Vs6 = Vs4 ? Math.max(0, ...Vs4.rows.slice(0, 30).map((r) => r.length)) : 0;
  const Vs7 = Array.from({ length: Vs6 }, (_Vs8, i) => ({
    i,
    label: (Vs5[i] || `Cột ${i + 1}`).slice(0, 30),
  }));

  const Vs9 = useMemo(() => {
    if (!Vs4) return null;
    const Vsa: V4b8 = { headerRow: Vrs, map: Vru, score: Object.keys(Vru).length };
    return V4be(Vs4.rows, Vsa, Vrq, Vrj);
  }, [Vs4, Vrs, Vru, Vrq, Vrj]);

  const Vsb = (Vsc: string, Vsd: number) =>
    Vrv((m) => {
      const n = { ...m };
      if (Vsd < 0) delete n[Vsc];
      else n[Vsc] = Vsd;
      return n;
    });

  return (
    <div className="overlay" onClick={Vrk}>
      <div className="dialog xl" onClick={(Vse) => Vse.stopPropagation()}>
        <h3>
          <Icon name="nhapTep" /> Nhập danh mục hồ sơ từ Excel
        </h3>

        {!Vrm ? (
          <div className="imp-drop" onClick={() => Vs0.current?.click()}>
            <div className="imp-drop-ic">
              <Icon name="taiLieu" size={28} />
            </div>
            <div>
              <b>Bấm để chọn file Excel</b> (.xlsx, .xls, .csv)
            </div>
            <small className="muted">
              Hệ thống tự nhận diện các cột: Hộp số · Hồ sơ số · Tiêu đề · Số tờ · THBQ · Ghi chú
            </small>
          </div>
        ) : (
          <>
            <div className="imp-filebar">
              <span className="imp-fname">
                <Icon name="taiLieu" />
                <b>{Vro}</b>
              </span>
              <select className="inp" value={Vrq} onChange={(Vsf) => Vs1(Vrm, Vsf.target.value)}>
                {Vrm.sheets.map((s) => (
                  <option key={s.name} value={s.name}>
                    {s.name} ({s.rows.length} dòng)
                  </option>
                ))}
              </select>
              <button
                className="btn sm"
                onClick={() => {
                  Vrn(null);
                  Vrp("");
                }}
              >
                Chọn file khác
              </button>
            </div>

            <div className="imp-map">
              {V4b5.map((f) => (
                <label key={f.key as string} className="imp-mapr">
                  <span>{f.label}</span>
                  <select
                    className="inp"
                    value={Vru[f.key as string] ?? -1}
                    onChange={(Vsg) => Vsb(f.key as string, Number(Vsg.target.value))}
                  >
                    <option value={-1}>— bỏ qua —</option>
                    {Vs7.map((c) => (
                      <option key={c.i} value={c.i}>
                        {c.label}
                      </option>
                    ))}
                  </select>
                </label>
              ))}
            </div>

            <div className="imp-stats">
              <span className="badge ok">{Vs9?.records.length ?? 0} hồ sơ</span>
              <span className="badge gray">{Vs9?.groups ?? 0} dòng nhóm</span>
              {(Vs9?.skipped ?? 0) > 0 && <span className="badge warn">{Vs9?.skipped} bỏ qua</span>}
            </div>

            <div className="imp-preview">
              <div className="tbl-scroll">
                <table className="tbl">
                  <thead>
                    <tr>
                      <th>Hộp</th>
                      <th>HS số</th>
                      <th>Tiêu đề</th>
                      <th>Bắt đầu</th>
                      <th>Kết thúc</th>
                      <th>Số tờ</th>
                      <th>THBQ</th>
                      <th>Nhóm / Năm</th>
                    </tr>
                  </thead>
                  <tbody>
                    {(Vs9?.records.slice(0, 8) ?? []).map((r) => (
                      <tr key={r.id}>
                        <td className="tnum">{r.hopSo}</td>
                        <td className="tnum">{r.hoSoSo}</td>
                        <td>{r.tieuDe.slice(0, 60)}</td>
                        <td className="tnum">{V4at(r).batDau}</td>
                        <td className="tnum">{V4at(r).ketThuc}</td>
                        <td className="num">{r.soTo}</td>
                        <td>{r.thbq}</td>
                        <td className="muted">
                          {[r.nhom, r.nam].filter(Boolean).join(" · ").slice(0, 40)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              {Vs9 && Vs9.records.length > 8 && (
                <div className="muted imp-more">… và {Vs9.records.length - 8} hồ sơ khác</div>
              )}
            </div>
          </>
        )}

        {Vrw && (
          <div className="banner err mt-3">
            <span className="bx">{Vrw}</span>
          </div>
        )}

        <input
          ref={Vs0}
          type="file"
          accept=".xlsx,.xls,.csv"
          className="hidden"
          onChange={(Vsh) => {
            const f = Vsh.target.files?.[0];
            if (f) Vs2(f);
            Vsh.target.value = "";
          }}
        />

        <div className="actions">
          <button className="btn" onClick={Vrk}>
            Hủy
          </button>
          <button
            className="btn primary"
            disabled={!Vs9 || Vs9.records.length === 0 || Vry}
            onClick={() => {
              if (Vs9 && Vs9.records.length) {
                Vrl(Vs9.records);
                Vrk();
              }
            }}
          >
            Nhập {Vs9?.records.length ?? 0} hồ sơ
          </button>
        </div>
      </div>
    </div>
  );
}
