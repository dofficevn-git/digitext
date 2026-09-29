// Copyright (c) 2026 DigiText. All rights reserved.
// Hộp thoại hồ sơ tạm (đủ trường: mô tả + TT05 + chứng từ kết thúc). Ghi vào bản sao, offline được.

import { useMemo, useState } from "react";
import { clSave, str, TIEP_CAN, TIN_CAY, TINH_TRANG_VAT_LY, type ClCatalog, type ClLocalErr, type ClRow } from "@/api/chinhly";
import { errText, Select, tenPhong } from "./common";
import { Badge, Button, Checkbox, Dialog, ErrorBanner, Field, FormGrid, Input, Kbd, Select as UiSelect, Textarea } from "@/ui";
import { Icon } from "@/components/Icon";

const CO_BAN: [string, string, "text" | "number" | "date", string?][] = [
  ["so_tam", "Số tạm", "text"],
  ["so_ky_hieu_hs", "Số ký hiệu hồ sơ", "text"],
  ["so_to", "Số tờ", "text"],
  ["ngay_bat_dau", "Ngày bắt đầu", "date", "d/m/yyyy, m/yyyy hoặc yyyy"],
  ["ngay_ket_thuc", "Ngày kết thúc", "date", "d/m/yyyy, m/yyyy hoặc yyyy"],
  ["tong_so_vb", "Tổng số văn bản", "number"],
  ["so_luong_trang", "Số lượng trang", "number"],
  ["ngon_ngu", "Ngôn ngữ", "text"],
];
const TT05: [string, string][] = [
  ["tu_khoa", "Từ khoá"],
  ["ky_hieu_thong_tin", "Ký hiệu thông tin"],
  ["ma_ho_so_giay", "Mã hồ sơ giấy"],
];
const CT: [string, string, "text" | "number"][] = [
  ["ct_so_to_bang_chu", "Số tờ (bằng chữ)", "text"],
  ["ct_danh_so_tu", "Đánh số từ", "number"],
  ["ct_den_so", "Đến số", "number"],
  ["ct_so_trung", "Số trùng", "text"],
  ["ct_so_khuyet", "Số khuyết", "text"],
  ["ct_so_muc_luc_vb", "Số mục lục văn bản", "number"],
  ["ct_so_muc_luc_vb_bang_chu", "Số mục lục văn bản (bằng chữ)", "text"],
  ["ct_dac_diem_tai_lieu", "Đặc điểm tài liệu", "text"],
  ["ct_dia_danh", "Địa danh", "text"],
  ["ct_ngay", "Ngày", "number"],
  ["ct_thang", "Tháng", "number"],
  ["ct_nam", "Năm", "number"],
  ["ct_nguoi_lap_hs", "Người lập hồ sơ", "text"],
  ["ct_phan_bo_sung", "Phần bổ sung", "text"],
];
const opts = (xs: [string, string][]) => xs.map(([value, l]) => ({ value, label: `${value} · ${l}` }));

