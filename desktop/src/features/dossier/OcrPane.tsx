// Copyright (c) 2026 DigiText. All rights reserved.
// OCR & bóc tách: Tiến độ (số liệu theo dự án) · Chạy xử lý (việc trên máy chủ) · Soát & duyệt (bảng tài liệu,
// mở màn bóc tách trường). Dự án mặc định lấy từ ô "Dự án đang làm" trên Topbar, vẫn đổi được tại chỗ.

import { useCallback, useEffect, useState } from "react";
import { V41p } from "@/live/useLive";
import { open } from "@tauri-apps/plugin-dialog";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { isTauri } from "@/api/ipc";
import { V3z9 } from "@/domain/permissions";
import { V4et, V4f3, V4er, V4ek, type V4f2, type V4f0, type V4f1 } from "@/api/docs";
import { inMetadata, layTruongNhieuTaiLieu, tenTepAnToan, xuatMetadataExcel, type MetaRow } from "./metadataXuat";
import { V4ak, type V4a9 } from "@/api/users";
import V4i7, { type V4i6 } from "../upload/OcrFieldsView";
import V4k5 from "../server/ServerJobs";
import ReviewHoSoTable from "./ReviewHoSoTable";
import KieExamplesPanel from "./KieExamplesPanel";
import { V4hg, pdfaSummary, V4hi, V4hj, V4hh, type V4he } from "./ocrReport";
import { Badge, Button, Card, Field, Input, KpiRow, KpiTile, Menu, notifyError, PageHeader, Select, Table, Tabs, toast, Toolbar, type Column } from "@/ui";

const fmtN = (n: number) => n.toLocaleString("vi-VN");
type Tab = "tiendo" | "chay" | "soat" | "vidu";

function BocTachCell({ c }: { c: V4f0 }) {
  return (
    <>
      <div>{fmtN(c.ocrDone)}</div>
      {c.fieldDocs != null && c.fieldDocs > 0 && <div className="text-xs text-fg-subtle">{fmtN(c.fieldDocs)} bóc trường</div>}
    </>
  );
}

function PdfaCell({ c }: { c: V4f0 }) {
  const t = pdfaSummary(c);
  if (!t.known) return <span className="text-fg-subtle">—</span>;
  return (
    <>
      <div>
        {fmtN(t.docs)}
        <span className="text-fg-subtle text-sm">/{fmtN(t.total)}</span>
      </div>
      {t.old > 0 && <div className="text-xs text-warning-700">{fmtN(t.old)} chưa đạt 2b</div>}
    </>
  );
}

