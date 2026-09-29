// Copyright (c) 2026 DigiText. All rights reserved.
// Khai thác dữ liệu chỉnh lý (bản sao, offline được): lọc như bản cũ (+ kho / giá, VV / GL / HET, trùng thừa),
// xem văn bản, và mọi kết xuất: metadata TT05 / TT02, mục lục hồ sơ 6 kiểu (Excel) + bản in, mục lục văn bản,
// nhãn hộp, sản phẩm mục lục có bìa, phụ lục hướng dẫn tra tìm, lịch sử.

import { useCallback, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41p } from "@/live/useLive";
import { V460 } from "@/state";
import { clHoSoList, clRows, str, THBQ_STATUS_LABEL, type ClHoSoFilter, type ClRow } from "@/api/chinhly";
import { errText, Field, n, Select, THBQ_BADGE, useClCatalog } from "./common";
import { inHtml, inMucLucHoSo, inMucLucVanBan, inNhanHop, inPhuLuc, inSanPham } from "./print";
import { KIEU_MLHS, xuatLichSu, xuatMucLucHoanThien, xuatMucLucVanBan, xuatPhuLuc, xuatTT02, xuatTT05, type KieuMLHS } from "./excelXuat";
import VanBanPanel from "./VanBanPanel";

export default function KhaiThacView() {
  const { session } = V460();
  const canEdit = (session?.permissions ?? []).includes("dossier.update");
  const { cat } = useClCatalog();
  const [phongId, setPhongId] = useState("");
  const [phongBanId, setPhongBanId] = useState("");
  const [mangId, setMangId] = useState("");
  const [thbqId, setThbqId] = useState("");
  const [khoId, setKhoId] = useState("");
  const [giaId, setGiaId] = useState("");
  const [status, setStatus] = useState("");
  const [trungThua, setTrungThua] = useState(false);
  const [q, setQ] = useState("");
  const [qA, setQA] = useState("");
  const [rows, setRows] = useState<ClRow[]>([]);
  const [total, setTotal] = useState(0);
  const [err, setErr] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);
  const [open, setOpen] = useState<ClRow | null>(null);
  const [kieu, setKieu] = useState<KieuMLHS | 0>(0);
  const [busy, setBusy] = useState(false);

  const phong = cat.phong.find((p) => p.id === phongId);
  const phongBans = useMemo(() => cat.phong_ban.filter((p) => str(p, "phong_id") === phongId), [cat.phong_ban, phongId]);
  const mangs = useMemo(() => cat.mang.filter((m) => str(m, "phong_ban_id") === phongBanId), [cat.mang, phongBanId]);
  const gias = useMemo(() => cat.gia.filter((g) => !khoId || str(g, "kho_id") === khoId), [cat.gia, khoId]);
  const filter: ClHoSoFilter = useMemo(
    () => ({ phongId: phongId || undefined, phongBanId: phongBanId || undefined, mangId: mangId || undefined, thbqIds: thbqId ? [thbqId] : [], khoId: khoId || undefined, giaId: giaId || undefined, status: (status || undefined) as ClHoSoFilter["status"], trungThua: trungThua ? true : undefined, q: qA || undefined, sort: "ho_so_so", limit: 200000 }),
    [phongId, phongBanId, mangId, thbqId, khoId, giaId, status, trungThua, qA],
  );
  const load = useCallback(async () => {
    try {
      const r = await clHoSoList(filter);
      setRows(r.rows);
      setTotal(r.total);
      setErr(null);
    } catch (e) {
      setErr(errText(e));
    }
  }, [filter]);
  V41p(load, { topics: ["chinhLy"], enabled: isTauri() });

  const vanBanCua = async (hs: ClRow[]): Promise<Map<string, ClRow[]>> => {
    const ids = new Set(hs.map((h) => h.id));
    const all = await clRows("van_ban", { limit: 200000, order: "so_thu_tu" });
    const m = new Map<string, ClRow[]>();
    const loai = new Map(cat.loai_van_ban.map((l) => [str(l, "ma"), str(l, "ten")]));
    for (const v of all.rows) {
      const k = str(v, "ho_so_id");
      if (!ids.has(k)) continue;
      v.ten_loai_vb = loai.get(str(v, "loai_vb")) ?? "";
      if (!m.has(k)) m.set(k, []);
      m.get(k)!.push(v);
    }
    return m;
  };
  const chay = async (ten: string, f: () => Promise<unknown>) => {
    if (rows.length === 0) {
      setErr("Không có hồ sơ khớp bộ lọc để kết xuất.");
      return;
    }
    setBusy(true);
    setErr(null);
    setMsg(null);
    try {
      await f();
      setMsg(`${ten}: ${n(rows.length)} hồ sơ.`);
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  };
  const tenPhong = str(phong, "ten") || (rows[0] ? str(rows[0], "ten_phong") : "");
  const tenPb = phongBanId ? str(phongBans.find((p) => p.id === phongBanId), "ten") : "";
  const coThoiHan = status === "GL" || status === "HET" || (thbqId !== "" && str(cat.thbq.find((t) => t.id === thbqId), "trang_thai") !== "1");

  return (
    <>
      <div className="page-head">
        <h2>Khai thác dữ liệu chỉnh lý</h2>
        <div className="sub">· tìm hồ sơ theo phông / phòng ban / mảng / THBQ / kho / giá · xuất & in mục lục, nhãn hộp, sản phẩm, phụ lục, metadata TT02 / TT05</div>
      </div>
      <div className="card mt-3.5">
        <div className="grid gap-3" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(190px, 1fr))" }}>
          <Field label="Phông">
            <Select value={phongId} onChange={(v) => { setPhongId(v); setPhongBanId(""); setMangId(""); }} rows={cat.phong} render={(r) => `${str(r, "ma_du_an")} — ${str(r, "ten")}`} empty="— Tất cả —" />
          </Field>
          <Field label="Phòng ban">
            <Select value={phongBanId} onChange={(v) => { setPhongBanId(v); setMangId(""); }} rows={phongBans} empty="— Tất cả —" />
          </Field>
          <Field label="Mảng">
            <Select value={mangId} onChange={setMangId} rows={mangs} empty="— Tất cả —" disabled={!phongBanId} />
          </Field>
          <Field label="THBQ">
            <Select value={thbqId} onChange={setThbqId} rows={cat.thbq} empty="— Tất cả —" />
          </Field>
          <Field label="Kho">
            <Select value={khoId} onChange={(v) => { setKhoId(v); setGiaId(""); }} rows={cat.kho} empty="— Tất cả —" />
          </Field>
          <Field label="Giá / kệ">
            <Select value={giaId} onChange={setGiaId} rows={gias} empty="— Tất cả —" render={(g) => `${str(g, "ten")} (hộp ${str(g, "tu_hop")}–${str(g, "den_hop")})`} />
          </Field>
          <Field label="Trạng thái">
            <select className="w-full" value={status} onChange={(e) => setStatus(e.target.value)}>
              <option value="">— Tất cả —</option>
              <option value="VV">Vĩnh viễn</option>
              <option value="GL">Giữ lại</option>
              <option value="HET">Hết THBQ</option>
            </select>
          </Field>
          <Field label="Tìm (tiêu đề, số tạm, số HS, mã HS)">
            <input className="inp w-full" value={q} onChange={(e) => setQ(e.target.value)} onKeyDown={(e) => e.key === "Enter" && setQA(q.trim())} placeholder="gõ rồi Enter" />
          </Field>
        </div>
        <div className="flex gap-2 flex-wrap mt-3 items-center">
          <label className="flex items-center gap-1 text-sm">
            <input type="checkbox" checked={trungThua} onChange={(e) => setTrungThua(e.target.checked)} /> Chỉ trùng thừa
          </label>
          <span className="muted text-sm">{n(total)} hồ sơ</span>
          <div className="flex-1" />
          <button className="btn sm" disabled={busy} onClick={() => void chay("Xuất metadata TT05", async () => xuatTT05(rows, await vanBanCua(rows), phong))}>
            Metadata TT05
          </button>
          <button className="btn sm" disabled={busy} onClick={() => void chay("Xuất metadata TT02", async () => xuatTT02(rows, await vanBanCua(rows), tenPhong))}>
            Metadata TT02
          </button>
        </div>
        <div className="flex gap-2 flex-wrap mt-2 items-center">
          <span className="muted text-sm">Mục lục hồ sơ:</span>
          <select className="max-w-72" value={kieu} onChange={(e) => setKieu(Number(e.target.value) as KieuMLHS | 0)}>
            <option value={0}>Theo số hồ sơ (mẫu kho)</option>
            {KIEU_MLHS.map(([k, l]) => (
              <option key={k} value={k}>
                {k}. {l}
              </option>
            ))}
          </select>
          <button className="btn sm" disabled={busy} onClick={() => void chay("Xuất MLHS", () => xuatMucLucHoanThien(rows, kieu, status === "HET" || trungThua, tenPhong))}>
            Xuất Excel
          </button>
          <button className="btn sm" disabled={busy} onClick={() => void chay("In MLHS", async () => inHtml("Mục lục hồ sơ", inMucLucHoSo(rows, tenPhong, tenPb)))}>
            In MLHS
          </button>
          <button className="btn sm" disabled={busy} onClick={() => void chay("In sản phẩm", async () => inHtml("Mục lục hồ sơ (sản phẩm)", inSanPham(rows, tenPhong, tenPb, str(phong, "phong_so"), str(phong, "muc_luc_so"), coThoiHan)))}>
            In sản phẩm (có bìa)
          </button>
          <span className="muted text-sm ml-2">Mục lục văn bản:</span>
          <button className="btn sm" disabled={busy} onClick={() => void chay("Xuất MLVB", async () => xuatMucLucVanBan(rows, await vanBanCua(rows)))}>
            Xuất Excel
          </button>
          <button className="btn sm" disabled={busy} onClick={() => void chay("In MLVB", async () => inHtml("Mục lục văn bản", inMucLucVanBan(rows, await vanBanCua(rows), tenPhong, tenPb)))}>
            In MLVB
          </button>
          <button className="btn sm" disabled={busy} onClick={() => void chay("In nhãn hộp", async () => inHtml("Nhãn hộp", inNhanHop(rows, tenPhong)))}>
            In nhãn hộp
          </button>
          <button className="btn sm" disabled={busy} onClick={() => void chay("In phụ lục", async () => inHtml("Hướng dẫn tra tìm mục lục", inPhuLuc(rows, tenPhong)))}>
            In phụ lục
          </button>
          <button className="btn sm" disabled={busy} onClick={() => void chay("Xuất phụ lục", () => xuatPhuLuc(rows, str(phong, "ma_du_an") || "phong"))}>
            Xuất phụ lục
          </button>
          <button
            className="btn sm"
            disabled={busy}
            onClick={() =>
              void chay("Xuất lịch sử", async () => {
                const ls = await clRows("lich_su", { limit: 200000, order: "at DESC" });
                const ids = new Set(rows.map((r) => r.id));
                const st = new Map(rows.map((r) => [r.id, str(r, "so_tam")]));
                const sub = ls.rows.filter((l) => ids.has(str(l, "ho_so_id"))).map((l) => ({ ...l, so_tam: st.get(str(l, "ho_so_id")) ?? "" }));
                await xuatLichSu(sub, str(phong, "ma_du_an") || "LichSu");
              })
            }
          >
            Xuất lịch sử
          </button>
        </div>
        {err && <div className="text-danger text-sm mt-2">{err}</div>}
        {msg && <div className="text-success-700 text-sm mt-2">{msg}</div>}
      </div>
      <div className="card mt-3.5">
        <div className="tbl-scroll">
          <table className="tbl">
            <thead>
              <tr>
                <th>Kho</th>
                <th>Giá / kệ</th>
                <th>Mã hồ sơ</th>
                <th className="num">Hộp</th>
                <th className="num">Bó</th>
                <th className="num">Tập</th>
                <th className="num">HS số</th>
                <th>Tiêu đề</th>
                <th>THBQ</th>
                <th className="num">Số tờ</th>
                <th>Bắt đầu</th>
                <th>Kết thúc</th>
                <th>T.thừa</th>
                <th className="num">VB</th>
                <th>Trạng thái</th>
              </tr>
            </thead>
            <tbody>
              {rows.slice(0, 2000).map((r) => {
                const st = str(r, "thbq_status");
                return (
                  <tr key={r.id}>
                    <td>{str(r, "ten_kho")}</td>
                    <td>{str(r, "ten_gia")}</td>
                    <td className="text-xs muted">{str(r, "ma_ho_so")}</td>
                    <td className="num">{str(r, "hop_so")}</td>
                    <td className="num">{str(r, "bo_so")}</td>
                    <td className="num">{str(r, "tap_so")}</td>
                    <td className="num">{str(r, "ho_so_so")}</td>
                    <td className="max-w-[420px]">
                      <button className="link text-left" onClick={() => setOpen(r)}>
                        {str(r, "tieu_de")}
                      </button>
                    </td>
                    <td className="text-sm">{str(r, "ten_thbq")}</td>
                    <td className="num">{str(r, "so_to")}</td>
                    <td>{str(r, "ngay_bat_dau")}</td>
                    <td>{str(r, "ngay_ket_thuc")}</td>
                    <td>{r.trung_thua === true ? "✓" : ""}</td>
                    <td className="num">{str(r, "so_van_ban")}</td>
                    <td>{st && <span className={`badge ${THBQ_BADGE[st] ?? "gray"}`}>{THBQ_STATUS_LABEL[st] ?? st}</span>}</td>
                  </tr>
                );
              })}
              {rows.length > 2000 && (
                <tr>
                  <td colSpan={15} className="muted">
                    Chỉ hiện 2.000 dòng đầu; kết xuất vẫn lấy đủ {n(rows.length)} hồ sơ.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>
      {open && <VanBanPanel cat={cat} hoSo={open} canEdit={canEdit} onClose={() => setOpen(null)} />}
    </>
  );
}
