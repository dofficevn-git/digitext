// Copyright (c) 2026 DigiText. All rights reserved.
// Khai thác lưu trữ số: người dùng gửi yêu cầu (đọc / bản sao / bản sao có ký / gói DIP), người xét duyệt chấm từng
// tài liệu, xác nhận phí, cấp gói DIP; người dùng đọc bản có watermark hoặc tải bản sao trong hạn.
// Căn cứ Điều 35–42 chỉ nằm ở tooltip / trợ giúp.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V4ei, type V4eg } from "@/api/docs";
import { V4to, V4tp, V4tr, V4ts, V4tu, V4tv, V4tw, V4ty, V4tz, type V4tl, type V4tn } from "@/api/access";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { V462 as cn } from "@/lib/utils";
import { Icon } from "@/components/Icon";
import { Badge, Banner, Button, Card, Checkbox, confirm, Dialog, EmptyState, ErrorBanner, Field, FormGrid, Help, IconButton, Input, Menu, notifyError, PageHeader, prompt, Select, Stepper, Table, Textarea, Tip, toast, Toolbar, type Column } from "@/ui";
import { V4u3, V4u5, V4u6 } from "./accessFlow";

const n = (v: number) => v.toLocaleString("vi-VN");
const tien = (v: number | null) => (v == null ? "—" : v === 0 ? "Miễn phí" : `${n(v)} đ`);
const ngay = (iso: string | null) => (iso ? new Date(iso).toLocaleDateString("vi-VN") : "—");

const KIND: Record<V4tl["kind"], { label: string; desc: string; legal: string; icon: "xem" | "tepPdf" | "kySo" | "goi" }> = {
  doc: { label: "Đọc trên hệ thống", desc: "Xem bản có watermark, không tải về.", legal: "Điều 35–37", icon: "xem" },
  ban_sao: { label: "Bản sao", desc: "Tải bản sao không xác thực.", legal: "Điều 40", icon: "tepPdf" },
  ban_sao_ky: { label: "Bản sao có ký số", desc: "Bản sao có xác thực bằng ký số.", legal: "Điều 41", icon: "kySo" },
  dip: { label: "Gói DIP có xác thực", desc: "Gói tin đầy đủ cho người nhận đã định danh.", legal: "Điều 42", icon: "goi" },
};
const STATUS: Record<V4tl["status"], { label: string; tone: "warn" | "ok" | "err" | "gray" }> = {
  submitted: { label: "Chờ xét duyệt", tone: "warn" },
  approved: { label: "Đã duyệt", tone: "ok" },
  refused: { label: "Từ chối", tone: "err" },
  expired: { label: "Hết hạn", tone: "gray" },
};
const PHUONG_THUC = [
  { value: "tai_khoan", label: "Qua tài khoản trong hệ thống" },
  { value: "email", label: "Qua thư điện tử" },
  { value: "giay", label: "Bản giấy tại lưu trữ" },
];
const STEPS = [
  { key: "gui", label: "Gửi yêu cầu" },
  { key: "duyet", label: "Xét duyệt" },
  { key: "phi", label: "Thanh toán" },
  { key: "dung", label: "Sử dụng" },
];
function stepOf(r: V4tl): number {
  if (r.status === "submitted") return 1;
  if (r.status === "refused") return 1;
  if (r.status === "approved") return V4u6(r) ? 3 : 2;
  return 4;
}

