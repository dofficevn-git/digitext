// Copyright (c) 2026 DigiText. All rights reserved.
// Lịch sử đơn vị hình thành phông / lịch sử phông có phiên bản (0039, 27/11/2026 — đợt 5, Digifile B02). Mỗi lần nộp bổ sung lập
// một phiên bản mới (chép sẵn từ bản mới nhất), bản cũ giữ nguyên để tra lại. Chỉ trực tuyến.

import { useCallback, useState } from "react";
import { V41t } from "@/live/useLive";
import { phongLichSuList, phongLichSuThem, type PhongLichSu, type PhongLichSuMoi } from "@/api/phongLichSu";
import { str, type ClRow } from "@/api/chinhly";
import { Badge, Button, Dialog, ErrorBanner, Field, FormGrid, Input, notifyError, Select, Table, Textarea, toast, Toolbar, type Column } from "@/ui";

const fmt = (iso: string) => {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleDateString("vi-VN");
};

type Form = Required<{ [K in keyof PhongLichSuMoi]: string }>;
const RONG: Form = { lichSuDonVi: "", lichSuPhong: "", thoiGianTaiLieu: "", tongSoTaiLieu: "", phuongAnPhanLoai: "", ngonNgu: "", trangThaiPhong: "mo", nguon: "", ghiChu: "" };
const tuBan = (b: PhongLichSu | null): Form =>
  b
    ? {
        lichSuDonVi: b.lichSuDonVi ?? "",
        lichSuPhong: b.lichSuPhong ?? "",
        thoiGianTaiLieu: b.thoiGianTaiLieu ?? "",
        tongSoTaiLieu: b.tongSoTaiLieu ?? "",
        phuongAnPhanLoai: b.phuongAnPhanLoai ?? "",
        ngonNgu: b.ngonNgu ?? "",
        trangThaiPhong: b.trangThaiPhong,
        nguon: "",
        ghiChu: "",
      }
    : RONG;

