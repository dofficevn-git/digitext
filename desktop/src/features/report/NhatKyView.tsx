// Copyright (c) 2026 DigiText. All rights reserved.
// Nhật ký số hoá (Bước 4 Điều 9 TT 05/2025, 3/11/2026): sản lượng theo ngày × người của dự án đang chọn lấy từ bảng tổng
// hợp dashboard (dashboard_daily, 2 phút một lần), không nhập tay. "Chốt nhật ký ngày…" lập biên bản kind nhat_ky_so_hoa
// (một ngày một bản, bất biến, có số và PDF) với bảng sản lượng điền sẵn + ô sự việc. Vai trò nhân viên chỉ thấy số của mình.

import { useCallback, useMemo, useState } from "react";
import { V41t } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { dashboardMe, dashboardProject, type DayMetrics, type Metrics } from "@/api/dashboard";
import { V4e3 } from "@/features/dossier/excel";
import { SoBienBan, type NguonDien, type NoiDungDien } from "@/features/records/RecordsView";
import { BarChart, Button, Card, EmptyState, Help, KpiRow, KpiTile, PageHeader, Seg, Table, type Column } from "@/ui";

const n = (v: number) => v.toLocaleString("vi-VN");
const fmtNgay = (iso: string) => {
  const [y, m, d] = iso.split("-");
  return d && m ? `${d}/${m}/${y}` : iso;
};
const COT: { key: keyof Metrics; label: string }[] = [
  { key: "pagesScanned", label: "Trang quét" },
  { key: "docsUploaded", label: "Tệp đưa lên" },
  { key: "pagesOcrDone", label: "Trang OCR" },
  { key: "docsPdfa", label: "PDF/A" },
  { key: "docsSigned", label: "Ký số" },
  { key: "docsReviewedApproved", label: "Duyệt" },
  { key: "clVanBanNew", label: "Văn bản lập" },
];
type Dong = { day: string; nguoi: string; userId: string } & Metrics;

