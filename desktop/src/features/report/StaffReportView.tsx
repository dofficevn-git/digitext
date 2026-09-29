// Copyright (c) 2026 DigiText. All rights reserved.
// Báo cáo nhân viên (số hoá): theo dự án + khoảng ngày, mỗi người × công việc (quét, kiểm tra quét, nhập trường,
// kiểm tra nhập liệu, xác nhận, kiểm tra xác nhận): sản lượng giao / thi công, năng suất, % hoàn thành, lỗi, KPI.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { useProject } from "@/state/project";
import { reportSetTarget, reportStaff, type StaffRow, type StaffTask } from "@/api/progressReport";
import { V4ak, type V4a9 } from "@/api/users";
import { V4e5 } from "@/features/dossier/excel";
import type { V3y0 } from "@/domain/types";
import { Badge, Button, Card, Dialog, Field, FormGrid, Help, Input, KpiRow, KpiTile, notifyError, PageHeader, Select, Table, Tip, toast, type Column } from "@/ui";

const n = (v: number) => v.toLocaleString("vi-VN");

export const TASKS: { key: StaffTask; label: string; unit: string }[] = [
  { key: "scan", label: "Quét tài liệu", unit: "trang" },
  { key: "kiem_tra_scan", label: "Kiểm tra quét", unit: "tài liệu" },
  { key: "nhap_lieu", label: "Nhập trường thông tin", unit: "tài liệu" },
  { key: "kiem_tra_nhap_lieu", label: "Kiểm tra nhập liệu", unit: "tài liệu" },
  { key: "xac_nhan", label: "Xác nhận dữ liệu", unit: "tài liệu" },
  { key: "kiem_tra_xac_nhan", label: "Kiểm tra xác nhận", unit: "tài liệu" },
];
const taskOf = (k: StaffTask) => TASKS.find((t) => t.key === k) ?? TASKS[0];

/** Ngưỡng KPI mặc định, cần bên dùng xác nhận. */
export const KPI = { xuatSac: { hoanThanh: 100, loi: 1 }, dat: { hoanThanh: 80, loi: 3 } };

export function kpiOf(hoanThanh: number | null, tyLeLoi: number): { label: string; tone: V3y0 } {
  if (hoanThanh == null) return tyLeLoi > KPI.dat.loi ? { label: "Cảnh báo lỗi", tone: "err" } : { label: "Chưa giao sản lượng", tone: "gray" };
  if (hoanThanh >= KPI.xuatSac.hoanThanh && tyLeLoi <= KPI.xuatSac.loi) return { label: "Xuất sắc", tone: "ok" };
  if (hoanThanh >= KPI.dat.hoanThanh && tyLeLoi <= KPI.dat.loi) return { label: "Đạt", tone: "info" };
  return { label: "Cảnh báo lỗi", tone: "err" };
}

interface Dong extends StaffRow {
  hoanThanh: number | null;
  tyLeLoi: number;
  nangSuat: number | null;
}

function tinh(r: StaffRow): Dong {
  const coSo = r.task === "scan" ? r.files : r.done;
  const tyLeLoi = coSo > 0 ? Math.round((r.errors / coSo) * 1000) / 10 : 0;
  return { ...r, hoanThanh: r.target && r.target > 0 ? Math.floor((r.done / r.target) * 100) : null, tyLeLoi, nangSuat: r.days > 0 ? Math.round(r.done / r.days) : null };
}

