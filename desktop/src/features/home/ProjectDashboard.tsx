// Copyright (c) 2026 DigiText. All rights reserved.
// Tổng quan MỘT DỰ ÁN (Quản trị dự án; Quản lý / Quản trị hệ thống khi đã chọn dự án): dây chuyền Chỉnh lý → Quét →
// Đưa lên → OCR → PDF/A → Ký số → Đóng gói → Nộp lưu, tiến độ so hợp đồng + dự kiến xong, sản lượng theo ngày, tồn đọng
// theo tuổi, nhân viên so định mức, SLA gói việc, cần xử lý, sức khoẻ hệ thống.

import { useCallback, useMemo, useState } from "react";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { dashboardProject, QUOTA_METRICS, type BacklogStageKey, type MetricKey, type ProjectDash } from "@/api/dashboard";
import { Badge, BarChart, Button, ChartLegend, CHART, KpiRow, KpiTile, Pipeline, Ring, Seg, Sparkline, StackedBar, Table, fmtDayFull, fmtNum, type Column } from "@/ui";
import KpiSettingsDialog from "@/features/report/KpiSettingsDialog";
import CanXuLyCard from "./blocks/CanXuLy";
import SucKhoeCard from "./blocks/SucKhoe";
import { DataCard, DaysSeg, RagBadge, duKienText, n, pct1, useSo, type Days } from "./blocks/common";
import { RatingBadge } from "./StaffDashboard";

type ThroughputMetric = "pagesScanned" | "pagesOcrDone" | "docsPdfa" | "docsReviewedApproved" | "docsSigned" | "workItemsDone";
const THROUGHPUT: { key: ThroughputMetric; label: string; unit: string; req?: "pagesScanned" | "docsPdfa" | "docsApproved" | "docsSigned" }[] = [
  { key: "pagesScanned", label: "Trang quét", unit: "trang", req: "pagesScanned" },
  { key: "pagesOcrDone", label: "Trang OCR xong", unit: "trang" },
  { key: "docsPdfa", label: "Tệp PDF/A", unit: "tệp", req: "docsPdfa" },
  { key: "docsSigned", label: "Tệp ký số", unit: "tệp", req: "docsSigned" },
  { key: "docsReviewedApproved", label: "Tệp duyệt bóc tách", unit: "tệp", req: "docsApproved" },
  { key: "workItemsDone", label: "Mục xác nhận", unit: "mục" },
];
const TUOI = ["≤ 3 ngày", "4–7 ngày", "8–14 ngày", "> 14 ngày"];