export default function NhatKyView() {
  const { session } = V460();
  const proj = useProject();
  const projectId = session?.projectId ?? proj.projectId ?? "";
  const laNhanVien = session?.role === "user" || session?.role === "customer";
  const [days, setDays] = useState<"14" | "30" | "90">("30");
  const loader = useCallback(async (): Promise<Dong[]> => {
    if (!projectId) return [];
    const d = Number(days);
    if (laNhanVien) {
      const me = await dashboardMe(d, projectId);
      return me.series.map((s) => ({ day: s.day, nguoi: session?.displayName ?? "Tôi", userId: me.userId, ...s.me }));
    }
    const p = await dashboardProject(projectId, d);
    const out: Dong[] = [];
    for (const st of p.staff) for (const s of st.series as DayMetrics[]) out.push({ ...s, nguoi: st.displayName || st.username, userId: st.userId });
    return out;
  }, [projectId, days, laNhanVien, session?.displayName]);
  const { data, error, loading, reload } = V41t(loader, { topics: ["dashboard"], pollMs: 120_000, enabled: (isTauri() || import.meta.env.DEV) && !!projectId, errorText: () => "Không tải được nhật ký." });
  const rows = useMemo(() => (data ?? []).filter((r) => COT.some((c) => (r[c.key] ?? 0) > 0)).sort((a, b) => b.day.localeCompare(a.day) || a.nguoi.localeCompare(b.nguoi, "vi")), [data]);
  const theoNgay = useMemo(() => {
    const m = new Map<string, Metrics & { nguoi: number }>();
    for (const r of rows) {
      const g = m.get(r.day) ?? ({ ...Object.fromEntries(COT.map((c) => [c.key, 0])), nguoi: 0 } as unknown as Metrics & { nguoi: number });
      for (const c of COT) (g as unknown as Record<string, number>)[c.key] += r[c.key] ?? 0;
      g.nguoi += 1;
      m.set(r.day, g);
    }
    return [...m.entries()].sort((a, b) => b[0].localeCompare(a[0]));
  }, [rows]);
  const tong = useMemo(() => {
    const t = Object.fromEntries(COT.map((c) => [c.key, 0])) as Record<string, number>;
    for (const r of rows) for (const c of COT) t[c.key] += r[c.key] ?? 0;
    return t;
  }, [rows]);
  const [ngayChon, setNgayChon] = useState<string>("");
  const ngayOptions = useMemo(() => theoNgay.map(([d, g]) => ({ value: d, label: `${fmtNgay(d)} · ${n(g.pagesScanned)} trang · ${n(g.nguoi)} người` })), [theoNgay]);

  const nguonDien: NguonDien = useMemo(
    () => ({
      label: "Ngày cần chốt",
      options: ngayOptions,
      dien: async (_kind, day): Promise<NoiDungDien> => {
        const cua = rows.filter((r) => r.day === day);
        return {
          fields: { duAn: proj.projectLabel || projectId, ngay: day, nguoiLap: session?.displayName ?? "" },
          lists: {
            sanLuong: cua.map((r) => ({
              nguoi: r.nguoi,
              trangQuet: String(r.pagesScanned),
              tepDuaLen: String(r.docsUploaded),
              trangOcr: String(r.pagesOcrDone),
              tepPdfa: String(r.docsPdfa),
              tepKy: String(r.docsSigned),
              tepDuyet: String(r.docsReviewedApproved),
            })),
          },
        };
      },
    }),
    [ngayOptions, rows, proj.projectLabel, projectId, session?.displayName],
  );

  const xuatExcel = () =>
    void V4e3(
      rows.map((r) => ({ Ngày: fmtNgay(r.day), Người: r.nguoi, ...Object.fromEntries(COT.map((c) => [c.label, r[c.key] ?? 0])) })),
      "Nhật ký số hoá",
      `nhat-ky-so-hoa-${projectId.slice(0, 8)}-${days}ngay.xlsx`,
    );

  const cols: Column<Dong>[] = [
    { key: "day", header: "Ngày", width: 110, sortable: true, cell: (r) => fmtNgay(r.day) },
    { key: "nguoi", header: "Người", sortable: true },
    ...COT.map((c) => ({ key: c.key, header: c.label, align: "right" as const, width: 100, sortable: true, cell: (r: Dong) => (r[c.key] ? n(r[c.key]) : "") })),
  ];
  const bieuDo = useMemo(() => [...theoNgay].reverse().map(([d, g]) => ({ label: d, value: g.pagesScanned })), [theoNgay]);

  if (!projectId) {
    return (
      <section className="view active">
        <PageHeader title="Nhật ký số hoá" description="Bước 4 Điều 9 TT 05/2025: sản lượng theo ngày của từng người và sổ nhật ký đã chốt." />
        <Card padding="md">
          <EmptyState icon="duAn" title="Chọn dự án đang làm ở thanh trên" description="Nhật ký ghi theo dự án." />
        </Card>
      </section>
    );
  }
  return (
    <section className="view active">
      <PageHeader
        title="Nhật ký số hoá"
        description={`Bước 4 Điều 9 TT 05/2025: sản lượng theo ngày của từng người (tự tổng hợp từ máy chủ) và sổ nhật ký đã chốt. Dự án ${proj.projectLabel || ""}.`}
        actions={
          <>
            <Seg<"14" | "30" | "90"> value={days} onChange={setDays} options={[{ value: "14", label: "14 ngày" }, { value: "30", label: "30 ngày" }, { value: "90", label: "90 ngày" }]} />
            <Button icon="tepExcel" onClick={xuatExcel} disabled={rows.length === 0}>
              Xuất Excel
            </Button>
          </>
        }
      />
      <Help title="Nhật ký ghi gì" className="mb-3">
        <p className="text-sm">Số liệu từng ngày lấy từ bảng tổng hợp của máy chủ (trang quét theo tệp đưa lên, trang OCR, PDF/A, ký số, duyệt, văn bản lập). Cuối ngày bấm <b>Chốt nhật ký ngày…</b>: bảng sản lượng được điền sẵn, thêm sự việc trong ngày rồi lập — nhật ký có số, có PDF, không sửa được; sai thì lập bản bổ sung.</p>
      </Help>
      <KpiRow className="mb-3">
        {COT.slice(0, 6).map((c) => (
          <KpiTile key={c.key} label={`${c.label} · ${days} ngày`} value={tong[c.key]} loading={loading && !data} />
        ))}
      </KpiRow>
      <div className="grid gap-3 lg:grid-cols-12">
        <Card className="lg:col-span-8" title="Trang quét theo ngày" padding="sm">
          <BarChart data={bieuDo} unit="trang" ariaLabel="Trang quét theo ngày" height={180} emptyText="Chưa có số liệu trong khoảng này" />
        </Card>
        <Card className="lg:col-span-4" title="Theo ngày" padding="none">
          <Table
            columns={[
              { key: "d", header: "Ngày", width: 100, cell: (x: [string, Metrics & { nguoi: number }]) => fmtNgay(x[0]) },
              { key: "p", header: "Người", align: "right", width: 70, cell: (x: [string, Metrics & { nguoi: number }]) => n(x[1].nguoi) },
              { key: "t", header: "Trang quét", align: "right", cell: (x: [string, Metrics & { nguoi: number }]) => n(x[1].pagesScanned) },
            ]}
            rows={theoNgay}
            getRowId={(x) => x[0]}
            dense
            maxHeight={220}
            onRowClick={(x) => setNgayChon(x[0])}
            rowClassName={(x) => (x[0] === ngayChon ? "bg-brand-50" : "")}
            state={{ loading: loading && !data, error: data ? undefined : error, onRetry: reload, empty: { title: "Chưa có ngày nào có sản lượng" } }}
          />
        </Card>
      </div>
      <Card className="mt-3" title={`Chi tiết theo người${ngayChon ? ` · ${fmtNgay(ngayChon)}` : ""}`} padding="none" actions={ngayChon && <Button size="sm" variant="ghost" onClick={() => setNgayChon("")}>Mọi ngày</Button>}>
        <Table columns={cols} rows={ngayChon ? rows.filter((r) => r.day === ngayChon) : rows} getRowId={(r) => `${r.day}|${r.userId}`} dense maxHeight={360} state={{ loading: loading && !data, error: data ? undefined : error, onRetry: reload, empty: { title: "Chưa có số liệu" } }} />
      </Card>
      {!laNhanVien && (
        <div className="mt-3">
          <SoBienBan kinds={["nhat_ky_so_hoa"]} projectId={projectId} title="Sổ nhật ký đã chốt" description="Mỗi ngày một bản; cột Phông / ngày là ngày đã chốt." nguonDien={nguonDien} nutLap="Chốt nhật ký ngày…" embedded />
        </div>
      )}
    </section>
  );
}
