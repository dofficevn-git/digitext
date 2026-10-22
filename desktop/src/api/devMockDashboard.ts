// Copyright (c) 2026 DigiText. All rights reserved.
// Dữ liệu giả cho dashboard khi chạy bản dev trong trình duyệt (xem trước / chụp màn 3 góc nhìn). Không vào bản
// phát hành (chỉ devMock.ts nạp). PRNG có seed để ảnh chụp lặp lại được. `?role=user|admin|manager|super_admin`
// chọn vai trò (sessionCore), `?project=1` giả lập đã chọn dự án ở Topbar.
import type { DayMetrics, Health, KpiLine, KpiSettings, MeDash, MeDay, Metrics, Overview, OverviewProject, ProjectDash, StaffBoard, StaffRow } from "./dashboard";
import { EMPTY_METRICS, QUOTA_METRICS } from "./dashboard";

function rng(seed: number) {
  let s = seed >>> 0;
  return () => {
    s = (s * 1664525 + 1013904223) >>> 0;
    return s / 4294967296;
  };
}
const iso = (d: Date) => d.toISOString().slice(0, 10);
const today = new Date();
const dayAgo = (n: number) => {
  const d = new Date(today);
  d.setDate(d.getDate() - n);
  return iso(d);
};
const addDays = (n: number) => dayAgo(-n);

interface Proj {
  id: string;
  code: string;
  name: string;
  seed: number;
  rag: Health["rag"];
  scale: number;
}
const PROJECTS: Proj[] = [
  { id: "dev-ltlsbn", code: "LTLSBN", name: "Lưu trữ lịch sử Bắc Ninh", seed: 7, rag: "red", scale: 1 },
  { id: "dev-ltlsbni", code: "LTLSBNI", name: "Lưu trữ lịch sử Bắc Ninh — đợt 2", seed: 11, rag: "green", scale: 0.35 },
  { id: "dev-thuky", code: "THUKY", name: "Thư ký Hội đồng nhân dân", seed: 19, rag: "amber", scale: 0.2 },
];

function series(p: Proj, days: number, userSeed = 0): DayMetrics[] {
  const r = rng(p.seed * 131 + userSeed);
  const out: DayMetrics[] = [];
  for (let i = days - 1; i >= 0; i--) {
    const d = new Date(today);
    d.setDate(d.getDate() - i);
    const wd = d.getDay();
    const work = wd === 0 ? 0.1 : wd === 6 ? 0.45 : 1;
    const k = p.scale * work * (userSeed ? 0.22 : 1);
    const pages = Math.round((4000 + r() * 9000) * k);
    out.push({
      ...EMPTY_METRICS,
      day: iso(d),
      pagesScanned: pages,
      docsUploaded: Math.round(pages / 4.2),
      pagesOcrDone: Math.round((3500 + r() * 8000) * k),
      docsPdfa: Math.round((900 + r() * 1800) * k),
      docsSigned: Math.round((700 + r() * 1600) * k * (i < 20 ? 1 : 0.3)),
      docsReviewedApproved: Math.round((60 + r() * 220) * k),
      docsReviewedRejected: Math.round(r() * 6 * k),
      fieldsEdited: Math.round((300 + r() * 500) * k),
      fieldsVerified: Math.round((100 + r() * 300) * k),
      workItemsDone: Math.round((40 + r() * 120) * k),
      workItemsCheckedOk: Math.round((10 + r() * 30) * k),
      workItemsCheckedErr: Math.round(r() * 3 * k),
      workItemsRejected: Math.round(r() * 2 * k),
      clHoSoNew: Math.round((5 + r() * 40) * k),
      clHoSoEdited: Math.round(r() * 20 * k),
      clVanBanNew: Math.round((20 + r() * 150) * k),
      clFieldsEntered: Math.round((200 + r() * 600) * k),
      packagesBuilt: i % 9 === 0 ? Math.round(2 * k) : 0,
      submissionsCreated: i % 15 === 0 ? 1 : 0,
      submissionsCompleted: i % 21 === 0 ? 1 : 0,
    });
  }
  return out;
}

