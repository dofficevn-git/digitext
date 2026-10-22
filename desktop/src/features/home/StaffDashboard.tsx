// Copyright (c) 2026 DigiText. All rights reserved.
// Tổng quan của NHÂN VIÊN: "Hôm nay của tôi" (mỗi việc một ô: số, định mức, 14 ngày, xếp loại), "Tuần này" (cột 7 ngày
// + vạch định mức), "So với nhóm" (tôi và trung bình nhóm ẩn danh), việc của tôi, cần xử lý. Chỉ số của chính mình —
// máy chủ không trả số của người khác.

import { useCallback, useMemo, useState } from "react";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { dashboardMe, QUOTA_METRICS, quotaLabel, type MeDash, type MetricKey, type MetricsF, type Rating } from "@/api/dashboard";
import { V4r1 } from "@/api/stats";
import { Badge, BarChart, Card, KpiRow, KpiTile, LineChart, Seg, Sparkline, fmtNum } from "@/ui";
import CanXuLyCard, { useCanXuLy } from "./blocks/CanXuLy";
import { DataCard, DaysSeg, n, useSo, type Days } from "./blocks/common";

export const RATING: Record<Rating, { label: string; tone: "ok" | "info" | "err" | "gray" }> = {
  xuat_sac: { label: "Xuất sắc", tone: "ok" },
  dat: { label: "Đạt", tone: "info" },
  canh_bao: { label: "Cảnh báo lỗi", tone: "err" },
  chua_giao: { label: "Chưa giao định mức", tone: "gray" },
};
export function RatingBadge({ r }: { r: Rating }) {
  const m = RATING[r];
  return <Badge tone={m.tone}>{m.label}</Badge>;
}

const metricOf = (key: string): MetricKey => QUOTA_METRICS.find((q) => q.key === key)?.metric ?? "pagesScanned";
const unitOf = (key: string) => QUOTA_METRICS.find((q) => q.key === key)?.unit ?? "";

