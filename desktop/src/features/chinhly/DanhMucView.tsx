// Copyright (c) 2026 DigiText. All rights reserved.
// Danh mục chỉnh lý (bản sao đồng bộ với máy chủ): khối phông, phông, phòng ban, mảng công việc,
// THBQ, ngôn ngữ, loại văn bản. Sửa được offline; dòng dùng chung (project_id trống) chỉ super admin sửa trên máy chủ.

import { useMemo, useState } from "react";
import Modal, { V49j } from "@/ui/Modal";
import { V460 } from "@/state";
import { clDelete, clSave, str, type ClCatalog, type ClRow, type ClTable } from "@/api/chinhly";
import { ChuaChonDuAn, errText, n, Select, SyncDot, useClCatalog } from "./common";

type Tab = "phong" | "phong_ban" | "mang" | "thbq" | "ngon_ngu" | "loai_van_ban" | "khoi_phong";
const TABS: [Tab, string][] = [
  ["phong", "Phông"],
  ["phong_ban", "Phòng ban"],
  ["mang", "Mảng công việc"],
  ["thbq", "Thời hạn bảo quản"],
  ["ngon_ngu", "Ngôn ngữ"],
  ["loai_van_ban", "Loại văn bản"],
  ["khoi_phong", "Khối phông"],
];

interface Col {
  k: string;
  l: string;
  type?: "text" | "number" | "select";
  opts?: [string, string][];
  w?: number;
}
const COLS: Record<Tab, Col[]> = {
  khoi_phong: [
    { k: "ma", l: "Mã khối *" },
    { k: "ten", l: "Tên khối *" },
    { k: "so_tt", l: "Thứ tự", type: "number", w: 80 },
  ],
  phong: [
    { k: "ma_du_an", l: "Mã phông (dùng tạo mã hồ sơ) *" },
    { k: "ten", l: "Tên phông *" },
    { k: "phong_so", l: "Phông số", w: 90 },
    { k: "muc_luc_so", l: "Mục lục số", w: 90 },
    { k: "khoi_id", l: "Khối phông", type: "select" },
    { k: "ghi_chu", l: "Ghi chú" },
    { k: "so_tt", l: "Thứ tự", type: "number", w: 80 },
  ],
  phong_ban: [
    { k: "phong_id", l: "Phông *", type: "select" },
    { k: "ma_don_vi", l: "Mã đơn vị (dùng tạo mã hồ sơ)" },
    { k: "ten", l: "Tên phòng ban *" },
    { k: "nhiem_ky", l: "Nhiệm kỳ" },
    { k: "so_tt", l: "Thứ tự", type: "number", w: 80 },
  ],
  mang: [
    { k: "phong_ban_id", l: "Phòng ban *", type: "select" },
    { k: "ma_mang", l: "Mã mảng (mặc định 00)" },
    { k: "ten", l: "Tên mảng công việc *" },
    { k: "so_tt", l: "Thứ tự", type: "number", w: 80 },
  ],
  thbq: [
    { k: "ten", l: "Tên *" },
    {
      k: "trang_thai",
      l: "Loại *",
      type: "select",
      opts: [
        ["1", "1 · Vĩnh viễn"],
        ["2", "2 · Có thời hạn"],
        ["3", "3 · Đánh giá lại"],
        ["4", "4 · Theo tuổi thọ công trình"],
      ],
    },
    { k: "gia_tri", l: "Số năm", type: "number", w: 80 },
    { k: "ma_tt05", l: "Mã TT05 (01–07)", w: 90 },
    { k: "so_tt", l: "Thứ tự", type: "number", w: 80 },
  ],
  ngon_ngu: [
    { k: "ma", l: "Mã *", w: 80 },
    { k: "ten", l: "Tên *" },
    { k: "so_tt", l: "Thứ tự", type: "number", w: 80 },
  ],
  loai_van_ban: [
    { k: "ma", l: "Mã *", w: 80 },
    { k: "ten", l: "Tên loại *" },
    { k: "so_tt", l: "Thứ tự", type: "number", w: 80 },
  ],
};

function Form({ tab, cat, row, onClose, onSaved }: { tab: Tab; cat: ClCatalog; row: ClRow | null; onClose: () => void; onSaved: () => void }) {
  const [d, setD] = useState<Record<string, unknown>>(() => (row ? { ...row } : tab === "mang" ? { ma_mang: "00" } : tab === "thbq" ? { trang_thai: "2", gia_tri: 0 } : {}));
  const [err, setErr] = useState<string | null>(null);
  const g = (k: string) => (d[k] === null || d[k] === undefined ? "" : String(d[k]));
  const set = (k: string, v: unknown) => setD((p) => ({ ...p, [k]: v }));
  const selRows = (k: string): ClRow[] => (k === "phong_id" ? cat.phong : k === "phong_ban_id" ? cat.phong_ban : k === "khoi_id" ? cat.khoi_phong : []);
  const luu = async () => {
    try {
      const patch: Record<string, unknown> = {};
      for (const c of COLS[tab]) patch[c.k] = d[c.k] === "" ? null : d[c.k];
      const r = await clSave(tab as ClTable, row ? row.id : null, patch);
      if (!r.ok) {
        setErr(r.errors.map((e) => `${e.field}: ${e.message}`).join("; "));
        return;
      }
      onSaved();
    } catch (e) {
      setErr(errText(e));
    }
  };
  return (
    <Modal title={row ? "Sửa" : "Thêm"} onClose={onClose} onSave={() => void luu()}>
      {err && <div className="text-danger text-sm mb-2">{err}</div>}
      {COLS[tab].map((c) => (
        <V49j key={c.k} label={c.l}>
          {c.type === "select" && c.opts ? (
            <select className="w-full" value={g(c.k)} onChange={(e) => set(c.k, e.target.value)}>
              <option value="">—</option>
              {c.opts.map(([v, l]) => (
                <option key={v} value={v}>
                  {l}
                </option>
              ))}
            </select>
          ) : c.type === "select" ? (
            <Select value={g(c.k)} onChange={(v) => set(c.k, v)} rows={selRows(c.k)} render={(r) => (c.k === "phong_id" ? `${str(r, "ma_du_an")} — ${str(r, "ten")}` : str(r, "ten"))} empty="— Chọn —" />
          ) : (
            <input className="inp w-full" type={c.type === "number" ? "number" : "text"} value={g(c.k)} onChange={(e) => set(c.k, e.target.value)} />
          )}
        </V49j>
      ))}
    </Modal>
  );
}