export default function V4u7() {
  const { session, goto } = V460();
  const perms = session?.permissions;
  const canReview = V3z9(perms, "access.review");
  const canRequest = V3z9(perms, "access.request");
  const [selId, setSelId] = useState<string | null>(null);
  const [q, setQ] = useState("");
  const [statusF, setStatusF] = useState("");
  const [moi, setMoi] = useState(false);
  const [fees, setFees] = useState(false);
  const [busy, setBusy] = useState(false);
  const [doc, setDoc] = useState<{ src: string; title: string } | null>(null);
  const [selItems, setSelItems] = useState<Set<string>>(new Set());

  const loadList = useCallback(() => V4to(), []);
  const { data: list, error, loading, refreshing, reload } = V41t(loadList, { topics: ["access", "docs"], enabled: isTauri() });
  const loadDetail = useCallback(() => (selId ? V4tp(selId) : Promise.resolve(null)), [selId]);
  const detail = V41t(loadDetail, { topics: ["access", "packages"], enabled: isTauri() && !!selId });
  const d: V4tn | null = detail.data ?? null;

  const shown = useMemo(() => {
    const k = q.trim().toLowerCase();
    const rows = (list ?? []).filter((r) => (!statusF || r.status === statusF) && (!k || r.mucDich.toLowerCase().includes(k) || (r.requester ?? "").toLowerCase().includes(k) || KIND[r.kind].label.toLowerCase().includes(k)));
    // Người xét duyệt: hàng chờ lên đầu
    return canReview ? [...rows].sort((a, b) => (a.status === "submitted" ? 0 : 1) - (b.status === "submitted" ? 0 : 1)) : rows;
  }, [list, q, statusF, canReview]);
  useEffect(() => {
    if (!selId && shown.length > 0) setSelId(shown[0].id);
  }, [shown, selId]);
  useEffect(() => setSelItems(new Set()), [selId]);

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

  const act = async (action: "approve" | "refuse" | "pay" | "dip") => {
    if (!d) return;
    let note: string | undefined;
    if (action === "refuse") {
      const r = await prompt({ title: "Từ chối yêu cầu", label: "Lý do từ chối", type: "textarea", required: true, confirmLabel: "Từ chối", danger: true });
      if (r === null) return;
      note = r.trim();
    }
    if (action === "pay") {
      const ok = await confirm({ title: "Xác nhận đã thu phí?", body: `Số tiền: ${tien(d.phi)}.`, confirmLabel: "Đã thu phí" });
      if (!ok) return;
    }
    await run(action === "approve" ? "duyệt yêu cầu" : action === "refuse" ? "từ chối yêu cầu" : action === "pay" ? "xác nhận thanh toán" : "cấp gói DIP", async () => {
      const r = await V4tu(d.id, action, note);
      if (action === "dip") toast.success(`Gói DIP ${r.maXacThuc ?? ""} đã xếp hàng.`, { action: { label: "Xem Đóng gói", onClick: () => goto("package") } });
      else toast.success(action === "approve" ? "Đã duyệt yêu cầu. Phí được tính tự động." : action === "refuse" ? "Đã từ chối yêu cầu." : "Đã ghi nhận thanh toán.");
    });
  };

  const cham = async (ids: string[], verdict: "duyet" | "tu_choi") => {
    if (!d || ids.length === 0) return;
    let lyDo: string | undefined;
    let dieuKien = false;
    if (verdict === "tu_choi") {
      const r = await prompt({ title: `Không duyệt ${ids.length > 1 ? `${n(ids.length)} tài liệu` : "tài liệu"}`, label: "Lý do", type: "textarea", required: true, confirmLabel: "Không duyệt", danger: true });
      if (r === null) return;
      lyDo = r.trim();
      dieuKien = await confirm({ title: "Tài liệu thuộc diện tiếp cận có điều kiện?", body: "Chọn Đồng ý nếu tài liệu chỉ được khai thác khi có điều kiện kèm theo.", confirmLabel: "Có điều kiện", cancelLabel: "Không" });
    }
    await run("chấm tài liệu", async () => {
      for (const it of ids) await V4ts(d.id, it, verdict, lyDo, dieuKien);
      toast.success(`Đã ${verdict === "duyet" ? "duyệt" : "không duyệt"} ${n(ids.length)} tài liệu.`);
      setSelItems(new Set());
    });
  };

  const doc_ = (itemId: string, title: string) =>
    run("mở bản đọc", async () => {
      if (!d) return;
      const b64 = await V4tv(d.id, itemId);
      setDoc({ src: `data:application/pdf;base64,${b64}`, title });
    });
  const tai = (itemId: string, name: string) =>
    run("tải bản sao", async () => {
      if (!d) return;
      const path = await V4tw(d.id, itemId, name);
      toast.success("Đã lưu bản sao.", { description: path });
    });

  const listCols: Column<V4tl>[] = [
    { key: "kind", header: "Yêu cầu", className: "whitespace-normal", cell: (r) => <div><div className="font-medium text-fg-strong">{KIND[r.kind].label}</div><div className="line-clamp-2 text-xs text-fg-subtle">{r.mucDich}</div></div> },
    { key: "status", header: "Trạng thái", width: 130, cell: (r) => <Badge tone={STATUS[r.status].tone}>{STATUS[r.status].label}</Badge> },
    { key: "phi", header: "Phí", width: 110, align: "right", cell: (r) => <span className={cn((r.phi ?? 0) > 0 && !r.paidAt && "text-warning-700")}>{tien(r.phi)}{(r.phi ?? 0) > 0 ? (r.paidAt ? " · đã thu" : " · chưa thu") : ""}</span> },
  ];

  return (
    <section className="view active">
      <PageHeader
        title="Khai thác"
        description="Yêu cầu đọc, sao hoặc nhận gói tài liệu lưu trữ số; xét duyệt và cấp cho người dùng."
        actions={
          <>
            <IconButton label="Tải lại" icon="lamMoi" variant="secondary" loading={refreshing} onClick={reload} />
            {canReview && <Menu items={[{ label: "Đơn giá phí", icon: "thanhToan", onSelect: () => setFees(true) }]} />}
            {canRequest && (
              <Button variant="primary" icon="them" onClick={() => setMoi(true)}>
                Yêu cầu mới
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
              <Input size="sm" prefix="tim" placeholder="Mục đích, người yêu cầu" value={q} onChange={(e) => setQ(e.target.value)} className="min-w-40 flex-1" aria-label="Tìm yêu cầu" />
              <Select size="sm" value={statusF} onChange={setStatusF} placeholder="Mọi trạng thái" options={Object.entries(STATUS).map(([k, v]) => ({ value: k, label: v.label }))} className="w-40" aria-label="Lọc trạng thái" />
            </Toolbar>
          </div>
          <Table
            columns={listCols}
            rows={shown}
            getRowId={(r) => r.id}
            onRowClick={(r) => setSelId(r.id)}
            rowClassName={(r) => (r.id === selId ? "bg-brand-50" : undefined)}
            state={{ loading: loading && !list, error: list ? undefined : error, onRetry: reload, empty: (list?.length ?? 0) === 0 ? { title: "Chưa có yêu cầu khai thác", description: canRequest ? "Bấm Yêu cầu mới để gửi." : undefined } : { title: "Không có yêu cầu khớp bộ lọc" } }}
            maxHeight="calc(100dvh - 300px)"
          />
        </Card>

        <div className="min-w-0">
          {!selId ? (
            <Card>
              <EmptyState icon="tim" title="Chọn một yêu cầu" />
            </Card>
          ) : !d ? (
            <Card>{detail.loading ? <p className="text-sm text-fg-subtle">Đang tải…</p> : <ErrorBanner error={detail.error ?? "Không tải được chi tiết."} onRetry={detail.reload} />}</Card>
          ) : (
            <div className="space-y-4">
              <Card
                title={
                  <span className="flex flex-wrap items-center gap-2">
                    {KIND[d.kind].label}
                    <Badge tone={STATUS[d.status].tone}>{STATUS[d.status].label}</Badge>
                  </span>
                }
                description={`${d.requester ? `${d.requester} · ` : ""}gửi ${ngay(d.createdAt)}${d.hanSuDung ? ` · hạn sử dụng ${ngay(d.hanSuDung)}` : ""}`}
              >
                <Stepper steps={STEPS} current={stepOf(d)} className="mb-4" />
                {d.status === "refused" && <Banner tone="danger" className="mb-3">Yêu cầu đã bị từ chối{d.reviewNote ? `: ${d.reviewNote}` : "."}</Banner>}
                {d.status === "expired" && <Banner tone="warning" className="mb-3">Đã hết hạn sử dụng. Bản đọc và bản sao trên hệ thống đã được xoá.</Banner>}
                <dl className="mb-3 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
                  <dt className="text-fg-subtle">Mục đích</dt>
                  <dd>{d.mucDich}</dd>
                  {d.phuongThuc && <><dt className="text-fg-subtle">Nhận qua</dt><dd>{PHUONG_THUC.find((p) => p.value === d.phuongThuc)?.label ?? d.phuongThuc}</dd></>}
                  {d.nguoiNhan && <><dt className="text-fg-subtle">Người nhận</dt><dd>{d.nguoiNhan}</dd></>}
                  <dt className="text-fg-subtle">Phí</dt>
                  <dd>
                    {tien(d.phi)}
                    {(d.phi ?? 0) > 0 && <span className={d.paidAt ? "text-success-700" : "text-warning-700"}> · {d.paidAt ? `đã thu ${ngay(d.paidAt)}` : "chưa thu"}</span>}
                  </dd>
                  {d.reviewNote && d.status !== "refused" && <><dt className="text-fg-subtle">Ghi chú</dt><dd>{d.reviewNote}</dd></>}
                </dl>
                {canReview && (
                  <Toolbar>
                    {d.status === "submitted" && (
                      <>
                        <Tip text="Duyệt yêu cầu, phí tính tự động (Điều 36.1.c)">
                          <Button variant="primary" icon="duyet" disabled={busy} onClick={() => void act("approve")}>
                            Duyệt yêu cầu
                          </Button>
                        </Tip>
                        <Button variant="dangerOutline" icon="tuChoi" disabled={busy} onClick={() => void act("refuse")}>
                          Từ chối
                        </Button>
                      </>
                    )}
                    {d.status === "approved" && (d.phi ?? 0) > 0 && !d.paidAt && (
                      <Tip text="Theo Điều 36.4">
                        <Button variant="primary" icon="thanhToan" disabled={busy} onClick={() => void act("pay")}>
                          Xác nhận đã thu phí
                        </Button>
                      </Tip>
                    )}
                    {d.kind === "dip" && d.status === "approved" && (
                      <Tip text={V4u6(d) ? "Tạo gói DIP có xác thực (Điều 42)" : "Cần thu phí trước"}>
                        <span>
                          <Button icon="goi" disabled={busy || !V4u6(d)} onClick={() => void act("dip")}>
                            Cấp gói DIP
                          </Button>
                        </span>
                      </Tip>
                    )}
                  </Toolbar>
                )}
              </Card>

              <Card
                title={`Tài liệu trong yêu cầu (${n(d.items.length)})`}
                padding="none"
                actions={
                  canReview && d.status === "submitted" && selItems.size > 0 && (
                    <>
                      <Button size="sm" variant="primary" icon="duyet" disabled={busy} onClick={() => void cham([...selItems], "duyet")}>
                        Duyệt ({n(selItems.size)})
                      </Button>
                      <Button size="sm" variant="dangerOutline" disabled={busy} onClick={() => void cham([...selItems], "tu_choi")}>
                        Không duyệt
                      </Button>
                    </>
                  )
                }
              >
                <Table
                  columns={[
                    { key: "stt", header: "#", align: "right", width: 50 },
                    { key: "filename", header: "Tệp", className: "whitespace-normal", cell: (i) => i.filename ?? i.documentId },
                    { key: "hoSoRef", header: "Hồ sơ", width: 130, cell: (i) => i.hoSoRef ?? "—" },
                    { key: "pageCount", header: "Trang", align: "right", width: 70, cell: (i) => i.pageCount ?? "—" },
                    {
                      key: "verdict",
                      header: "Kết quả",
                      width: 170,
                      cell: (i) => (
                        <span className="flex flex-wrap items-center gap-1">
                          {i.verdict === "pending" ? <span className="text-fg-subtle">Chưa chấm</span> : i.verdict === "duyet" ? <Badge tone="ok">Duyệt</Badge> : <Tip text={i.lyDo ?? ""}><span><Badge tone="err">Không duyệt</Badge></span></Tip>}
                          {i.dieuKien && <Badge tone="warn">Có điều kiện</Badge>}
                          {i.deleted && <Badge tone="gray">Đã xoá theo hạn</Badge>}
                        </span>
                      ),
                    },
                  ]}
                  rows={d.items}
                  getRowId={(i) => i.id}
                  selection={canReview && d.status === "submitted" ? { selected: selItems, onChange: setSelItems } : undefined}
                  rowActions={(i) => {
                    const usable = d.status === "approved" && i.verdict === "duyet" && !i.deleted && V4u6(d) && V4u5(d);
                    const why = d.status !== "approved" ? "Yêu cầu chưa được duyệt" : i.verdict !== "duyet" ? "Tài liệu không được duyệt" : i.deleted ? "Đã xoá theo hạn" : !V4u6(d) ? "Chưa thu phí" : !V4u5(d) ? "Hết hạn sử dụng" : "";
                    return (
                      <>
                        {canReview && d.status === "submitted" && (
                          <>
                            <IconButton label="Duyệt" icon="duyet" size="sm" disabled={busy} onClick={() => void cham([i.id], "duyet")} />
                            <IconButton label="Không duyệt" icon="tuChoi" size="sm" disabled={busy} onClick={() => void cham([i.id], "tu_choi")} />
                          </>
                        )}
                        {d.kind === "doc" && d.status === "approved" && i.verdict === "duyet" && (
                          <Tip text={usable ? "Đọc bản có watermark, không tải về" : why}>
                            <span>
                              <Button size="sm" icon="xem" disabled={busy || !usable} onClick={() => void doc_(i.id, i.filename ?? i.hoSoRef ?? "Tài liệu")}>
                                Đọc
                              </Button>
                            </span>
                          </Tip>
                        )}
                        {(d.kind === "ban_sao" || d.kind === "ban_sao_ky") && d.status === "approved" && i.verdict === "duyet" && (
                          <Tip text={usable ? "Tải bản sao về máy" : why}>
                            <span>
                              <Button size="sm" icon="taiVe" disabled={busy || !usable} onClick={() => void tai(i.id, `ban-sao-${i.hoSoRef ?? i.stt}.pdf`)}>
                                Tải
                              </Button>
                            </span>
                          </Tip>
                        )}
                      </>
                    );
                  }}
                  state={{ empty: { title: "Không có tài liệu" } }}
                  maxHeight={420}
                />
              </Card>
            </div>
          )}
        </div>
      </div>

      <Help className="mt-4" title="Quy trình khai thác">
        <p>Người dùng gửi yêu cầu kèm mục đích (Điều 35) → người xét duyệt chấm từng tài liệu và duyệt yêu cầu, phí tính theo đơn giá (Điều 36) → thu phí nếu có → người dùng đọc trên hệ thống có watermark (Điều 37), tải bản sao (Điều 40–41) hoặc nhận gói DIP có xác thực (Điều 42) trong hạn sử dụng.</p>
      </Help>

      <Dialog open={!!doc} onOpenChange={(o) => !o && setDoc(null)} title={doc?.title ?? "Bản đọc"} description="Bản đọc có watermark, không tải về." size="full" bodyClassName="p-0">
        {doc && <embed src={doc.src} type="application/pdf" className="h-full w-full" />}
      </Dialog>

      {fees && <FeesDialog onClose={() => setFees(false)} />}
      {moi && (
        <YeuCauMoiDialog
          projectId={session?.projectId ?? null}
          onClose={() => setMoi(false)}
          onDone={(r) => {
            setMoi(false);
            reload();
            setSelId(r.id);
          }}
        />
      )}
    </section>
  );
}

// ---------------- Đơn giá phí
function FeesDialog({ onClose }: { onClose: () => void }) {
  const [doc, setDocFee] = useState("");
  const [trang, setTrang] = useState("");
  const [busy, setBusy] = useState(false);
  const load = useCallback(() => V4ty(), []);
  const { data, error, loading } = V41t(load, { topics: ["access"] });
  useEffect(() => {
    if (data) {
      setDocFee(String(data.phiDocTaiLieu));
      setTrang(String(data.phiSaoTrang));
    }
  }, [data]);
  const save = async () => {
    setBusy(true);
    try {
      await V4tz(Number(doc) || 0, Number(trang) || 0);
      toast.success("Đã lưu đơn giá phí.");
      onClose();
    } catch (e) {
      notifyError(e, "lưu đơn giá");
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog open onOpenChange={(o) => !o && onClose()} title="Đơn giá phí khai thác" size="sm" footer={<><Button onClick={onClose}>Huỷ</Button><Button variant="primary" loading={busy} disabled={!data} onClick={() => void save()}>Lưu</Button></>}>
      {error && !data ? (
        <ErrorBanner error={error} />
      ) : (
        <FormGrid cols={2}>
          <Field label="Phí đọc mỗi tài liệu (đ)" hint="Điều 36.1.c">
            <Input type="number" min={0} value={doc} onChange={(e) => setDocFee(e.target.value)} disabled={loading && !data} />
          </Field>
          <Field label="Phí sao mỗi trang (đ)" hint="Điều 39.1.c">
            <Input type="number" min={0} value={trang} onChange={(e) => setTrang(e.target.value)} disabled={loading && !data} />
          </Field>
        </FormGrid>
      )}
    </Dialog>
  );
}

// ---------------- Yêu cầu mới (3 bước)
function YeuCauMoiDialog({ projectId, onClose, onDone }: { projectId: string | null; onClose: () => void; onDone: (r: V4tn) => void }) {
  const [step, setStep] = useState(0);
  const [kind, setKind] = useState<V4tl["kind"]>("doc");
  const [mucDich, setMucDich] = useState("");
  const [phuongThuc, setPhuongThuc] = useState("tai_khoan");
  const [nguoiNhan, setNguoiNhan] = useState("");
  const [soDinhDanh, setSoDinhDanh] = useState("");
  const [docs, setDocs] = useState<V4eg[] | null>(null);
  const [docErr, setDocErr] = useState<unknown>(null);
  const [q, setQ] = useState("");
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState(false);
  const [touched, setTouched] = useState(false);

  useEffect(() => {
    if (step === 1 && docs === null) V4ei().then(setDocs).catch(setDocErr);
  }, [step, docs]);

  const body = () => ({ projectId, kind, mucDich, phuongThuc, nguoiNhan: nguoiNhan || null, soDinhDanh: soDinhDanh || null, documentIds: [...sel] });
  const errs = V4u3(body());
  const shownDocs = useMemo(() => {
    const k = q.trim().toLowerCase();
    return (docs ?? []).filter((d) => !k || d.filename.toLowerCase().includes(k) || (d.hoSoRef ?? "").toLowerCase().includes(k));
  }, [docs, q]);
  const docCols: Column<V4eg>[] = [
    { key: "filename", header: "Tệp", className: "whitespace-normal" },
    { key: "hoSoRef", header: "Hồ sơ", width: 140, cell: (d) => d.hoSoRef ?? "—" },
    { key: "pageCount", header: "Trang", align: "right", width: 70, cell: (d) => d.pageCount ?? "—" },
  ];

  const gui = async () => {
    setTouched(true);
    if (errs.length > 0 || busy) return;
    setBusy(true);
    try {
      const r = await V4tr(body());
      toast.success("Đã gửi yêu cầu khai thác, chờ xét duyệt.");
      onDone(r);
    } catch (e) {
      notifyError(e, "gửi yêu cầu");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && onClose()}
      title="Yêu cầu khai thác mới"
      size="lg"
      footer={
        <>
          <Button onClick={onClose}>Huỷ</Button>
          <Toolbar.Spacer />
          {step > 0 && <Button icon="truoc" onClick={() => setStep(step - 1)}>Quay lại</Button>}
          {step < 2 ? (
            <Button variant="primary" iconRight="sau" disabled={step === 1 && sel.size === 0} onClick={() => setStep(step + 1)}>
              Tiếp
            </Button>
          ) : (
            <Button variant="primary" loading={busy} onClick={() => void gui()}>
              Gửi yêu cầu
            </Button>
          )}
        </>
      }
    >
      <Stepper className="mb-4" current={step} onSelect={(i) => i < step && setStep(i)} steps={[{ key: "ht", label: "Hình thức" }, { key: "tl", label: "Chọn tài liệu" }, { key: "md", label: "Mục đích và cách nhận" }]} />
      {step === 0 && (
        <div className="grid gap-3 sm:grid-cols-2">
          {(Object.keys(KIND) as V4tl["kind"][]).map((k) => (
            <button key={k} type="button" className={cn("flex items-start gap-3 rounded-md border p-3 text-left transition-colors", kind === k ? "border-brand-500 bg-brand-50" : "border-border hover:border-border-strong")} onClick={() => setKind(k)} aria-pressed={kind === k}>
              <span className={cn("mt-0.5 grid size-8 flex-none place-items-center rounded-full", kind === k ? "bg-brand-500 text-white" : "bg-gray-100 text-fg-subtle")}>
                <Icon name={KIND[k].icon} size={16} />
              </span>
              <span>
                <span className="block font-medium text-fg-strong">{KIND[k].label}</span>
                <span className="block text-sm text-fg-subtle">{KIND[k].desc}</span>
                <span className="block text-xs text-fg-subtle">{KIND[k].legal}</span>
              </span>
            </button>
          ))}
        </div>
      )}
      {step === 1 && (
        <div className="space-y-2">
          <Toolbar>
            <Input size="sm" prefix="tim" placeholder="Tên tệp, mã hồ sơ" value={q} onChange={(e) => setQ(e.target.value)} className="w-72" aria-label="Tìm tài liệu" />
            <Toolbar.Spacer />
            <span className="text-sm text-fg-muted">{n(sel.size)} đã chọn</span>
          </Toolbar>
          <Table columns={docCols} rows={shownDocs} getRowId={(d) => d.id} selection={{ selected: sel, onChange: setSel }} state={{ loading: docs === null && !docErr, error: docErr, empty: { title: "Không có tài liệu" } }} maxHeight={360} dense />
        </div>
      )}
      {step === 2 && (
        <FormGrid cols={2}>
          <Field label="Mục đích sử dụng" required className="col-span-2" hint="Điều 35.4">
            <Textarea value={mucDich} onChange={(e) => setMucDich(e.target.value)} rows={3} autoFocus />
          </Field>
          <Field label="Phương thức nhận" hint="Điều 38.3">
            <Select value={phuongThuc} onChange={setPhuongThuc} options={PHUONG_THUC} />
          </Field>
          {kind === "dip" && (
            <>
              <Field label="Người nhận" required hint="Điều 42.2.c">
                <Input value={nguoiNhan} onChange={(e) => setNguoiNhan(e.target.value)} />
              </Field>
              <Field label="Mã định danh hoặc số giấy tờ" required>
                <Input value={soDinhDanh} onChange={(e) => setSoDinhDanh(e.target.value)} />
              </Field>
            </>
          )}
          <div className="col-span-2 flex items-center gap-2 text-sm text-fg-muted">
            <Checkbox checked label="Đã chọn tài liệu" onChange={() => {}} disabled /> {n(sel.size)} tài liệu · {KIND[kind].label}
          </div>
          {touched && errs.length > 0 && (
            <Banner tone="danger" className="col-span-2" title="Chưa gửi được">
              <ul className="list-disc pl-4">
                {errs.map((e) => (
                  <li key={e}>{e}</li>
                ))}
              </ul>
            </Banner>
          )}
        </FormGrid>
      )}
    </Dialog>
  );
}

