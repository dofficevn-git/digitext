// Copyright (c) 2026 DigiText. All rights reserved.
// Văn bản của một hồ sơ chỉnh lý: bảng + hộp thoại thêm / sửa. Ghi vào bản sao, offline được.

import { useCallback, useMemo, useRef, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41p } from "@/live/useLive";
import { clDelete, clNextStt, clRemote, clSave, clVanBanList, str, TIEP_CAN, TIN_CAY, TINH_TRANG_VAT_LY, type ClCatalog, type ClLocalErr, type ClRow } from "@/api/chinhly";
import { V460 } from "@/state";
import { errText, n, SyncBadgeRow } from "./common";
import { docMauMlvb, xuatMauMlvb } from "./excel";
import { Badge, Banner, Button, Card, confirm, Dialog, ErrorBanner, Field, FormGrid, IconButton, Input, Menu, notifyError, Select, Table, Textarea, toast, type Column } from "@/ui";

const FIELDS: [string, string, string?][] = [
  ["so_van_ban", "Số văn bản"],
  ["so_ky_hieu", "Ký hiệu"],
  ["ngay_thang", "Ngày tháng", "d/m/yyyy, m/yyyy hoặc yyyy"],
  ["co_quan_ban_hanh", "Cơ quan ban hành"],
  ["co_quan_chu_quan", "Cơ quan chủ quản"],
  ["nguoi_ky", "Người ký"],
  ["trich_yeu", "Trích yếu nội dung"],
  ["tu_khoa", "Từ khoá"],
  ["but_tich", "Bút tích"],
  ["tai_lieu_so", "Tài liệu số"],
  ["ky_hieu_thong_tin", "Ký hiệu thông tin"],
  ["ghi_chu", "Ghi chú"],
];
const opts = (xs: [string, string][]) => xs.map(([value, l]) => ({ value, label: `${value} · ${l}` }));

function VanBanForm({ cat, hoSoId, row, onClose, onSaved }: { cat: ClCatalog; hoSoId: string; row: ClRow | null; onClose: () => void; onSaved: () => void }) {
  const [d, setD] = useState<Record<string, unknown>>(() => (row ? { ...row } : { ho_so_id: hoSoId, ngon_ngu: "01", che_do_su_dung: "01", do_tin_cay: "03", tinh_trang_vat_ly: "02" }));
  const [errs, setErrs] = useState<ClLocalErr[]>([]);
  const [busy, setBusy] = useState(false);
  const set = (k: string, v: unknown) => setD((p) => ({ ...p, [k]: v }));
  const g = (k: string) => (d[k] === null || d[k] === undefined ? "" : String(d[k]));
  const err = (k: string) => errs.find((e) => e.field === k)?.message;
  const luu = async () => {
    setErrs([]);
    setBusy(true);
    try {
      const patch: Record<string, unknown> = {};
      for (const k of ["ho_so_id", "so_thu_tu", "loai_vb", "to_so", "so_trang", "trang_so", "ngon_ngu", "che_do_su_dung", "do_tin_cay", "tinh_trang_vat_ly", ...FIELDS.map((f) => f[0])]) patch[k] = d[k] === "" ? null : d[k];
      if (!row && (patch.so_thu_tu === null || patch.so_thu_tu === undefined)) patch.so_thu_tu = await clNextStt(hoSoId);
      const r = await clSave("van_ban", row ? row.id : null, patch);
      if (!r.ok) {
        setErrs(r.errors);
        return;
      }
      onSaved();
    } catch (e) {
      setErrs([{ field: "", message: errText(e) }]);
    } finally {
      setBusy(false);
    }
  };
  const inp = (k: string, type = "text") => <Input type={type} value={g(k)} onChange={(e) => set(k, e.target.value)} />;
  const chung = errs.find((e) => !e.field)?.message;
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      closable={!busy}
      size="lg"
      title={row ? `Sửa văn bản số thứ tự ${str(row, "so_thu_tu")}` : "Thêm văn bản"}
      footer={
        <>
          <Button onClick={onClose} disabled={busy}>
            Huỷ
          </Button>
          <Button variant="primary" icon="luu" loading={busy} onClick={() => void luu()}>
            Lưu
          </Button>
        </>
      }
    >
      {/* eslint-disable-next-line jsx-a11y/no-static-element-interactions -- chỉ bắt Ctrl+Enter để lưu, không phải nút */}
      <div
        className="space-y-4"
        onKeyDown={(e) => {
          if (e.ctrlKey && e.key === "Enter") void luu();
        }}
      >
        {chung && <ErrorBanner error={chung} />}
        <FormGrid cols={3}>
          <Field label="Số thứ tự trong hồ sơ" hint="Để trống thì lấy số tiếp theo" error={err("so_thu_tu")}>
            {inp("so_thu_tu", "number")}
          </Field>
          <Field label="Loại văn bản">
            <Select value={g("loai_vb")} onChange={(v) => set("loai_vb", v)} placeholder="Chọn" options={cat.loai_van_ban.map((l) => ({ value: str(l, "ma"), label: str(l, "ten") }))} />
          </Field>
          <Field label="Ngôn ngữ">
            <Select value={g("ngon_ngu")} onChange={(v) => set("ngon_ngu", v)} placeholder="Chọn" options={cat.ngon_ngu.map((l) => ({ value: str(l, "ma"), label: str(l, "ten") }))} />
          </Field>
          {FIELDS.filter(([k]) => k !== "trich_yeu").map(([k, l, hint]) => (
            <Field key={k} label={l} hint={hint} error={err(k)}>
              {inp(k)}
            </Field>
          ))}
          <Field label="Tờ số">{inp("to_so", "number")}</Field>
          <Field label="Trang bắt đầu">{inp("trang_so", "number")}</Field>
          <Field label="Số trang">{inp("so_trang", "number")}</Field>
          <Field label="Chế độ sử dụng">
            <Select value={g("che_do_su_dung")} onChange={(v) => set("che_do_su_dung", v)} placeholder="Chọn" options={opts(TIEP_CAN)} />
          </Field>
          <Field label="Mức độ tin cậy">
            <Select value={g("do_tin_cay")} onChange={(v) => set("do_tin_cay", v)} placeholder="Chọn" options={opts(TIN_CAY)} />
          </Field>
          <Field label="Tình trạng vật lý">
            <Select value={g("tinh_trang_vat_ly")} onChange={(v) => set("tinh_trang_vat_ly", v)} placeholder="Chọn" options={opts(TINH_TRANG_VAT_LY)} />
          </Field>
        </FormGrid>
        <Field label="Trích yếu nội dung" error={err("trich_yeu")}>
          <Textarea rows={2} value={g("trich_yeu")} onChange={(e) => set("trich_yeu", e.target.value)} />
        </Field>
      </div>
    </Dialog>
  );
}

