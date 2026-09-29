// Copyright (c) 2026 DigiText. All rights reserved.
// Hệ thống hoá hồ sơ (chỉ online): lọc tập hồ sơ trong phông → chọn 1 trong 6 thứ tự → Xem trước (số hồ sơ
// dự kiến, mã hồ sơ, cảnh báo) → Áp dụng (đánh số chính thức) → Xuất mục lục tạm → điền hộp / bó / tập
// → Nhập lại (gán hộp, tự tạo hộp theo phòng ban) · gán lẻ cho các dòng đã chọn.

import { useMemo, useRef, useState } from "react";
import { V460 } from "@/state";
import { clRemote, str, THBQ_STATUS_LABEL } from "@/api/chinhly";
import { errText, Field, n, Select, THBQ_BADGE, useClCatalog, useClSync } from "./common";
import { docMucLucTam, xuatMucLucTam, type PreviewRow } from "./excel";

const ORDERS: [number, string][] = [
  [1, "Mặt hoạt động → Thời gian"],
  [2, "Thời gian → Mặt hoạt động"],
  [3, "THBQ tăng dần → Mặt hoạt động → Thời gian"],
  [4, "THBQ giảm dần → Mặt hoạt động → Thời gian"],
  [5, "THBQ tăng dần → Thời gian → Mặt hoạt động"],
  [6, "THBQ giảm dần → Thời gian → Mặt hoạt động"],
];

interface Preview {
  snapshotVer: number;
  order: number;
  fileType: number;
  rows: PreviewRow[];
}

