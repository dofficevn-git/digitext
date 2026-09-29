// Copyright (c) 2026 DigiText. All rights reserved.
// Hộp thoại thêm / sửa văn bản trong hồ sơ (bảng SQLite trên máy, tự đẩy sang chỉnh lý).

import type { V4bp } from "@/api/dossier";
import { Button, Dialog, Field, FormGrid, Input, Textarea } from "@/ui";

export default function V4ck({ mode, draft, onChange, onClose, onSave, busy }: { mode: "add" | "edit"; draft: V4bp; onChange: (p: Partial<V4bp>) => void; onClose: () => void; onSave: () => void; busy?: boolean }) {
  const num = (v: string): number | undefined => {
    const t = v.trim();
    if (t === "") return undefined;
    const k = Number.parseInt(t, 10);
    return Number.isNaN(k) ? undefined : k;
  };
  const text = (key: keyof V4bp, label: string, span?: boolean) => (
    <Field label={label} className={span ? "col-span-2" : undefined}>
      <Input value={(draft[key] as string | undefined) ?? ""} onChange={(e) => onChange({ [key]: e.target.value } as Partial<V4bp>)} />
    </Field>
  );
  return (
    <Dialog open onOpenChange={(o) => !o && onClose()} title={mode === "add" ? "Thêm văn bản" : "Sửa văn bản"} size="lg" footer={<><Button onClick={onClose}>Huỷ</Button><Button variant="primary" loading={busy} onClick={onSave}>Lưu</Button></>}>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          onSave();
        }}
      >
        <FormGrid cols={2}>
          <Field label="Số thứ tự" hint="Để trống thì tự đánh số">
            <Input type="number" value={draft.thuTu ? draft.thuTu : ""} onChange={(e) => onChange({ thuTu: num(e.target.value) ?? 0 })} autoFocus />
          </Field>
          {text("loaiVanBan", "Loại văn bản")}
          {text("soKyHieu", "Số, ký hiệu")}
          <Field label="Ngày tháng" hint="dd/mm/yyyy">
            <Input value={draft.ngayThang ?? ""} onChange={(e) => onChange({ ngayThang: e.target.value })} placeholder="dd/mm/yyyy" />
          </Field>
          {text("coQuanBanHanh", "Cơ quan ban hành", true)}
          <Field label="Trích yếu" className="col-span-2">
            <Textarea rows={3} value={draft.trichYeu ?? ""} onChange={(e) => onChange({ trichYeu: e.target.value })} />
          </Field>
          <Field label="Tờ số">
            <Input type="number" value={draft.toSo ?? ""} onChange={(e) => onChange({ toSo: num(e.target.value) })} />
          </Field>
          <Field label="Số trang">
            <Input type="number" value={draft.soTrang ?? ""} onChange={(e) => onChange({ soTrang: num(e.target.value) })} />
          </Field>
          <Field label="Trang số">
            <Input type="number" value={draft.trangSo ?? ""} onChange={(e) => onChange({ trangSo: num(e.target.value) })} />
          </Field>
          {text("ngonNgu", "Ngôn ngữ")}
          {text("cheDoSuDung", "Chế độ sử dụng")}
          {text("doTinCay", "Độ tin cậy")}
          {text("butTich", "Bút tích")}
          {text("tinhTrangVatLy", "Tình trạng vật lý")}
          {text("taiLieuSo", "Tài liệu số")}
          {text("ghiChu", "Ghi chú", true)}
        </FormGrid>
        <button type="submit" className="hidden" />
      </form>
    </Dialog>
  );
}
