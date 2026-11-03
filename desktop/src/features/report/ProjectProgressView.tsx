// Copyright (c) 2026 DigiText. All rights reserved.
// Báo cáo tiến độ dự án: tổng dự án / trang / tệp, lọc ngày, mỗi dự án có thông tin hợp đồng (ngày ký, kết thúc,
// hạn chót, cán bộ phụ trách), tiến độ thời gian và % hoàn thành từng bước (quét, PDF/A, ký số, OCR đã duyệt).

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { useProject } from "@/state/project";
import { reportProjectProgress, reportSetContract, type ProjectContract, type ProjectProgress } from "@/api/progressReport";
import { dashboardProject, type ProjectDash } from "@/api/dashboard";
import { V4e5 } from "@/features/dossier/excel";
import { RagBadge, duKienText } from "@/features/home/blocks/common";
import { Badge, BarChart, Button, Card, Dialog, EmptyState, ErrorBanner, Field, FormGrid, Input, KpiRow, KpiTile, notifyError, PageHeader, Ring, Select, Skeleton, Tip, toast } from "@/ui";

const n = (v: number) => v.toLocaleString("vi-VN");
const pct = (a: number, b: number | null | undefined): number | null => (b && b > 0 ? Math.min(100, Math.floor((a / b) * 100)) : null);
const ngay = (s: string | null) => (s ? s.split("-").reverse().join("/") : "—");

/** % thời gian hợp đồng đã trôi qua (theo hôm nay). */
function timePct(c: ProjectContract, today = new Date()): number | null {
  if (!c.signedOn || !c.endOn) return null;
  const a = new Date(`${c.signedOn}T00:00:00`).getTime();
  const b = new Date(`${c.endOn}T23:59:59`).getTime();
  if (!(b > a)) return null;
  return Math.max(0, Math.min(100, Math.floor(((today.getTime() - a) / (b - a)) * 100)));
}

interface Buoc {
  key: string;
  ten: string;
  done: number;
  total: number | null;
  donVi: string;
  moTa: string;
}

function cacBuoc(p: ProjectProgress): Buoc[] {
  return [
    { key: "scan", ten: "Quét tài liệu", done: p.pages, total: p.contract.contractPages, donVi: "trang", moTa: "Trang đã quét trên số trang hợp đồng" },
    { key: "pdfa", ten: "PDF/A", done: p.pdfaFiles, total: p.files, donVi: "tệp", moTa: "Tệp đã có bản PDF/A trên tổng tệp" },
    { key: "ky", ten: "Ký số", done: p.signedFiles, total: p.files, donVi: "tệp", moTa: "Tệp đã ký số trên tổng tệp" },
    { key: "ocr", ten: "OCR", done: p.approvedFiles, total: p.files, donVi: "tệp", moTa: `Tệp đã duyệt trên tổng tệp. Chưa duyệt ${n(p.files - p.approvedFiles - p.rejectedFiles)}, từ chối ${n(p.rejectedFiles)}` },
  ];
}

