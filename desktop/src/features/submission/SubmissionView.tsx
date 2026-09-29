// Copyright (c) 2026 DigiText. All rights reserved.
// Thu nộp vào lưu trữ lịch sử: danh sách lần nộp (trái) và chi tiết (phải) với dãy bước
// Đăng ký → Xác nhận thu → Nộp gói → Tiếp nhận → Chấm hồ sơ → Phê duyệt và hoàn thành.
// Căn cứ pháp lý (Điều 20–25) chỉ hiện ở tooltip / trợ giúp, không nằm trong nhãn nút.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V4rw } from "@/api/packaging";
import { V4s2, V4s3, V4s4, V4s6, V4s7, V4s9, V4sh, V4si, type V4rx, type V4rz, type V4s0, type V4s5 } from "@/api/submissions";
import { V4t3, type V4t2 } from "@/api/records";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { V462 as cn } from "@/lib/utils";
import { Badge, Banner, Button, Card, confirm, Dialog, EmptyState, ErrorBanner, Field, FormGrid, Help, IconButton, Input, Menu, notifyError, PageHeader, prompt, Select, Stepper, Table, Textarea, Tip, toast, Toolbar, type Column } from "@/ui";
import { V4t6, V4t7 } from "./submissionFlow";

const n = (v: number) => v.toLocaleString("vi-VN");

const STATUS: Record<string, { label: string; tone: "ok" | "info" | "warn" | "err" | "gray"; step: number }> = {
  registered: { label: "Chờ xác nhận thu", tone: "warn", step: 1 },
  confirmed: { label: "Đang nộp gói", tone: "info", step: 2 },
  refused: { label: "Từ chối thu", tone: "err", step: 1 },
  awaiting_reception: { label: "Chờ tiếp nhận", tone: "warn", step: 3 },
  received: { label: "Đang chấm hồ sơ", tone: "info", step: 4 },
  approved: { label: "Đã phê duyệt", tone: "ok", step: 5 },
  completed: { label: "Hoàn thành", tone: "ok", step: 6 },
};
const st = (s: string) => STATUS[s] ?? { label: s, tone: "gray" as const, step: 0 };

const STEPS = [
  { key: "dangky", label: "Đăng ký" },
  { key: "xacnhan", label: "Xác nhận thu" },
  { key: "nopgoi", label: "Nộp gói SIP" },
  { key: "tiepnhan", label: "Tiếp nhận" },
  { key: "cham", label: "Chấm hồ sơ" },
  { key: "pheduyet", label: "Phê duyệt và hoàn thành" },
];

const ACTION_LABEL: Record<V4s5 | "approve", { label: string; legal: string; icon: "duyet" | "tuChoi" | "kiemTra" | "nhapKho" | "thanhCong" | "kySo" }> = {
  confirm: { label: "Đồng ý thu", legal: "Theo Điều 21.2", icon: "duyet" },
  refuse: { label: "Từ chối thu", legal: "Theo Điều 21.3", icon: "tuChoi" },
  reconcile: { label: "Đối chiếu mục lục", legal: "Theo Điều 22.3.d", icon: "kiemTra" },
  receive: { label: "Tiếp nhận", legal: "Theo Điều 23", icon: "nhapKho" },
  approve: { label: "Phê duyệt", legal: "Theo Điều 25, chuyển SIP thành AIP", icon: "kySo" },
  complete: { label: "Hoàn thành", legal: "Theo Điều 25.5", icon: "thanhCong" },
};