function GiaoDialog({ projectId, row, onClose, onDone }: { projectId: string; row: Dong; onClose: () => void; onDone: () => void }) {
  const [task, setTask] = useState<StaffTask>(row.task);
  const [qty, setQty] = useState(row.target != null ? String(row.target) : "");
  const [busy, setBusy] = useState(false);
  const luu = async () => {
    setBusy(true);
    try {
      const q = qty.trim() === "" ? null : Math.max(0, Math.floor(Number(qty)) || 0);
      await reportSetTarget({ projectId, userId: row.userId, task, quantity: q });
      toast.success(q == null ? "Đã bỏ giao sản lượng." : `Đã giao ${n(q)} ${taskOf(task).unit}.`);
      onDone();
      onClose();
    } catch (e) {
      notifyError(e, "giao sản lượng");
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      closable={!busy}
      title="Giao sản lượng"
      description={row.displayName || row.username}
      size="sm"
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
      <div className="space-y-3">
        <Field label="Công việc">
          <Select value={task} onChange={(v) => setTask(v as StaffTask)} options={TASKS.map((t) => ({ value: t.key, label: `${t.label} (${t.unit})` }))} />
        </Field>
        <Field label="Số lượng giao" hint="Để trống để bỏ giao">
          <Input type="number" min={0} value={qty} onChange={(e) => setQty(e.target.value)} autoFocus />
        </Field>
      </div>
    </Dialog>
  );
}

export default function StaffReportView({ embedded }: { embedded?: boolean }) {
  const duAn = useProject();
  const [projects, setProjects] = useState<V4a9[]>([]);
  const [projectId, setProjectId] = useState("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [giao, setGiao] = useState<Dong | null>(null);

  useEffect(() => {
    if (!isTauri()) return;
    V4ak()
      .then((p) => setProjects(p.filter((x) => x.isActive)))
      .catch(() => {});
  }, []);
  useEffect(() => {
    if (!projectId && duAn.projectId) setProjectId(duAn.projectId);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [duAn.projectId]);

  const loader = useCallback(() => (isTauri() ? reportStaff({ projectId: projectId || undefined, from: from || undefined, to: to || undefined }) : Promise.resolve([] as StaffRow[])), [projectId, from, to]);
  const { data, error, loading, reload } = V41t(loader, { topics: ["work", "docs"], errorText: () => "Không lấy được số liệu nhân viên." });
  const dong = useMemo(() => (data ?? []).map(tinh), [data]);
  const tong = useMemo(
    () => ({
      trang: dong.filter((d) => d.task === "scan").reduce((a, d) => a + d.done, 0),
      xacNhan: dong.filter((d) => d.task === "xac_nhan").reduce((a, d) => a + d.done, 0),
      truong: dong.filter((d) => d.task === "nhap_lieu").reduce((a, d) => a + d.fields, 0),
    }),
    [dong],
  );

  const xuat = () => {
    const aoa: (string | number)[][] = [["STT", "Nhân viên", "Tài khoản", "Công việc", "Đơn vị", "Sản lượng giao", "Sản lượng thi công", "Năng suất TB/ngày", "% hoàn thành", "Số lỗi", "Tỷ lệ lỗi (%)", "Đánh giá KPI"]];
    dong.forEach((d, i) => {
      const t = taskOf(d.task);
      aoa.push([i + 1, d.displayName, d.username, t.label, t.unit, d.target ?? "", d.done, d.nangSuat ?? "", d.hoanThanh ?? "", d.errors, d.tyLeLoi, kpiOf(d.hoanThanh, d.tyLeLoi).label]);
    });
    void V4e5([{ name: "TongKetNhanVien", aoa }], `tong-ket-nhan-vien-${new Date().toISOString().slice(0, 10)}.xlsx`).then(() => toast.success("Đã xuất báo cáo nhân viên."));
  };

  const cols: Column<Dong>[] = [
    { key: "displayName", header: "Nhân viên", sortable: true, cell: (d) => <>{d.displayName} <span className="text-fg-subtle">({d.username})</span></> },
    { key: "task", header: "Công việc", sortable: true, sortValue: (d) => taskOf(d.task).label, cell: (d) => taskOf(d.task).label },
    { key: "target", header: "Giao", align: "right", width: 110, cell: (d) => (d.target != null ? `${n(d.target)} ${taskOf(d.task).unit}` : "—") },
    { key: "done", header: "Thi công", align: "right", width: 120, sortable: true, cell: (d) => (d.task === "nhap_lieu" ? <Tip text={`${n(d.fields)} trường`}><span>{n(d.done)} {taskOf(d.task).unit}</span></Tip> : `${n(d.done)} ${taskOf(d.task).unit}`) },
    { key: "nangSuat", header: "Năng suất / ngày", align: "right", width: 130, sortable: true, cell: (d) => (d.nangSuat != null ? n(d.nangSuat) : "—") },
    { key: "hoanThanh", header: "Hoàn thành", align: "right", width: 110, sortable: true, cell: (d) => (d.hoanThanh != null ? `${d.hoanThanh}%` : "—") },
    { key: "errors", header: "Lỗi", align: "right", width: 70, sortable: true, cell: (d) => n(d.errors) },
    { key: "tyLeLoi", header: "Tỷ lệ lỗi", align: "right", width: 90, sortable: true, cell: (d) => `${d.tyLeLoi}%` },
    { key: "kpi", header: "KPI", width: 150, cell: (d) => { const k = kpiOf(d.hoanThanh, d.tyLeLoi); return <Badge tone={k.tone}>{k.label}</Badge>; } },
  ];

  const body = (
    <>
      <Card className="mb-4" padding="sm">
        <FormGrid cols={4}>
          <Field label="Dự án">
            <Select size="sm" value={projectId} onChange={setProjectId} placeholder="Tất cả dự án" options={projects.map((p) => ({ value: p.id, label: `${p.code} · ${p.name}` }))} />
          </Field>
          <Field label="Từ ngày">
            <Input size="sm" type="date" value={from} max={to || undefined} onChange={(e) => setFrom(e.target.value)} />
          </Field>
          <Field label="Đến ngày">
            <Input size="sm" type="date" value={to} min={from || undefined} onChange={(e) => setTo(e.target.value)} />
          </Field>
        </FormGrid>
        <KpiRow className="mt-3">
          <KpiTile label="Trang đã quét" value={tong.trang} />
          <KpiTile label="Tệp đã xác nhận" value={tong.xacNhan} />
          <KpiTile label="Trường đã nhập" value={tong.truong} />
        </KpiRow>
      </Card>
      <Card padding="none" title="Theo nhân viên và công việc" description={projectId ? "Bấm Giao ở cuối dòng để giao sản lượng cho người đó." : "Chọn một dự án để giao sản lượng."} actions={<Button size="sm" icon="tepExcel" onClick={xuat} disabled={dong.length === 0}>Xuất Excel</Button>}>
        <Table columns={cols} rows={dong} getRowId={(d) => `${d.userId}-${d.task}`} dense rowActions={projectId ? (d) => <Button size="sm" onClick={() => setGiao(d)}>Giao</Button> : undefined} state={{ loading: loading && !data, error, onRetry: reload, empty: { title: "Chưa có số liệu trong phạm vi này", icon: "nguoi" } }} />
      </Card>
      <Help title="Cách tính" className="mt-4">
        <p>Quét tính theo trang người đó quét, tệp đẩy lên máy chủ đứng tên người quét. Lỗi là tài liệu của người đó bị từ chối khi xác nhận hoặc bị kiểm tra chéo chấm có lỗi. Xác nhận dữ liệu và kiểm tra nhập liệu lấy từ gói việc ở màn Giao việc; kiểm tra quét và kiểm tra xác nhận chưa có nơi ghi nhận.</p>
        <p>
          KPI: Xuất sắc khi hoàn thành từ {KPI.xuatSac.hoanThanh}% và lỗi tối đa {KPI.xuatSac.loi}%; Đạt khi hoàn thành từ {KPI.dat.hoanThanh}% và lỗi tối đa {KPI.dat.loi}%; còn lại là Cảnh báo lỗi.
        </p>
      </Help>
      {giao && projectId && <GiaoDialog projectId={projectId} row={giao} onClose={() => setGiao(null)} onDone={reload} />}
    </>
  );
  if (embedded) return body;
  return (
    <section className="view active">
      <PageHeader title="Báo cáo nhân viên" description="Sản lượng, năng suất, lỗi và KPI theo từng người, số liệu trên máy chủ." />
      {body}
    </section>
  );
}