export default function VanBanPanel({ cat, hoSo, canEdit, onClose }: { cat: ClCatalog; hoSo: ClRow; canEdit: boolean; onClose: () => void }) {
  const [rows, setRows] = useState<ClRow[]>([]);
  const [err, setErr] = useState<unknown>(null);
  const [form, setForm] = useState<{ row: ClRow | null } | null>(null);
  const [nhapKq, setNhapKq] = useState<{ text: string; errors: string[] } | null>(null);
  const hoSoId = hoSo.id;
  const load = useCallback(async () => {
    try {
      setRows(await clVanBanList(hoSoId));
      setErr(null);
    } catch (e) {
      setErr(e);
    }
  }, [hoSoId]);
  const { reload } = V41p(load, { topics: ["chinhLy"], enabled: isTauri() });
  const tenLoai = useMemo(() => new Map(cat.loai_van_ban.map((l) => [str(l, "ma"), str(l, "ten")])), [cat.loai_van_ban]);
  const { session } = V460();
  const fileRef = useRef<HTMLInputElement>(null);

  const nhapMlvb = async (f: File) => {
    setNhapKq(null);
    try {
      const d = await docMauMlvb(f);
      if (d.hoSoId && d.hoSoId !== hoSoId) {
        const ok = await confirm({ title: "Tệp ghi ID hồ sơ khác hồ sơ đang mở", body: "Vẫn nhập vào hồ sơ này?", confirmLabel: "Nhập" });
        if (!ok) return;
      }
      if (d.rows.length === 0) {
        toast.warning(d.errors[0]?.message ?? "Tệp không có dòng dữ liệu, dữ liệu bắt đầu từ dòng 5.");
        return;
      }
      const r = await clRemote<{ created: number; updated: number; skipped: number; errors: { row: number; field?: string; message: string }[] }>("POST", `/cl/ho-so/${hoSoId}/van-ban/import`, { projectId: session?.projectId, mode: "upsert", rows: d.rows });
      const errs = [...d.errors, ...r.errors.map((e) => ({ row: e.row + 4, message: `${e.field ? e.field + ": " : ""}${e.message}` }))];
      setNhapKq({ text: `Đã thêm ${n(r.created)}, cập nhật ${n(r.updated)}${errs.length ? `, ${errs.length} dòng lỗi` : ""}.`, errors: errs.map((e) => `Dòng ${e.row}: ${e.message}`) });
      reload();
    } catch (e) {
      notifyError(e, "nhập mục lục văn bản");
    }
  };

  const xoa = async (r: ClRow) => {
    const ok = await confirm({ title: `Xoá văn bản số thứ tự ${str(r, "so_thu_tu")}?`, confirmLabel: "Xoá", danger: true });
    if (!ok) return;
    try {
      await clDelete("van_ban", r.id);
      reload();
    } catch (e) {
      notifyError(e, "xoá văn bản");
    }
  };

  const cols: Column<ClRow>[] = [
    { key: "_s", header: "", width: 80, cell: (r) => <SyncBadgeRow sync={r._sync?.dirty ? "pending" : "clean"} /> },
    { key: "so_thu_tu", header: "STT", align: "right", width: 60, cell: (r) => str(r, "so_thu_tu"), sortable: true, sortValue: (r) => Number(str(r, "so_thu_tu")) || 0 },
    { key: "ma_van_ban", header: "Mã văn bản", className: "text-xs text-fg-subtle", cell: (r) => str(r, "ma_van_ban"), hideBelow: 1400 },
    { key: "loai_vb", header: "Loại", cell: (r) => tenLoai.get(str(r, "loai_vb")) ?? str(r, "loai_vb") },
    { key: "so", header: "Số / ký hiệu", cell: (r) => [str(r, "so_van_ban"), str(r, "so_ky_hieu")].filter(Boolean).join(" / ") },
    { key: "ngay_thang", header: "Ngày", cell: (r) => str(r, "ngay_thang") },
    { key: "co_quan_ban_hanh", header: "Tác giả", cell: (r) => str(r, "co_quan_ban_hanh") },
    { key: "trich_yeu", header: "Trích yếu", className: "max-w-[420px] whitespace-normal", cell: (r) => str(r, "trich_yeu") },
    { key: "to_so", header: "Tờ số", align: "right", width: 70, cell: (r) => str(r, "to_so") },
    { key: "so_trang", header: "Số trang", align: "right", width: 80, cell: (r) => str(r, "so_trang") },
    { key: "ghi_chu", header: "Ghi chú", className: "text-fg-subtle", cell: (r) => str(r, "ghi_chu"), hideBelow: 1400 },
  ];

  return (
    <Card
      className="mt-4"
      padding="none"
      title={
        <span className="flex flex-wrap items-center gap-2">
          Văn bản trong hồ sơ <Badge tone="gray">{n(rows.length)}</Badge>
        </span>
      }
      description={`${str(hoSo, "ma_ho_so") || `Số tạm ${str(hoSo, "so_tam")}`} · ${str(hoSo, "tieu_de")}`}
      actions={
        <>
          {canEdit && (
            <Button size="sm" variant="primary" icon="them" onClick={() => setForm({ row: null })}>
              Thêm văn bản
            </Button>
          )}
          <Menu
            size="sm"
            items={[
              { label: "Tải mẫu Excel mục lục văn bản", icon: "tepExcel", onSelect: () => void xuatMauMlvb(hoSoId, str(hoSo, "ma_ho_so"), str(hoSo, "tieu_de")).catch((e) => notifyError(e, "tải mẫu")) },
              { label: "Nhập mục lục văn bản từ Excel", icon: "nhapTep", disabled: !canEdit, hint: "Cần máy chủ", onSelect: () => fileRef.current?.click() },
            ]}
          />
          <input
            ref={fileRef}
            type="file"
            accept=".xlsx,.xls"
            hidden
            aria-hidden
            onChange={(e) => {
              const f = e.target.files?.[0];
              e.target.value = "";
              if (f) void nhapMlvb(f);
            }}
          />
          <IconButton label="Đóng" icon="dong" size="sm" onClick={onClose} />
        </>
      }
    >
      {err !== null && <ErrorBanner error={err} className="mx-4 mb-3" onRetry={reload} />}
      {nhapKq && (
        <Banner tone={nhapKq.errors.length ? "warning" : "success"} className="mx-4 mb-3" onClose={() => setNhapKq(null)} detail={nhapKq.errors.length ? nhapKq.errors.join("\n") : undefined}>
          {nhapKq.text}
        </Banner>
      )}
      <Table
        columns={cols}
        rows={rows}
        getRowId={(r) => r.id}
        dense
        maxHeight={420}
        rowActions={canEdit ? (r) => <Menu size="sm" items={[{ label: "Sửa", icon: "sua", onSelect: () => setForm({ row: r }) }, { label: "Xoá", icon: "xoa", danger: true, onSelect: () => void xoa(r) }]} /> : undefined}
        state={{ empty: { title: "Chưa có văn bản", description: "Thêm từng văn bản hoặc nhập mục lục văn bản từ Excel." } }}
      />
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
    </Card>
  );
}
