// Copyright (c) 2026 DigiText. All rights reserved.
// Thống kê chỉnh lý từ máy chủ: tổng kết chỉnh lý (theo phòng ban), nhân viên (từ lịch sử), phần mềm (theo phông),
// mượn trả. Xuất Excel. Cần máy chủ.

import { useCallback, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41p } from "@/live/useLive";
import { V460 } from "@/state";
import { clRemote, qs, str, num, type ClRow } from "@/api/chinhly";
import { errText, fmtTime, n, Select, useClCatalog, useClSync } from "./common";
import { writeXlsx } from "./excel";

type Kind = "chinh-ly" | "nhan-vien" | "phan-mem" | "muon-tra";
interface Col {
  k: string;
  l: string;
  num?: boolean;
  fmt?: (r: ClRow) => string;
}
const COLS: Record<Kind, Col[]> = {
  "chinh-ly": [
    { k: "ten_phong", l: "Phông" },
    { k: "ten_phong_ban", l: "Phòng ban" },
    { k: "hop_vv", l: "Hộp VV", num: true },
    { k: "hs_vv", l: "Hồ sơ VV", num: true },
    { k: "hop_co_thoi_han", l: "Hộp có thời hạn", num: true },
    { k: "hs_co_thoi_han", l: "HS có thời hạn", num: true },
    { k: "bo", l: "Bó loại", num: true },
    { k: "tap", l: "Tập loại", num: true },
    { k: "hs_het", l: "HS hết THBQ", num: true },
    { k: "hs_trung_thua", l: "HS trùng thừa", num: true },
    { k: "hs_chua_danh_so", l: "HS chưa đánh số", num: true },
    { k: "hs_tong", l: "Tổng HS", num: true },
    { k: "vb_tong", l: "Tổng VB", num: true },
  ],
  "nhan-vien": [
    { k: "actor_name", l: "Nhân viên" },
    { k: "username", l: "Tài khoản" },
    { k: "so_ho_so", l: "Số hồ sơ", num: true },
    { k: "so_van_ban", l: "Số văn bản", num: true },
    { k: "truong_tong", l: "Số trường thực hiện", num: true },
    { k: "truong_moi", l: "Trường thêm mới", num: true },
    { k: "truong_sua", l: "Trường đã sửa", num: true },
    { k: "ngay_lam_viec", l: "Ngày làm việc", num: true },
    { k: "dau", l: "Từ", fmt: (r) => fmtTime(str(r, "dau")) },
    { k: "cuoi", l: "Đến", fmt: (r) => fmtTime(str(r, "cuoi")) },
  ],
  "phan-mem": [
    { k: "ten_phong", l: "Phông" },
    { k: "khoi_tao", l: "Thời gian khởi tạo", fmt: (r) => fmtTime(str(r, "khoi_tao")) },
    { k: "so_phong_ban", l: "Phòng ban", num: true },
    { k: "hs_tao", l: "Hồ sơ tạo", num: true },
    { k: "vb_tong", l: "Văn bản", num: true },
    { k: "hop_vv", l: "Hộp VV", num: true },
    { k: "hs_vv", l: "HS VV", num: true },
    { k: "hop_co_thoi_han", l: "Hộp có thời hạn", num: true },
    { k: "hs_co_thoi_han", l: "HS có thời hạn", num: true },
    { k: "bo", l: "Bó loại", num: true },
    { k: "tap", l: "Tập loại", num: true },
  ],
  "muon-tra": [],
};
const TEN: Record<Kind, string> = { "chinh-ly": "Tổng kết chỉnh lý", "nhan-vien": "Tổng kết nhân viên", "phan-mem": "Tổng kết phần mềm", "muon-tra": "Mượn trả" };

