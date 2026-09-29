// Copyright (c) 2026 DigiText. All rights reserved.
// Lập hồ sơ điện tử (chỉnh lý, bản sao đồng bộ): lọc như bản cũ (phông, phòng ban, mảng, THBQ đa chọn,
// vĩnh viễn / giữ lại / hết THBQ, trùng thừa, tìm), lưới 50 dòng, thêm / sửa / xoá, văn bản của hồ sơ.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41p } from "@/live/useLive";
import { V460 } from "@/state";
import { clDelete, clHoSoList, clRemote, str, THBQ_STATUS_LABEL, type ClHoSoFilter, type ClRow } from "@/api/chinhly";
import { errText, Field, n, Select, SyncDot, THBQ_BADGE, useClCatalog, useClSync, syncLabel } from "./common";
import HoSoForm from "./HoSoForm";
import VanBanPanel from "./VanBanPanel";
import { docMauLapHoSo, xuatMauLapHoSo } from "./excel";

const PAGE = 50;

export default function LapHoSoView() {
  const { session } = V460();
  const perms = session?.permissions ?? [];
  const canCreate = perms.includes("dossier.create");
  const canEdit = perms.includes("dossier.update");
  const canDelete = perms.includes("dossier.delete");
  const { cat, error: catErr } = useClCatalog();
  const sync = useClSync();

  const [phongId, setPhongId] = useState("");
  const [phongBanId, setPhongBanId] = useState("");
  const [mangId, setMangId] = useState("");
  const [thbqIds, setThbqIds] = useState<string[]>([]);
  const [coThoiHan, setCoThoiHan] = useState(false);
  const [status, setStatus] = useState<"" | "VV" | "GL" | "HET">("");
  const [trungThua, setTrungThua] = useState(false);
  const [q, setQ] = useState("");
  const [qApplied, setQApplied] = useState("");
  const [page, setPage] = useState(0);
  const [sort, setSort] = useState<ClHoSoFilter["sort"]>("so_tam");
  const [desc, setDesc] = useState(false);

  const [rows, setRows] = useState<ClRow[]>([]);
  const [total, setTotal] = useState(0);
  const [err, setErr] = useState<string | null>(null);
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [form, setForm] = useState<{ row: ClRow | null } | null>(null);
  const [open, setOpen] = useState<ClRow | null>(null);

  useEffect(() => {
    if (!phongId && cat.phong.length > 0) setPhongId(str(cat.phong[0], "id"));
  }, [cat.phong, phongId]);

  const phongBans = useMemo(() => cat.phong_ban.filter((p) => str(p, "phong_id") === phongId), [cat.phong_ban, phongId]);
  const mangs = useMemo(() => cat.mang.filter((m) => str(m, "phong_ban_id") === phongBanId), [cat.mang, phongBanId]);

  const filter: ClHoSoFilter = useMemo(
    () => ({
      phongId: phongId || undefined,
      phongBanId: phongBanId || undefined,
      mangId: mangId || undefined,
      thbqIds,
      thbqCoThoiHan: coThoiHan,
      status: status || undefined,
      trungThua: trungThua ? true : undefined,
      q: qApplied || undefined,
      sort,
      desc,
      limit: PAGE,
      offset: page * PAGE,
    }),
    [phongId, phongBanId, mangId, thbqIds, coThoiHan, status, trungThua, qApplied, sort, desc, page],
  );

  const load = useCallback(async () => {
    try {
      const r = await clHoSoList(filter);
      setRows(r.rows);
      setTotal(r.total);
      setErr(null);
      if (open) {
        const again = r.rows.find((x) => x.id === open.id);
        if (again) setOpen(again);
      }
    } catch (e) {
      setErr(errText(e));
    }
  }, [filter, open]);
  const { reload } = V41p(load, { topics: ["chinhLy"], enabled: isTauri() });

  const xoaChon = async () => {
    if (sel.size === 0) return;
    if (!window.confirm(`Xoá ${sel.size} hồ sơ đã chọn (kèm văn bản bên trong)? Có thể khôi phục trên máy chủ.`)) return;
    try {
      for (const id of sel) await clDelete("ho_so", id);
      setSel(new Set());
      reload();
    } catch (e) {
      setErr(errText(e));
    }
  };

  const pages = Math.max(1, Math.ceil(total / PAGE));
  const sl = syncLabel(sync);
  const [importMsg, setImportMsg] = useState<string | null>(null);
  const fileRef = useRef<HTMLInputElement>(null);
  const phong = cat.phong.find((p) => p.id === phongId);
  const nhapExcel = async (f: File) => {
    setImportMsg(null);
    try {
      const d = await docMauLapHoSo(f);
      if (d.maPhong && phong && d.maPhong.toLowerCase() !== str(phong, "ma_du_an").toLowerCase()) {
        if (!window.confirm(`Ô A1 của tệp ghi mã phông “${d.maPhong}” khác phông đang chọn (${str(phong, "ma_du_an")}). Vẫn nhập vào phông đang chọn?`)) return;
      }
      if (d.rows.length === 0) {
        setImportMsg(d.errors[0]?.message ?? "Tệp không có dòng dữ liệu (dữ liệu từ dòng 5).");
        return;
      }
      const r = await clRemote<{ created: number; updated: number; skipped: number; errors: { row: number; field?: string; message: string }[] }>("POST", "/cl/ho-so/import", {
        projectId: session?.projectId,
        phongId,
        mode: "upsert",
        rows: d.rows,
      });
      const errs = [...d.errors, ...r.errors.map((e) => ({ row: e.row + 4, message: `${e.field ? e.field + ": " : ""}${e.message}` }))];
      setImportMsg(`Đã thêm ${n(r.created)}, cập nhật ${n(r.updated)}, bỏ qua ${n(r.skipped)}${errs.length ? ` · ${errs.length} dòng lỗi: ${errs.slice(0, 5).map((e) => `dòng ${e.row} ${e.message}`).join("; ")}` : ""}`);
      reload();
    } catch (e) {
      setImportMsg(errText(e));
    }
  };
  const toggleThbq = (id: string) => setThbqIds((v) => (v.includes(id) ? v.filter((x) => x !== id) : [...v, id]));

  const th = (label: string, key: ClHoSoFilter["sort"], cls = "") => (
    <th
      className={`${cls} cursor-pointer select-none`}
      onClick={() => {
        if (sort === key) setDesc((d) => !d);
        else {
          setSort(key);
          setDesc(false);
        }
        setPage(0);
      }}
      title="Bấm để sắp xếp"
    >
      {label}
      {sort === key ? (desc ? " ▾" : " ▴") : ""}
    </th>
  );

  return (
    <>
      <div className="page-head">
        <h2>Lập hồ sơ điện tử</h2>
        <div className="sub">
          · hồ sơ tạm trong dự án {session?.projectName ?? ""} · làm được khi mất mạng, tự đồng bộ ·{" "}
          <span className={sl.dot === "ok" ? "text-success-700" : sl.dot === "err" ? "text-danger" : "text-warning-700"} title={sl.title}>
            {sl.text}
          </span>
        </div>
      </div>
      {catErr && <div className="card mt-3.5 text-danger">{catErr}</div>}
      {sync?.phase === "offline" && <div className="card mt-3.5 text-warning-700">Đang ngoại tuyến — thay đổi được giữ ở máy và sẽ tự đẩy lên máy chủ khi có mạng.</div>}
      {cat.phong.length === 0 && !catErr && (
        <div className="card mt-3.5 muted">
          Chưa có phông nào trong dự án. Quản trị tạo phông ở Quản lý danh mục (khi có kết nối máy chủ), hoặc chờ đồng bộ kéo danh mục về.
        </div>
      )}
      <div className="card mt-3.5">
        <div className="grid gap-3" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))" }}>
          <Field label="Phông">
            <Select value={phongId} onChange={(v) => { setPhongId(v); setPhongBanId(""); setMangId(""); setPage(0); }} rows={cat.phong} render={(r) => `${str(r, "ma_du_an")} — ${str(r, "ten")}`} empty="— Tất cả phông —" />
          </Field>
          <Field label="Phòng ban">
            <Select value={phongBanId} onChange={(v) => { setPhongBanId(v); setMangId(""); setPage(0); }} rows={phongBans} empty="— Tất cả —" />
          </Field>
          <Field label="Mảng công việc">
            <Select value={mangId} onChange={(v) => { setMangId(v); setPage(0); }} rows={mangs} empty="— Tất cả —" disabled={!phongBanId} />
          </Field>
          <Field label="Trạng thái THBQ">
            <select className="w-full" value={status} onChange={(e) => { setStatus(e.target.value as typeof status); setPage(0); }}>
              <option value="">— Tất cả —</option>
              <option value="VV">Vĩnh viễn</option>
              <option value="GL">Giữ lại</option>
              <option value="HET">Hết thời hạn bảo quản</option>
            </select>
          </Field>
          <Field label="Tìm (số tạm, tiêu đề, số HS, mã HS)">
            <input
              className="inp w-full"
              value={q}
              onChange={(e) => setQ(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  setQApplied(q.trim());
                  setPage(0);
                }
              }}
              placeholder="gõ rồi Enter"
            />
          </Field>
        </div>
        <div className="flex gap-2 flex-wrap mt-2 items-center text-sm">
          <span className="muted">THBQ:</span>
          <label className="flex items-center gap-1 border border-border-subtle rounded px-2 py-0.5">
            <input type="checkbox" checked={coThoiHan} onChange={(e) => { setCoThoiHan(e.target.checked); setPage(0); }} /> Từ 5 tới 70 năm (có thời hạn)
          </label>
          {cat.thbq.map((t) => (
            <label key={t.id} className={`flex items-center gap-1 border rounded px-2 py-0.5 ${thbqIds.includes(t.id) ? "border-brand-500" : "border-border-subtle"}`}>
              <input type="checkbox" checked={thbqIds.includes(t.id)} onChange={() => { toggleThbq(t.id); setPage(0); }} /> {str(t, "ten")}
            </label>
          ))}
          <label className="flex items-center gap-1 border border-border-subtle rounded px-2 py-0.5 ml-2">
            <input type="checkbox" checked={trungThua} onChange={(e) => { setTrungThua(e.target.checked); setPage(0); }} /> Chỉ hồ sơ trùng thừa
          </label>
        </div>
        <div className="flex gap-2 flex-wrap mt-3 items-center">
          {canCreate && (
            <button className="btn primary" disabled={!phongId} onClick={() => setForm({ row: null })} title={phongId ? "" : "Chọn phông trước"}>
              + Thêm hồ sơ
            </button>
          )}
          {canDelete && (
            <button className="btn" disabled={sel.size === 0} onClick={() => void xoaChon()}>
              Xoá ({sel.size})
            </button>
          )}
          <button className="btn" disabled={!phong} onClick={() => void xuatMauLapHoSo(str(phong, "ma_du_an"), cat.thbq).catch((e) => setImportMsg(errText(e)))} title="Tải mẫu Excel lập hồ sơ (bố cục như bản cũ)">
            Mẫu H.Sơ
          </button>
          {canCreate && (
            <>
              <button className="btn" disabled={!phongId || sync?.phase === "offline"} onClick={() => fileRef.current?.click()} title="Nhập hồ sơ từ Excel theo mẫu (cần máy chủ)">
                Nhập Excel
              </button>
              <input ref={fileRef} type="file" accept=".xlsx,.xls" hidden onChange={(e) => { const f = e.target.files?.[0]; e.target.value = ""; if (f) void nhapExcel(f); }} />
            </>
          )}
          <div className="flex-1" />
          <span className="muted text-sm">
            {n(total)} hồ sơ · trang {page + 1}/{pages}
          </span>
          <button className="btn sm" disabled={page === 0} onClick={() => setPage((p) => p - 1)}>
            ‹
          </button>
          <input
            className="inp w-16 text-center"
            type="number"
            min={1}
            max={pages}
            value={page + 1}
            onChange={(e) => setPage(Math.min(pages - 1, Math.max(0, Number(e.target.value) - 1)))}
            title="Tới trang"
          />
          <button className="btn sm" disabled={page >= pages - 1} onClick={() => setPage((p) => p + 1)}>
            ›
          </button>
        </div>
        {err && <div className="text-danger text-sm mt-2">{err}</div>}
        {importMsg && <div className="text-sm mt-2">{importMsg}</div>}
      </div>

      {form && (
        <HoSoForm
          cat={cat}
          phongId={phongId}
          row={form.row}
          onClose={() => setForm(null)}
          onSaved={(_id, keepOpen) => {
            if (!keepOpen) setForm(null);
            reload();
          }}
        />
      )}

      <div className="card mt-3.5">
        <div className="tbl-scroll">
          <table className="tbl">
            <thead>
              <tr>
                <th style={{ width: 28 }}>
                  <input
                    type="checkbox"
                    checked={rows.length > 0 && rows.every((r) => sel.has(r.id))}
                    onChange={(e) => setSel(e.target.checked ? new Set(rows.map((r) => r.id)) : new Set())}
                  />
                </th>
                <th style={{ width: 24 }} title="Đồng bộ: ✓ khớp máy chủ · ● chờ đẩy · ⚠ xung đột" />
                <th>Mã hồ sơ</th>
                {th("Số tạm", "so_tam")}
                {th("HS số", "ho_so_so", "num")}
                <th>Tiêu đề</th>
                {th("Mảng", "mang")}
                {th("THBQ", "thbq")}
                <th className="num">Số tờ</th>
                <th>Bắt đầu</th>
                {th("Kết thúc", "ngay_kt")}
                <th>Trạng thái</th>
                <th className="num">VB</th>
                <th />
              </tr>
            </thead>
            <tbody>
              {rows.length === 0 && (
                <tr>
                  <td colSpan={14} className="muted">
                    Không có hồ sơ khớp bộ lọc.
                  </td>
                </tr>
              )}
              {rows.map((r) => {
                const st = str(r, "thbq_status");
                return (
                  <tr key={r.id} className={open?.id === r.id ? "bg-surface-sunken" : ""}>
                    <td>
                      <input
                        type="checkbox"
                        checked={sel.has(r.id)}
                        onChange={(e) =>
                          setSel((s) => {
                            const x = new Set(s);
                            if (e.target.checked) x.add(r.id);
                            else x.delete(r.id);
                            return x;
                          })
                        }
                      />
                    </td>
                    <td>
                      <SyncDot sync={str(r, "sync")} />
                    </td>
                    <td className="muted text-xs">{str(r, "ma_ho_so")}</td>
                    <td>{str(r, "so_tam")}</td>
                    <td className="num">{str(r, "ho_so_so")}</td>
                    <td className="max-w-[460px]">
                      <button className="link text-left" onClick={() => setOpen(r)} title="Xem văn bản trong hồ sơ">
                        {str(r, "tieu_de") || <span className="muted">(chưa có tiêu đề)</span>}
                      </button>
                      {r.trung_thua === true && <span className="badge gray ml-1">trùng thừa</span>}
                    </td>
                    <td className="text-sm">{str(r, "ten_mang")}</td>
                    <td className="text-sm">{str(r, "ten_thbq")}</td>
                    <td className="num">{str(r, "so_to")}</td>
                    <td>{str(r, "ngay_bat_dau")}</td>
                    <td>{str(r, "ngay_ket_thuc")}</td>
                    <td>{st && <span className={`badge ${THBQ_BADGE[st] ?? "gray"}`}>{THBQ_STATUS_LABEL[st] ?? st}</span>}</td>
                    <td className="num">{str(r, "so_van_ban")}</td>
                    <td className="whitespace-nowrap">
                      {canEdit && (
                        <button className="btn sm" onClick={() => setForm({ row: r })}>
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

      {open && <VanBanPanel cat={cat} hoSo={open} canEdit={canEdit} onClose={() => setOpen(null)} />}
    </>
  );
}
