// Copyright (c) 2026 DigiText. All rights reserved.
// Tách TIFF (tài liệu Đảng) — bước giữa Ký số và Đóng gói. Nguồn: bản đã ký trên máy chủ (dự án → phông → hồ sơ)
// hoặc thư mục PDF trên máy. Mỗi trang một TIFF lấy nguyên ảnh quét; tên theo mẫu của đơn vị (v1 / v2, nhớ theo
// dự án). Xem trước cây thư mục, chạy song song, tạm dừng / dừng, chờ khi ổ đích rớt, biên bản + SHA-256.

import { useEffect, useMemo, useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { isTauri } from "@/api/ipc";
import { useProject } from "@/state/project";
import { V4ak, type V4a9 } from "@/api/users";
import { docsForGroups, docsReviewSummary, type ReviewSummaryRow } from "@/api/docs";
import { onTiffProgress, tiffCancel, tiffPause, tiffPreset, tiffPreview, tiffRun, tiffScanLocal, type TiffInput, type TiffNaming, type TiffPreview, type TiffRule, type TiffRunResult } from "@/api/tiff";
import { V4e3 } from "@/features/dossier/excel";
import { errorText } from "@/lib/errors";
import { Badge, Banner, Button, Card, Checkbox, confirm, EmptyState, Field, FormGrid, Help, Input, KpiRow, KpiTile, notifyError, PageHeader, Seg, Select, Table, toast, usePagination, pageSlice, type Column } from "@/ui";

const fmtN = (n: number) => n.toLocaleString("vi-VN");
const LOG_MAX = 500;

function nho<T>(key: string, fallback: T): T {
  try {
    const v = localStorage.getItem(key);
    return v == null ? fallback : { ...fallback, ...(JSON.parse(v) as T) };
  } catch {
    return fallback;
  }
}
function luu(key: string, v: unknown): void {
  try {
    localStorage.setItem(key, JSON.stringify(v));
  } catch {
    /* chỉ là tiện ích nhớ */
  }
}
function hhmmss(ms: number): string {
  const s = Math.max(0, Math.round(ms / 1000));
  return [Math.floor(s / 3600), Math.floor((s % 3600) / 60), s % 60].map((x) => String(x).padStart(2, "0")).join(":");
}

type Nguon = "server" | "local";
interface Opts {
  outDir: string;
  onExist: "skip" | "overwrite";
  forceRgb: boolean;
  uncompressed: boolean;
}
const OPTS0: Opts = { outDir: "", onExist: "skip", forceRgb: false, uncompressed: false };
const hoSoKey = (r: ReviewSummaryRow) => `${r.phong}\u0001${r.hop}\u0001${r.hoSo}`;

const MO_TA_MAU: Record<TiffRule, string> = {
  v1: "Đổi thư mục chứa PDF thành mã hồ sơ: H49.44-ĐU-KHAC-0001-1976 → H49.44-ĐU-KHAC-1976; H49.44-001-01-0025-001 → H49.44-001-01-0025.",
  v2: "Văn bản Đảng (-ĐU-, -NQ-…) giữ tên thư mục gốc (01, 02…); tên dạng mã cơ quan H49.44-001-01-0042-001 → H49.44-001-01-0042.",
};

export default function TachTiffView() {
  const duAnTopbar = useProject();
  const [nguon, setNguon] = useState<Nguon>(() => (localStorage.getItem("tiff.nguon") as Nguon) || "server");
  const [projects, setProjects] = useState<V4a9[]>([]);
  const [projectId, setProjectId] = useState<string>(() => localStorage.getItem("tiff.projectId") ?? "");
  const [hoSo, setHoSo] = useState<ReviewSummaryRow[]>([]);
  const [phong, setPhong] = useState("");
  const [q, setQ] = useState("");
  const [chon, setChon] = useState<Set<string>>(new Set());
  const [folder, setFolder] = useState<string>(() => localStorage.getItem("tiff.folder") ?? "");
  const [inputs, setInputs] = useState<TiffInput[]>([]);
  const [nhanNguon, setNhanNguon] = useState("");
  const keyMau = nguon === "server" ? `tiff.naming.${projectId || "_"}` : "tiff.naming.local";
  const [naming, setNaming] = useState<TiffNaming>(() => nho(keyMau, tiffPreset("v1")));
  const [opts, setOpts] = useState<Opts>(() => nho("tiff.opts", OPTS0));
  const [preview, setPreview] = useState<TiffPreview | null>(null);
  const [busy, setBusy] = useState<null | "summary" | "load" | "preview" | "run">(null);
  const [run, setRun] = useState<{ done: number; total: number; pages: number; ok: number; skip: number; bad: number } | null>(null);
  const [startedAt, setStartedAt] = useState<number | null>(null);
  const [now, setNow] = useState(Date.now());
  const [paused, setPaused] = useState(false);
  const [choO, setChoO] = useState<string | null>(null);
  const [ketQua, setKetQua] = useState<TiffRunResult | null>(null);
  const [loiChay, setLoiChay] = useState<string | null>(null);
  const [nhatKy, setNhatKy] = useState<{ at: string; text: string; err: boolean }[]>([]);

  const ghi = (text: string, err = false) => setNhatKy((l) => [...l.slice(-(LOG_MAX - 1)), { at: new Date().toLocaleTimeString("vi-VN"), text, err }]);

  useEffect(() => localStorage.setItem("tiff.nguon", nguon), [nguon]);
  useEffect(() => localStorage.setItem("tiff.projectId", projectId), [projectId]);
  useEffect(() => localStorage.setItem("tiff.folder", folder), [folder]);
  useEffect(() => luu("tiff.opts", opts), [opts]);
  // Mẫu đặt tên nhớ theo dự án (mỗi đơn vị một mẫu) — đổi dự án / nguồn thì nạp mẫu tương ứng.
  useEffect(() => setNaming(nho(keyMau, tiffPreset("v1"))), [keyMau]);
  useEffect(() => luu(keyMau, naming), [keyMau, naming]);

  useEffect(() => {
    if (!projectId && duAnTopbar.projectId) setProjectId(duAnTopbar.projectId);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [duAnTopbar.projectId]);
  useEffect(() => {
    if (isTauri()) V4ak().then((p) => setProjects(p.filter((x) => x.isActive))).catch(() => {});
  }, []);
  useEffect(() => {
    if (busy !== "run") return;
    const t = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(t);
  }, [busy]);

  // Hồ sơ có bản đã ký của dự án.
  useEffect(() => {
    setHoSo([]);
    setChon(new Set());
    setPhong("");
    if (nguon !== "server" || !projectId || !isTauri()) return;
    setBusy("summary");
    docsReviewSummary({ projectId })
      .then((rows) => setHoSo(rows.filter((r) => r.signedDocs > 0)))
      .catch((e) => notifyError(e, "lấy danh sách hồ sơ đã ký"))
      .finally(() => setBusy((b) => (b === "summary" ? null : b)));
  }, [nguon, projectId]);

  // Đổi nguồn → bỏ danh sách tệp đã nạp.
  useEffect(() => {
    setInputs([]);
    setPreview(null);
    setNhanNguon("");
  }, [nguon, projectId]);

  const dsPhong = useMemo(() => [...new Set(hoSo.map((r) => r.phong))].sort(new Intl.Collator("vi", { numeric: true }).compare), [hoSo]);
  const hoSoHien = useMemo(() => {
    const t = q.trim().toLowerCase();
    return hoSo.filter((r) => (!phong || r.phong === phong) && (!t || `${r.phong} ${r.hop} ${r.hoSo}`.toLowerCase().includes(t)));
  }, [hoSo, phong, q]);

  const napTuMayChu = async () => {
    const groups = hoSo.filter((r) => chon.has(hoSoKey(r))).map((r) => ({ projectId: r.projectId, phong: r.phong, hop: r.hop, hoSo: r.hoSo }));
    if (groups.length === 0) return toast.warning("Tích chọn hồ sơ cần tách.");
    setBusy("load");
    try {
      const docs = await docsForGroups(groups);
      const ky = docs.filter((d) => d.status === "signed" || d.status === "archived");
      setInputs(
        ky.map((d) => {
          let dirs = (d.relPath ?? "").split(/[\\/]/).filter((s) => s.trim() !== "");
          if (dirs.length && dirs[dirs.length - 1].toLowerCase() === d.filename.toLowerCase()) dirs = dirs.slice(0, -1);
          if (dirs.length === 0) dirs = [d.hoSoRef || "HO SO"];
          return { sourceDirs: dirs, fileName: d.filename, docId: d.id, pages: d.pageCount ?? undefined };
        }),
      );
      setNhanNguon(`${fmtN(ky.length)} tài liệu đã ký trong ${fmtN(groups.length)} hồ sơ` + (docs.length > ky.length ? ` (bỏ ${fmtN(docs.length - ky.length)} tài liệu chưa ký)` : ""));
    } catch (e) {
      notifyError(e, "nạp danh sách tài liệu");
    } finally {
      setBusy(null);
    }
  };

  const chonThuMucNguon = async () => {
    const d = await openDialog({ directory: true, multiple: false, title: "Chọn thư mục PDF đã ký" });
    if (typeof d === "string") setFolder(d);
  };
  const quetThuMuc = async () => {
    if (!folder.trim()) return toast.warning("Chọn thư mục PDF.");
    setBusy("load");
    try {
      const l = await tiffScanLocal(folder);
      setInputs(l.map((f) => ({ sourceDirs: f.sourceDirs, fileName: f.fileName, path: f.path })));
      setNhanNguon(`${fmtN(l.length)} tệp PDF trong ${fmtN(new Set(l.map((f) => f.sourceDirs.join("/"))).size)} thư mục`);
    } catch (e) {
      notifyError(e, "quét thư mục");
    } finally {
      setBusy(null);
    }
  };
  const chonDauRa = async () => {
    const d = await openDialog({ directory: true, multiple: false, title: "Chọn thư mục lưu TIFF" });
    if (typeof d === "string") setOpts((o) => ({ ...o, outDir: d }));
  };

  // Xem trước tự cập nhật khi đổi danh sách / mẫu / đầu ra.
  useEffect(() => {
    if (inputs.length === 0 || !isTauri()) return setPreview(null);
    let huy = false;
    const t = window.setTimeout(() => {
      setBusy((b) => b ?? "preview");
      tiffPreview(inputs, naming, opts.outDir)
        .then((p) => !huy && setPreview(p))
        .catch((e) => !huy && notifyError(e, "xem trước tên TIFF"))
        .finally(() => setBusy((b) => (b === "preview" ? null : b)));
    }, 250);
    return () => {
      huy = true;
      window.clearTimeout(t);
    };
  }, [inputs, naming, opts.outDir]);

  const batDau = async () => {
    if (inputs.length === 0) return toast.warning("Chưa có PDF nào — nạp danh sách trước.");
    if (!opts.outDir.trim()) return toast.warning("Chọn thư mục lưu TIFF.");
    if (preview && preview.collisions.length > 0) return toast.error("Còn xung đột tên TIFF — xem mục Xem trước.");
    const daCo = preview?.folders.filter((f) => f.exists).length ?? 0;
    const ok = await confirm({
      title: `Tách TIFF ${fmtN(inputs.length)} tệp PDF?`,
      body: [
        `Nguồn: ${nhanNguon}.`,
        `Mẫu đặt tên: ${naming.rule.toUpperCase()}, STT ${naming.pad} chữ số từ ${naming.start}, phân cách "${naming.separator || "(không)"}".`,
        `Đầu ra: ${opts.outDir}.`,
        daCo ? `${fmtN(daCo)} thư mục xuất đã có sẵn — ${opts.onExist === "skip" ? "PDF đã đủ TIFF sẽ được bỏ qua" : "TIFF trùng tên sẽ bị ghi đè"}.` : "",
      ]
        .filter(Boolean)
        .join("\n"),
      confirmLabel: "Bắt đầu tách",
    });
    if (!ok) return;
    setRun({ done: 0, total: inputs.length, pages: 0, ok: 0, skip: 0, bad: 0 });
    setStartedAt(Date.now());
    setNow(Date.now());
    setPaused(false);
    setKetQua(null);
    setLoiChay(null);
    setChoO(null);
    ghi(`Bắt đầu tách ${fmtN(inputs.length)} tệp PDF.`);
    setBusy("run");
    const unlisten = await onTiffProgress((p) => {
      if (p.status === "waiting_dest") {
        setChoO(p.error);
        ghi(`${p.error} — đang chờ cắm lại ổ…`, true);
        return;
      }
      setChoO(null);
      setRun((r) => (r ? { ...r, done: p.doneFiles, total: p.totalFiles, pages: p.donePages, ok: r.ok + (p.status === "ok" ? 1 : 0), skip: r.skip + (p.status === "skipped" ? 1 : 0), bad: r.bad + (p.status === "failed" ? 1 : 0) } : r));
      if (p.status === "failed") ghi(`Lỗi ${p.file}: ${p.error ?? ""}`, true);
    });
    try {
      const r = await tiffRun({ inputs, naming, outDir: opts.outDir, onExist: opts.onExist, forceRgb: opts.forceRgb, uncompressed: opts.uncompressed });
      setKetQua(r);
      ghi(`Xong: ${fmtN(r.ok)} tệp tách mới, ${fmtN(r.skipped)} đã có, ${fmtN(r.failed)} lỗi, ${fmtN(r.pages)} TIFF.` + (r.reportPath ? ` Biên bản: ${r.reportPath}` : ""), r.failed > 0);
    } catch (e) {
      setLoiChay(errorText(e));
      ghi(`Tách TIFF không hoàn tất: ${errorText(e)}`, true);
    } finally {
      unlisten();
      setBusy(null);
      setPaused(false);
      setChoO(null);
    }
  };

  const tamDung = async () => {
    const moi = !paused;
    await tiffPause(moi);
    setPaused(moi);
    ghi(moi ? "Tạm dừng — tệp đang tách dở sẽ xong rồi chờ." : "Tiếp tục.");
  };

  const chayLaiLoi = () => {
    if (!ketQua) return;
    const loi = new Set(ketQua.failures.map((f) => f.source));
    setInputs((l) => l.filter((i) => loi.has(`${i.sourceDirs.join("/")}/${i.fileName}`)));
    setNhanNguon(`${fmtN(loi.size)} tệp lỗi của lần trước`);
    toast.info("Đã lọc còn các tệp lỗi — bấm Bắt đầu để chạy lại (TIFF đã có sẽ được bỏ qua).");
  };
  const xuatLoi = () => void V4e3((ketQua?.failures ?? []).map((f, i) => ({ STT: i + 1, "Tệp PDF": f.fileName, "Nguồn": f.source, "Lỗi": f.error })), "Lỗi tách TIFF", `loi-tach-tiff-${new Date().toISOString().slice(0, 10)}.xlsx`);

  const running = busy === "run";
  const khoa = busy !== null && busy !== "preview";
  const elapsed = startedAt ? now - startedAt : 0;
  const eta = run && run.done > 0 && run.done < run.total ? (elapsed / run.done) * (run.total - run.done) : null;
  const trang = usePagination(50);
  const trangHs = usePagination(100);
  const setN = (p: Partial<TiffNaming>) => setNaming((n) => ({ ...n, ...p }));

  const hsCols: Column<ReviewSummaryRow>[] = [
    { key: "phong", header: "Phông", className: "whitespace-normal" },
    { key: "hop", header: "Hộp" },
    { key: "hoSo", header: "Hồ sơ" },
    { key: "signedDocs", header: "Đã ký", align: "right", cell: (r) => `${fmtN(r.signedDocs)}/${fmtN(r.files)}` },
    { key: "pages", header: "Trang", align: "right", cell: (r) => fmtN(r.pages) },
  ];
  const pvCols: Column<TiffPreview["folders"][number]>[] = [
    { key: "src", header: "Thư mục nguồn", className: "whitespace-normal text-fg-subtle", cell: (f) => f.sourceDirs.join(" / ") },
    { key: "out", header: "Thư mục TIFF", className: "whitespace-normal font-medium", cell: (f) => f.outputDirs.join(" / ") },
    { key: "files", header: "PDF", align: "right", cell: (f) => fmtN(f.files) },
    { key: "first", header: "TIFF đầu tiên", className: "whitespace-normal font-mono text-xs", cell: (f) => f.firstTiff },
    { key: "exists", header: "", width: 80, cell: (f) => (f.exists ? <Badge tone="warn">Đã có</Badge> : null) },
  ];

  return (
    <section className="view active">
      <PageHeader
        title="Tách TIFF"
        description="Tách PDF đã ký số thành TIFF mỗi trang một tệp (tài liệu Đảng), lấy nguyên ảnh quét, đặt tên theo mẫu của đơn vị."
        actions={
          <>
            <Button variant="primary" icon="chay" loading={running} disabled={khoa || inputs.length === 0} onClick={() => void batDau()}>
              Bắt đầu ({fmtN(inputs.length)})
            </Button>
            <Button icon={paused ? "chay" : "tamDung"} disabled={!running} onClick={() => void tamDung()}>
              {paused ? "Tiếp tục" : "Tạm dừng"}
            </Button>
            <Button variant="dangerOutline" disabled={!running} onClick={() => void tiffCancel()}>
              Dừng
            </Button>
          </>
        }
      />

      {!isTauri() && <Banner tone="info" className="mb-4">Chạy trong ứng dụng desktop để tách TIFF.</Banner>}
      {choO && (
        <Banner tone="warning" className="mb-4">
          {choO}. Cắm lại ổ — việc tách sẽ tự chạy tiếp, không mất tệp nào.
        </Banner>
      )}
      {loiChay && (
        <Banner tone="danger" className="mb-4" onClose={() => setLoiChay(null)}>
          {loiChay}
        </Banner>
      )}
      {ketQua && (
        <Banner tone={ketQua.failed > 0 || ketQua.cancelled > 0 ? "warning" : "success"} className="mb-4" onClose={() => setKetQua(null)}>
          Đã tách {fmtN(ketQua.ok)} tệp ({fmtN(ketQua.pages)} TIFF, {(ketQua.bytes / 1e9).toLocaleString("vi-VN", { maximumFractionDigits: 2 })} GB)
          {ketQua.skipped ? ` · ${fmtN(ketQua.skipped)} tệp đã có TIFF, bỏ qua` : ""}
          {ketQua.failed ? ` · ${fmtN(ketQua.failed)} tệp lỗi` : ""}
          {ketQua.cancelled ? ` · ${fmtN(ketQua.cancelled)} tệp chưa chạy (đã dừng)` : ""}.
          {ketQua.reportPath ? ` Biên bản: ${ketQua.reportPath}` : ""}
        </Banner>
      )}

      {run && (
        <Card className="mb-4" padding="sm">
          <KpiRow>
            <KpiTile label="Đã xử lý" value={`${fmtN(run.done)}/${fmtN(run.total)}`} />
            <KpiTile label="TIFF đã ghi" value={fmtN(run.pages)} tone="ok" />
            <KpiTile label="Bỏ qua (đã có)" value={fmtN(run.skip)} />
            <KpiTile label="Lỗi" value={run.bad} tone={run.bad > 0 ? "err" : undefined} />
            <KpiTile label="Đã chạy" value={hhmmss(elapsed)} />
            <KpiTile label="Ước còn" value={eta != null ? hhmmss(eta) : "--"} />
          </KpiRow>
          <div className="mt-3 h-2 rounded bg-border-subtle" role="progressbar" aria-label="Tiến độ tách TIFF" aria-valuemin={0} aria-valuemax={run.total} aria-valuenow={run.done}>
            <div className="h-2 rounded bg-run-solid transition-[width]" style={{ width: `${run.total ? (run.done / run.total) * 100 : 0}%` }} />
          </div>
          {paused && <p className="mt-1 text-sm text-fg-subtle">Đang tạm dừng.</p>}
          {nhatKy.length > 0 && (
            <Help title={`Nhật ký (${fmtN(nhatKy.length)} dòng)`} className="mt-2">
              <div className="max-h-48 overflow-auto font-mono text-xs">
                {nhatKy.map((d, i) => (
                  <div key={i} className={d.err ? "text-danger-700" : undefined}>
                    [{d.at}] {d.text}
                  </div>
                ))}
              </div>
            </Help>
          )}
        </Card>
      )}

      <div className="grid gap-4 xl:grid-cols-2">
        <Card title="Nguồn PDF" description={nhanNguon ? `Đã nạp ${nhanNguon}.` : "Bản đã ký trên máy chủ, hoặc thư mục PDF trên máy."}>
          <div className="space-y-3">
            <Seg<Nguon> value={nguon} onChange={setNguon} options={[{ value: "server", label: "Bản đã ký trên máy chủ" }, { value: "local", label: "Thư mục trên máy" }]} />
            {nguon === "server" ? (
              <>
                <FormGrid cols={2}>
                  <Field label="Dự án">
                    <Select value={projectId} onChange={setProjectId} disabled={khoa} placeholder="Chọn dự án" options={projects.map((p) => ({ value: p.id, label: `${p.code} · ${p.name}` }))} />
                  </Field>
                  <Field label="Phông">
                    <Select value={phong} onChange={setPhong} disabled={khoa || dsPhong.length === 0} placeholder={`Tất cả phông (${dsPhong.length})`} options={dsPhong.map((p) => ({ value: p, label: p }))} />
                  </Field>
                </FormGrid>
                <div className="flex flex-wrap items-center gap-2">
                  <Input size="sm" prefix="tim" className="w-64" placeholder="Tìm phông, hộp, hồ sơ" value={q} onChange={(e) => setQ(e.target.value)} aria-label="Tìm hồ sơ" />
                  <Button variant="primary" loading={busy === "load"} disabled={khoa || chon.size === 0} onClick={() => void napTuMayChu()}>
                    Nạp tài liệu ({fmtN(chon.size)} hồ sơ)
                  </Button>
                </div>
                {!projectId ? (
                  <EmptyState compact icon="kySo" title="Chọn dự án để xem hồ sơ đã ký" />
                ) : (
                  <Table
                    columns={hsCols}
                    rows={pageSlice(hoSoHien, trangHs)}
                    getRowId={hoSoKey}
                    selection={{ selected: chon, onChange: setChon }}
                    dense
                    maxHeight={320}
                    pagination={{ page: trangHs.page, pageSize: trangHs.pageSize, total: hoSoHien.length, onChange: trangHs.setPage, onPageSizeChange: trangHs.setPageSize }}
                    state={{ loading: busy === "summary", empty: { title: "Chưa có hồ sơ nào có bản đã ký", icon: "kySo" } }}
                  />
                )}
                {hoSoHien.length > 0 && (
                  <Button size="sm" variant="ghost" disabled={khoa} onClick={() => setChon(new Set(hoSoHien.map(hoSoKey)))}>
                    Chọn cả {fmtN(hoSoHien.length)} hồ sơ đang lọc
                  </Button>
                )}
              </>
            ) : (
              <Field label="Thư mục PDF" hint="Quét cả thư mục con; tên thư mục được chọn là gốc của cây xuất">
                <div className="flex gap-2">
                  <Input value={folder} onChange={(e) => setFolder(e.target.value)} spellCheck={false} />
                  <Button onClick={() => void chonThuMucNguon()} disabled={khoa}>
                    Chọn
                  </Button>
                  <Button variant="primary" loading={busy === "load"} disabled={khoa} onClick={() => void quetThuMuc()}>
                    Quét
                  </Button>
                </div>
              </Field>
            )}
          </div>
        </Card>

        <Card title="Mẫu đặt tên và đầu ra" description={nguon === "server" ? "Mẫu được nhớ riêng cho từng dự án." : "Mẫu được nhớ cho chế độ thư mục trên máy."}>
          <div className="space-y-3">
            <Field label="Mẫu của đơn vị" hint={MO_TA_MAU[naming.rule]}>
              <Seg<TiffRule> value={naming.rule} onChange={(rule) => setN({ rule })} options={[{ value: "v1", label: "v1 · Bỏ STT, giữ năm" }, { value: "v2", label: "v2 · Văn bản Đảng giữ thư mục gốc" }]} />
            </Field>
            <FormGrid cols={3}>
              <Field label="Phân cách">
                <Select value={naming.separator === "" ? "none" : naming.separator} onChange={(v) => setN({ separator: v === "none" ? "" : v })} options={[{ value: "-", label: "Gạch ngang ( - )" }, { value: "_", label: "Gạch dưới ( _ )" }, { value: " ", label: "Khoảng trắng" }, { value: "none", label: "Không phân cách" }]} />
              </Field>
              <Field label="Độ dài STT">
                <Select value={String(naming.pad)} onChange={(v) => setN({ pad: Number(v) })} options={[3, 4, 5, 6].map((n) => ({ value: String(n), label: `${n} chữ số (${"0".repeat(n - 1)}1)` }))} />
              </Field>
              <Field label="Bắt đầu từ">
                <Input type="number" min={1} value={naming.start} onChange={(e) => setN({ start: Math.max(1, Number(e.target.value) || 1) })} />
              </Field>
            </FormGrid>
            <Checkbox checked={naming.mirror} onChange={(mirror) => setN({ mirror })} text="Giữ nguyên thư mục cha, chỉ đổi tên thư mục chứa PDF" />
            <Field label="Mã loại văn bản Đảng" hint="Ngăn cách bằng dấu phẩy; dùng để nhận dạng tên văn bản Đảng">
              <Input value={naming.docTypeCodes.join(", ")} onChange={(e) => setN({ docTypeCodes: e.target.value.split(",").map((s) => s.trim()).filter(Boolean) })} />
            </Field>
            <Field label="Thư mục lưu TIFF">
              <div className="flex gap-2">
                <Input value={opts.outDir} onChange={(e) => setOpts((o) => ({ ...o, outDir: e.target.value }))} spellCheck={false} />
                <Button onClick={() => void chonDauRa()} disabled={running}>
                  Chọn
                </Button>
              </div>
            </Field>
            <FormGrid cols={2}>
              <Field label="Khi TIFF đã có">
                <Select value={opts.onExist} onChange={(v) => setOpts((o) => ({ ...o, onExist: v as Opts["onExist"] }))} options={[{ value: "skip", label: "Bỏ qua PDF đã đủ TIFF" }, { value: "overwrite", label: "Ghi đè" }]} />
              </Field>
              <Field label="Ảnh">
                <div className="space-y-1 pt-1">
                  <Checkbox checked={opts.forceRgb} onChange={(forceRgb) => setOpts((o) => ({ ...o, forceRgb }))} text="Ép RGB 24 bit (bản quét xám cũng ra màu)" />
                  <Checkbox checked={opts.uncompressed} onChange={(uncompressed) => setOpts((o) => ({ ...o, uncompressed }))} text="Không nén (mặc định nén LZW, không mất dữ liệu)" />
                </div>
              </Field>
            </FormGrid>
          </div>
        </Card>
      </div>

      {ketQua && ketQua.failures.length > 0 && (
        <Card
          className="mt-4"
          tone="danger"
          padding="none"
          title={`Tệp lỗi (${fmtN(ketQua.failures.length)})`}
          actions={
            <>
              <Button variant="primary" icon="lamLai" disabled={khoa} onClick={chayLaiLoi}>
                Chạy lại tệp lỗi
              </Button>
              <Button icon="tepExcel" onClick={xuatLoi}>
                Xuất Excel
              </Button>
            </>
          }
        >
          <Table
            columns={[
              { key: "fileName", header: "Tệp", className: "whitespace-normal" },
              { key: "source", header: "Nguồn", className: "whitespace-normal text-fg-subtle" },
              { key: "error", header: "Lỗi", className: "whitespace-normal text-danger-700" },
            ]}
            rows={ketQua.failures}
            getRowId={(f) => f.source}
            dense
            maxHeight={320}
          />
        </Card>
      )}

      <Card className="mt-4" padding="none" title={preview ? `Xem trước: ${fmtN(preview.folderCount)} thư mục TIFF, ${fmtN(preview.fileCount)} PDF` : "Xem trước"}>
        {preview && preview.collisions.length > 0 && (
          <Banner tone="danger" className="m-3">
            {fmtN(preview.collisions.length)} xung đột tên TIFF (hai thư mục nguồn ra cùng chỗ): {preview.collisions.slice(0, 3).join("; ")}
            {preview.collisions.length > 3 ? "…" : ""} Bật "Giữ nguyên thư mục cha" hoặc đổi mẫu.
          </Banner>
        )}
        {!preview ? (
          <EmptyState compact icon="thuMuc" title="Nạp danh sách PDF để xem cây thư mục và tên TIFF sẽ tạo" />
        ) : (
          <Table
            columns={pvCols}
            rows={pageSlice(preview.folders, trang)}
            getRowId={(f) => f.sourceDirs.join("/")}
            dense
            maxHeight={420}
            pagination={{ page: trang.page, pageSize: trang.pageSize, total: preview.folders.length, onChange: trang.setPage, onPageSizeChange: trang.setPageSize }}
          />
        )}
        {preview && preview.folderCount > preview.folders.length && <p className="px-3 py-2 text-sm text-fg-subtle">Chỉ hiện {fmtN(preview.folders.length)} thư mục đầu.</p>}
      </Card>

      <Help title="Cách tách" className="mt-4">
        <p>Mỗi trang PDF ra một tệp TIFF lấy nguyên ảnh quét đã nhúng: giữ đúng độ phân giải, kích thước và màu của bản quét, không co giãn, không in ô chữ ký số lên trang.</p>
        <p>STT chạy liên tục qua các PDF (xếp theo tên) trong cùng một thư mục, sang thư mục khác đánh lại từ số bắt đầu. Chạy lại cùng cấu hình luôn ra đúng cùng tên tệp.</p>
        <p>Xong mỗi đợt, thư mục xuất có biên bản BIEN-BAN-TACH-TIFF-*.csv (mở bằng Excel) và manifest-sha256-*.txt để đối chiếu khi bàn giao.</p>
      </Help>
    </section>
  );
}