export default function StaffDashboard() {
  const { session, goto } = V460();
  const has = (p: Parameters<typeof V3z9>[1]) => V3z9(session?.permissions, p);
  const [days, setDays] = useState<Days>(14);
  const load = useCallback(() => dashboardMe(days), [days]);
  const me = useSo(load, ["dashboard", "work"], true);
  const can = useCanXuLy();
  const local = useSo(useCallback(() => V4r1(), []), ["docs", "collected"], true);

  // Các chỉ số có nghĩa với người này: có định mức trước, rồi theo sản lượng hôm nay; tối đa 6 ô để nhìn một lượt.
  const kpiRows = useMemo(() => {
    const d = me.data;
    if (!d) return [];
    const homNay = d.series.length ? d.series[d.series.length - 1].me : null;
    return [...d.kpi]
      .sort((a, b) => Number(!!b.quotaPerDay) - Number(!!a.quotaPerDay) || (homNay ? homNay[metricOf(b.metric)] - homNay[metricOf(a.metric)] : 0))
      .slice(0, 6);
  }, [me.data]);
  const metrics = useMemo(() => kpiRows.map((k) => k.metric), [kpiRows]);
  const [chon, setChon] = useState<string | null>(null);
  const primary = chon && metrics.includes(chon) ? chon : (metrics[0] ?? "pages_scanned");
  const pm = metricOf(primary);

  return (
    <div className="space-y-4">
      <div className="grid gap-4 xl:grid-cols-12">
        <DataCard
          title="Hôm nay của tôi"
          icon="nguoi"
          {...me}
          className="xl:col-span-8"
          description={me.data ? `${me.data.projectCode} · ${me.data.projectName} · ${n(me.data.activeDays)} ngày có làm trong ${me.data.days} ngày · TB = trung bình mỗi ngày có làm` : undefined}
          actions={<DaysSeg value={days} onChange={setDays} />}
        >
          {(d: MeDash) =>
            d.kpi.length === 0 ? (
              <p className="text-sm text-fg-subtle">Chưa có sản lượng nào ghi nhận trong {d.days} ngày qua. Số sẽ hiện khi bạn quét, nhập trường, xác nhận mục hoặc lập hồ sơ.</p>
            ) : (
              <div className="grid gap-3 sm:grid-cols-2">
                {kpiRows.map((k) => {
                  const mk = metricOf(k.metric);
                  const homNay = d.series.length ? d.series[d.series.length - 1].me[mk] : 0;
                  const last14 = d.series.slice(-14).map((s) => s.me[mk]);
                  const sub = k.quotaPerDay ? `định mức ${fmtNum(k.quotaPerDay)}/ngày · đạt ${k.attainmentPct == null ? "—" : `${Math.round(k.attainmentPct)} %`}` : `TB ${k.meAvgPerActiveDay == null ? "—" : fmtNum(k.meAvgPerActiveDay)}/ngày`;
                  return (
                    <KpiTile
                      key={k.metric}
                      label={`${quotaLabel(k.metric)} hôm nay`}
                      value={homNay}
                      sub={sub}
                      size="lg"
                      extra={
                        <span className="flex flex-col items-end gap-1">
                          <Sparkline values={last14} refValue={k.quotaPerDay} ariaLabel={`${quotaLabel(k.metric)} 14 ngày`} />
                          <RatingBadge r={k.rating} />
                        </span>
                      }
                    />
                  );
                })}
              </div>
            )
          }
        </DataCard>

        <DataCard title="Tuần này" icon="thongke" {...me} className="xl:col-span-4" description={`${quotaLabel(primary)} 7 ngày gần nhất`}>
          {(d: MeDash) => {
            const quota = d.quotas[primary] ?? null;
            const tuan = d.series.slice(-7);
            const tong = tuan.reduce((a, s) => a + s.me[pm], 0);
            const coLam = tuan.filter((s) => s.me[pm] > 0).length;
            return (
              <>
                <BarChart data={tuan.map((s) => ({ label: s.day, value: s.me[pm] }))} unit={unitOf(primary)} height={150} target={quota ? { value: quota, label: "Định mức" } : null} ariaLabel={`${quotaLabel(primary)} 7 ngày gần nhất`} />
                <dl className="mt-3 grid grid-cols-2 gap-x-3 gap-y-1 text-sm">
                  <dt className="text-fg-subtle">Tổng 7 ngày</dt>
                  <dd className="font-medium tabular-nums">{fmtNum(tong)} {unitOf(primary)}</dd>
                  <dt className="text-fg-subtle">Trung bình / ngày có làm</dt>
                  <dd className="font-medium tabular-nums">{coLam ? fmtNum(Math.round(tong / coLam)) : "—"}</dd>
                  {quota ? (
                    <>
                      <dt className="text-fg-subtle">So định mức</dt>
                      <dd className="font-medium tabular-nums">{coLam ? `${Math.round((tong / coLam / quota) * 100)} %` : "—"}</dd>
                    </>
                  ) : null}
                </dl>
              </>
            );
          }}
        </DataCard>
      </div>

      <div className="grid gap-4 xl:grid-cols-12">
        <DataCard
          title="So với nhóm"
          icon="thanhVien"
          {...me}
          className="xl:col-span-8"
          description="Trung bình mỗi người có làm trong ngày, không nêu tên. Ẩn những ngày nhóm chưa đủ người."
          actions={
            metrics.length > 1 ? (
              <Seg<string> value={primary} onChange={setChon} options={metrics.slice(0, 4).map((m) => ({ value: m, label: quotaLabel(m) }))} />
            ) : null
          }
        >
          {(d: MeDash) => {
            const gk = pm as keyof MetricsF;
            const labels = d.series.map((s) => s.day);
            const toi = d.series.map((s) => s.me[pm]);
            const nhom = d.series.map((s) => (s.groupAvg ? (s.groupAvg[gk] ?? null) : null));
            const myAvg = d.activeDays ? d.totals[pm] / d.activeDays : null;
            const gAvg = d.groupAvg ? d.groupAvg[gk] : null;
            const cau = myAvg != null && gAvg != null && gAvg > 0 ? (myAvg >= gAvg ? `Cao hơn trung bình nhóm ${Math.round(((myAvg - gAvg) / gAvg) * 100)} % (${fmtNum(Math.round(myAvg))} so ${fmtNum(Math.round(gAvg))} mỗi ngày có làm).` : `Thấp hơn trung bình nhóm ${Math.round(((gAvg - myAvg) / gAvg) * 100)} % (${fmtNum(Math.round(myAvg))} so ${fmtNum(Math.round(gAvg))} mỗi ngày có làm).`) : "Nhóm chưa đủ người để so sánh.";
            return (
              <>
                <LineChart labels={labels} series={[{ key: "toi", label: "Tôi", values: toi }, { key: "nhom", label: "Trung bình nhóm", values: nhom }]} unit={unitOf(primary)} height={180} target={d.quotas[primary] ? { value: d.quotas[primary], label: "Định mức" } : null} ariaLabel={`${quotaLabel(primary)}: tôi và trung bình nhóm ${d.days} ngày`} />
                <p className="mt-2 text-sm text-fg-muted">{cau}</p>
              </>
            );
          }}
        </DataCard>

        <div className="space-y-3 xl:col-span-4">
          <KpiRow className="grid grid-cols-1 gap-3">
            {has("work.execute") && <KpiTile icon="kiemTra" label="Việc của tôi" value={can.work?.pending} tone={can.work && can.work.pending > 0 ? "info" : undefined} sub={can.work ? `${n(can.work.packages)} gói đang mở${can.work.overdue ? ` · ${n(can.work.overdue)} quá hạn` : ""}` : undefined} onClick={() => goto("work")} />}
            {has("work.execute") && <KpiTile icon="tuChoi" label="Mục bị trả lại cho tôi" value={can.work?.rejected} tone={can.work && can.work.rejected > 0 ? "err" : undefined} sub="Sửa rồi xác nhận lại" onClick={() => goto("work")} />}
            <KpiTile icon="mayTram" label="Trang đã quét trên máy này" value={local.data?.pagesScanned} loading={local.loading} error={local.error} sub={local.data ? `${n(local.data.hoSoCount)} hồ sơ · ${n(local.data.pdfCount)} PDF` : undefined} onClick={() => goto("drive")} />
          </KpiRow>
          <CanXuLyCard />
        </div>
      </div>
      <Card padding="sm">
        <p className="text-xs text-fg-subtle">Số liệu tổng hợp theo ngày trên máy chủ, cập nhật mỗi 2 phút. Chỉ bạn và quản lý dự án xem được sản lượng của bạn.</p>
      </Card>
    </div>
  );
}
