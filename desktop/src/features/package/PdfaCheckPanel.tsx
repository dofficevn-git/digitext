// Copyright (c) 2026 DigiText. All rights reserved.
// Kiểm tra PDF/A cho tệp hoặc thư mục bất kỳ: loại tự khai (PDF/A-1B, 2B, 3B…), dấu hiệu bắt buộc, và khi máy có
// Docker thì veraPDF kết luận đạt / không đạt kèm luật vi phạm.

import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { onExtimportProgress, pdfaCheck, pdfaVerapdfAvailable, type ExtProgress, type PdfaFileReport, type PdfaSummary } from "@/api/extimport";
import { isTauri } from "@/api/ipc";
import { Badge, Banner, Button, Card, Checkbox, ErrorBanner, Field, Help, Table, Tip, Toolbar, type Column } from "@/ui";

const fmt = (n: number) => n.toLocaleString("vi-VN");
function badgeOf(f: PdfaFileReport): { tone: "err" | "ok" | "info"; text: string } {
  if (f.error) return { tone: "err", text: "Hỏng" };
  if (f.verapdf) return f.verapdf.compliant ? { tone: "ok", text: "Đạt" } : { tone: "err", text: "Không đạt" };
  if (f.looksPdfa) return { tone: "info", text: "Có khai PDF/A" };
  return { tone: "err", text: "Không phải PDF/A" };
}