export default function V4t9() {
  const { session, goto } = V460();
  const perms = session?.permissions ?? [];
  const can = (p: Parameters<typeof V3z9>[1]) => V3z9(perms, p);
  const [selId, setSelId] = useState<string | null>(null);
  const [q, setQ] = useState("");
  const [statusF, setStatusF] = useState("");
  const [dangKy, setDangKy] = useState(false);
  const [busy, setBusy] = useState(false);

  const loadList = useCallback(() => V4s2(), []);
  const { data: list, error, loading, refreshing, reload } = V41t(loadList, { topics: ["submissions", "docs"], enabled: isTauri() });

  const loadDetail = useCallback(async () => {
    if (!selId) return null;
    const d = await V4s3(selId);
    const returned = d.status === "approved" || d.status === "completed" ? await V4sh(selId).catch(() => null) : null;
    return { d, returned };
  }, [selId]);
  const detail = V41t(loadDetail, { topics: ["submissions", "packages"], enabled: isTauri() && !!selId });
  const d = detail.data?.d ?? null;
  const returned = detail.data?.returned ?? null;

  const shown = useMemo(() => {
    const k = q.trim().toLowerCase();
    return (list ?? []).filter((r) => (!statusF || r.status === statusF) && (!k || r.maYeuCau.toLowerCase().includes(k) || r.orgName.toLowerCase().includes(k) || r.fondCode.toLowerCase().includes(k)));
  }, [list, q, statusF]);
  useEffect(() => {
    if (!selId && shown.length > 0) setSelId(shown[0].id);
  }, [shown, selId]);

  const run = async (ctx: string, f: () => Promise<void>) => {
    setBusy(true);
    try {
      await f();
      reload();
      detail.reload();
    } catch (e) {
      notifyError(e, ctx);
    } finally {
      setBusy(false);
    }
  };

  const act = async (id: string, action: V4s5 | "approve", requiresNote?: boolean) => {
    const meta = ACTION_LABEL[action];
    let note: string | undefined;
    if (requiresNote) {
      const r = await prompt({ title: meta.label, label: "Lý do", type: "textarea", required: true, confirmLabel: meta.label, danger: action === "refuse" });
      if (r === null) return;
      note = r.trim();
    } else if (action === "confirm" || action === "receive") {
      const r = await prompt({ title: meta.label, label: "Ghi chú", body: "Có thể để trống.", type: "textarea", confirmLabel: meta.label });
      if (r === null) return;
      note = r.trim() || undefined;
    }
    if (action === "approve") {
      const qd = await prompt({ title: "Phê duyệt lần nộp", label: "Số quyết định phê duyệt", required: true, confirmLabel: "Phê duyệt", body: "Gói SIP của lần nộp sẽ được chuyển thành AIP." });
      if (qd === null) return;
      await run("phê duyệt", async () => {
        const r = await V4s9(id, qd.trim(), note);
        toast.success(`Đã phê duyệt. ${n(r.convertedAipIds.length)} gói AIP đang được dựng.`);
      });
      return;
    }
    if (action === "complete") {
      const ok = await confirm({ title: "Hoàn thành thu nộp?", body: "Lần nộp sẽ đóng lại, không sửa được nữa.", confirmLabel: "Hoàn thành" });
      if (!ok) return;
    }
    await run(meta.label.toLowerCase(), async () => {
      await V4s6(id, action, note);
      toast.success(`Đã ${meta.label.toLowerCase()}.`);
    });
  };

  const cham = async (ids: string[], result: "dat" | "khong_dat") => {
    if (!d || ids.length === 0) return;
    let note: string | undefined;
    if (result === "khong_dat") {
      const r = await prompt({ title: `Không đạt ${ids.length > 1 ? `${n(ids.length)} hồ sơ` : "hồ sơ"}`, label: "Lý do không đạt", type: "textarea", required: true, confirmLabel: "Không đạt", danger: true });
      if (r === null) return;
      note = r.trim();
    }
    await run("chấm hồ sơ", async () => {
      for (const it of ids) await V4s7(d.id, it, result, note);
      toast.success(`Đã chấm ${n(ids.length)} hồ sơ: ${result === "dat" ? "đạt" : "không đạt"}.`);
      setSelItems(new Set());
    });
  };
  const [selItems, setSelItems] = useState<Set<string>>(new Set());

  const bienBan = (loai: V4t2) =>
    run("lập biên bản", async () => {
      if (!d) return;
      const r = await V4t3(d.id, undefined, loai);
      toast.success(`Đã lập biên bản số ${r.soBienBan}.`, { action: { label: "Mở Biên bản", onClick: () => goto("records") } });
    });
  const taiHoSoQuaTrinh = () =>
    run("tải hồ sơ quá trình", async () => {
      if (!d) return;
      const path = await V4si(d.id, `ho-so-qua-trinh-${d.maYeuCau.split("/").join("-")}.zip`);
      toast.success("Đã lưu hồ sơ quá trình thu nộp.", { description: path });
    });

  const listCols: Column<V4rx>[] = [
    { key: "maYeuCau", header: "Mã", width: 120, cell: (r) => <span className="font-medium text-fg-strong">{r.maYeuCau}</span> },
    { key: "orgName", header: "Cơ quan nộp", className: "whitespace-normal", cell: (r) => <div><div>{r.orgName}</div><div className="text-xs text-fg-subtle">Phông {r.fondCode} · lần {r.soLanNop}</div></div> },
    { key: "status", header: "Trạng thái", width: 140, cell: (r) => <Badge tone={st(r.status).tone}>{st(r.status).label}</Badge> },
  ];

  return (
    <section className="view active">
      <PageHeader
        title="Thu nộp"
        description="Đăng ký, xác nhận, tiếp nhận và phê duyệt tài liệu nộp vào lưu trữ lịch sử."
        actions={
          <>
            <IconButton label="Tải lại" icon="lamMoi" variant="secondary" loading={refreshing} onClick={reload} />
            {can("submission.create") && (
              <Button variant="primary" icon="them" onClick={() => setDangKy(true)}>
                Đăng ký nộp
              </Button>
            )}
          </>
        }
      />
      {error && list && <ErrorBanner error={error} onRetry={reload} className="mb-3" />}

      <div className="grid gap-4 xl:grid-cols-[minmax(320px,2fr)_3fr]">
        <Card padding="none" className="self-start">
          <div className="border-b border-border-subtle p-3">
            <Toolbar>
              <Input size="sm" prefix="tim" placeholder="Mã, cơ quan, phông" value={q} onChange={(e) => setQ(e.target.value)} className="min-w-40 flex-1" aria-label="Tìm lần nộp" />
              <Select size="sm" value={statusF} onChange={setStatusF} placeholder="Mọi trạng thái" options={Object.entries(STATUS).map(([k, v]) => ({ value: k, label: v.label }))} className="w-44" aria-label="Lọc trạng thái" />
            </Toolbar>
          </div>
          <Table
            columns={listCols}
            rows={shown}
            getRowId={(r) => r.id}
            onRowClick={(r) => setSelId(r.id)}
            rowClassName={(r) => (r.id === selId ? "bg-brand-50" : undefined)}
            state={{ loading: loading && !list, error: list ? undefined : error, onRetry: reload, empty: (list?.length ?? 0) === 0 ? { title: "Chưa có lần nộp", description: can("submission.create") ? "Bấm Đăng ký nộp để bắt đầu." : undefined } : { title: "Không có lần nộp khớp bộ lọc" } }}
            maxHeight="calc(100dvh - 300px)"
          />
        </Card>

        <div className="min-w-0">
          {!selId ? (
            <Card>
              <EmptyState icon="nhapKho" title="Chọn một lần nộp" description="Chi tiết và các bước xử lý hiện ở đây." />
            </Card>
          ) : !d ? (
            <Card>{detail.loading ? <p className="text-sm text-fg-subtle">Đang tải chi tiết…</p> : <ErrorBanner error={detail.error ?? "Không tải được chi tiết."} onRetry={detail.reload} />}</Card>
          ) : (
            <div className="space-y-4">
              <Card
                title={
                  <span className="flex flex-wrap items-center gap-2">
                    {d.maYeuCau}
                    <Badge tone={st(d.status).tone}>{st(d.status).label}</Badge>
                  </span>
                }
                description={`${d.orgName} (mã ${d.orgCode}) · phông ${d.fondCode} · lần nộp ${d.soLanNop}${d.hanNop ? ` · hạn nộp ${d.hanNop.split("-").reverse().join("/")}` : ""}`}
                actions={
                  (d.status === "approved" || d.status === "completed") && (
                    <Menu
                      label="Biên bản"
                      trigger="button"
                      items={[
                        ...(can("submission.review")
                          ? [
                              { label: "Lập biên bản thu nộp", icon: "bienBan" as const, onSelect: () => void bienBan("thu_nop") },
                              { label: "Lập biên bản giao nhận", icon: "bienBan" as const, onSelect: () => void bienBan("giao_nhan") },
                              { label: "Lập biên bản trả lại", icon: "bienBan" as const, onSelect: () => void bienBan("tra_lai") },
                              "-" as const,
                            ]
                          : []),
                        { label: "Tải hồ sơ quá trình (zip)", icon: "hoSo", onSelect: () => void taiHoSoQuaTrinh() },
                      ]}
                    />
                  )
                }
              >
                <Stepper steps={STEPS} current={d.status === "completed" ? STEPS.length : st(d.status).step} className="mb-4" />
                {d.status === "refused" && <Banner tone="danger" className="mb-3">Lần nộp đã bị từ chối thu{d.decisionNote ? `: ${d.decisionNote}` : "."}</Banner>}
                {d.duplicateInSystem.length > 0 && (
                  <Banner tone="warning" className="mb-3" title="Trùng với hồ sơ đã có trong hệ thống">
                    {d.duplicateInSystem.join(", ")}. Người tiếp nhận quyết định có nhận hay không.
                  </Banner>
                )}
                {(d.summary || d.reconcileNote || d.decisionNote || d.receptionNote || d.approvalDoc) && (
                  <dl className="mb-3 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
                    {d.summary && <><dt className="text-fg-subtle">Tóm tắt</dt><dd>{d.summary}</dd></>}
                    {d.reconcileNote && <><dt className="text-fg-subtle">Đối chiếu</dt><dd>{d.reconcileNote}</dd></>}
                    {d.receptionNote && <><dt className="text-fg-subtle">Tiếp nhận</dt><dd>{d.receptionNote}</dd></>}
                    {d.decisionNote && <><dt className="text-fg-subtle">Ý kiến</dt><dd>{d.decisionNote}</dd></>}
                    {d.approvalDoc && <><dt className="text-fg-subtle">Quyết định</dt><dd>{d.approvalDoc}</dd></>}
                  </dl>
                )}
                <Toolbar>
                  {V4t6(d.status, perms).map((a) => {
                    const m = ACTION_LABEL[a.action];
                    return (
                      <Tip key={a.action} text={m.legal}>
                        <Button variant={a.action === "refuse" ? "dangerOutline" : "primary"} icon={m.icon} disabled={busy} onClick={() => void act(d.id, a.action, a.requiresNote)}>
                          {m.label}
                        </Button>
                      </Tip>
                    );
                  })}
                  {d.status === "confirmed" && can("package.view") && (
                    <Button icon="goi" onClick={() => goto("package")}>
                      Tạo gói SIP ở màn Đóng gói
                    </Button>
                  )}
                </Toolbar>
              </Card>

              <Card
                title={`Mục lục hồ sơ (${n(d.items.length)})`}
                padding="none"
                actions={
                  d.status === "received" && can("submission.review") && selItems.size > 0 && (
                    <>
                      <Button size="sm" variant="primary" icon="duyet" disabled={busy} onClick={() => void cham([...selItems], "dat")}>
                        Đạt ({n(selItems.size)})
                      </Button>
                      <Button size="sm" variant="dangerOutline" disabled={busy} onClick={() => void cham([...selItems], "khong_dat")}>
                        Không đạt
                      </Button>
                    </>
                  )
                }
              >
                <Table
                  columns={[
                    { key: "stt", header: "#", align: "right", width: 50 },
                    { key: "maHoSo", header: "Mã hồ sơ", width: 140 },
                    { key: "tieuDe", header: "Tiêu đề", className: "whitespace-normal" },
                    { key: "soTrang", header: "Trang", align: "right", width: 70, cell: (i) => i.soTrang ?? "—" },
                    {
                      key: "reviewResult",
                      header: "Kết quả",
                      width: 150,
                      cell: (i) =>
                        i.reviewResult === "pending" ? <span className="text-fg-subtle">Chưa chấm</span> : i.reviewResult === "dat" ? <Badge tone="ok">Đạt</Badge> : <Tip text={i.reviewNote ?? ""}><span><Badge tone="err">Không đạt</Badge></span></Tip>,
                    },
                  ]}
                  rows={d.items}
                  getRowId={(i) => i.id}
                  selection={d.status === "received" && can("submission.review") ? { selected: selItems, onChange: setSelItems } : undefined}
                  rowActions={
                    d.status === "received" && can("submission.review")
                      ? (i) => (
                          <>
                            <IconButton label="Đạt" icon="duyet" size="sm" disabled={busy} onClick={() => void cham([i.id], "dat")} />
                            <IconButton label="Không đạt" icon="tuChoi" size="sm" disabled={busy} onClick={() => void cham([i.id], "khong_dat")} />
                          </>
                        )
                      : undefined
                  }
                  state={{ empty: { title: "Mục lục trống" } }}
                  maxHeight={360}
                />
              </Card>

              {returned && returned.totalReturned > 0 && (
                <Card title={`Hồ sơ trả lại (${n(returned.totalReturned)}/${n(returned.totalRegistered)})`} description={`Đã duyệt ${n(returned.totalApproved)} hồ sơ.`} padding="none">
                  <Table
                    columns={[
                      { key: "stt", header: "#", align: "right", width: 50 },
                      { key: "maHoSo", header: "Mã hồ sơ", width: 140 },
                      { key: "tieuDe", header: "Tiêu đề", className: "whitespace-normal" },
                      { key: "reviewNote", header: "Lý do", className: "whitespace-normal text-sm", cell: (i) => i.reviewNote ?? "—" },
                    ]}
                    rows={returned.returnedItems}
                    getRowId={(i) => i.id}
                    rowActions={(i) =>
                      i.sipPackageId ? (
                        <Button
                          size="sm"
                          icon="taiVe"
                          disabled={busy}
                          onClick={() =>
                            void run("tải hồ sơ trả lại", async () => {
                              const path = await V4rw(i.sipPackageId as string, i.sipZipName ?? `${i.maHoSo}.zip`);
                              toast.success("Đã tải hồ sơ trả lại.", { description: path });
                            })
                          }
                        >
                          Tải
                        </Button>
                      ) : null
                    }
                  />
                </Card>
              )}

              <Card title={`Gói tin của lần nộp (${n(d.packages.length)})`} padding="none">
                <Table
                  columns={[
                    { key: "label", header: "Nhãn", cell: (p) => <Tip text={p.objid ?? ""}><span>{p.label}</span></Tip> },
                    { key: "kind", header: "Loại", width: 110, cell: (p) => `${p.kind}${p.convertedFrom ? " (chuyển hoá)" : ""}` },
                    { key: "status", header: "Trạng thái", width: 120, cell: (p) => <Badge tone={p.status === "done" || p.status === "ok" ? "ok" : p.status === "failed" ? "err" : "gray"}>{p.status}</Badge> },
                    { key: "documentCount", header: "Tài liệu", align: "right", width: 90 },
                    { key: "zipName", header: "Tệp", cell: (p) => p.zipName ?? "—" },
                  ]}
                  rows={d.packages}
                  getRowId={(p) => p.id}
                  state={{ empty: { title: "Chưa có gói", description: "Tạo gói SIP ở màn Đóng gói và gắn vào lần nộp này." } }}
                />
              </Card>
            </div>
          )}
        </div>
      </div>

      <Help className="mt-4" title="Quy trình thu nộp">
        <p>Cơ quan nộp đăng ký (Điều 20) → lưu trữ lịch sử xác nhận hoặc từ chối (Điều 21) → nộp gói SIP và đối chiếu mục lục (Điều 22) → tiếp nhận (Điều 23) → chấm từng hồ sơ đạt / không đạt (Điều 24) → phê duyệt, chuyển SIP thành AIP, lập biên bản và trả lại hồ sơ không đạt (Điều 25).</p>
      </Help>

      {dangKy && (
        <DangKyDialog
          projectId={session?.projectId ?? null}
          onClose={() => setDangKy(false)}
          onDone={(r) => {
            setDangKy(false);
            reload();
            setSelId(r.id);
          }}
        />
      )}
    </section>
  );
}