export default function HoSoForm({ cat, phongId, row, onSaved, onClose }: { cat: ClCatalog; phongId: string; row: ClRow | null; onSaved: (id: string, keepOpen: boolean) => void; onClose: () => void }) {
  const [d, setD] = useState<Record<string, unknown>>(() => (row ? { ...row } : { phong_id: phongId, trung_thua: false, che_do_su_dung: "01", tinh_trang_vat_ly: "02", muc_do_tin_cay: "03", ngon_ngu: "01" }));
  const [errs, setErrs] = useState<ClLocalErr[]>([]);
  const [busy, setBusy] = useState(false);
  const [moCt, setMoCt] = useState(false);
  const set = (k: string, v: unknown) => setD((p) => ({ ...p, [k]: v }));
  const g = (k: string) => (d[k] === null || d[k] === undefined ? "" : String(d[k]));
  const err = (k: string) => errs.find((e) => e.field === k)?.message;
  const loiChung = errs.filter((e) => !e.field).map((e) => e.message);
  const soLoiO = errs.filter((e) => !!e.field).length;

  const phongBans = useMemo(() => cat.phong_ban.filter((p) => str(p, "phong_id") === str(d as ClRow, "phong_id")), [cat.phong_ban, d]);
  const mangs = useMemo(() => cat.mang.filter((m) => str(m, "phong_ban_id") === str(d as ClRow, "phong_ban_id")), [cat.mang, d]);

  const luu = async (keepOpen: boolean) => {
    setBusy(true);
    setErrs([]);
    try {
      const patch: Record<string, unknown> = {};
      const keys = new Set<string>(["phong_id", "phong_ban_id", "mang_id", "thbq_id", "tieu_de", "trung_thua", "che_do_su_dung", "tinh_trang_vat_ly", "muc_do_tin_cay", "ghi_chu", ...CO_BAN.map((c) => c[0]), ...TT05.map((c) => c[0]), ...CT.map((c) => c[0])]);
      for (const k of keys) {
        const v = d[k];
        patch[k] = v === "" ? null : v;
      }
      const r = await clSave("ho_so", row ? row.id : null, patch);
      if (!r.ok) {
        setErrs(r.errors);
        if (r.errors.some((e) => CT.some((c) => c[0] === e.field))) setMoCt(true);
        return;
      }
      onSaved(r.id, keepOpen);
      if (keepOpen) setD({ phong_id: d.phong_id, phong_ban_id: d.phong_ban_id, mang_id: d.mang_id, thbq_id: d.thbq_id, trung_thua: false, che_do_su_dung: "01", tinh_trang_vat_ly: "02", muc_do_tin_cay: "03", ngon_ngu: d.ngon_ngu });
    } catch (e) {
      setErrs([{ field: "", message: errText(e) }]);
    } finally {
      setBusy(false);
    }
  };

  const inp = (k: string, type: "text" | "number" | "date" = "text") => <Input type={type === "number" ? "number" : "text"} value={g(k)} onChange={(e) => set(k, e.target.value)} />;

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      closable={!busy}
      size="xl"
      title={
        <span className="flex items-center gap-2">
          {row ? `Sửa hồ sơ số tạm ${str(row, "so_tam")}` : "Thêm hồ sơ tạm"}
          {row && str(row, "ho_so_so") && <Badge tone="info">Đã đánh số {str(row, "ho_so_so")}</Badge>}
        </span>
      }
      footer={
        <>
          <span className="self-center text-sm text-danger-700">{loiChung.length === 0 && soLoiO > 0 ? `Có ${soLoiO} ô chưa hợp lệ.` : ""}</span>
          <span className="flex-1" />
          <Button onClick={onClose} disabled={busy}>
            Huỷ
          </Button>
          {!row && (
            <Button disabled={busy} onClick={() => void luu(true)}>
              Lưu và thêm tiếp
            </Button>
          )}
          <Button variant="primary" icon="luu" loading={busy} onClick={() => void luu(false)}>
            Lưu <Kbd keys={["Ctrl", "Enter"]} className="ml-1" />
          </Button>
        </>
      }
    >
      <div
        className="space-y-4"
        onKeyDown={(e) => {
          if (e.ctrlKey && e.key === "Enter") void luu(false);
        }}
      >
        {loiChung.length > 0 && <ErrorBanner error={loiChung.join(" ")} />}
        <FormGrid cols={4}>
          <Field label="Phông" required>
            <Select
              value={g("phong_id")}
              onChange={(v) => {
                set("phong_id", v);
                set("phong_ban_id", "");
                set("mang_id", "");
              }}
              rows={cat.phong}
              render={tenPhong}
              disabled={!!row}
            />
          </Field>
          <Field label="Phòng ban">
            <Select
              value={g("phong_ban_id")}
              onChange={(v) => {
                set("phong_ban_id", v);
                set("mang_id", "");
              }}
              rows={phongBans}
              empty="Chưa chọn"
            />
          </Field>
          <Field label="Mảng công việc">
            <Select value={g("mang_id")} onChange={(v) => set("mang_id", v)} rows={mangs} empty="Chưa chọn" disabled={!g("phong_ban_id")} />
          </Field>
          <Field label="Thời hạn bảo quản">
            <Select value={g("thbq_id")} onChange={(v) => set("thbq_id", v)} rows={cat.thbq} empty="Chưa chọn" />
          </Field>
          {CO_BAN.map(([k, l, t, hint]) => (
            <Field key={k} label={l} required={k === "so_tam"} error={err(k)} hint={hint}>
              {inp(k, t)}
            </Field>
          ))}
          <Field label="Chế độ sử dụng">
            <UiSelect value={g("che_do_su_dung")} onChange={(v) => set("che_do_su_dung", v)} placeholder="Chọn" options={opts(TIEP_CAN)} />
          </Field>
          <Field label="Tình trạng vật lý">
            <UiSelect value={g("tinh_trang_vat_ly")} onChange={(v) => set("tinh_trang_vat_ly", v)} placeholder="Chọn" options={opts(TINH_TRANG_VAT_LY)} />
          </Field>
          <Field label="Mức độ tin cậy">
            <UiSelect value={g("muc_do_tin_cay")} onChange={(v) => set("muc_do_tin_cay", v)} placeholder="Chọn" options={opts(TIN_CAY)} />
          </Field>
          <Field label="Hồ sơ trùng thừa (loại)" inline className="self-end">
            <Checkbox checked={!!d.trung_thua} onChange={(v) => set("trung_thua", v)} />
          </Field>
          {TT05.map(([k, l]) => (
            <Field key={k} label={l} error={err(k)}>
              {inp(k)}
            </Field>
          ))}
        </FormGrid>
        <div className="grid gap-3 sm:grid-cols-[2fr_1fr]">
          <Field label="Tiêu đề hồ sơ" required error={err("tieu_de")}>
            <Textarea rows={2} value={g("tieu_de")} onChange={(e) => set("tieu_de", e.target.value)} />
          </Field>
          <Field label="Ghi chú">
            <Textarea rows={2} value={g("ghi_chu")} onChange={(e) => set("ghi_chu", e.target.value)} />
          </Field>
        </div>
        <div>
          <button type="button" className="inline-flex items-center gap-1 text-sm font-medium text-fg-muted hover:text-fg" onClick={() => setMoCt((v) => !v)} aria-expanded={moCt}>
            <Icon name={moCt ? "xuong" : "moNhanh"} size={14} /> Chứng từ kết thúc (14 trường)
          </button>
          {moCt && (
            <FormGrid cols={4} className="mt-3">
              {CT.map(([k, l, t]) => (
                <Field key={k} label={l} error={err(k)}>
                  {inp(k, t)}
                </Field>
              ))}
            </FormGrid>
          )}
        </div>
      </div>
    </Dialog>
  );
}