function health(p: Proj): Health {
  const base = { runRate7d: { pagesScanned: 6200 * p.scale, docsPdfa: 1300 * p.scale, docsSigned: 1100 * p.scale, docsApproved: 140 * p.scale }, requiredRate: { pagesScanned: 7100 * p.scale, docsPdfa: 1500 * p.scale, docsSigned: 1400 * p.scale, docsApproved: 900 * p.scale } };
  if (p.rag === "red") return { ...base, progressPct: 38.4, timePct: 61.2, lech: -22.8, daysRemaining: 142, projectedFinishOn: addDays(298), bottleneckStep: "review", rag: "red" };
  if (p.rag === "amber") return { ...base, progressPct: 52, timePct: 58.5, lech: -6.5, daysRemaining: 95, projectedFinishOn: addDays(104), bottleneckStep: "sign", rag: "amber" };
  return { ...base, progressPct: 71.3, timePct: 55, lech: 16.3, daysRemaining: 160, projectedFinishOn: addDays(120), bottleneckStep: "scan", rag: "green" };
}

function kpi(p: Proj): KpiSettings {
  return { projectId: p.id, quotas: { pages_scanned: 800, fields_edited: 360, work_items_done: 60, cl_ho_so_new: 25 }, excellentPct: 100, passPct: 80, excellentErrPct: 1, passErrPct: 3, ragAmberLech: -5, ragRedLech: -15, reviewBacklogWarnDays: 3, workItemSlaHours: 48, groupMinUsers: 3, updatedAt: null, updatedBy: null };
}

const NAMES = ["Nguyễn Thị Hoa", "Trần Văn Minh", "Lê Thu Hà", "Phạm Quốc Bảo", "Hoàng Lan Anh", "Vũ Đức Long", "Đỗ Mai Phương", "Bùi Tiến Dũng"];

function kpiLines(totals: Metrics, activeDays: number, q: Record<string, number>): KpiLine[] {
  return QUOTA_METRICS.filter((m) => q[m.key] || totals[m.metric] > 0).map((m) => {
    const quota = q[m.key] ?? null;
    const avg = activeDays ? Math.round((totals[m.metric] / activeDays) * 10) / 10 : null;
    const att = quota && avg != null ? Math.round((avg / quota) * 1000) / 10 : null;
    const err = 1.4;
    const rating = att == null ? "chua_giao" : att >= 100 && err <= 1 ? "xuat_sac" : att >= 80 && err <= 3 ? "dat" : "canh_bao";
    return { metric: m.key, quotaPerDay: quota, meAvgPerActiveDay: avg, attainmentPct: att, errorPct: err, rating };
  });
}

function sumSeries(s: DayMetrics[]): Metrics {
  const t: Metrics = { ...EMPTY_METRICS };
  for (const d of s) for (const k of Object.keys(EMPTY_METRICS) as (keyof Metrics)[]) t[k] += d[k];
  return t;
}

function staff(p: Proj, days: number): StaffRow[] {
  const q = kpi(p).quotas;
  return NAMES.slice(0, p.scale >= 1 ? 8 : 4).map((name, i) => {
    const s = series(p, days, i + 1);
    const totals = sumSeries(s);
    return { userId: `dev-u${i}`, username: `nv${i + 1}`, displayName: name, totals, activeDays: s.filter((d) => d.pagesScanned > 0).length, today: s[s.length - 1], series: s, kpi: kpiLines(totals, days, q) };
  });
}

