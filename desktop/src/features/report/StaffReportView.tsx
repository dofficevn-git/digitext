// Copyright (c) 2026 DigiText. All rights reserved.
// Báo cáo nhân viên (số hoá): theo dự án + khoảng ngày, mỗi người × công việc (quét, kiểm tra quét, nhập trường,
// kiểm tra nhập liệu, xác nhận, kiểm tra xác nhận): sản lượng giao / thi công, năng suất, % hoàn thành, lỗi, KPI.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { useProject } from "@/state/project";
import { reportSetTarget, reportStaff, type StaffRow, type StaffTask } from "@/api/progressReport";
import { dashboardStaff, type KpiSettings, type MetricKey, type StaffBoard } from "@/api/dashboard";
import { V4ak, type V4a9 } from "@/api/users";
import { V4e5 } from "@/features/dossier/excel";
import KpiSettingsDialog from "./KpiSettingsDialog";
import type { V3y0 } from "@/domain/types";
import { Badge, Button, Card, Dialog, Field, FormGrid, Help, Input, KpiRow, KpiTile, notifyError, PageHeader, Select, Sparkline, Table, Tip, toast, type Column } from "@/ui";

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

/** Ngưỡng KPI mặc định khi dự án chưa cài đặt (Định mức › Xếp loại KPI ghi đè theo dự án). */
export const KPI = { xuatSac: { hoanThanh: 100, loi: 1 }, dat: { hoanThanh: 80, loi: 3 } };
export type KpiNguong = typeof KPI;
export function nguongTu(s: KpiSettings | null | undefined): KpiNguong {
  return s ? { xuatSac: { hoanThanh: s.excellentPct, loi: s.excellentErrPct }, dat: { hoanThanh: s.passPct, loi: s.passErrPct } } : KPI;
}

export function kpiOf(hoanThanh: number | null, tyLeLoi: number, k: KpiNguong = KPI): { label: string; tone: V3y0 } {
  if (hoanThanh == null) return tyLeLoi > k.dat.loi ? { label: "Cảnh báo lỗi", tone: "err" } : { label: "Chưa giao sản lượng", tone: "gray" };
  if (hoanThanh >= k.xuatSac.hoanThanh && tyLeLoi <= k.xuatSac.loi) return { label: "Xuất sắc", tone: "ok" };
  if (hoanThanh >= k.dat.hoanThanh && tyLeLoi <= k.dat.loi) return { label: "Đạt", tone: "info" };
  return { label: "Cảnh báo lỗi", tone: "err" };
}

