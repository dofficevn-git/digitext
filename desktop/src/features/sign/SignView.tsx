// Copyright (c) 2026 DigiText. All rights reserved.
// Ký số hàng loạt theo dự án (PAdES, USB token): Nguồn & đầu ra · Token & chứng thư · Hình thức ký ·
// Bắt đầu / Tạm dừng / Huỷ, tiến độ + ước còn, nhật ký gập, tệp lỗi kèm ký lại. Dự án mặc định theo Topbar.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { V41p } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { useProject } from "@/state/project";
import { V4qu, V4qt, V4qv, onSignBatchProgress, signBatch, signBatchCancel, signBatchPause, signCertInfo, signLoadStamp, signRetry, type SignBatchProgress, type SignBatchResult, type SignCertInfo, type SignPlacement, type V4qr, type V4qs } from "@/api/sign";
import { V4el } from "@/api/docs";
import { V4ak, type V4a9 } from "@/api/users";
import { V4e3 } from "@/features/dossier/excel";
import { errorText } from "@/lib/errors";
import SignPlacementPicker from "./SignPlacementPicker";
import { Badge, Banner, Button, Card, Checkbox, confirm, Dialog, EmptyState, Field, FormGrid, Help, Input, KpiRow, KpiTile, notifyError, PageHeader, Seg, Select, Table, toast, usePagination, pageSlice, type Column } from "@/ui";

const LOG_MAX = 500;
const fmtN = (n: number) => n.toLocaleString("vi-VN");