export default function OcrPane() {
  const { session } = V460();
  const duAnTopbar = useProject();
  const [tab, setTab] = useState<Tab>("tiendo");
  const [dangTaiLen, setDangTaiLen] = useState(false);
  const [dangXepHang, setDangXepHang] = useState(false);
  const [xuatBan, setXuatBan] = useState(false);

  const [projects, setProjects] = useState<V4a9[]>([]);
  const [chonDuAn, setChonDuAn] = useState("");
  const [docMo, setDocMo] = useState<V4i6 | null>(null);

  const [khoang, setKhoang] = useState<V4he>(() => V4hg());
  const [report, setReport] = useState<V4f2 | null>(null);
  const [dangTinh, setDangTinh] = useState(true);

  const duAnTaiKhoan = session?.projectId ?? null;
  // Không gắn dự án ở tài khoản → mặc định theo Topbar, đổi tại chỗ được.
  useEffect(() => {
    if (!duAnTaiKhoan && duAnTopbar.projectId) setChonDuAn(duAnTopbar.projectId);
  }, [duAnTaiKhoan, duAnTopbar.projectId]);
  const duAnHieuLuc = duAnTaiKhoan ?? chonDuAn;
  const coQuyenDuyet = V3z9(session?.permissions, "document.review");
  const loiKhoang = V4hh(khoang);

  const napBaoCao = useCallback(async () => {
    if (!isTauri() || V4hh(khoang)) {
      setDangTinh(false);
      return;
    }
    try {
      setReport(await V4f3(khoang.from || undefined, khoang.to || undefined));
    } catch (e) {
      notifyError(e, "tính số liệu");
    } finally {
      setDangTinh(false);
    }
  }, [khoang]);
  V41p(napBaoCao, { topics: ["docs", "ocr"] });

  const napDuAn = useCallback(() => {
    V4ak()
      .then((all) => setProjects(all.filter((p) => p.isActive)))
      .catch(() => {});
  }, []);
  V41p(napDuAn, { topics: ["projects"], enabled: !duAnTaiKhoan && isTauri() });

  const taiPdfLen = async () => {
    if (!duAnHieuLuc) {
      toast.warning(projects.length > 1 ? "Chọn dự án đích trước khi tải tài liệu lên." : "Tài khoản chưa gắn dự án nào nên không tải tài liệu lên được.");
      return;
    }
    let path: string | null;
    try {
      const r = await open({ multiple: false, title: "Chọn PDF để gán nhãn", filters: [{ name: "PDF", extensions: ["pdf"] }] });
      path = typeof r === "string" ? r : null;
    } catch (e) {
      notifyError(e, "chọn tệp");
      return;
    }
    if (!path) return;
    setDangTaiLen(true);
    try {
      const r = await V4er(duAnHieuLuc, path);
      void napBaoCao();
      toast.success(r.duplicate ? `"${r.filename}" đã có sẵn trên máy chủ.` : `Đã tải "${r.filename}" lên máy chủ.`);
      setDocMo({ id: r.id, filename: r.filename, reviewStatus: "pending" });
    } catch (e) {
      notifyError(e, "tải PDF lên");
    } finally {
      setDangTaiLen(false);
    }
  };

  const ocrToanBo = async () => {
    setDangXepHang(true);
    try {
      const r = await V4et(duAnHieuLuc || undefined);
      const boQua = r.skippedNoPageCount > 0 ? ` Bỏ qua ${fmtN(r.skippedNoPageCount)} tài liệu chưa rõ số trang.` : "";
      if (r.documents === 0) toast.info(`Không có tài liệu mới để OCR.${boQua}`);
      else toast.success(`Đã đưa ${fmtN(r.documents)} tài liệu (${fmtN(r.pages)} trang) vào hàng đợi OCR.${boQua}`);
      void napBaoCao();
    } catch (e) {
      notifyError(e, "xếp hàng OCR");
    } finally {
      setDangXepHang(false);
    }
  };

  const xuatMeta = async (kieu: "excel" | "in") => {
    setXuatBan(true);
    const id = toast.info("Đang lấy trường đã bóc tách…");
    try {
      const all = await V4ek();
      const docs = all.filter((d) => d.status !== "disposed" && (!duAnHieuLuc || (d.projectId ?? "") === duAnHieuLuc));
      if (docs.length === 0) {
        toast.warning(duAnHieuLuc ? "Dự án đang chọn chưa có tài liệu nào." : "Chưa có tài liệu nào trên máy chủ.");
        return;
      }
      const truong = await layTruongNhieuTaiLieu(
        docs.map((d) => d.id),
        () => {},
      );
      const duyet: Record<string, string> = { pending: "Chờ duyệt", approved: "Đã duyệt", rejected: "Từ chối" };
      const boc: Record<string, string> = { done: "đã bóc", queued: "chờ bóc", running: "đang bóc", failed_transient: "bóc lỗi", page_failed: "bóc lỗi" };
      const rows: MetaRow[] = docs
        .sort((a, b) => (a.hoSoRef ?? "").localeCompare(b.hoSoRef ?? "", "vi", { numeric: true }) || a.filename.localeCompare(b.filename, "vi", { numeric: true }))
        .map((d) => {
          const f = truong.get(d.id);
          return {
            id: d.id,
            filename: d.filename,
            hoSoRef: d.hoSoRef ?? "",
            relPath: d.relPath ?? "",
            fields: f?.fields ?? {},
            daXacNhan: f?.daXacNhan ?? false,
            trangThai: `${duyet[d.reviewStatus ?? "pending"] ?? d.reviewStatus ?? ""}${d.fieldsStatus ? ` (${boc[d.fieldsStatus] ?? d.fieldsStatus})` : " (chưa bóc)"}`,
          };
        });
      const p = projects.find((x) => x.id === duAnHieuLuc);
      const ten = p ? `${p.code} — ${p.name}` : session?.projectName ?? "";
      if (kieu === "excel") await xuatMetadataExcel(rows, `metadata-${tenTepAnToan(p?.code ?? session?.projectCode ?? "du-an")}.xlsx`);
      else inMetadata(rows, "METADATA TÀI LIỆU ĐÃ BÓC TÁCH", ten);
      toast.success(`${kieu === "excel" ? "Đã xuất" : "Đã in"} metadata ${fmtN(rows.length)} tài liệu.`);
    } catch (e) {
      notifyError(e, "xuất metadata");
    } finally {
      toast.dismiss(id);
      setXuatBan(false);
    }
  };

  if (docMo) {
    return (
      <V4i7
        doc={docMo}
        onBack={() => {
          setDocMo(null);
          void napBaoCao();
        }}
      />
    );
  }

  const totals = report?.totals;
  const duAnRows = report?.projects ?? [];
  const cols: Column<V4f1>[] = [
    {
      key: "name",
      header: "Dự án",
      cell: (p) => (
        <>
          {p.name} <span className="text-sm text-fg-subtle">· {p.code}</span> {!p.isActive && <Badge tone="gray">ngưng</Badge>}
        </>
      ),
    },
    { key: "docs", header: "Tổng tệp", align: "right", width: 90, cell: (p) => fmtN(p.docs) },
    { key: "ocr", header: "Đã bóc tách", align: "right", width: 120, cell: (p) => <BocTachCell c={p} /> },
    { key: "pdfa", header: "Đã PDF/A", align: "right", width: 110, cell: (p) => <PdfaCell c={p} /> },
    { key: "approved", header: "Đã duyệt", align: "right", width: 90, cell: (p) => <span className={p.approved > 0 ? "text-success-700" : undefined}>{fmtN(p.approved)}</span> },
    { key: "rejected", header: "Từ chối", align: "right", width: 90, cell: (p) => <span className={p.rejected > 0 ? "text-danger-700" : undefined}>{fmtN(p.rejected)}</span> },
    { key: "pending", header: "Chờ xác nhận", align: "right", width: 110, cell: (p) => <span className={p.pending > 0 ? "text-warning-700" : undefined}>{fmtN(p.pending)}</span> },
    {
      key: "pct",
      header: "Hoàn thành",
      width: 150,
      cell: (p) => (
        <>
          <div className="progress mb-1 min-w-22.5">
            <span style={{ width: `${Math.min(100, V4hi(p))}%` }} />
          </div>
          <span className="text-sm">{V4hj(p)}</span>
        </>
      ),
    },
  ];

  // Topbar đã chọn dự án thì không lặp lại ô chọn ở đây; chỉ hiện khi Topbar đang để "mọi dự án".
  const chonDuAnUI = !duAnTaiKhoan && !duAnTopbar.projectId && projects.length > 0 && (
    <Select size="sm" className="w-56" value={chonDuAn} onChange={setChonDuAn} placeholder="Tất cả dự án" options={projects.map((p) => ({ value: p.id, label: `${p.code} · ${p.name}` }))} aria-label="Dự án" />
  );

  return (
    <section className="view active">
      <PageHeader
        title="OCR & bóc tách"
        description="Theo dõi tiến độ OCR, chạy xử lý trên máy chủ và soát, duyệt kết quả bóc tách 16 trường."
        actions={
          <>
            <Button variant="primary" icon="taiLen" loading={dangTaiLen} onClick={() => void taiPdfLen()}>
              Tải PDF lên
            </Button>
            <Menu
              items={[
                { label: "OCR toàn bộ tài liệu mới", icon: "chay", onSelect: () => void ocrToanBo(), disabled: dangXepHang },
                "-",
                { label: "Xuất metadata Excel", icon: "tepExcel", onSelect: () => void xuatMeta("excel"), disabled: xuatBan },
                { label: "In metadata", icon: "inRa", onSelect: () => void xuatMeta("in"), disabled: xuatBan },
              ]}
            />
          </>
        }
      />
      <Tabs<Tab>
        value={tab}
        onChange={setTab}
        items={[
          { value: "tiendo", label: "Tiến độ", icon: "thongke" },
          { value: "chay", label: "Chạy xử lý", icon: "chay" },
          { value: "soat", label: "Soát & duyệt", icon: "duyet" },
          ...(coQuyenDuyet ? [{ value: "vidu" as const, label: "Ví dụ dạy máy", icon: "danhSach" as const }] : []),
        ]}
      />
      <div className="mt-3 space-y-4">
        {tab === "tiendo" && (
          <>
            <Card>
              <KpiRow>
                <KpiTile label="Dự án" value={report?.projectsCount ?? 0} />
                <KpiTile label="Tổng tệp" value={totals?.docs ?? 0} sub={`${fmtN(totals?.pages ?? 0)} trang`} />
                <KpiTile label="Đã OCR" value={totals?.ocrDone ?? 0} sub={totals?.fieldDocs ? `${fmtN(totals.fieldDocs)} đã bóc 16 trường` : undefined} />
                <KpiTile label="Đã duyệt" value={totals?.approved ?? 0} tone="ok" />
                <KpiTile label="Từ chối" value={totals?.rejected ?? 0} tone="err" />
                <KpiTile label="Chờ xác nhận" value={totals?.pending ?? 0} tone="warn" />
              </KpiRow>
              <Toolbar className="mt-4 border-t border-border-subtle pt-3">
                <Field label="Tải lên từ" inline>
                  <Input size="sm" type="date" value={khoang.from} max={khoang.to || undefined} onChange={(e) => setKhoang((r) => ({ ...r, from: e.target.value }))} />
                </Field>
                <Field label="đến" inline>
                  <Input size="sm" type="date" value={khoang.to} min={khoang.from || undefined} onChange={(e) => setKhoang((r) => ({ ...r, to: e.target.value }))} />
                </Field>
                <Button size="sm" variant="ghost" onClick={() => setKhoang({ from: "", to: "" })}>
                  Toàn bộ
                </Button>
                <Button size="sm" variant="ghost" onClick={() => setKhoang(V4hg())}>
                  Tháng này
                </Button>
                <span className={`text-sm ${loiKhoang ? "text-danger-700" : "text-fg-subtle"}`}>{loiKhoang ?? (dangTinh ? "Đang tính số liệu…" : khoang.from || khoang.to ? `Đang xem ${khoang.from || "đầu kỳ"} đến ${khoang.to || "nay"}` : "Đang xem toàn bộ")}</span>
              </Toolbar>
            </Card>
            <Card title="Theo dự án" description="Bấm một dòng để lọc tab Soát & duyệt theo dự án đó." padding="none">
              <Table
                columns={cols}
                rows={duAnRows}
                getRowId={(p) => p.projectId}
                onRowClick={(p) => {
                  setChonDuAn((cu) => (cu === p.projectId ? "" : p.projectId));
                  setTab("soat");
                }}
                rowClassName={(p) => (p.projectId === chonDuAn ? "bg-brand-50" : undefined)}
                state={{ loading: dangTinh && !report, empty: { title: "Chưa có dự án nào trong phạm vi phụ trách" } }}
              />
              {totals && duAnRows.length > 1 && (
                <p className="px-4 py-2 text-sm text-fg-subtle">
                  Tổng cộng {fmtN(totals.docs)} tệp · đã duyệt {fmtN(totals.approved)} · từ chối {fmtN(totals.rejected)} · chờ {fmtN(totals.pending)} · {V4hj(totals)}
                </p>
              )}
            </Card>
          </>
        )}
        {tab === "chay" && <V4k5 />}
        {tab === "vidu" && coQuyenDuyet && <KieExamplesPanel projectId={duAnHieuLuc} projects={projects} />}
        {tab === "soat" && <ReviewHoSoTable canReview={coQuyenDuyet} onlyProject={duAnTaiKhoan ? undefined : chonDuAn} onOpenDoc={(f) => setDocMo({ ...f, reviewStatus: f.review })} onDone={() => void napBaoCao()} actions={chonDuAnUI} />}
      </div>
    </section>
  );
}