export default function ThongKeView({ kind }: { kind: Kind }) {
  const { session } = V460();
  const projectId = session?.projectId ?? "";
  const { cat } = useClCatalog();
  const sync = useClSync();
  const online = sync?.phase !== "offline";
  const [phongId, setPhongId] = useState("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [rows, setRows] = useState<ClRow[]>([]);
  const [mt, setMt] = useState<Record<string, number> | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const load = useCallback(async () => {
    if (!online || !projectId) return;
    try {
      const path = `/cl/thong-ke/${kind}${qs({ projectId, phongId, from: from ? from.split("-").reverse().join("/") : "", to: to ? to.split("-").reverse().join("/") : "" })}`;
      if (kind === "muon-tra") setMt(await clRemote<Record<string, number>>("GET", path));
      else setRows(await clRemote<ClRow[]>("GET", path));
      setErr(null);
    } catch (e) {
      setErr(errText(e));
    }
  }, [online, projectId, kind, phongId, from, to]);
  V41p(load, { topics: ["chinhLy"], enabled: isTauri() });
  const cols = COLS[kind];
  const tong = (k: string) => rows.reduce((s, r) => s + (num(r, k) ?? 0), 0);
  const xuat = async () => {
    const aoa = [[`BÁO CÁO ${TEN[kind].toUpperCase()}`], [], cols.map((c) => c.l), ...rows.map((r) => cols.map((c) => (c.fmt ? c.fmt(r) : c.num ? (num(r, c.k) ?? 0) : str(r, c.k)))), ["TỔNG CỘNG", ...cols.slice(1).map((c) => (c.num ? tong(c.k) : ""))]];
    await writeXlsx([{ name: TEN[kind].slice(0, 31), aoa, merges: [`A1:${String.fromCharCode(64 + cols.length)}1`] }], `${kind}_${new Date().toISOString().slice(0, 10)}.xlsx`).catch((e) => setErr(errText(e)));
  };
  return (
    <>
      <div className="page-head">
        <h2>{TEN[kind]}</h2>
        <div className="sub">· số liệu từ máy chủ (dữ liệu chỉnh lý đồng bộ) {kind === "nhan-vien" && "· tính từ lịch sử sửa từng trường"}</div>
      </div>
      {!online && <div className="card mt-3.5 text-warning-700">Đang ngoại tuyến — thống kê cần máy chủ.</div>}
      <div className="card mt-3.5">
        <div className="flex gap-2 items-center flex-wrap">
          {kind !== "phan-mem" && kind !== "muon-tra" && <Select value={phongId} onChange={setPhongId} rows={cat.phong} render={(r) => `${str(r, "ma_du_an")} — ${str(r, "ten")}`} empty="— Tất cả phông —" />}
          {kind !== "chinh-ly" && (
            <>
              <label className="muted text-sm">
                Từ ngày <input className="inp ml-1" type="date" value={from} onChange={(e) => setFrom(e.target.value)} />
              </label>
              <label className="muted text-sm">
                Đến ngày <input className="inp ml-1" type="date" value={to} onChange={(e) => setTo(e.target.value)} />
              </label>
            </>
          )}
          <div className="flex-1" />
          {kind !== "muon-tra" && (
            <button className="btn" disabled={rows.length === 0} onClick={() => void xuat()}>
              Xuất Excel
            </button>
          )}
        </div>
        {err && <div className="text-danger text-sm mt-2">{err}</div>}
        {kind === "muon-tra" && mt && (
          <div className="grid gap-3 mt-3" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(160px, 1fr))" }}>
            {[
              ["Đang mượn", mt.dangMuon],
              ["Quá hạn", mt.quaHan],
              ["Đã trả (trong kỳ)", mt.daTra],
              ["Phiếu lập (trong kỳ)", mt.lapPhieu],
            ].map(([l, v]) => (
              <div key={String(l)} className="card">
                <div className="muted text-sm">{l}</div>
                <div className="text-2xl font-semibold">{n(Number(v) || 0)}</div>
              </div>
            ))}
          </div>
        )}
        {kind !== "muon-tra" && (
          <div className="tbl-scroll mt-3">
            <table className="tbl">
              <thead>
                <tr>
                  <th className="num">STT</th>
                  {cols.map((c) => (
                    <th key={c.k} className={c.num ? "num" : ""}>
                      {c.l}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {rows.map((r, i) => (
                  <tr key={i}>
                    <td className="num">{i + 1}</td>
                    {cols.map((c) => (
                      <td key={c.k} className={c.num ? "num" : ""}>
                        {c.fmt ? c.fmt(r) : c.num ? n(num(r, c.k) ?? 0) : str(r, c.k)}
                      </td>
                    ))}
                  </tr>
                ))}
                {rows.length > 0 && (
                  <tr className="font-semibold">
                    <td />
                    <td>Tổng cộng</td>
                    {cols.slice(1).map((c) => (
                      <td key={c.k} className={c.num ? "num" : ""}>
                        {c.num ? n(tong(c.k)) : ""}
                      </td>
                    ))}
                  </tr>
                )}
                {rows.length === 0 && (
                  <tr>
                    <td colSpan={cols.length + 1} className="muted">
                      Chưa có số liệu.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </>
  );
}
