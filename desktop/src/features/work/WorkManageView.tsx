// Copyright (c) 2026 DigiText. All rights reserved.
// Giao việc xác nhận trường: tạo gói (theo hồ sơ liền mạch / ngẫu nhiên, chia thành nhiều gói), giao cộng tác viên,
// theo dõi tiến độ, duyệt / từ chối từng mục hoặc cả gói, xuất / in metadata.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V4ek, type V4ej } from "@/api/docs";
import { V4ac, V4ak, type V4a8, type V4a9 } from "@/api/users";
import {
  ITEM_STATUS_LABEL,
  KIND_LABEL,
  MODE_LABEL,
  PKG_STATUS_LABEL,
  workClosePackage,
  workCreatePackage,
  workGetPackage,
  workListPackages,
  workPatchPackage,
  workReview,
  workSetMembers,
  type CreatePackageResp,
  type ItemStatus,
  type WorkItem,
  type WorkKind,
  type WorkMode,
  type WorkPackage,
} from "@/api/work";
import V4i7 from "@/features/upload/OcrFieldsView";
import { inMetadata, layTruongNhieuTaiLieu, tenTepAnToan, xuatMetadataExcel, type MetaRow } from "@/features/dossier/metadataXuat";
import { useProject } from "@/state/project";
import { V462 as cn } from "@/lib/utils";
import { Badge, Button, Card, Checkbox, confirm, Dialog, ErrorBanner, Field, FormGrid, IconButton, Input, Menu, notifyError, PageHeader, prompt, Seg, Select, Stepper, Table, Tip, toast, Toolbar, type Column } from "@/ui";

const n = (v: number) => v.toLocaleString("vi-VN");
const ngay = (s: string | null) => (s ? s.split("-").reverse().join("/") : "—");
const quaHan = (p: WorkPackage) => !!p.dueAt && p.status !== "done" && p.status !== "closed" && new Date(`${p.dueAt}T23:59:59`).getTime() < Date.now();
const pkgTone = (s: WorkPackage["status"]) => (s === "done" ? "ok" : s === "under_review" ? "warn" : "gray") as "ok" | "warn" | "gray";
const itemTone = (s: ItemStatus) => (s === "approved" ? "ok" : s === "rejected" || s === "checked_err" ? "err" : s === "pending" ? "gray" : "info") as "ok" | "err" | "gray" | "info";

function TienDo({ p }: { p: WorkPackage }) {
  const t = p.nTotal || 1;
  const seg = [
    ["bg-success-600", p.nApproved, "đã duyệt"],
    ["bg-info-600", p.nDone + p.nChecked, "đã xác nhận / kiểm tra"],
    ["bg-danger-600", p.nRejected, "bị trả"],
  ] as const;
  return (
    <Tip text={seg.map(([, v, l]) => `${l}: ${n(v)}`).join(" · ")}>
      <div className="flex h-2 min-w-28 overflow-hidden rounded bg-border-subtle" role="img" aria-label={seg.map(([, v, l]) => `${l} ${n(v)}`).join(", ")}>
        {seg.map(([c, v]) => (
          <div key={c} className={c} style={{ width: `${(v / t) * 100}%` }} />
        ))}
      </div>
    </Tip>
  );
}

/** Phạm vi phông / hộp / hồ sơ suy từ cây tài liệu máy chủ. */
function useScope(docs: V4ej[], projectId: string) {
  return useMemo(() => {
    const phong = new Set<string>();
    const hop = new Map<string, Set<string>>();
    const hoso = new Map<string, Set<string>>();
    for (const d of docs) {
      if (d.projectId !== projectId) continue;
      const p = (d.relPath ?? "").split("/").filter((x) => x.trim() !== "");
      if (p.length < 1) continue;
      phong.add(p[0]);
      if (p.length >= 2) {
        if (!hop.has(p[0])) hop.set(p[0], new Set());
        hop.get(p[0])!.add(p[1]);
        if (p.length >= 3) {
          const k = `${p[0]}/${p[1]}`;
          if (!hoso.has(k)) hoso.set(k, new Set());
          hoso.get(k)!.add(p[2]);
        }
      }
    }
    const col = new Intl.Collator("vi", { numeric: true }).compare;
    return {
      phong: [...phong].sort(col),
      hop: (ph: string) => [...(hop.get(ph) ?? [])].sort(col),
      hoso: (ph: string, h: string) => [...(hoso.get(`${ph}/${h}`) ?? [])].sort(col),
    };
  }, [docs, projectId]);
}