function overview(days: number): Overview {
  const projects: OverviewProject[] = PROJECTS.map((p) => {
    const s = series(p, days);
    const files = Math.round(205008 * p.scale);
    return {
      projectId: p.id,
      code: p.code,
      name: p.name,
      contract: { signedOn: dayAgo(230), endOn: addDays(142), deadlineOn: addDays(142), contractPages: Math.round(2_250_000 * p.scale), contractFiles: Math.round(520_000 * p.scale), managerName: "Nguyễn Văn Hiếu" },
      counts: { files, pages: Math.round(866703 * p.scale), pdfaFiles: Math.round(183679 * p.scale), signedFiles: Math.round(160375 * p.scale), approvedFiles: Math.round(1200 * p.scale), rejectedFiles: Math.round(40 * p.scale), pendingReviewFiles: Math.round(182000 * p.scale) },
      health: health(p),
      trend: s,
      issues: { projectId: p.id, ocrFailedPages: p.rag === "red" ? 591 : 0, overduePackages: p.rag === "green" ? 0 : 2, rejectedUploads: 12 },
    };
  });
  const throughput = projects[0].trend.map((d, i) => {
    const t: DayMetrics = { ...d };
    for (const p of projects.slice(1)) for (const k of Object.keys(EMPTY_METRICS) as (keyof Metrics)[]) t[k] += p.trend[i][k];
    return t;
  });
  const sum7 = (from: number, to: number) => throughput.slice(Math.max(0, throughput.length - to), throughput.length - from).reduce((a, d) => a + d.pagesScanned, 0);
  return {
    generatedAt: new Date().toISOString(),
    today: iso(today),
    days,
    totals: { projects: 3, late: 1, atRisk: 1, pages7d: sum7(0, 7), pages7dPrev: sum7(7, 14), submittedFiles: projects.reduce((a, p) => a + p.counts.signedFiles, 0) },
    projects,
    throughput,
    risks: [
      { projectId: "dev-ltlsbn", projectCode: "LTLSBN", kind: "late", label: "LTLSBN chậm tiến độ", value: -23, severity: "err" },
      { projectId: "dev-thuky", projectCode: "THUKY", kind: "overdue_work", label: "THUKY có gói việc quá hạn", value: 2, severity: "err" },
      { projectId: "dev-ltlsbn", projectCode: "LTLSBN", kind: "ocr_failed", label: "LTLSBN có trang OCR lỗi", value: 591, severity: "warn" },
    ],
  };
}

function project(id: string, days: number): ProjectDash {
  const p = PROJECTS.find((x) => x.id === id) ?? PROJECTS[0];
  const ov = overview(days).projects.find((x) => x.projectId === p.id)!;
  return {
    projectId: p.id,
    code: p.code,
    name: p.name,
    today: iso(today),
    days,
    contract: ov.contract,
    counts: ov.counts,
    health: ov.health,
    pipeline: {
      clHoSo: Math.round(12840 * p.scale), clVanBan: Math.round(96000 * p.scale), scanPagesReported: Math.round(901200 * p.scale),
      files: ov.counts.files, pages: ov.counts.pages, ocrPendingDocs: Math.round(21304 * p.scale), ocrPendingPages: Math.round(176988 * p.scale), ocrDoneDocs: Math.round(183704 * p.scale), ocrFailedPages: ov.issues.ocrFailedPages,
      pdfaFiles: ov.counts.pdfaFiles, toSign: Math.round(23304 * p.scale), signedFiles: ov.counts.signedFiles, signFailed: 1,
      approvedFiles: ov.counts.approvedFiles, rejectedFiles: ov.counts.rejectedFiles, pendingReviewFiles: ov.counts.pendingReviewFiles,
      packagesBuilt: Math.round(14 * p.scale), docsPackaged: Math.round(9800 * p.scale), subRegistered: 1, subReceived: 1, subCompleted: 3,
    },
    series: ov.trend,
    backlog: [
      { stage: "ocr", label: "Chờ OCR", buckets: [Math.round(21303 * p.scale), 0, 0, 1] },
      { stage: "review", label: "Chờ duyệt", buckets: [Math.round(23308 * p.scale), 0, Math.round(37728 * p.scale), Math.round(122668 * p.scale)] },
      { stage: "sign", label: "Chờ ký", buckets: [Math.round(23303 * p.scale), 0, 0, 1] },
      { stage: "package", label: "Chờ đóng gói", buckets: [0, 0, Math.round(37728 * p.scale), Math.round(122647 * p.scale)] },
      { stage: "submit", label: "Chờ nộp", buckets: [2, 1, 0, 0] },
    ],
    workSla: { open: 6, overdue: p.rag === "green" ? 0 : 2, dueSoon3d: 1, closed30d: 9, closedOnTime30d: 7, pendingItems: 412, itemsOverSla: 38, avgTurnaroundHours: 19.4 },
    overduePackages: p.rag === "green" ? [] : [{ packageId: "dev-p1", name: "Gói xác nhận 14", dueAt: dayAgo(3), pendingItems: 57 }, { packageId: "dev-p2", name: "Gói kiểm tra chéo 3", dueAt: dayAgo(1), pendingItems: 12 }],
    staff: staff(p, days),
    kpi: kpi(p),
  };
}