/** Công việc của báo cáo ↔ chỉ số theo ngày của dashboard (cột "14 ngày") và khoá định mức. */
const TASK_METRIC: Record<StaffTask, { metric: MetricKey; quota: string } | null> = {
  scan: { metric: "pagesScanned", quota: "pages_scanned" },
  kiem_tra_scan: null,
  nhap_lieu: { metric: "fieldsEdited", quota: "fields_edited" },
  kiem_tra_nhap_lieu: { metric: "workItemsCheckedOk", quota: "work_items_checked_ok" },
  xac_nhan: { metric: "workItemsDone", quota: "work_items_done" },
  kiem_tra_xac_nhan: null,
};

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
  const [dinhMuc, setDinhMuc] = useState(false);

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
  const boardLoader = useCallback(() => dashboardStaff(projectId, from || undefined, to || undefined), [projectId, from, to]);
  const board = V41t(boardLoader, { topics: ["dashboard"], enabled: !!projectId && (isTauri() || import.meta.env.DEV), errorText: () => "Không lấy được số theo ngày." });
  const bd: StaffBoard | undefined = projectId && board.data?.projectId === projectId ? board.data : undefined;
  const nguong = nguongTu(bd?.kpi);
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
      aoa.push([i + 1, d.displayName, d.username, t.label, t.unit, d.target ?? "", d.done, d.nangSuat ?? "", d.hoanThanh ?? "", d.errors, d.tyLeLoi, kpiOf(d.hoanThanh, d.tyLeLoi, nguong).label]);
    });
    void V4e5([{ name: "TongKetNhanVien", aoa }], `tong-ket-nhan-vien-${new Date().toISOString().slice(0, 10)}.xlsx`).then(() => toast.success("Đã xuất báo cáo nhân viên."));
  };

  const cols: Column<Dong>[] = [
    { key: "displayName", header: "Nhân viên", sortable: true, cell: (d) => <>{d.displayName} <span className="text-fg-subtle">({d.username})</span></> },
    { key: "task", header: "Công việc", sortable: true, sortValue: (d) => taskOf(d.task).label, cell: (d) => taskOf(d.task).label },
    { key: "target", header: "Giao", align: "right", width: 110, cell: (d) => (d.target != null ? `${n(d.target)} ${taskOf(d.task).unit}` : "—") },
    { key: "done", header: "Thi công", align: "right", width: 120, sortable: true, cell: (d) => (d.task === "nhap_lieu" ? <Tip text={`${n(d.fields)} trường`}><span>{n(d.done)} {taskOf(d.task).unit}</span></Tip> : `${n(d.done)} ${taskOf(d.task).unit}`) },
    { key: "nangSuat", header: "Năng suất / ngày", align: "right", width: 130, sortable: true, cell: (d) => (d.nangSuat != null ? n(d.nangSuat) : "—") },
    {
      key: "14ngay",
      header: "14 ngày",
      width: 110,
      cell: (d) => {
        const tm = TASK_METRIC[d.task];
        const u = bd?.users.find((x) => x.userId === d.userId);
        if (!tm || !u) return <span className="text-fg-subtle">—</span>;
        return <Sparkline values={u.series.slice(-14).map((s) => s[tm.metric])} refValue={bd?.kpi.quotas[tm.quota] ?? null} ariaLabel={`${d.displayName} 14 ngày`} />;
      },
    },
    { key: "hoanThanh", header: "Hoàn thành", align: "right", width: 110, sortable: true, cell: (d) => (d.hoanThanh != null ? `${d.hoanThanh}%` : "—") },
    { key: "errors", header: "Lỗi", align: "right", width: 70, sortable: true, cell: (d) => n(d.errors) },
    { key: "tyLeLoi", header: "Tỷ lệ lỗi", align: "right", width: 90, sortable: true, cell: (d) => `${d.tyLeLoi}%` },
    { key: "kpi", header: "KPI", width: 150, cell: (d) => { const k = kpiOf(d.hoanThanh, d.tyLeLoi, nguong); return <Badge tone={k.tone}>{k.label}</Badge>; } },
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
      <Card
        padding="none"
        title="Theo nhân viên và công việc"
        description={projectId ? "Bấm Giao ở cuối dòng để giao sản lượng cho người đó. Cột 14 ngày lấy từ tổng hợp theo ngày của máy chủ." : "Chọn một dự án để giao sản lượng."}
        actions={
          <>
            {projectId && (
              <Button size="sm" icon="caiDat" onClick={() => setDinhMuc(true)}>
                Định mức
              </Button>
            )}
            <Button size="sm" icon="tepExcel" onClick={xuat} disabled={dong.length === 0}>
              Xuất Excel
            </Button>
          </>
        }
      >
        <Table columns={cols} rows={dong} getRowId={(d) => `${d.userId}-${d.task}`} dense rowActions={projectId ? (d) => <Button size="sm" onClick={() => setGiao(d)}>Giao</Button> : undefined} state={{ loading: loading && !data, error, onRetry: reload, empty: { title: "Chưa có số liệu trong phạm vi này", icon: "nguoi" } }} />
      </Card>
      <Help title="Cách tính" className="mt-4">
        <p>Quét tính theo trang người đó quét, tệp đẩy lên máy chủ đứng tên người quét. Lỗi là tài liệu của người đó bị từ chối khi xác nhận hoặc bị kiểm tra chéo chấm có lỗi. Xác nhận dữ liệu và kiểm tra nhập liệu lấy từ gói việc ở màn Giao việc; kiểm tra quét và kiểm tra xác nhận chưa có nơi ghi nhận.</p>
        <p>
          KPI: Xuất sắc khi hoàn thành từ {nguong.xuatSac.hoanThanh}% và lỗi tối đa {nguong.xuatSac.loi}%; Đạt khi hoàn thành từ {nguong.dat.hoanThanh}% và lỗi tối đa {nguong.dat.loi}%; còn lại là Cảnh báo lỗi. {bd ? "Ngưỡng theo cài đặt của dự án (nút Định mức)." : "Ngưỡng mặc định; chọn dự án và bấm Định mức để đặt riêng."}
        </p>
      </Help>
      {giao && projectId && <GiaoDialog projectId={projectId} row={giao} onClose={() => setGiao(null)} onDone={reload} />}
      {dinhMuc && projectId && <KpiSettingsDialog projectId={projectId} projectLabel={projects.find((p) => p.id === projectId)?.name} onClose={() => setDinhMuc(false)} onSaved={() => board.reload()} />}
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