// ---------------- Tạo gói (hộp thoại 3 bước)
function TaoGoi({ projectId, packages, docs, onCreated, onClose }: { projectId: string; packages: WorkPackage[]; docs: V4ej[]; onCreated: () => void; onClose: () => void }) {
  const scope = useScope(docs, projectId);
  const [step, setStep] = useState(0);
  const [name, setName] = useState("");
  const [kind, setKind] = useState<WorkKind>("xac_nhan");
  const [mode, setMode] = useState<WorkMode>("hoso");
  const [phong, setPhong] = useState("");
  const [hop, setHop] = useState("");
  const [hoso, setHoso] = useState("");
  const [size, setSize] = useState(50);
  const [count, setCount] = useState(1);
  const [source, setSource] = useState("");
  const [pct, setPct] = useState(20);
  const [dueAt, setDueAt] = useState("");
  const [note, setNote] = useState("");
  const [users, setUsers] = useState<V4a8[]>([]);
  const [q, setQ] = useState("");
  const [members, setMembers] = useState<Set<string>>(new Set());
  const [preview, setPreview] = useState<CreatePackageResp | null>(null);
  const [previewErr, setPreviewErr] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    V4ac(projectId)
      .then((u) => (u.length > 0 ? u : V4ac()))
      .then((u) => setUsers(u.filter((x) => x.isActive)))
      .catch(() => {});
  }, [projectId]);

  const prefixes = useMemo(() => {
    const p = [phong, hop, hoso].filter(Boolean);
    return p.length ? [p.join("/")] : [];
  }, [phong, hop, hoso]);

  const body = useCallback(
    () => ({
      projectId,
      name,
      kind,
      mode,
      prefixes,
      size,
      count,
      memberIds: [...members],
      dueAt: dueAt || null,
      note: note || null,
      sourcePackageId: kind === "kiem_tra" ? source || null : null,
      samplePct: kind === "kiem_tra" ? pct : null,
    }),
    [projectId, name, kind, mode, prefixes, size, count, members, dueAt, note, source, pct],
  );

  useEffect(() => {
    const t = window.setTimeout(() => {
      workCreatePackage({ ...body(), dryRun: true })
        .then((r) => {
          setPreview(r);
          setPreviewErr(null);
        })
        .catch((e) => {
          setPreview(null);
          setPreviewErr(e);
        });
    }, 300);
    return () => window.clearTimeout(t);
  }, [body]);

  const tao = async () => {
    if (members.size === 0) {
      const ok = await confirm({ title: "Chưa chọn cộng tác viên", body: "Tạo gói chưa giao cho ai? Bạn có thể giao sau ở chi tiết gói.", confirmLabel: "Tạo gói" });
      if (!ok) return;
    }
    setBusy(true);
    try {
      const r = await workCreatePackage(body());
      toast.success(`Đã tạo ${n(r.packageIds.length)} gói (${r.sizes.map(n).join(", ")} tài liệu).`, { description: r.toExtract ? `${n(r.toExtract)} tài liệu đang được máy bóc trường.` : undefined });
      onCreated();
    } catch (e) {
      notifyError(e, "tạo gói");
    } finally {
      setBusy(false);
    }
  };

  const dsUser = users.filter((u) => !q || `${u.displayName} ${u.username}`.toLowerCase().includes(q.toLowerCase()));
  const tongChon = preview?.sizes.reduce((a, b) => a + b, 0) ?? 0;
  const canNext = step === 0 ? (kind === "kiem_tra" ? !!source : true) : step === 1 ? !!preview?.sizes.length : true;

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      title="Tạo gói việc"
      size="lg"
      footer={
        <>
          <Button onClick={onClose}>Huỷ</Button>
          <Toolbar.Spacer />
          {step > 0 && <Button icon="truoc" onClick={() => setStep(step - 1)}>Quay lại</Button>}
          {step < 2 ? (
            <Button variant="primary" iconRight="sau" disabled={!canNext} onClick={() => setStep(step + 1)}>
              Tiếp
            </Button>
          ) : (
            <Button variant="primary" icon="them" loading={busy} disabled={!preview?.sizes.length} onClick={() => void tao()}>
              Tạo gói
            </Button>
          )}
        </>
      }
    >
      {/* eslint-disable-next-line jsx-a11y/no-static-element-interactions -- chỉ bắt Ctrl+Enter để lưu, không phải nút */}
      <div
        onKeyDown={(e) => {
          if (e.ctrlKey && e.key === "Enter" && step === 2) void tao();
        }}
      >
        <Stepper className="mb-4" current={step} onSelect={(i) => i < step && setStep(i)} steps={[{ key: "loai", label: "Loại và phạm vi" }, { key: "chia", label: "Cỡ và số gói" }, { key: "ctv", label: "Cộng tác viên và hạn" }]} />

        {step === 0 && (
          <FormGrid cols={2}>
            <Field label="Loại gói" hint={kind === "kiem_tra" ? "Cộng tác viên khác soát lại một phần mẫu của gói đã xác nhận." : "Cộng tác viên sửa và xác nhận các trường của từng tài liệu."}>
              <Select value={kind} onChange={(v) => setKind(v as WorkKind)} options={[{ value: "xac_nhan", label: KIND_LABEL.xac_nhan }, { value: "kiem_tra", label: KIND_LABEL.kiem_tra }]} />
            </Field>
            <Field label="Tên gói" hint="Để trống thì hệ thống tự đặt">
              <Input value={name} onChange={(e) => setName(e.target.value)} />
            </Field>
            {kind === "kiem_tra" ? (
              <>
                <Field label="Gói xác nhận nguồn" required>
                  <Select value={source} onChange={setSource} placeholder="— Chọn gói —" options={packages.filter((p) => p.kind === "xac_nhan" && p.nDone + p.nChecked + p.nApproved > 0).map((p) => ({ value: p.id, label: `${p.name} (${n(p.nDone + p.nChecked + p.nApproved)} đã xác nhận)` }))} />
                </Field>
                <Field label="Tỷ lệ mẫu kiểm tra (%)">
                  <Input type="number" min={1} max={100} value={pct} onChange={(e) => setPct(Number(e.target.value) || 20)} />
                </Field>
              </>
            ) : (
              <>
                <Field label="Cách chia" hint={mode === "hoso" ? "Lấy tài liệu theo thứ tự hồ sơ, không cắt giữa hồ sơ." : "Bốc ngẫu nhiên trong phạm vi đã chọn."}>
                  <Select value={mode} onChange={(v) => setMode(v as WorkMode)} options={[{ value: "hoso", label: MODE_LABEL.hoso }, { value: "ngau_nhien", label: MODE_LABEL.ngau_nhien }]} />
                </Field>
                <div />
                <Field label="Phông">
                  <Select value={phong} onChange={(v) => { setPhong(v); setHop(""); setHoso(""); }} placeholder="Cả dự án" options={scope.phong.map((p) => ({ value: p, label: p }))} />
                </Field>
                <Field label="Hộp">
                  <Select value={hop} disabled={!phong} onChange={(v) => { setHop(v); setHoso(""); }} placeholder="Cả phông" options={(phong ? scope.hop(phong) : []).map((h) => ({ value: h, label: h }))} />
                </Field>
                <Field label="Hồ sơ">
                  <Select value={hoso} disabled={!hop} onChange={setHoso} placeholder="Cả hộp" options={(phong && hop ? scope.hoso(phong, hop) : []).map((h) => ({ value: h, label: h }))} />
                </Field>
              </>
            )}
          </FormGrid>
        )}

        {step === 1 && (
          <div className="space-y-4">
            {kind === "xac_nhan" && (
              <FormGrid cols={2}>
                <Field label="Cỡ gói (tài liệu)">
                  <Input type="number" min={1} value={size} onChange={(e) => setSize(Math.max(1, Number(e.target.value) || 1))} />
                </Field>
                <Field label="Số gói">
                  <Input type="number" min={1} max={200} value={count} onChange={(e) => setCount(Math.max(1, Number(e.target.value) || 1))} />
                </Field>
              </FormGrid>
            )}
            <Card padding="sm" tone={preview && !preview.sizes.length ? "warning" : undefined}>
              {previewErr ? (
                <ErrorBanner error={previewErr} />
              ) : !preview ? (
                <p className="text-sm text-fg-subtle">Đang tính…</p>
              ) : preview.sizes.length ? (
                <p className="text-sm">
                  Sẽ tạo <b>{n(preview.sizes.length)}</b> gói: {preview.sizes.map(n).join(", ")} tài liệu.{" "}
                  <span className={tongChon === preview.available ? "text-success-700" : "text-warning-700"}>
                    Chiếm {Math.round((tongChon / Math.max(1, preview.available)) * 100)}% trong {n(preview.available)} tài liệu đủ điều kiện.
                  </span>
                  {preview.toExtract > 0 && <span className="block text-warning-700">{n(preview.toExtract)} tài liệu chưa bóc trường, máy sẽ bóc trước khi cộng tác viên mở.</span>}
                </p>
              ) : (
                <p className="text-sm text-warning-800">Không có tài liệu đủ điều kiện (đã OCR xong, chưa giao) trong phạm vi đã chọn.</p>
              )}
            </Card>
          </div>
        )}

        {step === 2 && (
          <div className="space-y-4">
            <FormGrid cols={2}>
              <Field label="Hạn hoàn thành">
                <Input type="date" value={dueAt} onChange={(e) => setDueAt(e.target.value)} />
              </Field>
              <Field label="Ghi chú">
                <Input value={note} onChange={(e) => setNote(e.target.value)} />
              </Field>
            </FormGrid>
            <div>
              <Toolbar className="mb-2">
                <span className="text-sm font-medium text-fg-muted">Cộng tác viên ({n(members.size)} đã chọn)</span>
                <Input size="sm" prefix="tim" placeholder="Tìm tên, tài khoản" value={q} onChange={(e) => setQ(e.target.value)} className="w-56" aria-label="Tìm cộng tác viên" />
                <Button size="sm" onClick={() => setMembers(new Set(dsUser.map((u) => u.id)))}>
                  Chọn tất cả
                </Button>
                <Button size="sm" onClick={() => setMembers(new Set())}>
                  Bỏ chọn
                </Button>
              </Toolbar>
              <div className="flex max-h-48 flex-wrap gap-2 overflow-auto">
                {dsUser.map((u) => (
                  <label key={u.id} className={cn("flex cursor-pointer items-center gap-2 rounded border px-2 py-1 text-sm", members.has(u.id) ? "border-brand-300 bg-brand-50" : "border-border-subtle")}>
                    <Checkbox
                      checked={members.has(u.id)}
                      onChange={(v) =>
                        setMembers((s) => {
                          const m = new Set(s);
                          if (v) m.add(u.id);
                          else m.delete(u.id);
                          return m;
                        })
                      }
                    />
                    {u.displayName} <span className="text-fg-subtle">({u.username})</span>
                  </label>
                ))}
                {dsUser.length === 0 && <span className="text-sm text-fg-subtle">Không có tài khoản nào trong dự án.</span>}
              </div>
            </div>
          </div>
        )}
      </div>
    </Dialog>
  );
}