function me(days: number): MeDash {
  const p = PROJECTS[0];
  const mine = series(p, days, 3);
  const grp = series(p, days, 9);
  const seriesOut: MeDay[] = mine.map((d, i) => ({
    day: d.day,
    me: d,
    groupAvg: i % 11 === 5 ? null : { pagesScanned: Math.round(grp[i].pagesScanned * 0.9), docsUploaded: Math.round(grp[i].docsUploaded * 0.9), fieldsEdited: Math.round(grp[i].fieldsEdited * 0.9), fieldsVerified: grp[i].fieldsVerified, workItemsDone: grp[i].workItemsDone, docsReviewedApproved: grp[i].docsReviewedApproved, docsSigned: grp[i].docsSigned, clHoSoNew: grp[i].clHoSoNew, clVanBanNew: grp[i].clVanBanNew, clFieldsEntered: grp[i].clFieldsEntered },
    activeUsers: i % 11 === 5 ? 2 : 6,
  }));
  const totals = sumSeries(mine);
  const q = kpi(p).quotas;
  const g = sumSeries(grp);
  return {
    userId: "dev-user",
    projectId: p.id,
    projectCode: p.code,
    projectName: p.name,
    today: iso(today),
    days,
    series: seriesOut,
    totals,
    activeDays: mine.filter((d) => d.pagesScanned > 0).length,
    groupAvg: { pagesScanned: g.pagesScanned / days * 0.9, docsUploaded: g.docsUploaded / days, fieldsEdited: g.fieldsEdited / days, fieldsVerified: g.fieldsVerified / days, workItemsDone: g.workItemsDone / days, docsReviewedApproved: g.docsReviewedApproved / days, docsSigned: g.docsSigned / days, clHoSoNew: g.clHoSoNew / days, clVanBanNew: g.clVanBanNew / days, clFieldsEntered: g.clFieldsEntered / days },
    groupUsers: 6,
    quotas: q,
    kpi: kpiLines(totals, days, q),
  };
}

const qs = () => (typeof window === "undefined" ? new URLSearchParams() : new URLSearchParams(window.location.search));

export function devMockDashboard(cmd: string, args?: Record<string, unknown>): unknown {
  const a = (args?.args ?? {}) as { days?: number | null; projectId?: string | null };
  const days = Math.max(1, Math.min(365, Number(a.days ?? 30) || 30));
  switch (cmd) {
    case "dashboard_overview":
      return overview(days);
    case "dashboard_project":
      return project(String(a.projectId ?? PROJECTS[0].id), days);
    case "dashboard_me":
      return me(days);
    case "dashboard_staff": {
      const p = project(String(a.projectId ?? PROJECTS[0].id), 30);
      return { projectId: p.projectId, from: dayAgo(29), to: iso(today), users: p.staff, kpi: p.kpi } satisfies StaffBoard;
    }
    case "dashboard_kpi_settings_get":
      return kpi(PROJECTS[0]);
    case "dashboard_kpi_settings_set":
      return { ...kpi(PROJECTS[0]), ...(args?.body as object), updatedAt: new Date().toISOString() };
    case "cl_project_get": {
      const role = qs().get("role") ?? "admin";
      const chosen = qs().get("project") === "1" || role === "admin" || role === "user";
      return { projectId: chosen ? PROJECTS[0].id : null, fromAccount: role === "admin" || role === "user", projects: PROJECTS.map((p) => ({ id: p.id, code: p.code, name: p.name })), cached: false };
    }
    case "cl_project_set":
      return null;
    default:
      return undefined;
  }
}