export default function PhongLichSuDialog({ phong, canEdit, onClose }: { phong: ClRow; canEdit: boolean; onClose: () => void }) {
  const [xem, setXem] = useState<PhongLichSu | null>(null);
  const [form, setForm] = useState<Form | null>(null);
  const [busy, setBusy] = useState(false);
  const load = useCallback(() => phongLichSuList(phong.id), [phong.id]);
  const { data, error, loading, reload } = V41t(load, { topics: ["chinhLy"] });
  const rows = data?.rows ?? [];
  const moiNhat = rows[0] ?? null;
  const hien = xem ?? moiNhat;

  const save = async () => {
    if (!form || busy) return;
    if (!form.lichSuDonVi.trim() && !form.lichSuPhong.trim()) return;
    setBusy(true);
    try {
      const b: PhongLichSuMoi = {
        lichSuDonVi: form.lichSuDonVi.trim() || null,
        lichSuPhong: form.lichSuPhong.trim() || null,
        thoiGianTaiLieu: form.thoiGianTaiLieu.trim() || null,
        tongSoTaiLieu: form.tongSoTaiLieu.trim() || null,
        phuongAnPhanLoai: form.phuongAnPhanLoai.trim() || null,
        ngonNgu: form.ngonNgu.trim() || null,
        trangThaiPhong: form.trangThaiPhong === "dong" ? "dong" : "mo",
        nguon: form.nguon.trim() || null,
        ghiChu: form.ghiChu.trim() || null,
      };
      const r = await phongLichSuThem(phong.id, b);
      toast.success(`Đã lưu phiên bản ${r.phienBan}.`);
      setForm(null);
      setXem(null);
      reload();
    } catch (e) {
      notifyError(e, "lưu lịch sử phông");
    } finally {
      setBusy(false);
    }
  };

  const cols: Column<PhongLichSu>[] = [
    { key: "phienBan", header: "Phiên bản", width: 90, cell: (r) => <span className="font-medium">v{r.phienBan}</span> },
    { key: "createdAt", header: "Ngày lập", width: 110, cell: (r) => fmt(r.createdAt) },
    { key: "nguoiLap", header: "Người lập", width: 150, cell: (r) => r.nguoiLap ?? "" },
    { key: "trangThaiPhong", header: "Phông", width: 80, cell: (r) => <Badge tone={r.trangThaiPhong === "dong" ? "gray" : "ok"}>{r.trangThaiPhong === "dong" ? "Đóng" : "Mở"}</Badge> },
    { key: "nguon", header: "Nguồn", cell: (r) => r.nguon ?? "" },
  ];
  const set = <K extends keyof Form>(k: K, v: string) => setForm((o) => (o ? { ...o, [k]: v } : o));

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      title={`Lịch sử phông — ${str(phong, "ma_du_an")} ${str(phong, "ten")}`}
      description="Lịch sử đơn vị hình thành phông và lịch sử phông theo Điều 4, Điều 22 TT05. Mỗi lần nộp bổ sung lập một phiên bản mới; bản cũ giữ nguyên."
      size="lg"
      footer={
        <>
          {form ? (
            <>
              <Button onClick={() => setForm(null)} disabled={busy}>
                Huỷ
              </Button>
              <Toolbar.Spacer />
              <Button variant="primary" loading={busy} disabled={!form.lichSuDonVi.trim() && !form.lichSuPhong.trim()} onClick={() => void save()}>
                Lưu phiên bản {rows.length + 1}
              </Button>
            </>
          ) : (
            <>
              <Toolbar.Spacer />
              <Button onClick={onClose}>Đóng</Button>
              {canEdit && (
                <Button variant="primary" icon="them" onClick={() => setForm(tuBan(moiNhat))}>
                  Phiên bản mới
                </Button>
              )}
            </>
          )}
        </>
      }
    >
      {error && <ErrorBanner error={error} className="mb-3" onRetry={reload} />}
      {form ? (
        <fieldset disabled={busy} className="m-0 border-0 p-0">
          <Field label="Lịch sử đơn vị hình thành phông" hint="Thành lập, chức năng nhiệm vụ, tổ chức, thay đổi tên gọi, giải thể…" hintAs="tip">
            <Textarea rows={6} value={form.lichSuDonVi} onChange={(e) => set("lichSuDonVi", e.target.value)} />
          </Field>
          <Field label="Lịch sử phông" className="mt-3" hint="Thời gian tài liệu, các lần thu nộp, chỉnh lý, tình trạng tài liệu…" hintAs="tip">
            <Textarea rows={5} value={form.lichSuPhong} onChange={(e) => set("lichSuPhong", e.target.value)} />
          </Field>
          <FormGrid className="mt-3">
            <Field label="Thời gian tài liệu">
              <Input value={form.thoiGianTaiLieu} onChange={(e) => set("thoiGianTaiLieu", e.target.value)} placeholder="1998 – 2024" />
            </Field>
            <Field label="Tổng số tài liệu">
              <Input value={form.tongSoTaiLieu} onChange={(e) => set("tongSoTaiLieu", e.target.value)} placeholder="1.250 hồ sơ, 36 mét giá" />
            </Field>
            <Field label="Phương án phân loại">
              <Input value={form.phuongAnPhanLoai} onChange={(e) => set("phuongAnPhanLoai", e.target.value)} placeholder="Thời gian – Cơ cấu tổ chức" />
            </Field>
            <Field label="Ngôn ngữ">
              <Input value={form.ngonNgu} onChange={(e) => set("ngonNgu", e.target.value)} placeholder="Tiếng Việt" />
            </Field>
            <Field label="Trạng thái phông">
              <Select value={form.trangThaiPhong} onChange={(v) => set("trangThaiPhong", v)} options={[{ value: "mo", label: "Mở (còn bổ sung)" }, { value: "dong", label: "Đóng" }]} />
            </Field>
            <Field label="Nguồn / lần nộp" hint="Ví dụ: Thu nộp đợt 2 năm 2026" hintAs="tip">
              <Input value={form.nguon} onChange={(e) => set("nguon", e.target.value)} />
            </Field>
          </FormGrid>
          <Field label="Ghi chú" className="mt-3">
            <Input value={form.ghiChu} onChange={(e) => set("ghiChu", e.target.value)} />
          </Field>
        </fieldset>
      ) : (
        <>
          <Table columns={cols} rows={rows} getRowId={(r) => r.id} dense onRowClick={(r) => setXem(r)} rowClassName={(r) => (r.id === hien?.id ? "bg-surface-sunken" : undefined)} state={{ loading: loading && !data, empty: { title: "Chưa có lịch sử phông", description: canEdit ? "Bấm Phiên bản mới để khai lần đầu." : undefined } }} />
          {hien && (
            <div className="mt-3 space-y-3 text-sm">
              <div className="flex flex-wrap items-center gap-2">
                <span className="font-semibold">Phiên bản {hien.phienBan}</span>
                <span className="text-fg-subtle">
                  {fmt(hien.createdAt)}
                  {hien.nguoiLap ? ` · ${hien.nguoiLap}` : ""}
                </span>
                {hien.nguon && <Badge tone="info">{hien.nguon}</Badge>}
              </div>
              {hien.lichSuDonVi && (
                <section>
                  <h4 className="m-0 mb-1 text-sm font-medium text-fg-muted">Lịch sử đơn vị hình thành phông</h4>
                  <p className="m-0 whitespace-pre-wrap">{hien.lichSuDonVi}</p>
                </section>
              )}
              {hien.lichSuPhong && (
                <section>
                  <h4 className="m-0 mb-1 text-sm font-medium text-fg-muted">Lịch sử phông</h4>
                  <p className="m-0 whitespace-pre-wrap">{hien.lichSuPhong}</p>
                </section>
              )}
              <dl className="m-0 grid grid-cols-2 gap-x-4 gap-y-1 md:grid-cols-4">
                {[
                  ["Thời gian tài liệu", hien.thoiGianTaiLieu],
                  ["Tổng số tài liệu", hien.tongSoTaiLieu],
                  ["Phương án phân loại", hien.phuongAnPhanLoai],
                  ["Ngôn ngữ", hien.ngonNgu],
                ]
                  .filter(([, v]) => v)
                  .map(([k, v]) => (
                    <div key={k}>
                      <dt className="text-xs text-fg-subtle">{k}</dt>
                      <dd className="m-0">{v}</dd>
                    </div>
                  ))}
              </dl>
              {hien.ghiChu && <p className="m-0 text-fg-subtle">{hien.ghiChu}</p>}
            </div>
          )}
        </>
      )}
    </Dialog>
  );
}