function nho<T>(key: string, fallback: T): T {
  try {
    const v = localStorage.getItem(key);
    return v == null ? fallback : (JSON.parse(v) as T);
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

const phongOf = (j: V4qr): string => {
  const parts = (j.relPath ?? "").split("/").filter((s) => s.trim() !== "");
  return parts.length >= 2 ? parts[0].trim() : "";
};
const duongDan = (j: V4qr): string => j.relPath || j.hoSoRef || "";

function hhmmss(ms: number): string {
  const s = Math.max(0, Math.round(ms / 1000));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;
}

interface LoiKy {
  id: string;
  filename: string;
  path: string;
  error: string;
}
interface DongNhatKy {
  at: string;
  text: string;
  kind: "ok" | "err" | "info";
}

export default function V4qy() {
  const duAnTopbar = useProject();
  const [hangDoi, setHangDoi] = useState<V4qr[]>([]);
  const [modulePath, setModulePath] = useState("");
  const [modules, setModules] = useState<string[]>([]);
  const [tokens, setTokens] = useState<V4qs[]>([]);
  const [slot, setSlot] = useState<number | null>(null);
  const [pin, setPin] = useState("");
  const [busy, setBusy] = useState<null | "jobs" | "tokens" | "cert" | "batch" | "retry">(null);
  const [ketQua, setKetQua] = useState<{ kind: "ok" | "err"; text: string } | null>(null);
  const [cert, setCert] = useState<SignCertInfo | null>(null);

  const [projects, setProjects] = useState<V4a9[]>([]);
  const [projectId, setProjectId] = useState<string>(() => nho("sign.projectId", ""));
  const [phongLoc, setPhongLoc] = useState("");
  const [q, setQ] = useState("");
  const [outDir, setOutDir] = useState<string>(() => nho("sign.outDir", ""));
  const [withStamp, setWithStamp] = useState<boolean>(() => nho("sign.withStamp", true));
  const [stampPath, setStampPath] = useState<string>(() => nho("sign.stampPath", ""));
  const [stampB64, setStampB64] = useState<string | null>(null);
  const [timestamp, setTimestamp] = useState<boolean>(() => nho("sign.timestamp", false));

  const [placement, setPlacement] = useState<SignPlacement | null>(null);
  const [moViTri, setMoViTri] = useState(false);
  const [batch, setBatch] = useState<{ done: number; total: number; signed: number; bad: number } | null>(null);
  const [startedAt, setStartedAt] = useState<number | null>(null);
  const [now, setNow] = useState(Date.now());
  const [paused, setPaused] = useState(false);
  const [loi, setLoi] = useState<LoiKy[]>([]);
  const [nhatKy, setNhatKy] = useState<DongNhatKy[]>([]);
  const taiVe = useRef<Promise<void>>(Promise.resolve());

  const ghi = (text: string, kind: DongNhatKy["kind"] = "info") => setNhatKy((l) => [...l.slice(-(LOG_MAX - 1)), { at: new Date().toLocaleTimeString("vi-VN"), text, kind }]);

  useEffect(() => luu("sign.outDir", outDir), [outDir]);
  useEffect(() => luu("sign.withStamp", withStamp), [withStamp]);
  useEffect(() => luu("sign.stampPath", stampPath), [stampPath]);
  useEffect(() => luu("sign.timestamp", timestamp), [timestamp]);
  useEffect(() => luu("sign.projectId", projectId), [projectId]);

  // Mặc định theo ô "Dự án đang làm" trên Topbar khi chưa chọn gì.
  useEffect(() => {
    if (!projectId && duAnTopbar.projectId) setProjectId(duAnTopbar.projectId);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [duAnTopbar.projectId]);

  useEffect(() => {
    if (!isTauri() || !stampPath) {
      setStampB64(null);
      return;
    }
    signLoadStamp(stampPath)
      .then(setStampB64)
      .catch(() => setStampB64(null));
  }, [stampPath]);

  const napHangDoi = useCallback(async () => {
    if (!isTauri() || !projectId) {
      setHangDoi([]);
      return;
    }
    setBusy((b) => b ?? "jobs");
    try {
      setHangDoi(await V4qt(projectId));
    } catch (e) {
      notifyError(e, "lấy hàng đợi ký");
    } finally {
      setBusy((b) => (b === "jobs" ? null : b));
    }
  }, [projectId]);

  const napModules = async () => {
    if (!isTauri()) return;
    try {
      const m = await V4qu();
      setModules(m);
      setModulePath((cu) => (cu.trim() === "" && m.length > 0 ? m[0] : cu));
    } catch {
      /* không dò được thì nhập tay */
    }
  };

  V41p(napHangDoi, { topics: ["sign", "docs"] });

  useEffect(() => {
    void napModules();
    if (isTauri()) {
      V4ak()
        .then((p) => setProjects(p.filter((x) => x.isActive)))
        .catch(() => {});
    }
  }, []);

  useEffect(() => {
    setPlacement(null);
    setPhongLoc("");
    setLoi([]);
    setBatch(null);
  }, [projectId]);

  useEffect(() => {
    if (busy !== "batch") return;
    const t = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(t);
  }, [busy]);

  const dsPhong = useMemo(() => {
    const s = new Set<string>();
    for (const j of hangDoi) {
      const p = phongOf(j);
      if (p) s.add(p);
    }
    return [...s].sort(new Intl.Collator("vi", { numeric: true }).compare);
  }, [hangDoi]);
  const jobs = useMemo(() => (phongLoc ? hangDoi.filter((j) => phongOf(j) === phongLoc) : hangDoi), [hangDoi, phongLoc]);
  const jobsHien = useMemo(() => {
    const t = q.trim().toLowerCase();
    return t ? jobs.filter((j) => j.filename.toLowerCase().includes(t) || duongDan(j).toLowerCase().includes(t)) : jobs;
  }, [jobs, q]);

  // Máy có thể cài nhiều phần mềm token (iCA, Viettel, Bit4id TokenME…): thử lần lượt từng thư viện PKCS#11 dò được,
  // dùng thư viện đầu tiên thấy token (13/10/2026: TokenME EVO v2 không dò được vì chỉ thử thư viện đầu tiên).
  const doToken = async () => {
    setBusy("tokens");
    try {
      let m: string[] = [];
      try {
        m = await V4qu();
        setModules(m);
      } catch {
        /* không dò được thì dùng đường dẫn nhập tay */
      }
      const thu = [...new Set([modulePath.trim(), ...m].filter(Boolean))];
      if (thu.length === 0) thu.push("");
      let found: V4qs[] = [];
      let dung = modulePath;
      const loi: string[] = [];
      for (const p of thu) {
        try {
          const t = await V4qv(p);
          if (t.length > 0) {
            found = t;
            dung = p;
            break;
          }
        } catch (e) {
          loi.push(`${p || "(tự dò)"}: ${errorText(e)}`);
        }
      }
      setTokens(found);
      setSlot(found.length > 0 ? found[0].index : null);
      setCert(null);
      if (found.length > 0) {
        setModulePath(dung);
        toast.success(`Thấy ${found.length} token.`, { description: dung ? `Thư viện: ${dung}` : undefined });
      } else if (loi.length === thu.length && thu[0] === "") {
        throw new Error(loi[0].replace(/^\(tự dò\): /, ""));
      } else {
        toast.error("Không thấy token nào đang cắm.", {
          detail: `Đã thử ${thu.length} thư viện PKCS#11: ${thu.join("; ")}. Kiểm tra USB token đã cắm và phần mềm token của nhà cung cấp chứng thư (iCA, Viettel-CA, Bit4id…) đã cài.`,
        });
      }
    } catch (e) {
      notifyError(e, "dò token");
    } finally {
      setBusy(null);
    }
  };

  const sanSang = (): boolean => {
    if (slot === null) {
      toast.warning("Chọn token trước khi ký, bấm Dò token.");
      return false;
    }
    if (!pin) {
      toast.warning("Nhập mã PIN của token.");
      return false;
    }
    if (withStamp && stampPath && !stampB64) {
      toast.warning("Ảnh con dấu chưa đọc được. Chọn lại ảnh hoặc chọn ký không dấu.");
      return false;
    }
    return true;
  };

  const xemChungThu = async () => {
    if (!sanSang()) return;
    setBusy("cert");
    try {
      const c = await signCertInfo({ modulePath, slotIndex: slot!, pin });
      setCert(c);
      toast.success(`PIN đúng. Chứng thư của ${c.subject}.`);
    } catch (e) {
      setCert(null);
      notifyError(e, "đọc chứng thư");
    } finally {
      setBusy(null);
    }
  };

  const chonDauRa = async () => {
    const d = await openDialog({ directory: true, multiple: false, title: "Chọn thư mục lưu bản đã ký" });
    if (typeof d === "string") setOutDir(d);
  };
  const chonConDau = async () => {
    const f = await openDialog({ multiple: false, title: "Chọn ảnh con dấu (PNG nền trong)", filters: [{ name: "Ảnh", extensions: ["png", "jpg", "jpeg"] }] });
    if (typeof f !== "string") return;
    try {
      setStampB64(await signLoadStamp(f));
      setStampPath(f);
    } catch (e) {
      notifyError(e, "đọc ảnh con dấu");
    }
  };

  const runBatch = async (list: V4qr[], nhan: string) => {
    if (!sanSang() || list.length === 0) return;
    const project = projects.find((p) => p.id === projectId);
    const where = placement ? `${placement.lastPage ? "trang cuối" : "trang đầu"}, cách lề trái ${Math.round(placement.x * 100)}%, cách mép trên ${Math.round(placement.y * 100)}%` : "góc trên-phải trang đầu";
    const kieu = withStamp ? (stampB64 ? "có dấu, ảnh con dấu đã chọn" : "có dấu") : "không dấu";
    const ok = await confirm({
      title: `${nhan} ${fmtN(list.length)} tài liệu?`,
      body: [`Dự án: ${project?.name ?? projectId}${phongLoc ? ` · phông ${phongLoc}` : ""}.`, `Vị trí chữ ký: ${where}.`, `Hình thức: ${kieu}; dấu thời gian: ${timestamp ? "có" : "không"}.`, `Đầu ra: ${outDir || "chỉ lưu trên máy chủ"}.`, "Chữ ký số không gỡ được sau khi ký."].join("\n"),
      confirmLabel: nhan,
    });
    if (!ok) return;
    const byId = new Map(list.map((j) => [j.id, j]));
    const jobIds = list.map((j) => j.id);
    setBatch({ done: 0, total: jobIds.length, signed: 0, bad: 0 });
    setStartedAt(Date.now());
    setNow(Date.now());
    setPaused(false);
    setLoi([]);
    setKetQua(null);
    ghi(`Bắt đầu: ${nhan.toLowerCase()} ${jobIds.length} tài liệu.`);
    setBusy("batch");
    const unlisten = await onSignBatchProgress((p: SignBatchProgress) => {
      const j = byId.get(p.jobId);
      const ten = j?.filename ?? p.jobId;
      setBatch((b) => (b ? { done: p.done, total: p.total, signed: b.signed + (p.status === "signed" ? 1 : 0), bad: b.bad + (p.status === "signed" ? 0 : 1) } : b));
      if (p.status === "signed") {
        setHangDoi((l) => l.filter((x) => x.id !== p.jobId));
        ghi(`Đã ký ${ten}`, "ok");
        if (outDir && j) {
          taiVe.current = taiVe.current.then(() =>
            V4el(j.id, j.filename, j.relPath ?? undefined, false, outDir, "signed")
              .then(() => undefined)
              .catch((e) => ghi(`Không tải được bản đã ký ${ten} về đầu ra: ${errorText(e)}`, "err")),
          );
        }
      } else {
        const error = p.error ?? p.status;
        setLoi((e) => [...e, { id: p.jobId, filename: ten, path: j ? duongDan(j) : "", error }]);
        ghi(`Lỗi ${ten}${j ? ` (${duongDan(j)})` : ""}: ${error}`, "err");
      }
    });
    try {
      const r: SignBatchResult = await signBatch({ jobIds, modulePath, slotIndex: slot!, pin, placement, style: { withStamp, stampPng: withStamp ? stampB64 : null, timestamp } });
      const text = `Đã ký ${fmtN(r.signed)}/${fmtN(r.total)} tài liệu` + (r.rejected ? ` · máy chủ từ chối ${fmtN(r.rejected)}` : "") + (r.failed ? ` · lỗi ${fmtN(r.failed)}` : "") + (r.cancelled ? ` · chưa ký ${fmtN(r.cancelled)}` : "") + (r.stoppedReason ? `. Dừng vì: ${r.stoppedReason}` : "") + ".";
      const xau = r.failed + r.rejected > 0 || !!r.stoppedReason;
      setKetQua({ kind: xau ? "err" : "ok", text });
      ghi(text, xau ? "err" : "ok");
    } catch (e) {
      setKetQua({ kind: "err", text: `Ký hàng loạt không hoàn tất: ${errorText(e)}` });
      ghi(`Ký hàng loạt lỗi: ${errorText(e)}`, "err");
    } finally {
      unlisten();
      setBusy(null);
      setPaused(false);
      await taiVe.current;
      if (outDir) ghi(`Bản đã ký đã lưu vào ${outDir}.`);
      void napHangDoi();
    }
  };

  const tamDung = async () => {
    const moi = !paused;
    await signBatchPause(moi);
    setPaused(moi);
    ghi(moi ? "Tạm dừng, ký xong tệp đang dở thì chờ." : "Tiếp tục ký.");
  };

  const kyLaiLoi = async () => {
    if (loi.length === 0) return;
    setBusy("retry");
    try {
      const ids = loi.map((e) => e.id);
      const r = await signRetry(ids);
      ghi(`Đưa ${r.requeued} tài liệu lỗi về hàng chờ ký${r.skipped ? ` (bỏ qua ${r.skipped})` : ""}.`);
      const list = await V4qt(projectId);
      setHangDoi(list);
      const set = new Set(ids);
      setBusy(null);
      await runBatch(
        list.filter((j) => set.has(j.id)),
        "Ký lại",
      );
    } catch (e) {
      notifyError(e, "ký lại");
      setBusy(null);
    }
  };

  const xuatLoi = () => void V4e3(loi.map((e, i) => ({ STT: i + 1, "Tên file": e.filename, "Đường dẫn": e.path, "Lý do": e.error })), "Lỗi ký số", `loi-ky-so-${new Date().toISOString().slice(0, 10)}.xlsx`);

  const trang = usePagination(100);
  const first = jobs[0];
  const running = busy === "batch";
  const elapsed = startedAt ? now - startedAt : 0;
  const eta = batch && batch.done > 0 && batch.done < batch.total ? (elapsed / batch.done) * (batch.total - batch.done) : null;
  const khoaForm = busy !== null;

  const loiCols: Column<LoiKy>[] = [
    { key: "filename", header: "Tệp", className: "whitespace-normal" },
    { key: "path", header: "Đường dẫn", className: "whitespace-normal text-fg-subtle", cell: (e) => e.path || "—" },
    { key: "error", header: "Lý do", className: "whitespace-normal text-danger-700" },
  ];
  const choCols: Column<V4qr>[] = [
    { key: "filename", header: "Tài liệu", className: "whitespace-normal", sortable: true },
    { key: "path", header: "Đường dẫn", className: "whitespace-normal text-fg-subtle", cell: (j) => duongDan(j) || "—", sortable: true, sortValue: duongDan },
    { key: "pageCount", header: "Trang", align: "right", width: 80, cell: (j) => j.pageCount ?? "—", sortable: true },
  ];

  return (
    <section className="view active">
      <PageHeader
        title="Ký số"
        description="Ký PAdES hàng loạt bằng USB token cho tài liệu đã có PDF/A của một dự án."
        actions={
          <>
            <Button variant="primary" icon="kySo" loading={running} disabled={khoaForm || jobs.length === 0} onClick={() => void runBatch(jobs, "Ký số")}>
              Bắt đầu ({fmtN(jobs.length)})
            </Button>
            <Button icon={paused ? "chay" : "tamDung"} disabled={!running} onClick={() => void tamDung()}>
              {paused ? "Tiếp tục" : "Tạm dừng"}
            </Button>
            <Button variant="dangerOutline" disabled={!running} onClick={() => void signBatchCancel()}>
              Huỷ
            </Button>
          </>
        }
      />

      {!isTauri() && <Banner tone="info" className="mb-4">Chạy trong ứng dụng desktop để ký. Chế độ trình duyệt không có token.</Banner>}
      {ketQua && (
        <Banner tone={ketQua.kind === "ok" ? "success" : "danger"} className="mb-4" onClose={() => setKetQua(null)}>
          {ketQua.text}
        </Banner>
      )}

      {batch && (
        <Card className="mb-4" padding="sm">
          <KpiRow>
            <KpiTile label="Đã xử lý" value={`${fmtN(batch.done)}/${fmtN(batch.total)}`} />
            <KpiTile label="Đã ký" value={batch.signed} tone="ok" />
            <KpiTile label="Lỗi" value={batch.bad} tone={batch.bad > 0 ? "err" : undefined} />
            <KpiTile label="Đã chạy" value={hhmmss(elapsed)} />
            <KpiTile label="Ước còn" value={eta != null ? hhmmss(eta) : "--"} />
          </KpiRow>
          <div className="mt-3 h-2 rounded bg-border-subtle" role="progressbar" aria-label="Tiến độ ký" aria-valuemin={0} aria-valuemax={batch.total} aria-valuenow={batch.done}>
            <div className="h-2 rounded bg-accent transition-[width]" style={{ width: `${batch.total ? (batch.done / batch.total) * 100 : 0}%` }} />
          </div>
          <p className="mt-1 text-sm text-fg-subtle">
            {running && batch.done === 0 ? "Đang mở token và kiểm PIN…" : ""}
            {paused ? "Đang tạm dừng." : ""}
          </p>
          {nhatKy.length > 0 && (
            <Help title={`Nhật ký (${fmtN(nhatKy.length)} dòng)`} className="mt-2">
              <div className="max-h-48 overflow-auto font-mono text-xs">
                {nhatKy.map((d, i) => (
                  <div key={i} className={d.kind === "err" ? "text-danger-700" : d.kind === "ok" ? "text-success-700" : undefined}>
                    [{d.at}] {d.text}
                  </div>
                ))}
              </div>
            </Help>
          )}
        </Card>
      )}

      <div className="grid gap-4 xl:grid-cols-3">
        <Card title="Nguồn và đầu ra" description="Dự án lấy theo ô Dự án đang làm, đổi được tại đây.">
          <div className="space-y-3">
            <Field label="Dự án">
              <Select value={projectId} onChange={setProjectId} disabled={khoaForm} placeholder="Chọn dự án" options={projects.map((p) => ({ value: p.id, label: `${p.code} · ${p.name}` }))} />
            </Field>
            {dsPhong.length > 0 && (
              <Field label="Phông">
                <Select value={phongLoc} onChange={setPhongLoc} disabled={khoaForm} placeholder={`Tất cả phông (${dsPhong.length})`} options={dsPhong.map((p) => ({ value: p, label: p }))} />
              </Field>
            )}
            <Field label="Thư mục lưu bản đã ký" hint="Để trống thì chỉ lưu trên máy chủ">
              <div className="flex gap-2">
                <Input value={outDir} onChange={(e) => setOutDir(e.target.value)} spellCheck={false} />
                <Button onClick={() => void chonDauRa()} disabled={running}>
                  Chọn
                </Button>
              </div>
            </Field>
          </div>
        </Card>

        <Card title="Token và chứng thư">
          <div className="space-y-3">
            <Field label="Thư viện PKCS#11" hint="Để trống thì tự dò phần mềm token đã cài">
              <div className="flex gap-2">
                <Input value={modulePath} onChange={(e) => setModulePath(e.target.value)} spellCheck={false} list="pkcs11-modules" />
                <datalist id="pkcs11-modules">
                  {modules.map((m) => (
                    <option key={m} value={m} />
                  ))}
                </datalist>
                <Button icon="token" loading={busy === "tokens"} disabled={khoaForm} onClick={() => void doToken()}>
                  Dò token
                </Button>
              </div>
            </Field>
            <FormGrid cols={2}>
              <Field label="Token">
                <Select value={slot == null ? "" : String(slot)} onChange={(v) => setSlot(v === "" ? null : Number(v))} disabled={tokens.length === 0} placeholder={tokens.length === 0 ? "Chưa dò" : undefined} options={tokens.map((t) => ({ value: String(t.index), label: t.label || `Token #${t.index}` }))} />
              </Field>
              <Field label="Mã PIN">
                <Input
                  type="password"
                  value={pin}
                  onChange={(e) => {
                    setPin(e.target.value);
                    setCert(null);
                  }}
                  autoComplete="off"
                />
              </Field>
            </FormGrid>
            <div className="flex flex-wrap items-center gap-2">
              <Button loading={busy === "cert"} disabled={khoaForm} onClick={() => void xemChungThu()}>
                Kiểm tra chứng thư
              </Button>
              {cert && <Badge tone="ok">PIN đúng</Badge>}
            </div>
            {cert && (
              <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
                <dt className="text-fg-subtle">Chủ thể</dt>
                <dd>{[cert.subject, cert.organization].filter(Boolean).join(" — ")}</dd>
                <dt className="text-fg-subtle">Cấp bởi</dt>
                <dd>{cert.issuer}</dd>
                <dt className="text-fg-subtle">Hết hạn</dt>
                <dd>{cert.notAfter}</dd>
                <dt className="text-fg-subtle">Vân tay</dt>
                <dd className="break-all font-mono text-xs">{cert.thumbprint}</dd>
              </dl>
            )}
          </div>
        </Card>

        <Card title="Hình thức ký">
          <div className="space-y-3">
            <Field label="Con dấu">
              <Seg<"co" | "khong"> value={withStamp ? "co" : "khong"} onChange={(v) => setWithStamp(v === "co")} options={[{ value: "co", label: "Ký có dấu" }, { value: "khong", label: "Ký không dấu" }]} />
            </Field>
            {withStamp && (
              <Field label="Ảnh con dấu" hint="Không chọn ảnh thì dùng dấu đỏ dựng từ chứng thư">
                <div className="flex flex-wrap items-center gap-2">
                  <Input className="min-w-40 flex-1" readOnly value={stampPath} placeholder="Chưa chọn ảnh" />
                  <Button onClick={() => void chonConDau()} disabled={running}>
                    Chọn
                  </Button>
                  {stampPath && (
                    <Button variant="ghost" onClick={() => setStampPath("")} disabled={running}>
                      Bỏ ảnh
                    </Button>
                  )}
                  {stampB64 && <img src={`data:image/png;base64,${stampB64}`} alt="Ảnh con dấu" className="h-12 w-auto border border-border-subtle" />}
                </div>
              </Field>
            )}
            <Checkbox checked={timestamp} onChange={setTimestamp} disabled={running} text="Gắn dấu thời gian (TSA)" />
            <Field label="Vị trí chữ ký">
              <div className="flex flex-wrap items-center gap-2 text-sm">
                <span>{placement ? `${placement.lastPage ? "Trang cuối" : "Trang đầu"}, trái ${Math.round(placement.x * 100)}%, trên ${Math.round(placement.y * 100)}%` : "Góc trên-phải trang đầu"}</span>
                <Button size="sm" disabled={!first || running} onClick={() => setMoViTri(true)}>
                  Chọn trên trang
                </Button>
              </div>
            </Field>
          </div>
        </Card>
      </div>

      {loi.length > 0 && (
        <Card
          className="mt-4"
          tone="danger"
          padding="none"
          title={`Tệp ký lỗi (${fmtN(loi.length)})`}
          actions={
            <>
              <Button variant="primary" icon="lamLai" loading={busy === "retry"} disabled={khoaForm} onClick={() => void kyLaiLoi()}>
                Ký lại tệp lỗi
              </Button>
              <Button icon="tepExcel" onClick={xuatLoi}>
                Xuất Excel
              </Button>
            </>
          }
        >
          <Table columns={loiCols} rows={loi} getRowId={(e) => e.id} dense maxHeight={320} />
        </Card>
      )}

      <Card className="mt-4" padding="none" title={`Chờ ký (${fmtN(jobs.length)})`} actions={<Input size="sm" prefix="tim" className="w-64" placeholder="Tìm tài liệu, đường dẫn" value={q} onChange={(e) => setQ(e.target.value)} aria-label="Tìm trong hàng chờ ký" />}>
        {!projectId ? (
          <EmptyState compact icon="kySo" title="Chọn dự án để xem tài liệu chờ ký" />
        ) : (
          <Table columns={choCols} rows={pageSlice(jobsHien, trang)} getRowId={(j) => j.id} dense maxHeight={420} pagination={{ page: trang.page, pageSize: trang.pageSize, total: jobsHien.length, onChange: trang.setPage, onPageSizeChange: trang.setPageSize }} state={{ loading: busy === "jobs" && hangDoi.length === 0, empty: { title: q ? "Không có tài liệu nào khớp từ khoá" : "Không có tài liệu nào đang chờ ký", icon: "kySo" } }} />
        )}
      </Card>

      <Help title="Điều kiện ký" className="mt-4">
        <p>Chỉ tài liệu đã có bản PDF/A mới vào hàng chờ ký. Chữ ký PAdES-LTV ghi kèm chứng thư và dấu thời gian nên xác thực được lâu dài mà không cần token.</p>
        <p>Thư viện PKCS#11 là tệp .dll do nhà cung cấp chứng thư cài cùng phần mềm token. Máy chưa cài thì không dò thấy token.</p>
      </Help>

      <Dialog open={moViTri && !!first} onOpenChange={setMoViTri} title="Vị trí chữ ký" description="Chọn trên tệp đầu tiên, áp cho cả lô." size="lg" footer={<Button variant="primary" onClick={() => setMoViTri(false)}>Xong</Button>}>
        {first && <SignPlacementPicker jobId={first.id} filename={first.filename} value={placement} onChange={setPlacement} />}
      </Dialog>
    </section>
  );
}