// ---------------- Đăng ký nộp (2 bước)
const EMPTY_ITEM: V4s0 = { maHoSo: "", tieuDe: "" };

function DangKyDialog({ projectId, onClose, onDone }: { projectId: string | null; onClose: () => void; onDone: (r: V4rz) => void }) {
  const [step, setStep] = useState(0);
  const [orgName, setOrgName] = useState("");
  const [orgCode, setOrgCode] = useState("");
  const [fondCode, setFondCode] = useState("");
  const [summary, setSummary] = useState("");
  const [hanNop, setHanNop] = useState("");
  const [items, setItems] = useState<V4s0[]>([{ ...EMPTY_ITEM }]);
  const [paste, setPaste] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [touched, setTouched] = useState(false);

  const body = () => ({ projectId, orgName, orgCode, fondCode, summary: summary.trim() || null, cachThuc: "truc_tuyen" as const, hanNop: hanNop || null, items: items.filter((i) => i.maHoSo.trim() || i.tieuDe.trim()) });
  const errs = V4t7(body());
  const step0Ok = orgName.trim() && orgCode.trim() && fondCode.trim();
  const set = (i: number, p: Partial<V4s0>) => setItems((s) => s.map((x, j) => (j === i ? { ...x, ...p } : x)));

  const apDung = () => {
    if (paste === null) return;
    const rows = paste
      .split(/\r?\n/)
      .map((l) => l.split(/\t|;/).map((c) => c.trim()))
      .filter((c) => c[0] || c[1]);
    if (rows.length === 0) return;
    setItems((s) => [...s.filter((x) => x.maHoSo.trim() || x.tieuDe.trim()), ...rows.map((c) => ({ maHoSo: c[0] ?? "", tieuDe: c[1] ?? "", soTrang: c[2] && !Number.isNaN(Number(c[2])) ? Number(c[2]) : null }))]);
    setPaste(null);
  };

  const gui = async () => {
    setTouched(true);
    if (errs.length > 0 || busy) return;
    setBusy(true);
    try {
      const r = await V4s4(body());
      toast.success(`Đã đăng ký lần nộp, mã yêu cầu ${r.maYeuCau}.`);
      onDone(r);
    } catch (e) {
      notifyError(e, "đăng ký nộp");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      title="Đăng ký nộp tài liệu"
      size="lg"
      footer={
        <>
          <Button onClick={onClose}>Huỷ</Button>
          <Toolbar.Spacer />
          {step === 1 && <Button icon="truoc" onClick={() => setStep(0)}>Quay lại</Button>}
          {step === 0 ? (
            <Button variant="primary" iconRight="sau" disabled={!step0Ok} onClick={() => setStep(1)}>
              Tiếp
            </Button>
          ) : (
            <Button variant="primary" loading={busy} onClick={() => void gui()}>
              Gửi đăng ký
            </Button>
          )}
        </>
      }
    >
      <Stepper className="mb-4" current={step} onSelect={(i) => i < step && setStep(i)} steps={[{ key: "coquan", label: "Cơ quan và phông" }, { key: "mucluc", label: "Mục lục hồ sơ" }]} />
      {step === 0 ? (
        <FormGrid cols={2}>
          <Field label="Tên cơ quan, tổ chức nộp" required className="col-span-2">
            <Input value={orgName} onChange={(e) => setOrgName(e.target.value)} autoFocus />
          </Field>
          <Field label="Mã cơ quan nộp" required>
            <Input value={orgCode} onChange={(e) => setOrgCode(e.target.value)} />
          </Field>
          <Field label="Mã phông" required>
            <Input value={fondCode} onChange={(e) => setFondCode(e.target.value)} />
          </Field>
          <Field label="Tóm tắt khối tài liệu" className="col-span-2">
            <Textarea value={summary} onChange={(e) => setSummary(e.target.value)} rows={2} />
          </Field>
          <Field label="Hạn nộp tài liệu" hint="Đăng ký trước ít nhất 12 tháng (Điều 20.2)">
            <Input type="date" value={hanNop} onChange={(e) => setHanNop(e.target.value)} />
          </Field>
        </FormGrid>
      ) : (
        <div className="space-y-3">
          <Toolbar>
            <span className="text-sm text-fg-muted">{n(items.filter((i) => i.maHoSo.trim() || i.tieuDe.trim()).length)} hồ sơ</span>
            <Toolbar.Spacer />
            <Button size="sm" icon="them" onClick={() => setItems((s) => [...s, { ...EMPTY_ITEM }])}>
              Thêm dòng
            </Button>
            <Button size="sm" icon="nhapTep" onClick={() => setPaste((p) => (p === null ? "" : null))}>
              Dán từ bảng tính
            </Button>
          </Toolbar>
          {paste !== null && (
            <Card padding="sm" tone="info">
              <Field label="Dán các dòng: mã hồ sơ, tiêu đề, số trang (ngăn bằng Tab)">
                <Textarea value={paste} onChange={(e) => setPaste(e.target.value)} rows={5} placeholder={"HS001\tKế hoạch năm 2020\t120"} autoFocus />
              </Field>
              <Toolbar className="mt-2">
                <Button size="sm" variant="primary" onClick={apDung}>
                  Thêm vào mục lục
                </Button>
                <Button size="sm" onClick={() => setPaste(null)}>
                  Đóng
                </Button>
              </Toolbar>
            </Card>
          )}
          <Table.Root maxHeight={320}>
            <Table.Head>
              <tr>
                <Table.HeadCell width={44}>#</Table.HeadCell>
                <Table.HeadCell width={180}>Mã hồ sơ</Table.HeadCell>
                <Table.HeadCell>Tiêu đề</Table.HeadCell>
                <Table.HeadCell width={90}>Số trang</Table.HeadCell>
                <Table.HeadCell width={44} />
              </tr>
            </Table.Head>
            <Table.Body>
              {items.map((it, i) => (
                <Table.Row key={i}>
                  <Table.Cell align="right" className="text-fg-subtle">
                    {i + 1}
                  </Table.Cell>
                  <Table.Cell>
                    <Input size="sm" value={it.maHoSo} onChange={(e) => set(i, { maHoSo: e.target.value })} aria-label={`Mã hồ sơ dòng ${i + 1}`} />
                  </Table.Cell>
                  <Table.Cell>
                    <Input size="sm" value={it.tieuDe} onChange={(e) => set(i, { tieuDe: e.target.value })} aria-label={`Tiêu đề dòng ${i + 1}`} />
                  </Table.Cell>
                  <Table.Cell>
                    <Input size="sm" type="number" min={0} value={it.soTrang ?? ""} onChange={(e) => set(i, { soTrang: e.target.value ? Number(e.target.value) : null })} aria-label={`Số trang dòng ${i + 1}`} />
                  </Table.Cell>
                  <Table.Cell>
                    <IconButton label="Bỏ dòng" icon="xoa" size="sm" disabled={items.length === 1} onClick={() => setItems((s) => s.filter((_, j) => j !== i))} />
                  </Table.Cell>
                </Table.Row>
              ))}
            </Table.Body>
          </Table.Root>
          {touched && errs.length > 0 && (
            <Banner tone="danger" title="Chưa gửi được">
              <ul className={cn("list-disc pl-4")}>
                {errs.map((e) => (
                  <li key={e}>{e}</li>
                ))}
              </ul>
            </Banner>
          )}
        </div>
      )}
    </Dialog>
  );
}