export default function HeThongHoaView() {
  const { session } = V460();
  const projectId = session?.projectId ?? "";
  const canDo = (session?.permissions ?? []).includes("dossier.systematize");
  const { cat } = useClCatalog();
  const sync = useClSync();
  const online = sync?.phase !== "offline" && sync?.phase !== "error";

  const [phongId, setPhongId] = useState("");
  const [phongBanId, setPhongBanId] = useState("");
  const [mangId, setMangId] = useState("");
  const [thbqIds, setThbqIds] = useState<string[]>([]);
  const [status, setStatus] = useState("");
  const [trungThua, setTrungThua] = useState<"" | "1" | "0">("");
  const [chuaDanhSo, setChuaDanhSo] = useState(true);
  const [order, setOrder] = useState(1);
  const [fileType, setFileType] = useState<1 | 2>(1);
  const [startAt, setStartAt] = useState(1);

  const [preview, setPreview] = useState<Preview | null>(null);
  const [applied, setApplied] = useState(false);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [importRes, setImportRes] = useState<{ applied: number; hopsCreated: number; boTapCreated: number; errors: { row: number; message: string }[] } | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  const phong = cat.phong.find((p) => p.id === phongId);
  const phongBans = useMemo(() => cat.phong_ban.filter((p) => str(p, "phong_id") === phongId), [cat.phong_ban, phongId]);
  const mangs = useMemo(() => cat.mang.filter((m) => str(m, "phong_ban_id") === phongBanId), [cat.mang, phongBanId]);

  const body = () => ({
    projectId,
    phongId,
    filter: { phongBanId: phongBanId || undefined, mangIds: mangId ? [mangId] : [], thbqIds, status: status || undefined, trungThua: trungThua === "" ? undefined : trungThua === "1", chuaDanhSo },
    sortOrder: order,
    fileType,
    startAt,
  });

  const xemTruoc = async () => {
    setBusy(true);
    setErr(null);
    setMsg(null);
    setApplied(false);
    try {
      setPreview(await clRemote<Preview>("POST", "/cl/he-thong-hoa/preview", body()));
      setSel(new Set());
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  };
  const apDung = async () => {
    if (!preview) return;
    if (!window.confirm(`Đánh số chính thức ${n(preview.rows.length)} hồ sơ từ ${startAt} theo thứ tự “${ORDERS.find((o) => o[0] === order)?.[1]}”? Số cũ (nếu có) sẽ bị thay.`)) return;
    setBusy(true);
    setErr(null);
    try {
      const p = await clRemote<Preview>("POST", "/cl/he-thong-hoa/apply", { ...body(), snapshotVer: preview.snapshotVer });
      setPreview(p);
      setApplied(true);
      setMsg(`Đã đánh số ${n(p.rows.length)} hồ sơ. Xuất mục lục tạm, điền ${fileType === 1 ? "Hộp số" : "Bó số / Tập số"} rồi nhập lại.`);
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  };
  const xuat = async () => {
    if (!preview) return;
    try {
      await xuatMucLucTam(preview.rows, fileType, phongId, str(phong, "ten"), ORDERS.find((o) => o[0] === order)?.[1] ?? "");
    } catch (e) {
      setErr(errText(e));
    }
  };
  const nhap = async (f: File) => {
    setBusy(true);
    setErr(null);
    setImportRes(null);
    try {
      const d = await docMucLucTam(f);
      if (d.rows.length === 0) {
        setErr(d.errors[0]?.message ?? "Tệp không có dòng dữ liệu");
        setImportRes({ applied: 0, hopsCreated: 0, boTapCreated: 0, errors: d.errors });
        return;
      }
      const r = await clRemote<{ applied: number; hopsCreated: number; boTapCreated: number; errors: { row: number; hoSoId: string; message: string }[] }>("POST", "/cl/he-thong-hoa/boxing", { projectId, fileType: d.fileType, rows: d.rows });
      setImportRes({ ...r, errors: [...d.errors, ...r.errors.map((e) => ({ row: e.row + 2, message: e.message }))] });
      setMsg(`Đã gán ${n(r.applied)} hồ sơ · tạo mới ${n(r.hopsCreated)} hộp, ${n(r.boTapCreated)} bó/tập.`);
      if (preview) void xemTruoc();
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  };
  const ganLe = async () => {
    if (sel.size === 0) return;
    const v = window.prompt(fileType === 1 ? `Số hộp cho ${sel.size} hồ sơ đã chọn:` : `Bó số (và tập số, cách nhau dấu /) cho ${sel.size} hồ sơ:`);
    if (v === null) return;
    const t = v.trim();
    setBusy(true);
    setErr(null);
    try {
      const rows = [...sel].map((id) => {
        if (fileType === 1) return { hoSoId: id, hopSo: t };
        const [b, tp] = t.split("/").map((x) => Number.parseInt(x.trim(), 10));
        return { hoSoId: id, boSo: Number.isNaN(b) ? undefined : b, tapSo: Number.isNaN(tp) ? undefined : tp, clear: t === "" };
      });
      const r = await clRemote<{ applied: number; errors: { message: string }[] }>("POST", "/cl/he-thong-hoa/boxing", { projectId, fileType, rows });
      setMsg(`Đã gán ${n(r.applied)} hồ sơ${r.errors.length ? ` · ${r.errors.length} lỗi: ${r.errors[0].message}` : ""}.`);
      void xemTruoc();
    } catch (e) {
      setErr(errText(e));
    } finally {
      setBusy(false);
    }
  };

  const khoa = !online || !canDo;
  const canhBao = preview?.rows.filter((r) => r.warnings.length > 0).length ?? 0;

  return (
    <>
      <div className="page-head">
        <h2>Hệ thống hoá hồ sơ</h2>
        <div className="sub">· đánh số hồ sơ chính thức, xuất mục lục tạm, gán hộp / bó / tập · cần kết nối máy chủ</div>
      </div>
      {!canDo && <div className="card mt-3.5 text-warning-700">Tài khoản chưa có quyền hệ thống hoá (dossier.systematize).</div>}
      {!online && <div className="card mt-3.5 text-warning-700">Đang ngoại tuyến — hệ thống hoá thay đổi số hồ sơ của cả tập nên chỉ làm khi có máy chủ.</div>}
      <div className="card mt-3.5">
        <div className="grid gap-3" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))" }}>
          <Field label="Phông *">
            <Select value={phongId} onChange={(v) => { setPhongId(v); setPhongBanId(""); setMangId(""); setPreview(null); }} rows={cat.phong} render={(r) => `${str(r, "ma_du_an")} — ${str(r, "ten")}`} />
          </Field>
          <Field label="Phòng ban">
            <Select value={phongBanId} onChange={(v) => { setPhongBanId(v); setMangId(""); }} rows={phongBans} empty="— Cả phông —" />
          </Field>
          <Field label="Mảng công việc">
            <Select value={mangId} onChange={setMangId} rows={mangs} empty="— Cả phòng ban —" disabled={!phongBanId} />
          </Field>
          <Field label="Trạng thái THBQ">
            <select className="w-full" value={status} onChange={(e) => setStatus(e.target.value)}>
              <option value="">— Tất cả —</option>
              <option value="VV">Vĩnh viễn</option>
              <option value="GL">Giữ lại</option>
              <option value="HET">Hết THBQ</option>
            </select>
          </Field>
          <Field label="Trùng thừa">
            <select className="w-full" value={trungThua} onChange={(e) => setTrungThua(e.target.value as typeof trungThua)}>
              <option value="">— Tất cả —</option>
              <option value="0">Không trùng thừa</option>
              <option value="1">Chỉ trùng thừa</option>
            </select>
          </Field>
          <Field label="Thứ tự đánh số">
            <select className="w-full" value={order} onChange={(e) => setOrder(Number(e.target.value))}>
              {ORDERS.map(([k, l]) => (
                <option key={k} value={k}>
                  {k}. {l}
                </option>
              ))}
            </select>
          </Field>
          <Field label="Loại tệp">
            <select className="w-full" value={fileType} onChange={(e) => setFileType(Number(e.target.value) as 1 | 2)}>
              <option value={1}>Hộp số / Hồ sơ số (hồ sơ chuẩn)</option>
              <option value={2}>Bó số / Tập số (hết hạn, trùng thừa)</option>
            </select>
          </Field>
          <Field label="Bắt đầu từ số">
            <input className="inp w-full" type="number" min={1} value={startAt} onChange={(e) => setStartAt(Math.max(1, Number(e.target.value) || 1))} />
          </Field>
        </div>
        <div className="flex gap-2 flex-wrap mt-2 items-center text-sm">
          <span className="muted">THBQ:</span>
          {cat.thbq.map((t) => (
            <label key={t.id} className={`flex items-center gap-1 border rounded px-2 py-0.5 ${thbqIds.includes(t.id) ? "border-brand-500" : "border-border-subtle"}`}>
              <input type="checkbox" checked={thbqIds.includes(t.id)} onChange={() => setThbqIds((v) => (v.includes(t.id) ? v.filter((x) => x !== t.id) : [...v, t.id]))} /> {str(t, "ten")}
            </label>
          ))}
          <label className="flex items-center gap-1 border border-border-subtle rounded px-2 py-0.5 ml-2">
            <input type="checkbox" checked={chuaDanhSo} onChange={(e) => setChuaDanhSo(e.target.checked)} /> Chỉ hồ sơ chưa đánh số
          </label>
        </div>
        <div className="flex gap-2 flex-wrap mt-3 items-center">
          <button className="btn primary" disabled={khoa || busy || !phongId} onClick={() => void xemTruoc()}>
            1. Xem trước
          </button>
          <button className="btn primary" disabled={khoa || busy || !preview || preview.rows.length === 0 || applied} onClick={() => void apDung()}>
            2. Áp dụng đánh số
          </button>
          <button className="btn" disabled={!preview || preview.rows.length === 0} onClick={() => void xuat()}>
            3. Xuất mục lục tạm (Excel)
          </button>
          <button className="btn" disabled={khoa || busy} onClick={() => fileRef.current?.click()}>
            4. Nhập tệp đã điền {fileType === 1 ? "hộp" : "bó/tập"}
          </button>
          <input ref={fileRef} type="file" accept=".xlsx,.xls" hidden onChange={(e) => { const f = e.target.files?.[0]; e.target.value = ""; if (f) void nhap(f); }} />
          <button className="btn" disabled={khoa || busy || sel.size === 0} onClick={() => void ganLe()}>
            Gán {fileType === 1 ? "hộp" : "bó/tập"} cho {sel.size} dòng chọn
          </button>
          <div className="flex-1" />
          {preview && (
            <span className="muted text-sm">
              {n(preview.rows.length)} hồ sơ{canhBao > 0 && <span className="text-warning-700"> · {n(canhBao)} cảnh báo</span>}
            </span>
          )}
        </div>
        {err && <div className="text-danger text-sm mt-2">{err}</div>}
        {msg && <div className="text-success-700 text-sm mt-2">{msg}</div>}
        {importRes && importRes.errors.length > 0 && (
          <div className="mt-2 text-sm">
            <b className="text-danger">{importRes.errors.length} dòng lỗi:</b>
            <ul className="m-0 pl-5">
              {importRes.errors.slice(0, 30).map((e, i) => (
                <li key={i}>
                  Dòng {e.row}: {e.message}
                </li>
              ))}
            </ul>
          </div>
        )}
      </div>

      {preview && (
        <div className="card mt-3.5">
          <div className="tbl-scroll">
            <table className="tbl">
              <thead>
                <tr>
                  <th style={{ width: 28 }}>
                    <input type="checkbox" checked={preview.rows.length > 0 && preview.rows.every((r) => sel.has(r.hoSoId))} onChange={(e) => setSel(e.target.checked ? new Set(preview.rows.map((r) => r.hoSoId)) : new Set())} />
                  </th>
                  <th className="num">{applied ? "HS số" : "HS số (dự kiến)"}</th>
                  <th className="num">Số cũ</th>
                  <th>{fileType === 1 ? "Hộp" : "Bó / Tập"}</th>
                  <th>Mã hồ sơ mới</th>
                  <th>Số tạm</th>
                  <th>Tiêu đề</th>
                  <th>Phòng ban</th>
                  <th>Mảng</th>
                  <th>THBQ</th>
                  <th>Trạng thái</th>
                  <th>KT</th>
                  <th className="num">Số tờ</th>
                  <th>Cảnh báo</th>
                </tr>
              </thead>
              <tbody>
                {preview.rows.map((r) => (
                  <tr key={r.hoSoId} className={r.warnings.length ? "bg-warning-50" : ""}>
                    <td>
                      <input type="checkbox" checked={sel.has(r.hoSoId)} onChange={(e) => setSel((s) => { const x = new Set(s); if (e.target.checked) x.add(r.hoSoId); else x.delete(r.hoSoId); return x; })} />
                    </td>
                    <td className="num">
                      <b>{r.stt}</b>
                    </td>
                    <td className="num muted">{r.hoSoSoCu ?? ""}</td>
                    <td>{fileType === 1 ? r.hopSo ?? "" : [r.boSo, r.tapSo].filter((x) => x != null).join(" / ")}</td>
                    <td className="text-xs">{r.maHoSoMoi ?? <span className="muted">—</span>}</td>
                    <td>{r.soTam}</td>
                    <td className="max-w-[420px] truncate" title={r.tieuDe}>
                      {r.tieuDe}
                    </td>
                    <td className="text-sm">{r.tenPhongBan ?? ""}</td>
                    <td className="text-sm">{r.tenMang ?? ""}</td>
                    <td className="text-sm">{r.tenThbq ?? ""}</td>
                    <td>{r.thbqStatus && <span className={`badge ${THBQ_BADGE[r.thbqStatus] ?? "gray"}`}>{THBQ_STATUS_LABEL[r.thbqStatus] ?? r.thbqStatus}</span>}</td>
                    <td>{r.ngayKetThuc ?? ""}</td>
                    <td className="num">{r.soTo ?? ""}</td>
                    <td className="text-warning-700 text-xs">{r.warnings.join("; ")}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}
    </>
  );
}