export default function ProjectDashboard({ projectId }: { projectId: string }) {
  const nav = V460();
  const { session, goto, gotoHoSo, gotoReport } = nav;
  const has = (p: Parameters<typeof V3z9>[1]) => V3z9(session?.permissions, p);
  const [days, setDays] = useState<Days>(30);
  const [tp, setTp] = useState<ThroughputMetric>("pagesScanned");
  const [dinhMuc, setDinhMuc] = useState(false);
  const load = useCallback(() => dashboardProject(projectId, days), [projectId, days]);
  const dash = useSo(load, ["dashboard", "work", "sign", "packages"], true);
  const d = dash.data;

  const chips = useMemo(() => {
    if (!d) return [];
    const p = d.pipeline;
    return [
      { label: "Hồ sơ chỉnh lý", value: p.clHoSo, sub: `${n(p.clVanBan)} văn bản`, run: () => gotoHoSo("laphoso") },
      { label: "Trang quét", value: p.pages, sub: `${n(p.files)} tệp đưa lên`, run: () => goto("upload") },
      { label: "Chờ OCR", value: p.ocrPendingDocs, sub: `${n(p.ocrPendingPages)} trang${p.ocrFailedPages ? ` · ${n(p.ocrFailedPages)} trang lỗi` : ""}`, run: () => gotoHoSo("ocr"), tone: p.ocrFailedPages ? ("err" as const) : undefined },
      { label: "Đã PDF/A", value: p.pdfaFiles, sub: `${n(p.ocrDoneDocs)} tệp OCR xong`, run: () => gotoHoSo("ocr") },
      { label: "Chờ duyệt bóc tách", value: p.pendingReviewFiles, sub: `${n(p.approvedFiles)} đã duyệt · ${n(p.rejectedFiles)} từ chối`, run: () => gotoHoSo("ocr") },
      { label: "Chờ ký số", value: p.toSign, sub: `${n(p.signedFiles)} đã ký${p.signFailed ? ` · ${n(p.signFailed)} lỗi` : ""}`, run: () => goto("sign"), tone: p.toSign > 0 ? ("warn" as const) : undefined },
      { label: "Gói tin đã dựng", value: p.packagesBuilt, sub: `${n(p.docsPackaged)} tài liệu`, run: () => goto("package") },
      { label: "Lần nộp lưu", value: p.subRegistered + p.subReceived + p.subCompleted, sub: `${n(p.subCompleted)} hoàn thành · ${n(p.subRegistered + p.subReceived)} đang xử lý`, run: () => goto("submission") },
    ];
  }, [d, goto, gotoHoSo]);

  const staffCols: Column<ProjectDash["staff"][number]>[] = [
    { key: "displayName", header: "Nhân viên", cell: (r) => <span className="font-medium">{r.displayName}</span> },
    { key: "today", header: "Hôm nay", align: "right", cell: (r) => homNayText(r.today, r.kpi.map((k) => k.metric)) },
    { key: "series", header: "14 ngày", width: 110, cell: (r) => <Sparkline values={r.series.slice(-14).map((s) => s[chinhMetric(r.kpi.map((k) => k.metric))])} refValue={r.kpi[0]?.quotaPerDay} ariaLabel={`${r.displayName} 14 ngày`} /> },
    { key: "dat", header: "Đạt định mức", align: "right", width: 110, cell: (r) => (r.kpi[0]?.attainmentPct == null ? "—" : `${Math.round(r.kpi[0].attainmentPct)} %`) },
    { key: "kpi", header: "KPI", width: 150, cell: (r) => (r.kpi[0] ? <RatingBadge r={r.kpi[0].rating} /> : <span className="text-fg-subtle">—</span>) },
  ];

  const header = d && (
    <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
      <h2 className="m-0 text-base font-semibold text-fg-strong">
        {d.code} · {d.name}
      </h2>
      <RagBadge rag={d.health.rag} />
      <span className="text-sm text-fg-muted">{duKienText(d.health, d.contract.deadlineOn ?? d.contract.endOn)}</span>
      <span className="ml-auto flex items-center gap-2">
        {has("project.update") && (
          <Button size="sm" icon="caiDat" onClick={() => setDinhMuc(true)}>
            Định mức
          </Button>
        )}
        <DaysSeg value={days} onChange={setDays} />
      </span>
    </div>
  );

  return (
    <div className="space-y-4">
      {header}
      <div className="grid gap-4 xl:grid-cols-12">
        <DataCard title="Dây chuyền xử lý" icon="xuLy" {...dash} className="xl:col-span-8" description="Bốn bước so với khối lượng hợp đồng; bấm một ô để mở màn tương ứng.">
          {(p: ProjectDash) => {
            const tf = p.contract.contractFiles ?? p.counts.files;
            return (
              <>
                <Pipeline
                  steps={[
                    { label: "Quét", done: p.counts.pages, total: p.contract.contractPages ?? 0, unit: "trang" },
                    { label: "PDF/A", done: p.counts.pdfaFiles, total: tf, unit: "tệp" },
                    { label: "Ký số", done: p.counts.signedFiles, total: tf, unit: "tệp" },
                    { label: "Duyệt bóc tách", done: p.counts.approvedFiles, total: tf, unit: "tệp" },
                  ]}
                  doneText="Đã xong khối lượng hợp đồng"
                />
                <div className="mt-4 grid grid-cols-2 gap-2 md:grid-cols-4">
                  {chips.map((c) => (
                    <KpiTile key={c.label} label={c.label} value={c.value} sub={c.sub} tone={c.tone} onClick={c.run} />
                  ))}
                </div>
                <div className="mt-4">
                  <StackedBar
                    ariaLabel="Tệp theo trạng thái"
                    segments={[
                      { key: "ocr", label: "Chờ OCR", value: p.pipeline.ocrPendingDocs },
                      { key: "pdfa", label: "Chờ PDF/A", value: Math.max(0, p.pipeline.ocrDoneDocs - p.pipeline.pdfaFiles) },
                      { key: "sign", label: "Chờ ký", value: p.pipeline.toSign },
                      { key: "signed", label: "Đã ký", value: p.pipeline.signedFiles },
                    ]}
                  />
                </div>
              </>
            );
          }}
        </DataCard>

        <DataCard title="Tiến độ so hợp đồng" icon="baoCao" {...dash} className="xl:col-span-4" open={() => gotoReport("sohoa")} openLabel="Mở báo cáo">
          {(p: ProjectDash) => (
            <div className="space-y-3">
              <div className="flex items-center justify-around">
                <div className="text-center">
                  <Ring value={p.health.progressPct} label="Hoàn thành" />
                  <div className="mt-1 text-sm">Hoàn thành</div>
                </div>
                <div className="text-center">
                  <Ring value={p.health.timePct} label="Thời gian đã qua" />
                  <div className="mt-1 text-sm">Thời gian đã qua</div>
                </div>
              </div>
              <dl className="grid grid-cols-2 gap-x-4 gap-y-1 text-sm">
                <dt className="text-fg-subtle">Hạn chót</dt>
                <dd className="font-medium">{fmtDayFull(p.contract.deadlineOn ?? p.contract.endOn)}</dd>
                <dt className="text-fg-subtle">Còn lại</dt>
                <dd className="font-medium">{p.health.daysRemaining == null ? "—" : p.health.daysRemaining < 0 ? `quá hạn ${n(-p.health.daysRemaining)} ngày` : `${n(p.health.daysRemaining)} ngày`}</dd>
                <dt className="text-fg-subtle">Lệch tiến độ</dt>
                <dd className="font-medium">{p.health.lech == null ? "—" : `${p.health.lech > 0 ? "+" : ""}${Math.round(p.health.lech)} điểm`}</dd>
                <dt className="text-fg-subtle">Quét: đang / cần mỗi ngày</dt>
                <dd className="font-medium tabular-nums">
                  {fmtNum(Math.round(p.health.runRate7d.pagesScanned))} / {fmtNum(Math.round(p.health.requiredRate.pagesScanned))} trang
                </dd>
                <dt className="text-fg-subtle">Ký số: đang / cần mỗi ngày</dt>
                <dd className="font-medium tabular-nums">
                  {fmtNum(Math.round(p.health.runRate7d.docsSigned))} / {fmtNum(Math.round(p.health.requiredRate.docsSigned))} tệp
                </dd>
              </dl>
              {!p.contract.signedOn && <p className="text-xs text-warning-700">Chưa nhập ngày ký và hạn hợp đồng — vào Báo cáo › Tiến độ dự án › Hợp đồng để nhập.</p>}
            </div>
          )}
        </DataCard>
      </div>

      <div className="grid gap-4 xl:grid-cols-12">
        <DataCard
          title="Sản lượng theo ngày"
          icon="thongke"
          {...dash}
          className="xl:col-span-8"
          actions={<Seg<ThroughputMetric> value={tp} onChange={setTp} options={THROUGHPUT.slice(0, 4).map((t) => ({ value: t.key, label: t.label }))} />}
        >
          {(p: ProjectDash) => {
            const def = THROUGHPUT.find((t) => t.key === tp)!;
            const req = def.req ? p.health.requiredRate[def.req] : 0;
            return <BarChart data={p.series.map((s) => ({ label: s.day, value: s[tp] }))} unit={def.unit} height={190} target={req > 0 ? { value: Math.round(req), label: "Cần/ngày" } : null} ariaLabel={`${def.label} ${p.days} ngày`} />;
          }}
        </DataCard>

        <DataCard title="Tồn đọng theo tuổi" icon="dangCho" {...dash} className="xl:col-span-4" description="Số tài liệu đang chờ ở mỗi bước, theo số ngày đã chờ.">
          {(p: ProjectDash) => (
            <div className="space-y-3">
              <ChartLegend items={TUOI.map((t, i) => ({ label: t, color: CHART.seq[i + 1] }))} />
              {p.backlog.map((b) => {
                const tong = b.buckets.reduce((a, x) => a + x, 0);
                const run: Record<BacklogStageKey, () => void> = { ocr: () => gotoHoSo("ocr"), review: () => gotoHoSo("ocr"), sign: () => goto("sign"), package: () => goto("package"), submit: () => goto("submission") };
                return (
                  <button key={b.stage} type="button" className="block w-full rounded px-1 py-1 text-left hover:bg-surface-hover" onClick={run[b.stage]}>
                    <div className="mb-1 flex justify-between text-sm">
                      <span>{b.label}</span>
                      <span className="tabular-nums text-fg-muted">{n(tong)}</span>
                    </div>
                    <StackedBar ariaLabel={b.label} legend={false} height={10} segments={b.buckets.map((v, i) => ({ key: String(i), label: TUOI[i], value: v, color: CHART.seq[i + 1] }))} />
                  </button>
                );
              })}
            </div>
          )}
        </DataCard>
      </div>

      <div className="grid gap-4 xl:grid-cols-12">
        <DataCard
          title="Nhân viên so định mức"
          icon="thanhVien"
          {...dash}
          className="xl:col-span-8"
          padding="none"
          open={() => gotoReport("nhanvien")}
          openLabel="Xem báo cáo"
          description={d?.kpi && Object.keys(d.kpi.quotas).length === 0 ? "Chưa đặt định mức — bấm Định mức để đặt, KPI sẽ tính theo đó." : undefined}
        >
          {(p: ProjectDash) => <Table columns={staffCols} rows={p.staff.slice(0, 8)} getRowId={(r) => r.userId} dense state={{ empty: { title: "Chưa có sản lượng nhân viên trong kỳ", icon: "nguoi" } }} />}
        </DataCard>

        <DataCard title="Gói việc" icon="kiemTra" {...dash} className="xl:col-span-4" open={() => goto("workManage")}>
          {(p: ProjectDash) => (
            <div className="space-y-3">
              <KpiRow>
                <KpiTile label="Đang mở" value={p.workSla.open} />
                <KpiTile label="Quá hạn" value={p.workSla.overdue} tone={p.workSla.overdue > 0 ? "err" : undefined} />
                <KpiTile label="Đến hạn 3 ngày" value={p.workSla.dueSoon3d} tone={p.workSla.dueSoon3d > 0 ? "warn" : undefined} />
                <KpiTile label="Đúng hạn 30 ngày" value={p.workSla.closed30d ? `${Math.round((p.workSla.closedOnTime30d / p.workSla.closed30d) * 100)} %` : "—"} sub={`${n(p.workSla.closedOnTime30d)}/${n(p.workSla.closed30d)} gói đóng`} />
              </KpiRow>
              <p className="text-sm text-fg-muted">
                {n(p.workSla.pendingItems)} mục chờ làm{p.workSla.itemsOverSla ? ` · ${n(p.workSla.itemsOverSla)} mục quá SLA ${p.kpi.workItemSlaHours} giờ` : ""}
                {p.workSla.avgTurnaroundHours != null ? ` · trung bình ${fmtNum(Math.round(p.workSla.avgTurnaroundHours * 10) / 10)} giờ/mục` : ""}
              </p>
              {p.overduePackages.length > 0 && (
                <ul className="divide-y divide-border-subtle text-sm">
                  {p.overduePackages.map((o) => (
                    <li key={o.packageId} className="flex items-center gap-2 py-1.5">
                      <span className="flex-1 truncate">{o.name}</span>
                      <Badge tone="err">quá hạn {fmtDayFull(o.dueAt)}</Badge>
                      <span className="tabular-nums text-fg-muted">{n(o.pendingItems)} mục</span>
                    </li>
                  ))}
                </ul>
              )}
            </div>
          )}
        </DataCard>
      </div>

      <div className="grid gap-4 xl:grid-cols-12">
        <CanXuLyCard className="xl:col-span-5" />
        {has("bk.view") && <SucKhoeCard className="xl:col-span-7" />}
      </div>
      {dinhMuc && d && <KpiSettingsDialog projectId={projectId} projectLabel={`${d.code} · ${d.name}`} onClose={() => setDinhMuc(false)} onSaved={() => dash.reload()} />}
      <p className="text-xs text-fg-subtle">{pct1(d?.health.progressPct)} hoàn thành · số theo ngày cập nhật mỗi 2 phút, số hiện tại mỗi 60 giây.</p>
    </div>
  );
}

/** Chỉ số chính của một người: chỉ số đầu tiên có định mức / sản lượng, mặc định trang quét. */
function chinhMetric(keys: string[]): MetricKey {
  return (QUOTA_METRICS.find((q) => q.key === keys[0])?.metric ?? "pagesScanned") as MetricKey;
}
function homNayText(today: ProjectDash["staff"][number]["today"], keys: string[]): string {
  const q = QUOTA_METRICS.find((x) => x.key === keys[0]);
  if (!q) return "—";
  return `${n(today[q.metric])} ${q.unit}`;
}