function HopDongDialog({ p, onSaved, onClose }: { p: ProjectProgress; onSaved: (c: ProjectContract) => void; onClose: () => void }) {
  const [c, setC] = useState<ProjectContract>(p.contract);
  const [busy, setBusy] = useState(false);
  const so = (v: string) => (v.trim() === "" ? null : Math.max(0, Math.floor(Number(v)) || 0));
  const luu = async () => {
    setBusy(true);
    try {
      onSaved(await reportSetContract(p.projectId, c));
      toast.success("Đã lưu thông tin hợp đồng.");
    } catch (e) {
      notifyError(e, "lưu hợp đồng");
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      closable={!busy}
      title={`Hợp đồng dự án ${p.code}`}
      description={p.name}
      size="md"
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
      <FormGrid cols={2}>
        <Field label="Ngày ký hợp đồng">
          <Input type="date" value={c.signedOn ?? ""} onChange={(e) => setC({ ...c, signedOn: e.target.value || null })} />
        </Field>
        <Field label="Ngày kết thúc hợp đồng">
          <Input type="date" value={c.endOn ?? ""} onChange={(e) => setC({ ...c, endOn: e.target.value || null })} />
        </Field>
        <Field label="Hạn chót hoàn thành">
          <Input type="date" value={c.deadlineOn ?? ""} onChange={(e) => setC({ ...c, deadlineOn: e.target.value || null })} />
        </Field>
        <Field label="Cán bộ phụ trách">
          <Input value={c.managerName ?? ""} onChange={(e) => setC({ ...c, managerName: e.target.value })} />
        </Field>
        <Field label="Số trang hợp đồng">
          <Input type="number" min={0} value={c.contractPages ?? ""} onChange={(e) => setC({ ...c, contractPages: so(e.target.value) })} />
        </Field>
        <Field label="Số tệp hợp đồng">
          <Input type="number" min={0} value={c.contractFiles ?? ""} onChange={(e) => setC({ ...c, contractFiles: so(e.target.value) })} />
        </Field>
        <Field label="Số hộp hợp đồng" hint="Khối lượng TT05 để nghiệm thu">
          <Input type="number" min={0} value={c.contractBoxes ?? ""} onChange={(e) => setC({ ...c, contractBoxes: so(e.target.value) })} />
        </Field>
        <Field label="Số hồ sơ hợp đồng">
          <Input type="number" min={0} value={c.contractDossiers ?? ""} onChange={(e) => setC({ ...c, contractDossiers: so(e.target.value) })} />
        </Field>
        <Field label="Số tờ hợp đồng">
          <Input type="number" min={0} value={c.contractSheets ?? ""} onChange={(e) => setC({ ...c, contractSheets: so(e.target.value) })} />
        </Field>
      </FormGrid>
      <Field label="Ghi chú" className="mt-3">
        <Input value={c.note ?? ""} onChange={(e) => setC({ ...c, note: e.target.value })} />
      </Field>
    </Dialog>
  );
}

export default function ProjectProgressView() {
  const duAn = useProject();
  const [projectId, setProjectId] = useState("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [sua, setSua] = useState<ProjectProgress | null>(null);
  const [tatCa, setTatCa] = useState<ProjectProgress[]>([]);
  useEffect(() => {
    if (!projectId && duAn.projectId) setProjectId(duAn.projectId);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [duAn.projectId]);

  const loader = useCallback(async () => {
    if (!isTauri()) return [] as ProjectProgress[];
    const r = await reportProjectProgress({ projectId: projectId || undefined, from: from || undefined, to: to || undefined });
    if (!projectId) setTatCa(r);
    return r;
  }, [projectId, from, to]);
  const { data, error, loading, reload, setData } = V41t(loader, { topics: ["docs", "sign", "ocr"], errorText: () => "Không lấy được số liệu tiến độ." });
  // Khi xem MỘT dự án: sức khoẻ (RAG, dự kiến xong) và sản lượng theo ngày lấy từ dashboard máy chủ.
  const dashLoader = useCallback(() => dashboardProject(projectId, 30), [projectId]);
  const dash = V41t(dashLoader, { topics: ["dashboard"], enabled: !!projectId && (isTauri() || import.meta.env.DEV), errorText: () => "Không lấy được số theo ngày." });
  const dd: ProjectDash | undefined = projectId && dash.data?.projectId === projectId ? dash.data : undefined;
  const rows = useMemo(() => data ?? [], [data]);
  // Danh sách dự án cho ô chọn: lấy từ lần tải "tất cả" gần nhất hoặc chính kết quả hiện tại.
  const dsDuAn = tatCa.length ? tatCa : rows;

  const tong = useMemo(() => rows.reduce((a, r) => ({ duAn: a.duAn + 1, trang: a.trang + r.pages, file: a.file + r.files }), { duAn: 0, trang: 0, file: 0 }), [rows]);

  const xuat = () => {
    const aoa: (string | number)[][] = [["STT", "Mã dự án", "Tên dự án", "Ngày ký HĐ", "Ngày kết thúc HĐ", "Hạn chót", "Cán bộ phụ trách", "Trang đã quét", "Trang hợp đồng", "% Quét", "Tổng tệp", "Tệp PDF/A", "% PDF/A", "Tệp đã ký số", "% Ký số", "Tệp OCR đã duyệt", "% OCR", "% thời gian đã qua", "% hoàn thành công việc", "Tình trạng", "Dự kiến xong", "Trang quét/ngày (7 ngày)"]];
    const ragTen = { green: "Đúng tiến độ", amber: "Có rủi ro", red: "Chậm tiến độ", grey: "Chưa có hợp đồng" } as const;
    rows.forEach((r, i) => {
      const b = cacBuoc(r).map((x) => pct(x.done, x.total));
      const cv = trungBinh(b);
      const h = dd && dd.projectId === r.projectId ? dd.health : null;
      aoa.push([i + 1, r.code, r.name, ngay(r.contract.signedOn), ngay(r.contract.endOn), ngay(r.contract.deadlineOn), r.contract.managerName ?? "", r.pages, r.contract.contractPages ?? "", b[0] ?? "", r.files, r.pdfaFiles, b[1] ?? "", r.signedFiles, b[2] ?? "", r.approvedFiles, b[3] ?? "", timePct(r.contract) ?? "", cv ?? "", h ? ragTen[h.rag] : "", h?.projectedFinishOn ? ngay(h.projectedFinishOn) : "", h ? Math.round(h.runRate7d.pagesScanned) : ""]);
    });
    void V4e5([{ name: "TienDoDuAn", aoa }], `tien-do-du-an-${new Date().toISOString().slice(0, 10)}.xlsx`).then(() => toast.success("Đã xuất báo cáo tiến độ."));
  };

  return (
    <section className="view active">
      <PageHeader
        title="Tiến độ dự án"
        description="So tiến độ công việc với thời gian hợp đồng của từng dự án, số liệu trên máy chủ."
        actions={
          <Button variant="primary" icon="tepExcel" onClick={xuat} disabled={rows.length === 0}>
            Xuất Excel
          </Button>
        }
      />
      <Card className="mb-4" padding="sm">
        <FormGrid cols={4}>
          <Field label="Dự án">
            <Select size="sm" value={projectId} onChange={setProjectId} placeholder="Tất cả dự án" options={dsDuAn.map((p) => ({ value: p.projectId, label: `${p.code} · ${p.name}` }))} />
          </Field>
          <Field label="Từ ngày">
            <Input size="sm" type="date" value={from} max={to || undefined} onChange={(e) => setFrom(e.target.value)} />
          </Field>
          <Field label="Đến ngày">
            <Input size="sm" type="date" value={to} min={from || undefined} onChange={(e) => setTo(e.target.value)} />
          </Field>
        </FormGrid>
        <KpiRow className="mt-3">
          <KpiTile label="Dự án" value={tong.duAn} />
          <KpiTile label="Trang đã quét" value={tong.trang} />
          <KpiTile label="Tệp" value={tong.file} />
        </KpiRow>
      </Card>
      {error != null && <ErrorBanner error={error} className="mb-4" onRetry={reload} />}
      {loading && !data && <Skeleton.Rows rows={4} />}
      {data && rows.length === 0 && <EmptyState icon="baoCao" title="Không có dự án nào trong phạm vi được xem" />}

      {rows.map((r) => {
        const buoc = cacBuoc(r);
        const ptBuoc = buoc.map((b) => pct(b.done, b.total));
        const cv = trungBinh(ptBuoc);
        const tg = timePct(r.contract);
        const lech = cv != null && tg != null ? cv - tg : null;
        const quaHan = r.contract.deadlineOn != null && new Date(`${r.contract.deadlineOn}T23:59:59`).getTime() < Date.now() && (cv ?? 0) < 100;
        return (
          <Card
            key={r.projectId}
            className="mb-4"
            title={
              <span className="flex flex-wrap items-center gap-2">
                {r.code} · {r.name}
                {dd ? (
                  <RagBadge rag={dd.health.rag} />
                ) : (
                  <>
                    {quaHan && <Badge tone="err">Quá hạn chót</Badge>}
                    {lech != null && (lech >= 0 ? <Badge tone="ok">Đúng tiến độ, +{lech} điểm</Badge> : <Badge tone={lech < -10 ? "err" : "warn"}>Chậm {-lech} điểm</Badge>)}
                  </>
                )}
              </span>
            }
            actions={
              <Button size="sm" icon="sua" onClick={() => setSua(r)}>
                Hợp đồng
              </Button>
            }
          >
            <dl className="mb-3 grid grid-cols-2 gap-x-6 gap-y-1 text-sm sm:grid-cols-4">
              <div>
                <dt className="text-fg-subtle">Ngày ký hợp đồng</dt>
                <dd className="font-medium">{ngay(r.contract.signedOn)}</dd>
              </div>
              <div>
                <dt className="text-fg-subtle">Kết thúc hợp đồng</dt>
                <dd className="font-medium">{ngay(r.contract.endOn)}</dd>
              </div>
              <div>
                <dt className="text-fg-subtle">Hạn chót hoàn thành</dt>
                <dd className="font-medium">{ngay(r.contract.deadlineOn)}</dd>
              </div>
              <div>
                <dt className="text-fg-subtle">Cán bộ phụ trách</dt>
                <dd className="font-medium">{r.contract.managerName || "—"}</dd>
              </div>
            </dl>
            <div className="flex justify-between text-sm">
              <span>
                Thời gian hợp đồng đã qua: <b>{tg == null ? "chưa nhập ngày ký và kết thúc" : `${tg}%`}</b>
              </span>
              <span>
                Hoàn thành công việc: <b>{cv == null ? "—" : `${cv}%`}</b>
              </span>
            </div>
            <Tip text={tg != null ? `Vạch đỏ: ${tg}% thời gian đã qua` : "Chưa có thời gian hợp đồng"}>
              <div className="relative mt-1 h-2 rounded bg-border-subtle">
                <div className="h-2 rounded bg-brand-500" style={{ width: `${cv ?? 0}%` }} />
                {tg != null && <div className="absolute -top-[3px] h-[14px] w-0.5 bg-danger-600" style={{ left: `${tg}%` }} />}
              </div>
            </Tip>
            <div className="mt-4 grid gap-4" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(220px, 1fr))" }}>
              {buoc.map((b, i) => (
                <div key={b.key} className="flex items-center gap-3">
                  <Ring value={ptBuoc[i]} label={b.ten} />
                  <div className="text-sm">
                    <div className="font-semibold">{b.ten}</div>
                    <div>
                      {n(b.done)}/{b.total != null ? n(b.total) : "?"} {b.donVi}
                    </div>
                    <div className="text-xs text-fg-subtle">{b.moTa}</div>
                  </div>
                </div>
              ))}
            </div>
            {dd && (
              <div className="mt-4 border-t border-border-subtle pt-3">
                <p className="mb-2 text-sm text-fg-muted">{duKienText(dd.health, dd.contract.deadlineOn ?? dd.contract.endOn)} · quét {n(Math.round(dd.health.runRate7d.pagesScanned))} trang/ngày (7 ngày), cần {n(Math.round(dd.health.requiredRate.pagesScanned))}/ngày để kịp hạn.</p>
                <BarChart data={dd.series.map((s) => ({ label: s.day, value: s.pagesScanned }))} unit="trang" height={160} target={dd.health.requiredRate.pagesScanned > 0 ? { value: Math.round(dd.health.requiredRate.pagesScanned), label: "Cần/ngày" } : null} ariaLabel={`Trang quét theo ngày, ${dd.days} ngày`} />
              </div>
            )}
          </Card>
        );
      })}
      {sua && (
        <HopDongDialog
          p={sua}
          onClose={() => setSua(null)}
          onSaved={(c) => {
            setSua(null);
            setData((l) => (l ?? []).map((x) => (x.projectId === sua.projectId ? { ...x, contract: c } : x)));
          }}
        />
      )}
    </section>
  );
}

function trungBinh(v: (number | null)[]): number | null {
  const co = v.filter((x): x is number => x != null);
  return co.length ? Math.floor(co.reduce((a, b) => a + b, 0) / co.length) : null;
}
