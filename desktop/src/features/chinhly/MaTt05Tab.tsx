// Copyright (c) 2026 DigiText. All rights reserved.
// Tab "Mã TT05" của màn Danh mục (0040, đợt 6 — Digifile B03): chế độ sử dụng, mức độ tin cậy, tình trạng vật lý, tình trạng
// dự phòng. Dòng dùng chung (chuẩn) chỉ super admin sửa; dự án thêm mã riêng. Chỉ trực tuyến.

import { useCallback, useState } from "react";
import { isTauri } from "@/api/ipc";
import { maTt05List, maTt05Sua, maTt05Them, maTt05Xoa, NHOM_MA, type MaTt05, type MaTt05Moi, type NhomMa } from "@/api/maTt05";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { Badge, Banner, Button, Card, Checkbox, confirm, Dialog, ErrorBanner, Field, FormGrid, Input, Menu, notifyError, Select, Table, Tip, toast, Toolbar, type Column } from "@/ui";

export default function MaTt05Tab({ canUpdate }: { canUpdate: boolean }) {
  const { session } = V460();
  const isSuper = session?.role === "super_admin";
  const { projectId } = useProject();
  const duAn = session?.projectId ?? projectId ?? null;
  const trucTuyen = isTauri() && session?.mode !== "offline";
  const [nhom, setNhom] = useState<NhomMa>("che_do_su_dung");
  const [form, setForm] = useState<{ row: MaTt05 | null } | null>(null);
  const load = useCallback(() => (duAn ? maTt05List(duAn).then((r) => r.rows) : Promise.resolve([] as MaTt05[])), [duAn]);
  const { data, error, loading, reload } = V41t(load, { topics: ["chinhLy"], enabled: trucTuyen && !!duAn });
  const rows = (data ?? []).filter((r) => r.nhom === nhom);
  const chung = (r: MaTt05) => r.projectId === null;

  const xoa = async (r: MaTt05) => {
    const ok = await confirm({ title: `Xoá mã ${r.ma} · ${r.ten}?`, body: "Hồ sơ, văn bản đã dùng mã này vẫn giữ giá trị cũ.", confirmLabel: "Xoá", danger: true });
    if (!ok) return;
    try {
      await maTt05Xoa(r.id);
      reload();
    } catch (e) {
      notifyError(e, "xoá mã");
    }
  };

  const cols: Column<MaTt05>[] = [
    { key: "_tt", header: "", width: 90, cell: (r) => (chung(r) ? <Tip text="Mã chuẩn TT05, dùng chung mọi kho"><span><Badge tone="info">Chuẩn TT05</Badge></span></Tip> : <Badge tone="gray">Riêng</Badge>) },
    { key: "ma", header: "Mã", width: 80, sortable: true },
    { key: "ten", header: "Tên", sortable: true },
    { key: "soTt", header: "Thứ tự", width: 80, align: "right", cell: (r) => r.soTt ?? "" },
    { key: "ghiChu", header: "Ghi chú", className: "whitespace-normal text-sm", cell: (r) => r.ghiChu ?? "" },
  ];

  if (!trucTuyen) return <Banner tone="info" className="mt-3">Bảng mã TT05 chỉ xem và sửa khi nối máy chủ.</Banner>;
  return (
    <>
      <Toolbar className="mt-3">
        <Select value={nhom} onChange={(v) => setNhom(v as NhomMa)} options={NHOM_MA.map(([value, label]) => ({ value, label }))} className="w-60" aria-label="Nhóm mã" />
        <Toolbar.Spacer />
        {canUpdate && duAn && (
          <Button icon="them" onClick={() => setForm({ row: null })}>
            Thêm mã
          </Button>
        )}
      </Toolbar>
      {error && <ErrorBanner error={error} className="mt-3" onRetry={reload} />}
      <Card className="mt-3" padding="none">
        <Table
          columns={cols}
          rows={rows}
          getRowId={(r) => r.id}
          dense
          rowActions={(r) => {
            const locked = chung(r) && !isSuper;
            if (locked || !canUpdate) return null;
            return <Menu size="sm" items={[{ label: "Sửa", icon: "sua", onSelect: () => setForm({ row: r }) }, { label: "Xoá", icon: "xoa", danger: true, onSelect: () => void xoa(r) }]} />;
          }}
          state={{ loading: loading && !data, empty: { title: "Chưa có mã trong nhóm này" } }}
        />
      </Card>
      {form && duAn && (
        <FormDialog
          projectId={duAn}
          nhom={nhom}
          row={form.row}
          isSuper={isSuper}
          onClose={() => setForm(null)}
          onSaved={() => {
            setForm(null);
            reload();
          }}
        />
      )}
    </>
  );
}

function FormDialog({ projectId, nhom, row, isSuper, onClose, onSaved }: { projectId: string; nhom: NhomMa; row: MaTt05 | null; isSuper: boolean; onClose: () => void; onSaved: () => void }) {
  const [f, setF] = useState<MaTt05Moi>({ nhom: row?.nhom ?? nhom, ma: row?.ma ?? "", ten: row?.ten ?? "", soTt: row?.soTt ?? null, ghiChu: row?.ghiChu ?? "" });
  const [dungChung, setDungChung] = useState(false);
  const [busy, setBusy] = useState(false);
  const ok = f.ma.trim() && f.ten.trim();
  const luu = async () => {
    if (!ok || busy) return;
    setBusy(true);
    try {
      if (row) await maTt05Sua(row.id, f);
      else await maTt05Them(projectId, f, dungChung);
      toast.success("Đã lưu mã.");
      onSaved();
    } catch (e) {
      notifyError(e, "lưu mã TT05");
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      title={row ? `Sửa mã ${row.ma}` : `Thêm mã · ${NHOM_MA.find(([k]) => k === nhom)?.[1] ?? ""}`}
      size="sm"
      footer={
        <>
          <Button onClick={onClose} disabled={busy}>
            Huỷ
          </Button>
          <Button variant="primary" icon="luu" loading={busy} disabled={!ok} onClick={() => void luu()}>
            Lưu
          </Button>
        </>
      }
    >
      <FormGrid cols={2}>
        <Field label="Mã" required>
          <Input value={f.ma} onChange={(e) => setF((o) => ({ ...o, ma: e.target.value }))} autoFocus />
        </Field>
        <Field label="Thứ tự">
          <Input type="number" value={f.soTt ?? ""} onChange={(e) => setF((o) => ({ ...o, soTt: e.target.value ? Number(e.target.value) : null }))} />
        </Field>
        <Field label="Tên" required className="col-span-2">
          <Input value={f.ten} onChange={(e) => setF((o) => ({ ...o, ten: e.target.value }))} />
        </Field>
        <Field label="Ghi chú" className="col-span-2">
          <Input value={f.ghiChu ?? ""} onChange={(e) => setF((o) => ({ ...o, ghiChu: e.target.value }))} />
        </Field>
        {!row && isSuper && (
          <Field label="Dùng chung mọi kho" inline className="col-span-2">
            <Checkbox checked={dungChung} onChange={setDungChung} />
          </Field>
        )}
      </FormGrid>
    </Dialog>
  );
}