// ---------------- Chi tiết gói
function ChiTietGoi({ id, onBack, onChanged }: { id: string; onBack: () => void; onChanged: () => void }) {
  const load = useCallback(() => workGetPackage(id), [id]);
  const { data: pkg, error, loading, reload } = V41t(load, { topics: ["work"] });
  const [loc, setLoc] = useState<ItemStatus | "">("");
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [open, setOpen] = useState<WorkItem | null>(null);
  const [suaTv, setSuaTv] = useState<{ users: V4a8[]; sel: Set<string> } | null>(null);
  const [busy, setBusy] = useState(false);
  const [tienDo, setTienDo] = useState<string | null>(null);

  const items = useMemo(() => (pkg?.items ?? []).filter((i) => !loc || i.status === loc), [pkg, loc]);
  const duyetDuoc = (i: WorkItem) => ["done", "redone", "checked_ok", "checked_err"].includes(i.status);

  const review = async (decision: "approve" | "reject", all: boolean) => {
    if (!pkg) return;
    let note: string | undefined;
    if (decision === "reject") {
      const r = await prompt({ title: "Từ chối mục đã chọn", label: "Lý do từ chối", body: "Cộng tác viên sẽ thấy lý do này để sửa lại.", type: "textarea", required: true, confirmLabel: "Từ chối", danger: true });
      if (r === null) return;
      note = r.trim();
    }
    const ids = all ? undefined : [...sel];
    if (!all && ids!.length === 0) return;
    if (all) {
      const ok = await confirm({ title: `${decision === "approve" ? "Duyệt" : "Từ chối"} tất cả mục đã xác nhận trong gói?`, confirmLabel: decision === "approve" ? "Duyệt tất cả" : "Từ chối tất cả", danger: decision === "reject" });
      if (!ok) return;
    }
    setBusy(true);
    try {
      const r = await workReview(pkg.id, { itemIds: ids, decision, note });
      toast.success(`${decision === "approve" ? "Đã duyệt" : "Đã từ chối"} ${n(r.changed)} mục.`, { description: r.skipped ? `Bỏ qua ${n(r.skipped)} mục chưa xác nhận hoặc do chính bạn xác nhận.` : undefined });
      setSel(new Set());
      reload();
      onChanged();
    } catch (e) {
      notifyError(e, decision === "approve" ? "duyệt" : "từ chối");
    } finally {
      setBusy(false);
    }
  };

  const doiHan = async () => {
    if (!pkg) return;
    const d = await prompt({ title: "Đổi hạn hoàn thành", label: "Hạn mới", type: "date", initial: pkg.dueAt ?? "", body: "Để trống để bỏ hạn.", confirmLabel: "Lưu" });
    if (d === null) return;
    try {
      await workPatchPackage(pkg.id, { dueAt: d.trim() || null });
      toast.success(d.trim() ? `Đã đặt hạn ${ngay(d.trim())}.` : "Đã bỏ hạn hoàn thành.");
      reload();
      onChanged();
    } catch (e) {
      notifyError(e, "đổi hạn");
    }
  };
  const dongGoi = async () => {
    if (!pkg) return;
    const ok = await confirm({ title: "Đóng gói việc này?", body: "Mục chưa duyệt sẽ huỷ và tài liệu được giao lại ở gói khác.", confirmLabel: "Đóng gói", danger: true });
    if (!ok) return;
    try {
      await workClosePackage(pkg.id);
      toast.success("Đã đóng gói việc.");
      reload();
      onChanged();
    } catch (e) {
      notifyError(e, "đóng gói việc");
    }
  };
  const moSuaTv = async () => {
    if (!pkg) return;
    try {
      let u = await V4ac(pkg.projectId);
      if (u.length === 0) u = await V4ac();
      setSuaTv({ users: u.filter((x) => x.isActive), sel: new Set(pkg.members.map((m) => m.userId)) });
    } catch (e) {
      notifyError(e, "lấy danh sách cộng tác viên");
    }
  };
  const luuTv = async () => {
    if (!pkg || !suaTv) return;
    setBusy(true);
    try {
      await workSetMembers(pkg.id, [...suaTv.sel]);
      setSuaTv(null);
      toast.success("Đã cập nhật cộng tác viên.");
      reload();
      onChanged();
    } catch (e) {
      notifyError(e, "lưu cộng tác viên");
    } finally {
      setBusy(false);
    }
  };
  const xuatMeta = async (kieu: "excel" | "in") => {
    if (!pkg || items.length === 0) return;
    setBusy(true);
    try {
      const truong = await layTruongNhieuTaiLieu(items.map((i) => i.documentId), (d, t) => setTienDo(`Đang lấy trường ${d}/${t}…`));
      const rows: MetaRow[] = items.map((i) => {
        const f = truong.get(i.documentId);
        return { id: i.documentId, filename: i.filename, hoSoRef: i.hoSoRef ?? "", relPath: i.relPath ?? "", fields: f?.fields ?? {}, daXacNhan: f?.daXacNhan ?? false, trangThai: ITEM_STATUS_LABEL[i.status], nguoiLam: i.doneByName ?? "", kiemTra: i.checkedByName ?? "", ghiChu: [i.checkNote, i.reviewNote].filter(Boolean).join(" · ") };
      });
      const ten = `${pkg.name}${loc ? ` - ${ITEM_STATUS_LABEL[loc]}` : ""}`;
      if (kieu === "excel") await xuatMetadataExcel(rows, `metadata-${tenTepAnToan(pkg.name)}.xlsx`);
      else inMetadata(rows, "METADATA TÀI LIỆU ĐÃ BÓC TÁCH", ten);
      toast.success(`${kieu === "excel" ? "Đã xuất" : "Đã in"} metadata ${n(rows.length)} tài liệu.`);
    } catch (e) {
      notifyError(e, "xuất metadata");
    } finally {
      setBusy(false);
      setTienDo(null);
    }
  };

  if (open) {
    return (
      <div className="mt-2">
        <V4i7
          doc={{ id: open.documentId, filename: open.filename, reviewStatus: "pending" }}
          onBack={() => {
            setOpen(null);
            reload();
          }}
        />
      </div>
    );
  }
  if (!pkg) {
    return (
      <Card className="mt-2">
        {loading ? <p className="text-sm text-fg-subtle">Đang tải gói…</p> : <ErrorBanner error={error ?? "Không tải được gói."} onRetry={reload} />}
      </Card>
    );
  }

  const tiles: { value: ItemStatus | ""; label: string; count: number }[] = [
    { value: "", label: "Tất cả", count: pkg.nTotal },
    { value: "pending", label: "Chờ làm", count: pkg.nPending },
    { value: "done", label: "Đã xác nhận", count: pkg.nDone },
    { value: "checked_ok", label: "Kiểm tra đạt", count: pkg.items.filter((i) => i.status === "checked_ok").length },
    { value: "checked_err", label: "Kiểm tra lỗi", count: pkg.items.filter((i) => i.status === "checked_err").length },
    { value: "rejected", label: "Bị trả", count: pkg.nRejected },
    { value: "approved", label: "Đã duyệt", count: pkg.nApproved },
  ];
  const cols: Column<WorkItem>[] = [
    { key: "ord", header: "#", align: "right", width: 56 },
    { key: "filename", header: "Tài liệu", sortable: true },
    { key: "relPath", header: "Đường dẫn", cell: (i) => <span className="text-fg-subtle">{i.relPath ?? i.hoSoRef ?? "—"}</span>, hideBelow: 1366 },
    { key: "status", header: "Trạng thái", width: 140, cell: (i) => <Badge tone={itemTone(i.status)}>{ITEM_STATUS_LABEL[i.status]}</Badge> },
    { key: "doneByName", header: "Người làm", cell: (i) => i.doneByName ?? "—", sortable: true },
    { key: "checkedByName", header: "Kiểm tra", cell: (i) => i.checkedByName ?? "—", hideBelow: 1440 },
    { key: "note", header: "Ghi chú", className: "whitespace-normal text-sm", cell: (i) => [i.checkNote, i.reviewNote].filter(Boolean).join(" · ") },
  ];

  return (
    <>
      <PageHeader
        onBack={onBack}
        title={pkg.name}
        meta={
          <>
            <Badge tone="gray">{KIND_LABEL[pkg.kind]}</Badge>
            <Badge tone={pkgTone(pkg.status)}>{PKG_STATUS_LABEL[pkg.status]}</Badge>
            <span className={cn("text-sm", quaHan(pkg) ? "text-danger-700" : "text-fg-subtle")}>Hạn: {ngay(pkg.dueAt)}</span>
          </>
        }
        description={`Cộng tác viên: ${pkg.members.map((m) => m.displayName).join(", ") || "chưa giao"}${pkg.note ? ` · ${pkg.note}` : ""}`}
        actions={
          <>
            <Button variant="primary" icon="duyet" disabled={sel.size === 0 || busy} onClick={() => void review("approve", false)}>
              Duyệt đã chọn ({n(sel.size)})
            </Button>
            <Button variant="dangerOutline" disabled={sel.size === 0 || busy} onClick={() => void review("reject", false)}>
              Từ chối đã chọn
            </Button>
            <Menu
              items={[
                { label: "Duyệt tất cả đã xác nhận", icon: "duyet", onSelect: () => void review("approve", true) },
                "-",
                { label: "Sửa cộng tác viên", icon: "thanhVien", onSelect: () => void moSuaTv() },
                { label: "Đổi hạn hoàn thành", icon: "nhacHan", onSelect: () => void doiHan() },
                "-",
                { label: "Xuất metadata Excel", icon: "tepExcel", onSelect: () => void xuatMeta("excel"), disabled: items.length === 0 },
                { label: "In metadata", icon: "inRa", onSelect: () => void xuatMeta("in"), disabled: items.length === 0 },
                "-",
                { label: "Đóng gói việc", icon: "daHuy", danger: true, onSelect: () => void dongGoi(), disabled: pkg.status === "closed" },
              ]}
            />
          </>
        }
      />
      {error && <ErrorBanner error={error} onRetry={reload} className="mb-3" />}
      {tienDo && <p className="mb-2 text-sm text-fg-subtle">{tienDo}</p>}
      <Card padding="sm" className="mb-3">
        <TienDo p={pkg} />
      </Card>
      <Card padding="none">
        <div className="border-b border-border-subtle p-3">
          <Seg value={loc} onChange={setLoc} options={tiles.map((t) => ({ value: t.value, label: `${t.label} ${n(t.count)}` }))} />
        </div>
        <Table
          columns={cols}
          rows={items}
          getRowId={(i) => i.id}
          selection={{ selected: sel, onChange: setSel, canSelect: duyetDuoc }}
          onRowClick={(i) => setOpen(i)}
          rowActions={(i) => <IconButton label="Mở tài liệu" icon="xem" size="sm" onClick={() => setOpen(i)} />}
          state={{ loading: loading && !pkg, empty: { title: "Không có mục nào ở trạng thái này" } }}
          maxHeight="calc(100dvh - 360px)"
        />
      </Card>

      <Dialog open={!!suaTv} onOpenChange={(o) => !o && setSuaTv(null)} title="Cộng tác viên của gói" size="md" footer={<><Button onClick={() => setSuaTv(null)}>Huỷ</Button><Button variant="primary" loading={busy} onClick={() => void luuTv()}>Lưu</Button></>}>
        {suaTv && (
          <div className="flex max-h-72 flex-wrap gap-2 overflow-auto">
            {suaTv.users.map((u) => (
              <label key={u.id} className={cn("flex cursor-pointer items-center gap-2 rounded border px-2 py-1 text-sm", suaTv.sel.has(u.id) ? "border-brand-300 bg-brand-50" : "border-border-subtle")}>
                <Checkbox
                  checked={suaTv.sel.has(u.id)}
                  onChange={(v) => {
                    const m = new Set(suaTv.sel);
                    if (v) m.add(u.id);
                    else m.delete(u.id);
                    setSuaTv({ ...suaTv, sel: m });
                  }}
                />
                {u.displayName} <span className="text-fg-subtle">({u.username})</span>
              </label>
            ))}
            {suaTv.users.length === 0 && <span className="text-sm text-fg-subtle">Không có tài khoản nào trong dự án.</span>}
          </div>
        )}
      </Dialog>
    </>
  );
}

