// Copyright (c) 2026 DigiText. All rights reserved.
// Form hồ sơ tạm (đủ trường của bản cũ: mô tả + TT05 + chứng từ kết thúc). Ghi vào bản sao, offline được.

import { useMemo, useState } from "react";
import { clSave, str, TIEP_CAN, TIN_CAY, TINH_TRANG_VAT_LY, type ClCatalog, type ClLocalErr, type ClRow } from "@/api/chinhly";
import { errText, Field, Select } from "./common";

const CO_BAN: [string, string, "text" | "number" | "date"][] = [
  ["so_tam", "Số tạm *", "text"],
  ["so_ky_hieu_hs", "Số ký hiệu hồ sơ", "text"],
  ["so_to", "Số tờ", "text"],
  ["ngay_bat_dau", "Ngày bắt đầu (d/m/yyyy · m/yyyy · yyyy)", "date"],
  ["ngay_ket_thuc", "Ngày kết thúc (d/m/yyyy · m/yyyy · yyyy)", "date"],
  ["tong_so_vb", "Tổng số văn bản", "number"],
  ["so_luong_trang", "Số lượng trang", "number"],
  ["ngon_ngu", "Ngôn ngữ", "text"],
];
const TT05: [string, string, "text" | "number"][] = [
  ["tu_khoa", "Từ khoá", "text"],
  ["ky_hieu_thong_tin", "Ký hiệu thông tin", "text"],
  ["ma_ho_so_giay", "Mã hồ sơ giấy", "text"],
];
const CT: [string, string, "text" | "number"][] = [
  ["ct_so_to_bang_chu", "Số tờ (bằng chữ)", "text"],
  ["ct_danh_so_tu", "Đánh số từ", "number"],
  ["ct_den_so", "Đến số", "number"],
  ["ct_so_trung", "Số trùng", "text"],
  ["ct_so_khuyet", "Số khuyết", "text"],
  ["ct_so_muc_luc_vb", "Số mục lục văn bản", "number"],
  ["ct_so_muc_luc_vb_bang_chu", "Số mục lục VB (bằng chữ)", "text"],
  ["ct_dac_diem_tai_lieu", "Đặc điểm tài liệu", "text"],
  ["ct_dia_danh", "Địa danh", "text"],
  ["ct_ngay", "Ngày", "number"],
  ["ct_thang", "Tháng", "number"],
  ["ct_nam", "Năm", "number"],
  ["ct_nguoi_lap_hs", "Người lập hồ sơ", "text"],
  ["ct_phan_bo_sung", "Phần bổ sung", "text"],
];

