// Copyright (c) 2026 DigiText. All rights reserved.
// Văn bản của một hồ sơ: bảng + form (Modal). Ghi vào bản sao, offline được.

import { useCallback, useMemo, useState } from "react";
import Modal, { V49j } from "@/ui/Modal";
import { isTauri } from "@/api/ipc";
import { V41p } from "@/live/useLive";
import { clDelete, clNextStt, clRemote, clSave, clVanBanList, str, TIEP_CAN, TIN_CAY, TINH_TRANG_VAT_LY, type ClCatalog, type ClLocalErr, type ClRow } from "@/api/chinhly";
import { V460 } from "@/state";
import { errText, n, SyncDot } from "./common";
import { docMauMlvb, xuatMauMlvb } from "./excel";
import { useRef } from "react";

const FIELDS: [string, string][] = [
  ["so_van_ban", "Số văn bản"],
  ["so_ky_hieu", "Ký hiệu"],
  ["ngay_thang", "Ngày tháng (d/m/yyyy · m/yyyy · yyyy)"],
  ["co_quan_ban_hanh", "Cơ quan ban hành (tác giả)"],
  ["co_quan_chu_quan", "Cơ quan chủ quản"],
  ["nguoi_ky", "Người ký"],
  ["trich_yeu", "Trích yếu nội dung"],
  ["tu_khoa", "Từ khoá"],
  ["but_tich", "Bút tích"],
  ["tai_lieu_so", "Tài liệu số"],
  ["ky_hieu_thong_tin", "Ký hiệu thông tin"],
  ["ghi_chu", "Ghi chú"],
];

function VanBanForm({ cat, hoSoId, row, onClose, onSaved }: { cat: ClCatalog; hoSoId: string; row: ClRow | null; onClose: () => void; onSaved: () => void }) {
  const [d, setD] = useState<Record<string, unknown>>(() => (row ? { ...row } : { ho_so_id: hoSoId, ngon_ngu: "01", che_do_su_dung: "01", do_tin_cay: "03", tinh_trang_vat_ly: "02" }));
  const [errs, setErrs] = useState<ClLocalErr[]>([]);
  const set = (k: string, v: unknown) => setD((p) => ({ ...p, [k]: v }));
  const g = (k: string) => (d[k] === null || d[k] === undefined ? "" : String(d[k]));
  const err = (k: string) => errs.find((e) => e.field === k)?.message;
  const luu = async () => {
    setErrs([]);
    try {
      const patch: Record<string, unknown> = {};
      for (const k of ["ho_so_id", "so_thu_tu", "loai_vb", "to_so", "so_trang", "trang_so", "ngon_ngu", "che_do_su_dung", "do_tin_cay", "tinh_trang_vat_ly", ...FIELDS.map((f) => f[0])]) {
        patch[k] = d[k] === "" ? null : d[k];
      }
      if (!row && (patch.so_thu_tu === null || patch.so_thu_tu === undefined)) patch.so_thu_tu = await clNextStt(hoSoId);
      const r = await clSave("van_ban", row ? row.id : null, patch);
      if (!r.ok) {
        setErrs(r.errors);
        return;
      }
      onSaved();
    } catch (e) {
      setErrs([{ field: "", message: errText(e) }]);
    }
  };
  const inp = (k: string, type = "text") => <input className={`inp w-full ${err(k) ? "border-danger" : ""}`} type={type} value={g(k)} onChange={(e) => set(k, e.target.value)} title={err(k)} />;
  const sel = (k: string, opts: [string, string][]) => (
    <select className="w-full" value={g(k)} onChange={(e) => set(k, e.target.value)}>
      <option value="">—</option>
      {opts.map(([m, l]) => (
        <option key={m} value={m}>
          {m} · {l}
        </option>
      ))}
    </select>
  );
  return (
    <Modal title={row ? `Sửa văn bản STT ${str(row, "so_thu_tu")}` : "Thêm văn bản"} onClose={onClose} onSave={() => void luu()}>
      {errs.some((e) => !e.field) && <div className="text-danger text-sm mb-2">{errs.find((e) => !e.field)?.message}</div>}
      <V49j label={err("so_thu_tu") ? `Số thứ tự — ${err("so_thu_tu")}` : "Số thứ tự trong hồ sơ (trống = tiếp theo)"}>{inp("so_thu_tu", "number")}</V49j>
      <V49j label="Loại văn bản">{sel("loai_vb", cat.loai_van_ban.map((l) => [str(l, "ma"), str(l, "ten")]))}</V49j>
      {FIELDS.map(([k, l]) => (
        <V49j key={k} label={err(k) ? `${l} — ${err(k)}` : l}>
          {k === "trich_yeu" ? <textarea className="inp w-full" rows={2} value={g(k)} onChange={(e) => set(k, e.target.value)} /> : inp(k)}
        </V49j>
      ))}
      <V49j label="Tờ số / Trang bắt đầu / Số trang">
        <div className="flex gap-2">
          <input className="inp" type="number" placeholder="Tờ số" value={g("to_so")} onChange={(e) => set("to_so", e.target.value)} />
          <input className="inp" type="number" placeholder="Trang BĐ" value={g("trang_so")} onChange={(e) => set("trang_so", e.target.value)} />
          <input className="inp" type="number" placeholder="Số trang" value={g("so_trang")} onChange={(e) => set("so_trang", e.target.value)} />
        </div>
      </V49j>
      <V49j label="Ngôn ngữ">{sel("ngon_ngu", cat.ngon_ngu.map((l) => [str(l, "ma"), str(l, "ten")]))}</V49j>
      <V49j label="Chế độ sử dụng">{sel("che_do_su_dung", TIEP_CAN)}</V49j>
      <V49j label="Mức độ tin cậy">{sel("do_tin_cay", TIN_CAY)}</V49j>
      <V49j label="Tình trạng vật lý">{sel("tinh_trang_vat_ly", TINH_TRANG_VAT_LY)}</V49j>
    </Modal>
  );
}

