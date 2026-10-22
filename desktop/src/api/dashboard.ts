// Copyright (c) 2026 DigiText. All rights reserved.
// Dashboard 3 góc nhìn — số liệu từ máy chủ /dashboard/* (migration 0021; crates/digitext-api/src/routes/dashboard.rs).
// Kiểu dữ liệu khớp đúng tên trường camelCase của máy chủ.

import { invoke } from "@/api/ipc";

/** 20 chỉ số của dashboard_daily (tên cột máy chủ ở dạng camelCase). */
export interface Metrics {
  pagesScanned: number;
  docsUploaded: number;
  pagesOcrDone: number;
  docsPdfa: number;
  docsSigned: number;
  docsReviewedApproved: number;
  docsReviewedRejected: number;
  fieldsEdited: number;
  fieldsVerified: number;
  workItemsDone: number;
  workItemsCheckedOk: number;
  workItemsCheckedErr: number;
  workItemsRejected: number;
  clHoSoNew: number;
  clHoSoEdited: number;
  clVanBanNew: number;
  clFieldsEntered: number;
  packagesBuilt: number;
  submissionsCreated: number;
  submissionsCompleted: number;
}
export type MetricKey = keyof Metrics;
export const EMPTY_METRICS: Metrics = {
  pagesScanned: 0, docsUploaded: 0, pagesOcrDone: 0, docsPdfa: 0, docsSigned: 0, docsReviewedApproved: 0, docsReviewedRejected: 0,
  fieldsEdited: 0, fieldsVerified: 0, workItemsDone: 0, workItemsCheckedOk: 0, workItemsCheckedErr: 0, workItemsRejected: 0,
  clHoSoNew: 0, clHoSoEdited: 0, clVanBanNew: 0, clFieldsEntered: 0, packagesBuilt: 0, submissionsCreated: 0, submissionsCompleted: 0,
};
/** Khoá snake_case của định mức (máy chủ) ↔ khoá camelCase của Metrics. */
export const QUOTA_METRICS: { key: string; metric: MetricKey; label: string; unit: string }[] = [
  { key: "pages_scanned", metric: "pagesScanned", label: "Trang quét", unit: "trang" },
  { key: "docs_uploaded", metric: "docsUploaded", label: "Tệp đưa lên", unit: "tệp" },
  { key: "fields_edited", metric: "fieldsEdited", label: "Trường đã nhập", unit: "trường" },
  { key: "fields_verified", metric: "fieldsVerified", label: "Trường đã xác nhận", unit: "trường" },
  { key: "work_items_done", metric: "workItemsDone", label: "Mục xác nhận", unit: "mục" },
  { key: "work_items_checked_ok", metric: "workItemsCheckedOk", label: "Mục kiểm tra đạt", unit: "mục" },
  { key: "docs_reviewed_approved", metric: "docsReviewedApproved", label: "Tài liệu đã duyệt", unit: "tài liệu" },
  { key: "docs_signed", metric: "docsSigned", label: "Tệp đã ký", unit: "tệp" },
  { key: "cl_ho_so_new", metric: "clHoSoNew", label: "Hồ sơ lập mới", unit: "hồ sơ" },
  { key: "cl_van_ban_new", metric: "clVanBanNew", label: "Văn bản lập mới", unit: "văn bản" },
  { key: "cl_fields_entered", metric: "clFieldsEntered", label: "Trường chỉnh lý", unit: "trường" },
];
export const quotaLabel = (k: string) => QUOTA_METRICS.find((q) => q.key === k)?.label ?? k;

export type DayMetrics = Metrics & { day: string };

export type Rag = "green" | "amber" | "red" | "grey";
export type Step = "scan" | "pdfa" | "sign" | "review";
export interface Rates {
  pagesScanned: number;
  docsPdfa: number;
  docsSigned: number;
  docsApproved: number;
}
export interface Health {
  progressPct: number | null;
  timePct: number | null;
  lech: number | null;
  daysRemaining: number | null;
  runRate7d: Rates;
  requiredRate: Rates;
  projectedFinishOn: string | null;
  bottleneckStep: Step | null;
  rag: Rag;
}
export interface Contract {
  signedOn: string | null;
  endOn: string | null;
  deadlineOn: string | null;
  contractPages: number | null;
  contractFiles: number | null;
  managerName: string | null;
}
export interface Counts {
  files: number;
  pages: number;
  pdfaFiles: number;
  signedFiles: number;
  approvedFiles: number;
  rejectedFiles: number;
  pendingReviewFiles: number;
}
export interface ProjectIssues {
  projectId: string;
  ocrFailedPages: number;
  overduePackages: number;
  rejectedUploads: number;
}
export interface OverviewProject {
  projectId: string;
  code: string;
  name: string;
  contract: Contract;
  counts: Counts;
  health: Health;
  trend: DayMetrics[];
  issues: ProjectIssues;
}
export type RiskKind = "late" | "ocr_failed" | "overdue_work";
export interface Risk {
  projectId: string | null;
  projectCode: string | null;
  kind: RiskKind;
  label: string;
  value: number;
  severity: "warn" | "err";
}
export interface Overview {
  generatedAt: string;
  today: string;
  days: number;
  totals: { projects: number; late: number; atRisk: number; pages7d: number; pages7dPrev: number; submittedFiles: number };
  projects: OverviewProject[];
  throughput: DayMetrics[];
  risks: Risk[];
}