export default function HoSoForm({
  cat,
  phongId,
  row,
  onSaved,
  onClose,
}: {
  cat: ClCatalog;
  /** Phông đang chọn (bắt buộc khi thêm mới). */
  phongId: string;
  /** Dòng đang sửa; null = thêm mới. */
  row: ClRow | null;
  onSaved: (id: string, keepOpen: boolean) => void;
  onClose: () => void;
}) {
  const [d, setD] = useState<Record<string, unknown>>(() => {
    if (row) return { ...row };
    return { phong_id: phongId, trung_thua: false, che_do_su_dung: "01", tinh_trang_vat_ly: "02", muc_do_tin_cay: "03", ngon_ngu: "01" };
  });
  const [errs, setErrs] = useState<ClLocalErr[]>([]);
  const [busy, setBusy] = useState(false);
  const [moCt, setMoCt] = useState(false);
  const set = (k: string, v: unknown) => setD((p) => ({ ...p, [k]: v }));
  const g = (k: string) => (d[k] === null || d[k] === undefined ? "" : String(d[k]));
  const err = (k: string) => errs.find((e) => e.field === k)?.message;

  const phongBans = useMemo(() => cat.phong_ban.filter((p) => str(p, "phong_id") === str(d as ClRow, "phong_id")), [cat.phong_ban, d]);
  const mangs = useMemo(() => cat.mang.filter((m) => str(m, "phong_ban_id") === str(d as ClRow, "phong_ban_id")), [cat.mang, d]);

  const luu = async (keepOpen: boolean) => {
    setBusy(true);
    setErrs([]);
    try {
      const patch: Record<string, unknown> = {};
      const keys = new Set<string>([
        "phong_id", "phong_ban_id", "mang_id", "thbq_id", "tieu_de", "trung_thua", "che_do_su_dung", "tinh_trang_vat_ly", "muc_do_tin_cay", "ghi_chu",
        ...CO_BAN.map((c) => c[0]), ...TT05.map((c) => c[0]), ...CT.map((c) => c[0]),
      ]);
      for (const k of keys) {
        const v = d[k];
        patch[k] = v === "" ? null : v;
      }
      const r = await clSave("ho_so", row ? row.id : null, patch);
      if (!r.ok) {
        setErrs(r.errors);
        return;
      }
      onSaved(r.id, keepOpen);
      if (keepOpen) {
        setD({ phong_id: d.phong_id, phong_ban_id: d.phong_ban_id, mang_id: d.mang_id, thbq_id: d.thbq_id, trung_thua: false, che_do_su_dung: "01", tinh_trang_vat_ly: "02", muc_do_tin_cay: "03", ngon_ngu: d.ngon_ngu });
      }
    } catch (e) {
      setErrs([{ field: "", message: errText(e) }]);
    } finally {
      setBusy(false);
    }
  };

  const inp = (k: string, type: "text" | "number" | "date" = "text") => (
    <input className={`inp w-full min-w-0 ${err(k) ? "border-danger" : ""}`} type={type === "number" ? "number" : "text"} value={g(k)} onChange={(e) => set(k, e.target.value)} title={err(k)} />
  );

  return (
    <div
      className="card mt-3"
      onKeyDown={(e) => {
        if (e.ctrlKey && e.key === "Enter") void luu(false);
        if (e.key === "Escape") onClose();
      }}
    >
      <div className="flex items-center gap-2">
        <h3 className="m-0">{row ? `Sửa hồ sơ — số tạm ${str(row, "so_tam")}` : "Thêm hồ sơ tạm"}</h3>
        {row && str(row, "ho_so_so") && <span className="badge info">Đã đánh số: HS {str(row, "ho_so_so")}</span>}
        <div className="flex-1" />
        <button className="btn" onClick={onClose}>
          Đóng
        </button>
      </div>
      <div className="grid gap-3 mt-3" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))" }}>
        <Field label="Phông *">
          <Select value={g("phong_id")} onChange={(v) => { set("phong_id", v); set("phong_ban_id", ""); set("mang_id", ""); }} rows={cat.phong} render={(r) => `${str(r, "ma_du_an")} — ${str(r, "ten")}`} disabled={!!row} />
        </Field>
        <Field label="Phòng ban">
          <Select value={g("phong_ban_id")} onChange={(v) => { set("phong_ban_id", v); set("mang_id", ""); }} rows={phongBans} empty="— Chưa chọn —" />
        </Field>
        <Field label="Mảng công việc">
          <Select value={g("mang_id")} onChange={(v) => set("mang_id", v)} rows={mangs} empty="— Chưa chọn —" disabled={!g("phong_ban_id")} />
        </Field>
        <Field label="Thời hạn bảo quản">
          <Select value={g("thbq_id")} onChange={(v) => set("thbq_id", v)} rows={cat.thbq} empty="— Chưa chọn —" />
        </Field>
        {CO_BAN.map(([k, l, t]) => (
          <Field key={k} label={err(k) ? `${l} — ${err(k)}` : l} className={err(k) ? "text-danger" : ""}>
            {inp(k, t)}
          </Field>
        ))}
        <Field label="Chế độ sử dụng">
          <select className="w-full" value={g("che_do_su_dung")} onChange={(e) => set("che_do_su_dung", e.target.value)}>
            <option value="">—</option>
            {TIEP_CAN.map(([m, l]) => (
              <option key={m} value={m}>
                {m} · {l}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Tình trạng vật lý">
          <select className="w-full" value={g("tinh_trang_vat_ly")} onChange={(e) => set("tinh_trang_vat_ly", e.target.value)}>
            <option value="">—</option>
            {TINH_TRANG_VAT_LY.map(([m, l]) => (
              <option key={m} value={m}>
                {m} · {l}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Mức độ tin cậy">
          <select className="w-full" value={g("muc_do_tin_cay")} onChange={(e) => set("muc_do_tin_cay", e.target.value)}>
            <option value="">—</option>
            {TIN_CAY.map(([m, l]) => (
              <option key={m} value={m}>
                {m} · {l}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Trùng thừa">
          <label className="flex items-center gap-2 h-9">
            <input type="checkbox" checked={!!d.trung_thua} onChange={(e) => set("trung_thua", e.target.checked)} /> Hồ sơ trùng thừa (loại)
          </label>
        </Field>
        {TT05.map(([k, l, t]) => (
          <Field key={k} label={l}>
            {inp(k, t)}
          </Field>
        ))}
      </div>
      <div className="grid gap-3 mt-3" style={{ gridTemplateColumns: "2fr 1fr" }}>
        <Field label={err("tieu_de") ? `Tiêu đề hồ sơ * — ${err("tieu_de")}` : "Tiêu đề hồ sơ *"} className={err("tieu_de") ? "text-danger" : ""}>
          <textarea className={`inp w-full ${err("tieu_de") ? "border-danger" : ""}`} rows={2} value={g("tieu_de")} onChange={(e) => set("tieu_de", e.target.value)} />
        </Field>
        <Field label="Ghi chú">
          <textarea className="inp w-full" rows={2} value={g("ghi_chu")} onChange={(e) => set("ghi_chu", e.target.value)} />
        </Field>
      </div>
      <button className="btn sm mt-3" onClick={() => setMoCt((v) => !v)}>
        {moCt ? "▾" : "▸"} Chứng từ kết thúc (14 trường)
      </button>
      {moCt && (
        <div className="grid gap-3 mt-2" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))" }}>
          {CT.map(([k, l, t]) => (
            <Field key={k} label={err(k) ? `${l} — ${err(k)}` : l} className={err(k) ? "text-danger" : ""}>
              {inp(k, t)}
            </Field>
          ))}
        </div>
      )}
      <div className="flex items-center gap-2 mt-3 flex-wrap">
        {errs.filter((e) => !e.field || e.field === "").map((e, i) => (
          <span key={i} className="text-danger text-sm">
            {e.message}
          </span>
        ))}
        {errs.length > 0 && errs.every((e) => e.field) && <span className="text-danger text-sm">Có {errs.length} trường chưa hợp lệ (đánh dấu đỏ).</span>}
        <div className="flex-1" />
        {!row && (
          <button className="btn" disabled={busy} onClick={() => void luu(true)}>
            Lưu & Tiếp
          </button>
        )}
        <button className="btn primary" disabled={busy} onClick={() => void luu(false)}>
          {row ? "Lưu (Ctrl+Enter)" : "Lưu & Đóng (Ctrl+Enter)"}
        </button>
      </div>
    </div>
  );
}