export default function DanhMucView() {
  const { session } = V460();
  const perms = session?.permissions ?? [];
  const isSuper = session?.role === "super_admin";
  const canCreate = perms.includes("catalog.create");
  const canUpdate = perms.includes("catalog.update");
  const canDelete = perms.includes("catalog.delete");
  const { cat, reload, error } = useClCatalog();
  const [tab, setTab] = useState<Tab>("phong");
  const [form, setForm] = useState<{ row: ClRow | null } | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const rows = cat[tab];
  const ten = useMemo(() => {
    const m = new Map<string, string>();
    for (const r of [...cat.phong, ...cat.phong_ban, ...cat.khoi_phong]) m.set(r.id, str(r, "ten"));
    return m;
  }, [cat]);
  const chung = (r: ClRow) => r.project_id === null || r.project_id === undefined;
  const xoa = async (r: ClRow) => {
    if (!window.confirm(`Xoá "${str(r, "ten")}"? (bị chặn nếu còn hồ sơ / danh mục con đang dùng)`)) return;
    try {
      await clDelete(tab as ClTable, r.id);
      reload();
    } catch (e) {
      setErr(errText(e));
    }
  };
  return (
    <>
      <div className="page-head">
        <h2>Danh mục chỉnh lý</h2>
        <div className="sub">· dùng chung toàn dự án, đồng bộ với máy chủ · dòng có dấu ★ là bảng mã chuẩn TT05 (chỉ super admin sửa)</div>
      </div>
      <ChuaChonDuAn />
      {error && <div className="card mt-3.5 text-danger">{error}</div>}
      <div className="card mt-3.5">
        <div className="flex gap-2 flex-wrap items-center">
          {TABS.map(([k, l]) => (
            <button key={k} className={`btn sm ${tab === k ? "primary" : ""}`} onClick={() => setTab(k)}>
              {l} <span className="muted">({n(cat[k].length)})</span>
            </button>
          ))}
          <div className="flex-1" />
          {canCreate && (
            <button className="btn primary" onClick={() => setForm({ row: null })}>
              + Thêm
            </button>
          )}
        </div>
        {err && <div className="text-danger text-sm mt-2">{err}</div>}
        <div className="tbl-scroll mt-3">
          <table className="tbl">
            <thead>
              <tr>
                <th style={{ width: 24 }} />
                {COLS[tab].map((c) => (
                  <th key={c.k} style={c.w ? { width: c.w } : undefined} className={c.type === "number" ? "num" : ""}>
                    {c.l.replace(" *", "")}
                  </th>
                ))}
                <th />
              </tr>
            </thead>
            <tbody>
              {rows.length === 0 && (
                <tr>
                  <td colSpan={COLS[tab].length + 2} className="muted">
                    Chưa có dòng nào.
                  </td>
                </tr>
              )}
              {rows.map((r) => {
                const locked = chung(r) && !isSuper;
                return (
                  <tr key={r.id}>
                    <td>{chung(r) ? <span title="Bảng mã dùng chung">★</span> : <SyncDot sync={r._sync?.dirty ? "pending" : "clean"} />}</td>
                    {COLS[tab].map((c) => (
                      <td key={c.k} className={c.type === "number" ? "num" : ""}>
                        {c.type === "select" && c.opts ? c.opts.find(([v]) => v === str(r, c.k))?.[1] ?? str(r, c.k) : c.type === "select" ? ten.get(str(r, c.k)) ?? "" : str(r, c.k)}
                      </td>
                    ))}
                    <td className="whitespace-nowrap">
                      {canUpdate && !locked && (
                        <button className="btn sm" onClick={() => setForm({ row: r })}>
                          Sửa
                        </button>
                      )}{" "}
                      {canDelete && !locked && (
                        <button className="btn sm" onClick={() => void xoa(r)}>
                          Xoá
                        </button>
                      )}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
      {form && (
        <Form
          tab={tab}
          cat={cat}
          row={form.row}
          onClose={() => setForm(null)}
          onSaved={() => {
            setForm(null);
            reload();
          }}
        />
      )}
      {tab === "phong" && (
        <div className="card mt-3.5 muted text-sm">
          Mã hồ sơ = <code>Mã phông.Phông số.Năm kết thúc.Số hồ sơ[.Mục lục số]</code>. Mã văn bản theo TT05 = mã hồ sơ + số thứ tự 7 chữ số.
        </div>
      )}
    </>
  );
}