export default function VanBanPanel({ cat, hoSo, canEdit, onClose }: { cat: ClCatalog; hoSo: ClRow; canEdit: boolean; onClose: () => void }) {
  const [rows, setRows] = useState<ClRow[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const [form, setForm] = useState<{ row: ClRow | null } | null>(null);
  const hoSoId = hoSo.id;
  const load = useCallback(async () => {
    try {
      setRows(await clVanBanList(hoSoId));
      setErr(null);
    } catch (e) {
      setErr(errText(e));
    }
  }, [hoSoId]);
  const { reload } = V41p(load, { topics: ["chinhLy"], enabled: isTauri() });
  const tenLoai = useMemo(() => new Map(cat.loai_van_ban.map((l) => [str(l, "ma"), str(l, "ten")])), [cat.loai_van_ban]);
  const { session } = V460();
  const fileRef = useRef<HTMLInputElement>(null);
  const [msg, setMsg] = useState<string | null>(null);
  const nhapMlvb = async (f: File) => {
    setMsg(null);
    try {
      const d = await docMauMlvb(f);
      if (d.hoSoId && d.hoSoId !== hoSoId && !window.confirm("Tệp ghi ID hồ sơ khác hồ sơ đang mở. Vẫn nhập vào hồ sơ này?")) return;
      if (d.rows.length === 0) {
        setMsg(d.errors[0]?.message ?? "Tệp không có dòng dữ liệu (dữ liệu từ dòng 5).");
        return;
      }
      const r = await clRemote<{ created: number; updated: number; skipped: number; errors: { row: number; field?: string; message: string }[] }>("POST", `/cl/ho-so/${hoSoId}/van-ban/import`, {
        projectId: session?.projectId,
        mode: "upsert",
        rows: d.rows,
      });
      const errs = [...d.errors, ...r.errors.map((e) => ({ row: e.row + 4, message: `${e.field ? e.field + ": " : ""}${e.message}` }))];
      setMsg(`Đã thêm ${n(r.created)}, cập nhật ${n(r.updated)}${errs.length ? ` · ${errs.length} dòng lỗi: ${errs.slice(0, 5).map((e) => `dòng ${e.row} ${e.message}`).join("; ")}` : ""}`);
      reload();
    } catch (e) {
      setMsg(errText(e));
    }
  };

  const xoa = async (r: ClRow) => {
    if (!window.confirm(`Xoá văn bản STT ${str(r, "so_thu_tu")}?`)) return;
    try {
      await clDelete("van_ban", r.id);
      reload();
    } catch (e) {
      setErr(errText(e));
    }
  };

  return (
    <div className="card mt-3">
      <div className="flex items-center gap-2 flex-wrap">
        <h3 className="m-0">Văn bản trong hồ sơ</h3>
        <span className="muted text-sm">
          {str(hoSo, "ma_ho_so") || `số tạm ${str(hoSo, "so_tam")}`} · {str(hoSo, "tieu_de")}
        </span>
        <span className="badge gray">{n(rows.length)} văn bản</span>
        <div className="flex-1" />
        {canEdit && (
          <button className="btn sm primary" onClick={() => setForm({ row: null })}>
            + Thêm văn bản
          </button>
        )}
        <button className="btn sm" onClick={() => void xuatMauMlvb(hoSoId, str(hoSo, "ma_ho_so"), str(hoSo, "tieu_de")).catch((e) => setMsg(errText(e)))} title="Tải mẫu Excel mục lục văn bản của hồ sơ này">
          Mẫu MLVB
        </button>
        {canEdit && (
          <>
            <button className="btn sm" onClick={() => fileRef.current?.click()} title="Nhập mục lục văn bản từ Excel (cần máy chủ)">
              Nhập MLVB
            </button>
            <input ref={fileRef} type="file" accept=".xlsx,.xls" hidden onChange={(e) => { const f = e.target.files?.[0]; e.target.value = ""; if (f) void nhapMlvb(f); }} />
          </>
        )}
        <button className="btn sm" onClick={onClose}>
          Đóng
        </button>
      </div>
      {err && <div className="text-danger text-sm mt-2">{err}</div>}
      {msg && <div className="text-sm mt-2">{msg}</div>}
      <div className="tbl-scroll mt-2">
        <table className="tbl">
          <thead>
            <tr>
              <th style={{ width: 28 }} />
              <th className="num">STT</th>
              <th>Mã VB</th>
              <th>Loại</th>
              <th>Số / ký hiệu</th>
              <th>Ngày</th>
              <th>Tác giả</th>
              <th>Trích yếu</th>
              <th className="num">Tờ số</th>
              <th className="num">Số trang</th>
              <th>Ghi chú</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {rows.length === 0 && (
              <tr>
                <td colSpan={12} className="muted">
                  Chưa có văn bản. Bấm “Thêm văn bản” hoặc nhập mục lục văn bản từ Excel.
                </td>
              </tr>
            )}
            {rows.map((r) => (
              <tr key={r.id}>
                <td>
                  <SyncDot sync={r._sync?.dirty ? "pending" : "clean"} />
                </td>
                <td className="num">{str(r, "so_thu_tu")}</td>
                <td className="muted text-xs">{str(r, "ma_van_ban")}</td>
                <td>{tenLoai.get(str(r, "loai_vb")) ?? str(r, "loai_vb")}</td>
                <td>{[str(r, "so_van_ban"), str(r, "so_ky_hieu")].filter(Boolean).join(" / ")}</td>
                <td>{str(r, "ngay_thang")}</td>
                <td>{str(r, "co_quan_ban_hanh")}</td>
                <td className="max-w-[420px] truncate" title={str(r, "trich_yeu")}>
                  {str(r, "trich_yeu")}
                </td>
                <td className="num">{str(r, "to_so")}</td>
                <td className="num">{str(r, "so_trang")}</td>
                <td className="muted">{str(r, "ghi_chu")}</td>
                <td className="whitespace-nowrap">
                  {canEdit && (
                    <>
                      <button className="btn sm" onClick={() => setForm({ row: r })}>
                        Sửa
                      </button>{" "}
                      <button className="btn sm" onClick={() => void xoa(r)}>
                        Xoá
                      </button>
                    </>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      {form && (
        <VanBanForm
          cat={cat}
          hoSoId={hoSoId}
          row={form.row}
          onClose={() => setForm(null)}
          onSaved={() => {
            setForm(null);
            reload();
          }}
        />
      )}
    </div>
  );
}