export default function PdfaCheckPanel() {
  const [paths, setPaths] = useState<string[]>([]);
  const [useVera, setUseVera] = useState(true);
  const [veraOk, setVeraOk] = useState<boolean | null>(null);
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState<ExtProgress | null>(null);
  const [res, setRes] = useState<PdfaSummary | null>(null);
  const [err, setErr] = useState<unknown>(null);

  useEffect(() => {
    if (!isTauri()) return;
    pdfaVerapdfAvailable()
      .then(setVeraOk)
      .catch(() => setVeraOk(false));
    let off: (() => void) | undefined;
    void onExtimportProgress((p) => {
      if (p.phase === "pdfa") setProgress(p);
    }).then((u) => {
      off = u;
    });
    return () => off?.();
  }, []);

  const pickFiles = async () => {
    const r = await open({ multiple: true, title: "Chọn tệp PDF cần kiểm", filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (Array.isArray(r) && r.length) setPaths(r);
    else if (typeof r === "string") setPaths([r]);
  };
  const pickDir = async () => {
    const r = await open({ directory: true, multiple: false, title: "Chọn thư mục chứa PDF (quét cả thư mục con)" });
    if (typeof r === "string") setPaths([r]);
  };
  const run = async () => {
    setErr(null);
    setRes(null);
    setProgress(null);
    setBusy(true);
    try {
      setRes(await pdfaCheck({ paths, useVerapdf: useVera && veraOk === true }));
    } catch (e) {
      setErr(e);
    } finally {
      setBusy(false);
    }
  };

  const cols: Column<PdfaFileReport>[] = [
    { key: "kq", header: "Kết luận", width: 140, cell: (f) => { const b = badgeOf(f); return <Badge tone={b.tone}>{b.text}</Badge>; } },
    { key: "declared", header: "Loại tự khai", width: 120, cell: (f) => f.declared ?? "Không khai" },
    { key: "name", header: "Tệp", className: "whitespace-normal", cell: (f) => <Tip text={f.path}><span>{f.name}</span></Tip> },
    { key: "hasOutputIntent", header: "Hồ sơ màu", width: 100, cell: (f) => (f.hasOutputIntent ? "Có" : <span className="text-danger-700">Không</span>) },
    { key: "textPages", header: "Lớp chữ", width: 100, cell: (f) => `${f.textPages}/${f.pages} trang` },
    { key: "minDpi", header: "dpi", width: 110, cell: (f) => `${f.minDpi ?? "—"}${f.isMap ? " (bản đồ, cần 300)" : ""}` },
    { key: "signed", header: "Ký số", width: 80, cell: (f) => (f.signed ? "Có" : "Không") },
    { key: "producer", header: "Phần mềm tạo", className: "text-sm text-fg-subtle", cell: (f) => f.producer ?? "—", hideBelow: 1440 },
  ];

  return (
    <div className="space-y-4">
      <Card title="Kiểm tra PDF/A" description="Chọn tệp hoặc cả thư mục để biết từng tệp là loại PDF/A nào hay không phải PDF/A, kèm lý do.">
        <Toolbar>
          <Button icon="tepPdf" disabled={busy} onClick={() => void pickFiles()}>
            Chọn tệp PDF
          </Button>
          <Button icon="thuMuc" disabled={busy} onClick={() => void pickDir()}>
            Chọn thư mục
          </Button>
          <span className="text-sm text-fg-subtle">{paths.length === 0 ? "Chưa chọn gì" : paths.length === 1 ? paths[0] : `${fmt(paths.length)} tệp đã chọn`}</span>
        </Toolbar>
        <Toolbar className="mt-3">
          <Tip text={veraOk === false ? "Máy này chưa có Docker và veraPDF, chỉ xem được loại tự khai và dấu hiệu bắt buộc" : "Kết quả veraPDF dùng để nghiệm thu"}>
            <span>
              <Field label="Kiểm đúng chuẩn ISO bằng veraPDF" inline>
                <Checkbox checked={useVera && veraOk === true} onChange={setUseVera} disabled={busy || veraOk !== true} />
              </Field>
            </span>
          </Tip>
          <Toolbar.Spacer />
          <Button variant="primary" icon="kiemTraPdfa" loading={busy} disabled={!isTauri() || paths.length === 0} onClick={() => void run()}>
            Kiểm tra PDF/A
          </Button>
        </Toolbar>
      </Card>

      {busy && progress && progress.total > 0 && (
        <Banner tone="info">
          Đang kiểm PDF/A: {fmt(progress.done)}/{fmt(progress.total)} bước
        </Banner>
      )}
      {err !== null && <ErrorBanner error={err} />}

      {res && (
        <Card
          tone={res.notOk === 0 ? undefined : "warning"}
          padding="none"
          title={`${fmt(res.total)} tệp: ${fmt(res.ok)} ${res.verapdfUsed ? "đạt veraPDF" : "có khai PDF/A đủ dấu hiệu"}, ${fmt(res.notOk)} không đạt`}
          description={`Theo loại: ${Object.entries(res.byKind).map(([k, v]) => `${k} ${fmt(v)}`).join(" · ")}${!res.verapdfUsed ? " · chưa chạy veraPDF, đây là loại tự khai" : ""}${res.truncated ? " · quá 5.000 tệp, chỉ kiểm 5.000 tệp đầu" : ""}`}
        >
          <Table columns={cols} rows={res.files} getRowId={(f) => f.path} maxHeight={480} dense renderExpanded={(f) => { const d = [...(f.error ? [f.error] : f.problems), ...(f.verapdf?.rules ?? [])]; return d.length ? <ul className="list-disc px-8 py-2 text-sm">{d.map((x, i) => <li key={i}>{x}</li>)}</ul> : null; }} />
          <p className="px-4 py-2 text-xs text-fg-subtle">
            {res.total > res.files.length ? `Bảng hiện ${fmt(res.files.length)} tệp đầu, tệp không đạt xếp trước. ` : ""}
            {res.reportFile ? (
              <>
                Bảng đầy đủ: <code>{res.reportFile}</code>
              </>
            ) : null}
          </p>
        </Card>
      )}

      <Help title="Lớp chữ không phải PDF/A">
        <p>Bôi được chữ bằng Ctrl+A chỉ chứng minh tệp có lớp chữ. PDF/A còn đòi hồ sơ màu, siêu dữ liệu XMP, font nhúng và không có thành phần bị cấm; veraPDF kiểm đủ các luật đó theo ISO 19005.</p>
      </Help>
    </div>
  );
}