export interface PipelineCounts {
  clHoSo: number;
  clVanBan: number;
  scanPagesReported: number;
  files: number;
  pages: number;
  ocrPendingDocs: number;
  ocrPendingPages: number;
  ocrDoneDocs: number;
  ocrFailedPages: number;
  pdfaFiles: number;
  toSign: number;
  signedFiles: number;
  signFailed: number;
  approvedFiles: number;
  rejectedFiles: number;
  pendingReviewFiles: number;
  packagesBuilt: number;
  docsPackaged: number;
  subRegistered: number;
  subReceived: number;
  subCompleted: number;
}
export type BacklogStageKey = "ocr" | "review" | "sign" | "package" | "submit";
export interface BacklogStage {
  stage: BacklogStageKey;
  label: string;
  /** 0–3 · 4–7 · 8–14 · >14 ngày */
  buckets: [number, number, number, number];
}
export interface WorkSla {
  open: number;
  overdue: number;
  dueSoon3d: number;
  closed30d: number;
  closedOnTime30d: number;
  pendingItems: number;
  itemsOverSla: number;
  avgTurnaroundHours: number | null;
}
export interface OverduePackage {
  packageId: string;
  name: string;
  dueAt: string;
  pendingItems: number;
}
export type Rating = "xuat_sac" | "dat" | "canh_bao" | "chua_giao";
export interface KpiLine {
  metric: string;
  quotaPerDay: number | null;
  meAvgPerActiveDay: number | null;
  attainmentPct: number | null;
  errorPct: number;
  rating: Rating;
}
export interface StaffRow {
  userId: string;
  username: string;
  displayName: string;
  totals: Metrics;
  activeDays: number;
  today: Metrics;
  series: DayMetrics[];
  kpi: KpiLine[];
}
export interface KpiSettings {
  projectId: string;
  quotas: Record<string, number>;
  excellentPct: number;
  passPct: number;
  excellentErrPct: number;
  passErrPct: number;
  ragAmberLech: number;
  ragRedLech: number;
  reviewBacklogWarnDays: number;
  workItemSlaHours: number;
  groupMinUsers: number;
  updatedAt: string | null;
  updatedBy: string | null;
}
export type KpiSettingsBody = Omit<KpiSettings, "projectId" | "updatedAt" | "updatedBy">;

export interface ProjectDash {
  projectId: string;
  code: string;
  name: string;
  today: string;
  days: number;
  contract: Contract;
  counts: Counts;
  health: Health;
  pipeline: PipelineCounts;
  series: DayMetrics[];
  backlog: BacklogStage[];
  workSla: WorkSla;
  overduePackages: OverduePackage[];
  staff: StaffRow[];
  kpi: KpiSettings;
}

export interface MetricsF {
  pagesScanned: number;
  docsUploaded: number;
  fieldsEdited: number;
  fieldsVerified: number;
  workItemsDone: number;
  docsReviewedApproved: number;
  docsSigned: number;
  clHoSoNew: number;
  clVanBanNew: number;
  clFieldsEntered: number;
}
export interface MeDay {
  day: string;
  me: Metrics;
  groupAvg: MetricsF | null;
  activeUsers: number;
}
export interface MeDash {
  userId: string;
  projectId: string;
  projectCode: string;
  projectName: string;
  today: string;
  days: number;
  series: MeDay[];
  totals: Metrics;
  activeDays: number;
  groupAvg: MetricsF | null;
  groupUsers: number;
  quotas: Record<string, number>;
  kpi: KpiLine[];
}
export interface StaffBoard {
  projectId: string;
  from: string;
  to: string;
  users: StaffRow[];
  kpi: KpiSettings;
}

export const dashboardOverview = (days?: number, projectId?: string | null) => invoke<Overview>("dashboard_overview", { args: { days: days ?? null, projectId: projectId ?? null } });
export const dashboardProject = (projectId: string, days?: number) => invoke<ProjectDash>("dashboard_project", { args: { projectId, days: days ?? null } });
export const dashboardMe = (days?: number, projectId?: string | null) => invoke<MeDash>("dashboard_me", { args: { days: days ?? null, projectId: projectId ?? null } });
export const dashboardStaff = (projectId: string, from?: string, to?: string) => invoke<StaffBoard>("dashboard_staff", { args: { projectId, from: from ?? null, to: to ?? null } });
export const kpiSettingsGet = (projectId: string) => invoke<KpiSettings>("dashboard_kpi_settings_get", { projectId });
export const kpiSettingsSet = (projectId: string, body: KpiSettingsBody) => invoke<KpiSettings>("dashboard_kpi_settings_set", { projectId, body });
