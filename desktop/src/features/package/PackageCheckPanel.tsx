// Copyright (c) 2026 DigiText. All rights reserved.
// Đóng gói › Kiểm tra gói tin: kiểm nhiều gói SIP/AIP/DIP do phần mềm KHÁC tạo (tệp .zip, zip bọc zip, hoặc thư mục)
// theo TT 05/2025/TT-BNV + E-ARK CSIP — cấu trúc, METS, EAD, PREMIS, từng tệp PDF (PDF/A, lớp chữ, dpi, chữ ký số) —
// rồi xuất báo cáo PDF có đầu văn bản cơ quan và chỗ ký người kiểm / người duyệt.

import { useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { isTauri } from "@/api/ipc";
import { onPkgcheckProgress, pkgcheckCancel, pkgcheckReport, pkgcheckRun, type PkgFinding, type PkgPdfRow, type PkgProgress, type PkgReportHeader, type PkgResult } from "@/api/pkgcheck";
import { Badge, Banner, Button, Card, ErrorBanner, Field, FormGrid, Help, Input, KpiRow, KpiTile, Table, Toolbar, toast, type Column } from "@/ui";

const fmt = (n: number) => n.toLocaleString("vi-VN");
const HEADER0: PkgReportHeader = { parentOrg: "", org: "", place: "", inspector: "", inspectorTitle: "", approver: "", approverTitle: "" };

function nhoHeader(): PkgReportHeader {
  try {
    return { ...HEADER0, ...(JSON.parse(localStorage.getItem("pkgcheck.header") ?? "{}") as Partial<PkgReportHeader>) };
  } catch {
    return HEADER0;
  }
}
const toneOf = (v: string) => (v === "ĐẠT" ? "ok" : v.startsWith("ĐẠT") ? "warn" : "err");
const levelTone = { err: "err", warn: "warn", ok: "ok" } as const;
const levelText = { err: "Lỗi", warn: "Cảnh báo", ok: "Đạt" } as const;

function ChiTiet({ r }: { r: PkgResult }) {
  const [xemDat, setXemDat] = useState(false);
  const findings = r.findings.filter((f) => f.level !== "ok" || xemDat).sort((a, b) => ["err", "warn", "ok"].indexOf(a.level) - ["err", "warn", "ok"].indexOf(b.level));
  const fCols: Column<PkgFinding & { i: number }>[] = [
    { key: "level", header: "Mức", width: 96, cell: (f) => <Badge tone={levelTone[f.level]}>{levelText[f.level]}</Badge> },
    { key: "group", header: "Nhóm", width: 110 },
    { key: "message", header: "Nội dung", className: "whitespace-normal" },
  ];
  const pCols: Column<PkgPdfRow>[] = [
    { key: "file", header: "Tệp", className: "whitespace-normal" },
    { key: "pages", header: "Trang / lớp chữ", width: 110, cell: (p) => `${p.pages} / ${p.textPages}` },
    { key: "pdfa", header: "PDF/A", width: 140, cell: (p) => (p.pdfa ? (p.outputIntent ? p.pdfa : `${p.pdfa}, thiếu hồ sơ màu`) : <span className="text-danger-700">Không (PDF {p.pdfVersion})</span>) },
    { key: "minDpi", header: "Dpi", width: 60, align: "right", cell: (p) => p.minDpi ?? "—" },
    {
      key: "sig",
      header: "Chữ ký số",
      className: "whitespace-normal",
      cell: (p) =>
        !p.signed ? (
          <span className="text-warning-700">Chưa ký</span>
        ) : (
          <div className="text-xs leading-5">
            <div className={p.sigValid ? "text-success-700" : "text-danger-700"}>{p.sigValid ? "Toàn vẹn" : "KHÔNG toàn vẹn"}</div>
            {p.signer && <div>{p.signer}</div>}
            {p.signingTime && <div className="text-fg-subtle">ký {p.signingTime}</div>}
            <div className="text-fg-subtle">
              TSA {p.tsa ? "có" : "không"} · LTV {p.ltv ? "có" : "không"}
              {p.certUntil ? ` · CT hết hạn ${p.certUntil}` : ""}
            </div>
          </div>
        ),
    },
    { key: "verdict", header: "Kết luận", width: 220, className: "whitespace-normal", cell: (p) => <div className="text-xs leading-5"><Badge tone={p.verdict === "Đạt" ? "ok" : p.verdict.startsWith("Đạt") ? "warn" : "err"}>{p.verdict}</Badge>{p.problems.length > 0 && <div className="mt-1 text-fg-subtle">{p.problems.join("; ")}</div>}</div> },
  ];
  return (
    <div className="space-y-3 bg-surface-sunken p-3">
      <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
        <dt className="text-fg-subtle">Tệp</dt>
        <dd className="break-all">{r.source}</dd>
        <dt className="text-fg-subtle">OBJID</dt>
        <dd className="break-all">{r.objid || "—"}</dd>
        <dt className="text-fg-subtle">Đơn vị / phần mềm tạo</dt>
        <dd>{[r.creatorOrg, r.creatorSoftware].filter(Boolean).join(" · ") || "—"}</dd>
        {r.profile && (
          <>
            <dt className="text-fg-subtle">PROFILE</dt>
            <dd className="break-all">{r.profile}</dd>
          </>
        )}
      </dl>
      <div className="flex items-center justify-between">
        <h4 className="text-sm font-medium">
          {fmt(r.errors)} lỗi · {fmt(r.warnings)} cảnh báo · {fmt(r.findings.filter((f) => f.level === "ok").length)} mục đạt
        </h4>
        <Button size="sm" variant="ghost" onClick={() => setXemDat((v) => !v)}>
          {xemDat ? "Ẩn mục đạt" : "Hiện cả mục đạt"}
        </Button>
      </div>
      <Table columns={fCols} rows={findings.map((f, i) => ({ ...f, i }))} getRowId={(f) => String(f.i)} dense />
      {r.pdfs.length > 0 && (
        <>
          <h4 className="text-sm font-medium">Chi tiết {fmt(r.pdfs.length)} tệp PDF</h4>
          <Table columns={pCols} rows={r.pdfs} getRowId={(p) => p.file} dense maxHeight={420} />
        </>
      )}
    </div>
  );
}

export default function PackageCheckPanel() {
  const [paths, setPaths] = useState<string[]>([]);
  const [busy, setBusy] = useState<null | "run" | "report">(null);
  const [progress, setProgress] = useState<PkgProgress | null>(null);
  const [results, setResults] = useState<PkgResult[] | null>(null);
  const [err, setErr] = useState<unknown>(null);
  const [header, setHeader] = useState<PkgReportHeader>(nhoHeader);
  const [reportPath, setReportPath] = useState<string | null>(null);

  useEffect(() => {
    try {
      localStorage.setItem("pkgcheck.header", JSON.stringify(header));
    } catch {
      /* chỉ là tiện ích nhớ */
    }
  }, [header]);
  useEffect(() => {
    let off: (() => void) | undefined;
    void onPkgcheckProgress(setProgress).then((u) => {
      off = u;
    });
    return () => off?.();
  }, []);

  const pickFiles = async () => {
    const r = await open({ multiple: true, title: "Chọn tệp gói tin (.zip)", filters: [{ name: "Gói tin", extensions: ["zip"] }] });
    if (Array.isArray(r) && r.length) setPaths(r);
    else if (typeof r === "string") setPaths([r]);
  };
  const pickDir = async () => {
    const r = await open({ directory: true, multiple: false, title: "Chọn thư mục gói (hoặc thư mục chứa nhiều gói)" });
    if (typeof r === "string") setPaths([r]);
  };
  const run = async () => {
    setErr(null);
    setResults(null);
    setReportPath(null);
    setProgress(null);
    setBusy("run");
    try {
      setResults(await pkgcheckRun(paths));
    } catch (e) {
      setErr(e);
    } finally {
      setBusy(null);
    }
  };
  const xuatPdf = async () => {
    if (!results) return;
    const ngay = new Date().toISOString().slice(0, 10);
    setBusy("report");
    try {
      const p = await pkgcheckReport(results, header, `BAO-CAO-KIEM-TRA-GOI-${ngay}.pdf`);
      if (!p) return;
      setReportPath(p);
      toast.success("Đã xuất báo cáo PDF.", { description: p });
    } catch (e) {
      setErr(e);
    } finally {
      setBusy(null);
    }
  };

  const tong = useMemo(() => {
    const r = results ?? [];
    return {
      goi: r.length,
      dat: r.filter((x) => x.errors === 0).length,
      tep: r.reduce((s, x) => s + x.pdfs.length, 0),
      trang: r.reduce((s, x) => s + x.pdfs.reduce((a, p) => a + p.pages, 0), 0),
    };
  }, [results]);

  const cols: Column<PkgResult>[] = [
    { key: "verdict", header: "Kết luận", width: 150, cell: (r) => <Badge tone={toneOf(r.verdict)}>{r.verdict}</Badge> },
    { key: "kind", header: "Loại", width: 60 },
    { key: "name", header: "Gói", className: "whitespace-normal break-all" },
    { key: "fileCode", header: "Hồ sơ", className: "whitespace-normal", cell: (r) => (r.fileCode ? `${r.fileCode} — ${r.title}` : r.title || "—") },
    { key: "pdfs", header: "Tệp / trang", width: 100, align: "right", cell: (r) => `${fmt(r.pdfs.length)} / ${fmt(r.pdfs.reduce((a, p) => a + p.pages, 0))}` },
    { key: "errors", header: "Lỗi", width: 60, align: "right", cell: (r) => <span className={r.errors ? "text-danger-700 font-medium" : undefined}>{r.errors}</span> },
    { key: "warnings", header: "Cảnh báo", width: 80, align: "right" },
  ];
  const setH = (k: keyof PkgReportHeader) => (e: React.ChangeEvent<HTMLInputElement>) => setHeader((h) => ({ ...h, [k]: e.target.value }));

  return (
    <div className="space-y-4">
      {!isTauri() && <Banner tone="info">Chạy trong ứng dụng desktop để kiểm tra gói tin.</Banner>}
      <Card title="Gói cần kiểm" description="Chọn một hay nhiều tệp .zip, hoặc một thư mục (thư mục gói có METS.xml, hay thư mục chứa nhiều gói — quét cả thư mục con). Gói bị bọc thêm một lớp zip cũng đọc được.">
        <Toolbar>
          <Button icon="goi" onClick={() => void pickFiles()} disabled={busy !== null}>
            Chọn tệp gói
          </Button>
          <Button icon="thuMuc" onClick={() => void pickDir()} disabled={busy !== null}>
            Chọn thư mục
          </Button>
          <Button variant="primary" icon="kiemTra" loading={busy === "run"} disabled={paths.length === 0 || busy !== null} onClick={() => void run()}>
            Kiểm tra
          </Button>
          {busy === "run" && (
            <Button variant="dangerOutline" onClick={() => void pkgcheckCancel()}>
              Dừng
            </Button>
          )}
        </Toolbar>
        {paths.length > 0 && <p className="mt-2 break-all text-sm text-fg-subtle">{paths.length === 1 ? paths[0] : `${fmt(paths.length)} tệp: ${paths.slice(0, 3).join("; ")}${paths.length > 3 ? "…" : ""}`}</p>}
        {busy === "run" && progress && progress.total > 0 && (
          <div className="mt-3">
            <div className="h-2 rounded bg-border-subtle" role="progressbar" aria-label="Tiến độ kiểm tra gói" aria-valuemin={0} aria-valuemax={progress.total} aria-valuenow={progress.done}>
              <div className="h-2 rounded bg-run-solid transition-[width]" style={{ width: `${(progress.done / progress.total) * 100}%` }} />
            </div>
            <p className="mt-1 text-sm text-fg-subtle">
              Gói {fmt(Math.min(progress.done + 1, progress.total))}/{fmt(progress.total)}: {progress.name}
            </p>
          </div>
        )}
      </Card>

      <ErrorBanner error={err} />

      {results && (
        <>
          <Card padding="sm">
            <KpiRow>
              <KpiTile label="Gói đã kiểm" value={fmt(tong.goi)} />
              <KpiTile label="Đạt" value={fmt(tong.dat)} tone="ok" />
              <KpiTile label="Không đạt" value={fmt(tong.goi - tong.dat)} tone={tong.goi - tong.dat > 0 ? "err" : undefined} />
              <KpiTile label="Tệp PDF" value={fmt(tong.tep)} />
              <KpiTile label="Trang" value={fmt(tong.trang)} />
            </KpiRow>
          </Card>
          <Card padding="none" title="Kết quả" description="Bấm vào một gói để xem lỗi, cảnh báo và chi tiết từng tệp PDF.">
            <Table columns={cols} rows={results} getRowId={(r) => r.source} renderExpanded={(r) => <ChiTiet r={r} />} dense />
          </Card>

          <Card title="Xuất báo cáo PDF" description="Thông tin đầu văn bản và khối ký — được nhớ cho lần sau.">
            <FormGrid cols={3}>
              <Field label="Cơ quan chủ quản">
                <Input value={header.parentOrg} onChange={setH("parentOrg")} placeholder="Có thể để trống" />
              </Field>
              <Field label="Đơn vị kiểm tra">
                <Input value={header.org} onChange={setH("org")} />
              </Field>
              <Field label="Địa danh">
                <Input value={header.place} onChange={setH("place")} placeholder="Hà Nội" />
              </Field>
              <Field label="Người kiểm tra">
                <Input value={header.inspector} onChange={setH("inspector")} />
              </Field>
              <Field label="Chức danh người kiểm">
                <Input value={header.inspectorTitle} onChange={setH("inspectorTitle")} placeholder="(Ký, ghi rõ họ tên)" />
              </Field>
              <div />
              <Field label="Người duyệt">
                <Input value={header.approver} onChange={setH("approver")} />
              </Field>
              <Field label="Chức danh người duyệt">
                <Input value={header.approverTitle} onChange={setH("approverTitle")} placeholder="(Ký, ghi rõ họ tên)" />
              </Field>
            </FormGrid>
            <div className="mt-3 flex flex-wrap items-center gap-3">
              <Button variant="primary" icon="tepPdf" loading={busy === "report"} disabled={busy !== null} onClick={() => void xuatPdf()}>
                Xuất báo cáo PDF
              </Button>
              {reportPath && <span className="break-all text-sm text-fg-subtle">Đã lưu: {reportPath}</span>}
            </div>
          </Card>
        </>
      )}

      <Help title="Kiểm những gì">
        <p>Cấu trúc gói (một thư mục gốc, OBJID khớp tên, loại gói OAIS, PROFILE E-ARK CSIP), METS (mọi tham chiếu tồn tại, đúng kích thước và mã băm SHA-256, không có tệp mồ côi), EAD hồ sơ và văn bản (trường bắt buộc, bảng mã TT05, ngày tháng, totalDoc / số trang, đối chiếu schema đi kèm gói), PREMIS (mã băm từng tệp).</p>
        <p>Từng tệp PDF: khai PDF/A và hồ sơ màu, lớp chữ, độ phân giải; chữ ký số còn toàn vẹn, người ký, đơn vị cấp chứng thư, thời điểm ký, dấu thời gian (TSA), thông tin xác minh lâu dài (LTV).</p>
        <p>PDF/A ở đây dựa trên dấu hiệu tệp tự khai; muốn kết luận theo ISO 19005 đầy đủ, dùng thêm tab Kiểm PDF/A với veraPDF.</p>
      </Help>
    </div>
  );
}