// ---------------- Danh sách gói
export default function WorkManageView() {
  const project = useProject();
  const [projects, setProjects] = useState<V4a9[]>([]);
  const [projectId, setProjectId] = useState(project.projectId ?? "");
  const [docs, setDocs] = useState<V4ej[]>([]);
  const [tao, setTao] = useState(false);
  const [open, setOpen] = useState<string | null>(null);
  const [q, setQ] = useState("");

  useEffect(() => setProjectId(project.projectId ?? ""), [project.projectId, project.version]);
  useEffect(() => {
    if (!isTauri()) return;
    V4ak()
      .then((p) => setProjects(p.filter((x) => x.isActive)))
      .catch(() => {});
    V4ek().then(setDocs).catch(() => {});
  }, []);
  const load = useCallback(() => workListPackages(projectId || undefined), [projectId]);
  const { data: pkgs, error, loading, refreshing, reload } = V41t(load, { topics: ["work"], enabled: isTauri() });
  const shown = useMemo(() => {
    const k = q.trim().toLowerCase();
    return (pkgs ?? []).filter((p) => !k || p.name.toLowerCase().includes(k) || p.members.some((m) => m.displayName.toLowerCase().includes(k)));
  }, [pkgs, q]);

  const cols: Column<WorkPackage>[] = [
    { key: "name", header: "Gói", sortable: true, cell: (p) => <div><div className="font-medium text-fg-strong">{p.name}</div><div className="text-xs text-fg-subtle">{KIND_LABEL[p.kind]} · {MODE_LABEL[p.mode]}</div></div> },
    { key: "members", header: "Cộng tác viên", className: "whitespace-normal text-sm", cell: (p) => p.members.map((m) => m.displayName).join(", ") || <span className="text-warning-700">Chưa giao</span> },
    { key: "dueAt", header: "Hạn", sortable: true, width: 100, cell: (p) => <span className={quaHan(p) ? "font-medium text-danger-700" : undefined}>{ngay(p.dueAt)}</span> },
    { key: "nTotal", header: "Tổng", align: "right", sortable: true, cell: (p) => n(p.nTotal), width: 70 },
    { key: "nPending", header: "Chờ", align: "right", sortable: true, cell: (p) => n(p.nPending), width: 70 },
    { key: "nDone", header: "Đã XN", align: "right", sortable: true, sortValue: (p) => p.nDone + p.nChecked, cell: (p) => n(p.nDone + p.nChecked), width: 80 },
    { key: "nRejected", header: "Bị trả", align: "right", sortable: true, cell: (p) => n(p.nRejected), width: 70 },
    { key: "nApproved", header: "Đã duyệt", align: "right", sortable: true, cell: (p) => n(p.nApproved), width: 90 },
    { key: "tienDo", header: "Tiến độ", width: 140, cell: (p) => <TienDo p={p} /> },
    { key: "status", header: "Trạng thái", width: 130, cell: (p) => <Badge tone={pkgTone(p.status)}>{PKG_STATUS_LABEL[p.status]}</Badge> },
  ];

  return (
    <section className="view active">
      {open ? (
        <ChiTietGoi id={open} onBack={() => setOpen(null)} onChanged={reload} />
      ) : (
        <>
          <PageHeader
            title="Giao việc"
            description="Chia tài liệu đã OCR thành gói để cộng tác viên xác nhận trường, rồi duyệt kết quả."
            actions={
              <>
                <IconButton label="Tải lại" icon="lamMoi" variant="secondary" loading={refreshing} onClick={reload} />
                <Tip text={projectId ? "" : "Chọn dự án trước"}>
                  <span>
                    <Button variant="primary" icon="them" disabled={!projectId} onClick={() => setTao(true)}>
                      Tạo gói
                    </Button>
                  </span>
                </Tip>
              </>
            }
          />
          {error && pkgs && <ErrorBanner error={error} onRetry={reload} className="mb-3" />}
          <Card padding="none">
            <div className="border-b border-border-subtle p-3">
              <Toolbar>
                <Select size="sm" value={projectId} onChange={setProjectId} placeholder="Tất cả dự án" options={projects.map((p) => ({ value: p.id, label: `${p.code} · ${p.name}` }))} className="w-64" aria-label="Dự án" />
                <Input size="sm" prefix="tim" placeholder="Tìm gói, cộng tác viên" value={q} onChange={(e) => setQ(e.target.value)} className="w-60" aria-label="Tìm gói" />
                <Toolbar.Spacer />
                <span className="text-sm text-fg-subtle">{n(shown.length)} gói</span>
              </Toolbar>
            </div>
            <Table
              columns={cols}
              rows={shown}
              getRowId={(p) => p.id}
              onRowClick={(p) => setOpen(p.id)}
              state={{ loading: loading && !pkgs, error: pkgs ? undefined : error, onRetry: reload, empty: { title: "Chưa có gói việc", description: projectId ? "Bấm Tạo gói để chia tài liệu cho cộng tác viên." : "Chọn dự án rồi bấm Tạo gói." } }}
            />
          </Card>
          {tao && projectId && (
            <TaoGoi
              projectId={projectId}
              packages={pkgs ?? []}
              docs={docs}
              onCreated={() => {
                setTao(false);
                reload();
              }}
              onClose={() => setTao(false)}
            />
          )}
        </>
      )}
    </section>
  );
}
