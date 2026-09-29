// Copyright (c) 2026 DigiText. All rights reserved.
// Kho và giá lưu trữ (chỉ online): kho (tên, ghi chú), giá = dải hộp [từ–đến] của một phòng ban trong kho;
// hồ sơ thuộc giá suy ra từ số hộp (như bản cũ), hoặc gán tay qua trường gia_id.

import { useMemo, useState } from "react";
import Modal, { V49j } from "@/ui/Modal";
import { V460 } from "@/state";
import { clRemoteDelete, clRemoteSave, str, num, type ClRow } from "@/api/chinhly";
import { ChonDuAn, errText, n, Select, useClCatalog, useClSync } from "./common";

export default function KhoGiaView({ tab }: { tab: "kho" | "gia" }) {
  const { session } = V460();
  const projectId = session?.projectId ?? "";
  const can = (session?.permissions ?? []).includes("khogia.manage");
  const { cat, reload } = useClCatalog();
  const sync = useClSync();
  const online = sync?.phase !== "offline";
  const [form, setForm] = useState<{ row: ClRow | null } | null>(null);
  const [d, setD] = useState<Record<string, unknown>>({});
  const [err, setErr] = useState<string | null>(null);
  const [phongId, setPhongId] = useState("");
  const phongBans = useMemo(() => cat.phong_ban.filter((p) => !phongId || str(p, "phong_id") === phongId), [cat.phong_ban, phongId]);
  const tenKho = new Map(cat.kho.map((k) => [k.id, str(k, "ten")]));
  const tenPb = new Map(cat.phong_ban.map((k) => [k.id, str(k, "ten")]));
  const table = tab === "kho" ? "kho" : "gia";
  const g = (k: string) => (d[k] === null || d[k] === undefined ? "" : String(d[k]));
  const set = (k: string, v: unknown) => setD((p) => ({ ...p, [k]: v }));
  const mo = (row: ClRow | null) => {
    setD(row ? { ...row } : tab === "gia" ? { kho_id: cat.kho[0]?.id ?? "" } : {});
    setForm({ row });
    setErr(null);
  };
  const luu = async () => {
    try {
      const keys = tab === "kho" ? ["ten", "ghi_chu", "so_tt"] : ["kho_id", "phong_ban_id", "ten", "tu_hop", "den_hop", "ghi_chu", "so_tt"];
      const body: Record<string, unknown> = {};
      for (const k of keys) body[k] = d[k] === "" ? null : d[k];
      await clRemoteSave(table, projectId, form?.row?.id ?? null, body, form?.row?._sync?.ver);
      setForm(null);
      reload();
    } catch (e) {
      setErr(errText(e));
    }
  };
  const xoa = async (r: ClRow) => {
    if (!window.confirm(`Xoá "${str(r, "ten")}"?`)) return;
    try {
      await clRemoteDelete(table, projectId, r.id);
      reload();
    } catch (e) {
      setErr(errText(e));
    }
  };
  const soHoSoTrenGia = (gid: string) => 0 * gid.length; // đếm chi tiết ở màn Khai thác (lọc theo giá)
  void soHoSoTrenGia;

  return (
    <>
      <div className="page-head">
        <h2>{tab === "kho" ? "Kho lưu trữ" : "Giá lưu trữ"}</h2>
        <div className="sub">· dùng chung toàn dự án · giá = dải hộp của một phòng ban trong kho; hồ sơ trên giá suy ra từ số hộp</div>
      </div>
      <ChonDuAn />
      {!online && <div className="card mt-3.5 text-warning-700">Đang ngoại tuyến — kho / giá chỉ sửa được khi có máy chủ (danh sách dưới là bản sao).</div>}
      <div className="card mt-3.5">
        <div className="flex gap-2 items-center flex-wrap">
          {tab === "gia" && <Select value={phongId} onChange={setPhongId} rows={cat.phong} render={(r) => `${str(r, "ma_du_an")} — ${str(r, "ten")}`} empty="— Mọi phông —" />}
          <div className="flex-1" />
          {can && (
            <button className="btn primary" disabled={!online} onClick={() => mo(null)}>
              + Thêm {tab === "kho" ? "kho" : "giá"}
            </button>
          )}
        </div>
        {err && <div className="text-danger text-sm mt-2">{err}</div>}
        <div className="tbl-scroll mt-3">
          <table className="tbl">
            <thead>
              {tab === "kho" ? (
                <tr>
                  <th>Tên kho</th>
                  <th className="num">Số giá</th>
                  <th>Ghi chú</th>
                  <th />
                </tr>
              ) : (
                <tr>
                  <th>Kho</th>
                  <th>Phòng ban</th>
                  <th>Tên giá</th>
                  <th className="num">Từ hộp</th>
                  <th className="num">Đến hộp</th>
                  <th>Ghi chú</th>
                  <th />
                </tr>
              )}
            </thead>
            <tbody>
              {(tab === "kho" ? cat.kho : cat.gia.filter((g) => !phongId || phongBans.some((p) => p.id === str(g, "phong_ban_id")))).map((r) => (
                <tr key={r.id}>
                  {tab === "kho" ? (
                    <>
                      <td>{str(r, "ten")}</td>
                      <td className="num">{n(cat.gia.filter((g) => str(g, "kho_id") === r.id).length)}</td>
                      <td className="muted">{str(r, "ghi_chu")}</td>
                    </>
                  ) : (
                    <>
                      <td>{tenKho.get(str(r, "kho_id")) ?? ""}</td>
                      <td>{tenPb.get(str(r, "phong_ban_id")) ?? ""}</td>
                      <td>{str(r, "ten")}</td>
                      <td className="num">{num(r, "tu_hop") ?? ""}</td>
                      <td className="num">{num(r, "den_hop") ?? ""}</td>
                      <td className="muted">{str(r, "ghi_chu")}</td>
                    </>
                  )}
                  <td className="whitespace-nowrap">
                    {can && (
                      <>
                        <button className="btn sm" disabled={!online} onClick={() => mo(r)}>
                          Sửa
                        </button>{" "}
                        <button className="btn sm" disabled={!online} onClick={() => void xoa(r)}>
                          Xoá
                        </button>
                      </>
                    )}
                  </td>
                </tr>
              ))}
              {(tab === "kho" ? cat.kho : cat.gia).length === 0 && (
                <tr>
                  <td colSpan={7} className="muted">
                    Chưa có {tab === "kho" ? "kho" : "giá"} nào.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
      {form && (
        <Modal title={form.row ? "Sửa" : "Thêm"} onClose={() => setForm(null)} onSave={() => void luu()}>
          {err && <div className="text-danger text-sm mb-2">{err}</div>}
          {tab === "gia" && (
            <>
              <V49j label="Kho *">
                <Select value={g("kho_id")} onChange={(v) => set("kho_id", v)} rows={cat.kho} />
              </V49j>
              <V49j label="Phòng ban (chủ dải hộp) *">
                <Select value={g("phong_ban_id")} onChange={(v) => set("phong_ban_id", v)} rows={cat.phong_ban} render={(p) => `${str(cat.phong.find((x) => x.id === str(p, "phong_id")), "ma_du_an")} / ${str(p, "ten")}`} />
              </V49j>
            </>
          )}
          <V49j label={tab === "kho" ? "Tên kho *" : "Tên giá *"}>
            <input className="inp w-full" value={g("ten")} onChange={(e) => set("ten", e.target.value)} />
          </V49j>
          {tab === "gia" && (
            <V49j label="Từ hộp – đến hộp">
              <div className="flex gap-2">
                <input className="inp" type="number" placeholder="Từ hộp" value={g("tu_hop")} onChange={(e) => set("tu_hop", e.target.value)} />
                <input className="inp" type="number" placeholder="Đến hộp" value={g("den_hop")} onChange={(e) => set("den_hop", e.target.value)} />
              </div>
            </V49j>
          )}
          <V49j label="Ghi chú">
            <input className="inp w-full" value={g("ghi_chu")} onChange={(e) => set("ghi_chu", e.target.value)} />
          </V49j>
          <V49j label="Thứ tự">
            <input className="inp w-full" type="number" value={g("so_tt")} onChange={(e) => set("so_tt", e.target.value)} />
          </V49j>
        </Modal>
      )}
    </>
  );
}
