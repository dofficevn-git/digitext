// Copyright (c) 2026 DigiText. All rights reserved.
// Tổng quan LÃNH ĐẠO (Quản trị hệ thống / Quản lý): mọi dự án trong một màn — số dẫn đầu, bảng sức khoẻ dự án (tình
// trạng, hoàn thành so thời gian, dự kiến xong, 14 ngày), sản lượng theo dự án, rủi ro hàng đầu, cần xử lý, sức khoẻ
// hệ thống. Bấm "Mở" một dự án → chuyển sang Tổng quan của dự án đó.

import { useCallback, useMemo, useState } from "react";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { dashboardOverview, type Overview, type OverviewProject } from "@/api/dashboard";
import { V4e5 } from "@/features/dossier/excel";
import { Icon } from "@/components/Icon";
import { Button, KpiRow, KpiTile, LineChart, Seg, Sparkline, Table, fmtDayFull, fmtNum, toast, type Column } from "@/ui";
import CanXuLyCard from "./blocks/CanXuLy";
import SucKhoeCard from "./blocks/SucKhoe";
import { DataCard, DaysSeg, RagBadge, duKienText, n, soVoiKyTruoc, useSo, type Days } from "./blocks/common";

type Metric = "pagesScanned" | "docsPdfa" | "docsSigned" | "docsReviewedApproved";
const METRICS: { key: Metric; label: string; unit: string }[] = [
  { key: "pagesScanned", label: "Trang quét", unit: "trang" },
  { key: "docsPdfa", label: "Tệp PDF/A", unit: "tệp" },
  { key: "docsSigned", label: "Tệp ký số", unit: "tệp" },
  { key: "docsReviewedApproved", label: "Tệp duyệt", unit: "tệp" },
];

