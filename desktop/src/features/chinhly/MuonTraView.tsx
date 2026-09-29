// Copyright (c) 2026 DigiText. All rights reserved.
// Mượn / trả hồ sơ giấy (chỉ online): lập phiếu (người mượn, đơn vị, SĐT, email, nội dung, ngày mượn, hạn trả),
// ghi trả, quá hạn; một hồ sơ chỉ có một phiếu đang mượn.

import { useCallback, useState } from "react";
import Modal, { V49j } from "@/ui/Modal";
import { isTauri } from "@/api/ipc";
import { V41p } from "@/live/useLive";
import { V460 } from "@/state";
import { clHoSoList, clRemote, clRemoteSave, qs, str, type ClRow } from "@/api/chinhly";
import { ChonDuAn, errText, fmtTime, n, useClSync } from "./common";

const TT: Record<string, [string, string]> = { dang_muon: ["Đang mượn", "info"], da_tra: ["Đã trả", "ok"] };

export default function MuonTraView() {
  const { session } = V460();
  const projectId = session?.projectId ?? "";
  const perms = session?.permissions ?? [];
  const canManage = perms.includes("loan.manage");
  const sync = useClSync();
  const online = sync?.phase !== "offline";
  const [trangThai, setTrangThai] = useState("dang_muon");
  const [q, setQ] = useState("");
  const [qA, setQA] = useState("");
  const [rows, setRows] = useState<ClRow[]>([]);
  const [total, setTotal] = useState(0);
  const [err, setErr] = useState<string | null>(null);
  const [form, setForm] = useState<{ row: ClRow | null } | null>(null);
  const [d, setD] = useState<Record<string, unknown>>({});
  const [timHs, setTimHs] = useState("");
  const [hsGoiY, setHsGoiY] = useState<ClRow[]>([]);
  const [hsChon, setHsChon] = useState<ClRow | null>(null);

  const load = useCallback(async () => {
    if (!online) return;
    try {
      const r = await clRemote<{ rows: ClRow[]; total: number }>("GET", `/cl/muon-tra${qs({ projectId, trangThai, q: qA, limit: 500 })}`);
      setRows(r.rows);
      setTotal(r.total);
      setErr(null);
    } catch (e) {
      setErr(errText(e));
    }
  }, [online, projectId, trangThai, qA]);
  const { reload } = V41p(load, { topics: ["chinhLy"], enabled: isTauri() });

  const g = (k: string) => (d[k] === null || d[k] === undefined ? "" : String(d[k]));
  const set = (k: string, v: unknown) => setD((p) => ({ ...p, [k]: v }));
  const mo = (row: ClRow | null) => {
    const today = new Date().toISOString().slice(0, 10);
    setD(row ? { ...row } : { ngay_muon: today, ngay_du_kien_tra: new Date(Date.now() + 7 * 86400000).toISOString().slice(0, 10) });
    setHsChon(null);
    setTimHs("");
    setHsGoiY([]);
    setForm({ row });
    setErr(null);
  };
  const timHoSo = async (t: string) => {
    setTimHs(t);
    if (t.trim().length < 2) {
      setHsGoiY([]);
      return;
    }
    try {
      const r = await clHoSoList({ q: t.trim(), limit: 10 });
      setHsGoiY(r.rows);
    } catch {
      setHsGoiY([]);
    }
  };
  const luu = async () => {
    try {
      const body: Record<string, unknown> = {};
      for (const k of ["nguoi_muon", "don_vi", "dien_thoai", "email", "noi_dung", "ngay_muon", "ngay_du_kien_tra", "ghi_chu"]) body[k] = d[k] === "" ? null : d[k];
      if (!form?.row) {
        if (!hsChon) {
          setErr("Chọn hồ sơ cho mượn.");
          return;
        }
        body.ho_so_id = hsChon.id;
        body.trang_thai = "dang_muon";
      }
      await clRemoteSave("muon_tra", projectId, form?.row?.id ?? null, body, form?.row ? Number(form.row.ver) : undefined);
      setForm(null);
      reload();
    } catch (e) {
      setErr(errText(e));
    }
  };
  const tra = async (r: ClRow) => {
    if (!window.confirm(`Ghi trả hồ sơ "${str(r, "tieu_de")}" hôm nay?`)) return;
    try {
      await clRemoteSave("muon_tra", projectId, r.id, { trang_thai: "da_tra", ngay_thuc_te_tra: new Date().toISOString().slice(0, 10) }, Number(r.ver));
      reload();
    } catch (e) {
      setErr(errText(e));
    }
  };

  return (
    <>
      <div className="page-head">
        <h2>Mượn – trả hồ sơ giấy</h2>
        <div className="sub">· phiếu mượn, hạn trả, quá hạn · cần kết nối máy chủ</div>
      </div>
      <ChonDuAn />
      {!online && <div className="card mt-3.5 text-warning-700">Đang ngoại tuyến — mượn / trả cần máy chủ.</div>}
      <div className="card mt-3.5">
        <div className="flex gap-2 items-center flex-wrap">
          <select value={trangThai} onChange={(e) => setTrangThai(e.target.value)}>
            <option value="dang_muon">Đang mượn</option>
            <option value="qua_han">Quá hạn</option>
            <option value="da_tra">Đã trả</option>
            <option value="">Tất cả</option>
          </select>
          <input className="inp max-w-72" placeholder="tìm người mượn / đơn vị / hồ sơ (Enter)" value={q} onChange={(e) => setQ(e.target.value)} onKeyDown={(e) => e.key === "Enter" && setQA(q.trim())} />
          <span className="muted text-sm">{n(total)} phiếu</span>
          <div className="flex-1" />
          {canManage && (
            <button className="btn primary" disabled={!online} onClick={() => mo(null)}>
              + Lập phiếu mượn
            </button>
          )}
        </div>
        {err && <div className="text-danger text-sm mt-2">{err}</div>}
        <div className="tbl-scroll mt-3">
          <table className="tbl">
            <thead>
              <tr>
                <th>Hồ sơ</th>
                <th>Hộp / HS số</th>
                <th>Người mượn</th>
                <th>Đơn vị</th>
                <th>Điện thoại</th>
                <th>Nội dung</th>
                <th>Ngày mượn</th>
                <th>Hạn trả</th>
                <th>Ngày trả</th>
                <th>Trạng thái</th>
                <th>Người lập</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {rows.length === 0 && (
                <tr>
                  <td colSpan={12} className="muted">
                    Không có phiếu.
                  </td>
                </tr>
              )}
              {rows.map((r) => {
                const [l, cls] = TT[str(r, "trang_thai")] ?? [str(r, "trang_thai"), "gray"];
                const quaHan = r.qua_han === true;
                return (
                  <tr key={r.id}>
                    <td className="max-w-[360px] truncate" title={str(r, "tieu_de")}>
                      <span className="muted text-xs">{str(r, "ma_ho_so") || `số tạm ${str(r, "so_tam")}`}</span>
                      <br />
                      {str(r, "tieu_de")}
                    </td>
                    <td>
                      {str(r, "hop_so")} / {str(r, "ho_so_so")}
                    </td>
                    <td>{str(r, "nguoi_muon")}</td>
                    <td>{str(r, "don_vi")}</td>
                    <td>{str(r, "dien_thoai")}</td>
                    <td className="max-w-[260px] truncate" title={str(r, "noi_dung")}>
                      {str(r, "noi_dung")}
                    </td>
                    <td>{fmtNgay(str(r, "ngay_muon"))}</td>
                    <td className={quaHan ? "text-danger" : ""}>{fmtNgay(str(r, "ngay_du_kien_tra"))}</td>
                    <td>{fmtNgay(str(r, "ngay_thuc_te_tra"))}</td>
                    <td>
                      <span className={`badge ${quaHan ? "err" : cls}`}>{quaHan ? "Quá hạn" : l}</span>
                    </td>
                    <td className="text-sm">
                      {str(r, "nguoi_lap_ten")} <span className="muted text-xs">{fmtTime(str(r, "created_at"))}</span>
                    </td>
                    <td className="whitespace-nowrap">
                      {canManage && str(r, "trang_thai") === "dang_muon" && (
                        <button className="btn sm primary" disabled={!online} onClick={() => void tra(r)}>
                          Ghi trả
                        </button>
                      )}{" "}
                      {canManage && (
                        <button className="btn sm" disabled={!online} onClick={() => mo(r)}>
                          Sửa
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
        <Modal title={form.row ? "Sửa phiếu mượn" : "Lập phiếu mượn"} onClose={() => setForm(null)} onSave={() => void luu()}>
          {err && <div className="text-danger text-sm mb-2">{err}</div>}
          {!form.row && (
            <V49j label="Hồ sơ cho mượn * (gõ tiêu đề / số tạm / mã HS)">
              <input className="inp w-full" value={timHs} onChange={(e) => void timHoSo(e.target.value)} placeholder="tìm hồ sơ…" />
              {hsGoiY.length > 0 && !hsChon && (
                <div className="border border-border-subtle rounded mt-1 max-h-48 overflow-auto">
                  {hsGoiY.map((h) => (
                    <button key={h.id} className="block w-full text-left px-2 py-1 hover:bg-surface-sunken text-sm" onClick={() => { setHsChon(h); setTimHs(str(h, "tieu_de")); }}>
                      <span className="muted">{str(h, "ma_ho_so") || `số tạm ${str(h, "so_tam")}`}</span> · {str(h, "tieu_de")}
                      {h.dang_muon === true && <span className="badge warn ml-1">đang mượn</span>}
                    </button>
                  ))}
                </div>
              )}
              {hsChon && (
                <div className="text-sm mt-1">
                  Đã chọn: <b>{str(hsChon, "tieu_de")}</b>{" "}
                  <button className="link" onClick={() => { setHsChon(null); setTimHs(""); }}>
                    đổi
                  </button>
                </div>
              )}
            </V49j>
          )}
          <V49j label="Người mượn *">
            <input className="inp w-full" value={g("nguoi_muon")} onChange={(e) => set("nguoi_muon", e.target.value)} />
          </V49j>
          <V49j label="Đơn vị công tác">
            <input className="inp w-full" value={g("don_vi")} onChange={(e) => set("don_vi", e.target.value)} />
          </V49j>
          <V49j label="Điện thoại / Email">
            <div className="flex gap-2">
              <input className="inp" value={g("dien_thoai")} onChange={(e) => set("dien_thoai", e.target.value)} placeholder="Điện thoại" />
              <input className="inp" value={g("email")} onChange={(e) => set("email", e.target.value)} placeholder="Email" />
            </div>
          </V49j>
          <V49j label="Nội dung / mục đích">
            <textarea className="inp w-full" rows={2} value={g("noi_dung")} onChange={(e) => set("noi_dung", e.target.value)} />
          </V49j>
          <V49j label="Ngày mượn / Hạn trả">
            <div className="flex gap-2">
              <input className="inp" type="date" value={g("ngay_muon")} onChange={(e) => set("ngay_muon", e.target.value)} />
              <input className="inp" type="date" value={g("ngay_du_kien_tra")} onChange={(e) => set("ngay_du_kien_tra", e.target.value)} />
            </div>
          </V49j>
          <V49j label="Ghi chú">
            <input className="inp w-full" value={g("ghi_chu")} onChange={(e) => set("ghi_chu", e.target.value)} />
          </V49j>
        </Modal>
      )}
    </>
  );
}

function fmtNgay(s: string): string {
  if (!s) return "";
  const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(s);
  return m ? `${m[3]}/${m[2]}/${m[1]}` : s;
}