export default function LeaderDashboard({ onOpenProject }: { onOpenProject: (projectId: string) => void }) {
  const { session, goto, gotoReport } = V460();
  const has = (p: Parameters<typeof V3z9>[1]) => V3z9(session?.permissions, p);
  const [days, setDays] = useState<Days>(30);
  const [metric, setMetric] = useState<Metric>("pagesScanned");
  const [loc, setLoc] = useState<"all" | "late">("all");
  const load = useCallback(() => dashboardOverview(days), [days]);
  const ov = useSo(load, ["dashboard"], true, 120_000);
  const d = ov.data;

  const rows = useMemo(() => (d ? d.projects.filter((p) => loc === "all" || p.health.rag === "red" || p.health.rag === "amber") : []), [d, loc]);

  const cols: Column<OverviewProject>[] = [
    { key: "code", header: "Dự án", cell: (p) => (
        <span className="block min-w-0">
          <span className="block font-medium">{p.code}</span>
          <span className="block truncate text-xs text-fg-subtle">{p.name}</span>
        </span>
      ) },
    { key: "rag", header: "Tình trạng", width: 150, cell: (p) => <RagBadge rag={p.health.rag} /> },
    { key: "progress", header: "Hoàn thành", width: 150, align: "right", sortable: true, sortValue: (p) => p.health.progressPct ?? -1, cell: (p) => (
        <span className="block">
          <span className="tabular-nums">{p.health.progressPct == null ? "—" : `${Math.round(p.health.progressPct)} %`}</span>
          <span className="mt-1 block h-0.5 w-full rounded bg-border-subtle">
            <span className="block h-0.5 rounded bg-brand-500" style={{ width: `${Math.min(100, p.health.progressPct ?? 0)}%` }} />
          </span>
        </span>
      ) },
    { key: "time", header: "Thời gian đã qua", width: 120, align: "right", sortable: true, sortValue: (p) => p.health.timePct ?? -1, cell: (p) => (p.health.timePct == null ? "—" : `${Math.round(p.health.timePct)} %`) },
    { key: "eta", header: "Dự kiến xong", width: 170, cell: (p) => <span className="text-sm">{p.health.projectedFinishOn ? fmtDayFull(p.health.projectedFinishOn) : "Chưa đủ số liệu"}</span> },
    { key: "trend", header: "Trang quét 14 ngày", width: 120, cell: (p) => <Sparkline values={p.trend.slice(-14).map((t) => t.pagesScanned)} ariaLabel={`${p.code} trang quét 14 ngày`} /> },
    { key: "deadline", header: "Hạn chót", width: 110, cell: (p) => fmtDayFull(p.contract.deadlineOn ?? p.contract.endOn) },
  ];

  const xuat = () => {
    if (!d) return;
    const aoa: (string | number)[][] = [["STT", "Mã dự án", "Tên dự án", "Tình trạng", "% hoàn thành", "% thời gian đã qua", "Lệch (điểm)", "Dự kiến xong", "Hạn chót", "Trang đã quét", "Tệp", "Tệp PDF/A", "Tệp ký số", "Tệp đã duyệt", "Trang quét/ngày (7 ngày)", "Cần trang/ngày"]];
    const rag = { green: "Đúng tiến độ", amber: "Có rủi ro", red: "Chậm tiến độ", grey: "Chưa có hợp đồng" };
    d.projects.forEach((p, i) => aoa.push([i + 1, p.code, p.name, rag[p.health.rag], p.health.progressPct ?? "", p.health.timePct ?? "", p.health.lech ?? "", fmtDayFull(p.health.projectedFinishOn), fmtDayFull(p.contract.deadlineOn ?? p.contract.endOn), p.counts.pages, p.counts.files, p.counts.pdfaFiles, p.counts.signedFiles, p.counts.approvedFiles, Math.round(p.health.runRate7d.pagesScanned), Math.round(p.health.requiredRate.pagesScanned)]));
    void V4e5([{ name: "SucKhoeDuAn", aoa }], `suc-khoe-du-an-${d.today}.xlsx`).then(() => toast.success("Đã xuất bảng sức khoẻ dự án."));
  };

  return (
    <div className="space-y-4">
      <KpiRow className="grid grid-cols-2 gap-3 md:grid-cols-4">
        <KpiTile size="lg" icon="duAn" label="Dự án đang chạy" value={d?.totals.projects} loading={ov.loading} error={ov.error} />
        <KpiTile size="lg" icon="loi" label="Chậm tiến độ" value={d?.totals.late} tone={d && d.totals.late > 0 ? "err" : undefined} sub={d ? `${n(d.totals.atRisk)} có rủi ro` : undefined} onClick={() => setLoc((v) => (v === "late" ? "all" : "late"))} loading={ov.loading} />
        <KpiTile size="lg" icon="quet" label="Trang quét 7 ngày" value={d?.totals.pages7d} sub={d ? soVoiKyTruoc(d.totals.pages7d, d.totals.pages7dPrev) : undefined} loading={ov.loading} />
        <KpiTile size="lg" icon="kySo" label="Tệp đã ký số" value={d?.totals.submittedFiles} sub="Toàn bộ dự án" loading={ov.loading} />
      </KpiRow>

      <DataCard
        title="Sức khoẻ dự án"
        icon="duAn"
        {...ov}
        padding="none"
        description="Hoàn thành = trung bình 4 bước quét, PDF/A, ký số, duyệt so khối lượng hợp đồng. Dự kiến theo tốc độ 7 ngày gần nhất."
        actions={
          <>
            <Seg<"all" | "late"> value={loc} onChange={setLoc} options={[{ value: "all", label: "Tất cả" }, { value: "late", label: "Cần chú ý" }]} />
            <Button size="sm" icon="tepExcel" onClick={xuat} disabled={!d}>
              Xuất Excel
            </Button>
          </>
        }
      >
        {() => (
          <Table
            columns={cols}
            rows={rows}
            getRowId={(p) => p.projectId}
            dense
            rowActions={(p) => (
              <Button size="sm" variant="ghost" iconRight="sau" onClick={() => onOpenProject(p.projectId)}>
                Mở
              </Button>
            )}
            renderExpanded={(p) => <p className="px-3 py-2 text-sm text-fg-muted">{duKienText(p.health, p.contract.deadlineOn ?? p.contract.endOn)} · {n(p.counts.files)} tệp · {n(p.counts.pendingReviewFiles)} chờ duyệt · {p.contract.managerName ? `phụ trách ${p.contract.managerName}` : "chưa ghi cán bộ phụ trách"}</p>}
            state={{ empty: { title: loc === "late" ? "Không có dự án nào cần chú ý" : "Chưa có dự án nào trong phạm vi" } }}
          />
        )}
      </DataCard>

      <div className="grid gap-4 xl:grid-cols-12">
        <DataCard
          title="Sản lượng theo dự án"
          icon="thongke"
          {...ov}
          className="xl:col-span-8"
          actions={
            <>
              <Seg<Metric> value={metric} onChange={setMetric} options={METRICS.map((m) => ({ value: m.key, label: m.label }))} />
              <DaysSeg value={days} onChange={setDays} />
            </>
          }
        >
          {(o: Overview) => {
            const def = METRICS.find((m) => m.key === metric)!;
            const labels = o.throughput.map((t) => t.day);
            // ≤ 4 dự án: mỗi dự án một đường (màu theo dự án, thứ tự cố định); nhiều hơn: tổng toàn hệ thống.
            const series = o.projects.length <= 4 && o.projects.length > 1
              ? o.projects.map((p) => ({ key: p.projectId, label: p.code, values: labels.map((day) => p.trend.find((t) => t.day === day)?.[metric] ?? 0) }))
              : [{ key: "all", label: o.projects.length > 4 ? "Toàn bộ dự án" : (o.projects[0]?.code ?? "Toàn bộ"), values: o.throughput.map((t) => t[metric]) }];
            return (
              <>
                <LineChart labels={labels} series={series} unit={def.unit} height={200} ariaLabel={`${def.label} theo ngày, ${o.days} ngày`} />
                {o.projects.length > 4 && <p className="mt-1 text-xs text-fg-subtle">Hơn 4 dự án nên hiện tổng; mở từng dự án để xem riêng.</p>}
              </>
            );
          }}
        </DataCard>

        <DataCard title="Rủi ro hàng đầu" icon="canhBao" {...ov} className="xl:col-span-4">
          {(o: Overview) =>
            o.risks.length === 0 ? (
              <p className="text-sm text-fg-subtle">Không có rủi ro nào đáng chú ý.</p>
            ) : (
              <ul className="divide-y divide-border-subtle">
                {o.risks.slice(0, 6).map((r, i) => (
                  <li key={`${r.kind}-${r.projectId}-${i}`}>
                    <button type="button" className="flex w-full items-center gap-2 rounded px-1 py-1.5 text-left text-sm hover:bg-surface-hover" onClick={() => (r.kind === "overdue_work" ? goto("workManage") : r.projectId ? onOpenProject(r.projectId) : undefined)}>
                      <Icon name={r.severity === "err" ? "loi" : "canhBao"} size={14} className={r.severity === "err" ? "text-danger-700" : "text-warning-700"} />
                      <span className="flex-1">{r.label}</span>
                      <span className="tabular-nums text-fg-muted">{r.kind === "late" ? `${r.value} điểm` : fmtNum(r.value)}</span>
                      <Icon name="sau" size={13} className="text-fg-subtle" />
                    </button>
                  </li>
                ))}
              </ul>
            )
          }
        </DataCard>
      </div>

      <div className="grid gap-4 xl:grid-cols-12">
        <CanXuLyCard className="xl:col-span-5" />
        {has("bk.view") && <SucKhoeCard className="xl:col-span-7" />}
      </div>
      <p className="text-xs text-fg-subtle">
        Số theo ngày tổng hợp trên máy chủ mỗi 2 phút; màn này làm mới mỗi 2 phút.{" "}
        <button type="button" className="underline hover:text-fg" onClick={() => gotoReport("sohoa")}>
          Báo cáo tiến độ dự án
        </button>
      </p>
    </div>
  );
}
