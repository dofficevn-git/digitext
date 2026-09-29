// Copyright (c) 2026 DigiText. All rights reserved.
// Cung cấp thông tin từ tài liệu lưu trữ: người dùng gửi yêu cầu (danh mục / trích xuất / tổng hợp), người xét duyệt
// duyệt, thu phí, kết xuất và trả kết quả. Cùng khung với màn Khai thác. Căn cứ Điều 44–47 chỉ ở tooltip / trợ giúp.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V4ut, V4uv, V4ux, V4uy, type V4us, type V4uw } from "@/api/infoops";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { V462 as cn } from "@/lib/utils";
import { Icon, type V41v as IconName } from "@/components/Icon";
import { Badge, Banner, Button, Card, confirm, Dialog, EmptyState, ErrorBanner, Field, Help, IconButton, Input, notifyError, PageHeader, prompt, Select, Stepper, Table, Textarea, Tip, toast, Toolbar, type Column } from "@/ui";
import { V4v9 } from "./infoFlow";

const n = (v: number) => v.toLocaleString("vi-VN");
const tien = (v: number | null) => (v == null ? "—" : v === 0 ? "Miễn phí" : `${n(v)} đ`);
const ngay = (iso: string | null) => (iso ? new Date(iso).toLocaleDateString("vi-VN") : "—");

const HINH_THUC: Record<V4us["hinhThuc"], { label: string; desc: string; legal: string; icon: IconName }> = {
  danh_muc: { label: "Danh mục tài liệu", desc: "Danh sách tài liệu theo chủ đề, tối đa 4 trường tin.", legal: "Điều 44.1", icon: "danhSach" },
  trich_xuat: { label: "Trích xuất nguyên văn", desc: "Trích nội dung kèm địa chỉ lưu trữ.", legal: "Điều 44.2", icon: "taiLieu" },
  tong_hop: { label: "Thông tin tổng hợp", desc: "Báo cáo tổng hợp do hệ thống lập.", legal: "Điều 44.3", icon: "baoCao" },
};
const STATUS: Record<V4us["status"], { label: string; tone: "warn" | "ok" | "err" | "info" }> = {
  submitted: { label: "Chờ xét duyệt", tone: "warn" },
  approved: { label: "Đã duyệt", tone: "info" },
  refused: { label: "Từ chối", tone: "err" },
  fulfilled: { label: "Đã trả kết quả", tone: "ok" },
};
const STEPS = [
  { key: "gui", label: "Gửi yêu cầu" },
  { key: "duyet", label: "Xét duyệt" },
  { key: "phi", label: "Thanh toán" },
  { key: "ketxuat", label: "Kết xuất" },
  { key: "tra", label: "Trả kết quả" },
];
function stepOf(r: V4us): number {
  if (r.status === "submitted" || r.status === "refused") return 1;
  if (r.status === "fulfilled") return 5;
  if ((r.phi ?? 0) > 0 && !r.paidAt) return 2;
  return r.coKetQua ? 4 : 3;
}
/** Số ngày làm việc còn lại của hạn xử lý 6 ngày (Điều 46.5). */
function conNgayLamViec(createdAt: string): number {
  const start = new Date(createdAt);
  let d = new Date(start);
  let count = 0;
  while (count < 6) {
    d = new Date(d.getTime() + 86_400_000);
    if (d.getDay() !== 0 && d.getDay() !== 6) count += 1;
  }
  return Math.ceil((d.getTime() - Date.now()) / 86_400_000);
}

export default function V4vc() {
  const { session } = V460();
  const perms = session?.permissions;
  const canReview = V3z9(perms, "access.review");
  const canRequest = V3z9(perms, "access.request");
  const [selId, setSelId] = useState<string | null>(null);
  const [q, setQ] = useState("");
  const [statusF, setStatusF] = useState("");
  const [moi, setMoi] = useState(false);
  const [busy, setBusy] = useState(false);

  const load = useCallback(() => V4ut(), []);
  const { data: list, error, loading, refreshing, reload } = V41t(load, { topics: ["info"], enabled: isTauri() });
  const shown = useMemo(() => {
    const k = q.trim().toLowerCase();
    const rows = (list ?? []).filter((r) => (!statusF || r.status === statusF) && (!k || r.chuDe.toLowerCase().includes(k) || r.mucDich.toLowerCase().includes(k) || (r.requester ?? "").toLowerCase().includes(k)));
    return canReview ? [...rows].sort((a, b) => (a.status === "submitted" ? 0 : 1) - (b.status === "submitted" ? 0 : 1)) : rows;
  }, [list, q, statusF, canReview]);
  useEffect(() => {
    if (!selId && shown.length > 0) setSelId(shown[0].id);
  }, [shown, selId]);
  const d = useMemo(() => (list ?? []).find((r) => r.id === selId) ?? null, [list, selId]);

  const run = async (ctx: string, f: () => Promise<void>) => {
    setBusy(true);
    try {
      await f();
      reload();
    } catch (e) {
      notifyError(e, ctx);
    } finally {
      setBusy(false);
    }
  };
  const act = async (row: V4us, action: V4uw) => {
    let note: string | undefined;
    let bienTap: string | undefined;
    if (action === "refuse") {
      const r = await prompt({ title: "Từ chối yêu cầu", label: "Lý do từ chối", type: "textarea", required: true, confirmLabel: "Từ chối", danger: true });
      if (r === null) return;
      note = r.trim();
    }
    if (action === "generate") {
      const r = await prompt({ title: row.coKetQua ? "Kết xuất lại" : "Kết xuất từ hệ thống", label: "Nội dung rà soát, biên tập", body: "Có thể để trống.", type: "textarea", confirmLabel: "Kết xuất" });
      if (r === null) return;
      bienTap = r.trim() || undefined;
    }
    if (action === "pay") {
      const ok = await confirm({ title: "Xác nhận đã thu phí?", body: `Số tiền: ${tien(row.phi)}.`, confirmLabel: "Đã thu phí" });
      if (!ok) return;
    }
    if (action === "fulfill") {
      const ok = await confirm({ title: "Phê duyệt và trả kết quả cho người yêu cầu?", confirmLabel: "Trả kết quả" });
      if (!ok) return;
    }
    const ctx: Record<V4uw, string> = { approve: "duyệt yêu cầu", refuse: "từ chối yêu cầu", pay: "xác nhận thanh toán", generate: "kết xuất", fulfill: "trả kết quả" };
    const done: Record<V4uw, string> = { approve: "Đã duyệt yêu cầu. Phí tính tự động.", refuse: "Đã từ chối yêu cầu.", pay: "Đã ghi nhận thanh toán.", generate: "Đã kết xuất kết quả.", fulfill: "Đã trả kết quả cho người yêu cầu." };
    await run(ctx[action], async () => {
      await V4ux(row.id, action, { note, bienTap });
      toast.success(done[action]);
    });
  };
  const tai = (row: V4us) =>
    run("tải kết quả", async () => {
      const path = await V4uy(row.id, `cung-cap-thong-tin-${row.chuDe.slice(0, 30)}.pdf`);
      toast.success("Đã lưu kết quả.", { description: path });
    });

  const cols: Column<V4us>[] = [
    { key: "chuDe", header: "Yêu cầu", className: "whitespace-normal", cell: (r) => <div><div className="font-medium text-fg-strong">{r.chuDe}</div><div className="text-xs text-fg-subtle">{HINH_THUC[r.hinhThuc].label}</div></div> },
    { key: "status", header: "Trạng thái", width: 130, cell: (r) => <Badge tone={STATUS[r.status].tone}>{STATUS[r.status].label}</Badge> },
    { key: "phi", header: "Phí", width: 100, align: "right", cell: (r) => <span className={cn((r.phi ?? 0) > 0 && !r.paidAt && "text-warning-700")}>{tien(r.phi)}</span> },
  ];

  return (
    <section className="view active">
      <PageHeader
        title="Cung cấp thông tin"
        description="Yêu cầu danh mục, trích xuất hoặc thông tin tổng hợp từ tài liệu lưu trữ; xét duyệt, kết xuất và trả kết quả."
        actions={
          <>
            <IconButton label="Tải lại" icon="lamMoi" variant="secondary" loading={refreshing} onClick={reload} />
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
              <Input size="sm" prefix="tim" placeholder="Chủ đề, mục đích, người yêu cầu" value={q} onChange={(e) => setQ(e.target.value)} className="min-w-40 flex-1" aria-label="Tìm yêu cầu" />
              <Select size="sm" value={statusF} onChange={setStatusF} placeholder="Mọi trạng thái" options={Object.entries(STATUS).map(([k, v]) => ({ value: k, label: v.label }))} className="w-40" aria-label="Lọc trạng thái" />
            </Toolbar>
          </div>
          <Table
            columns={cols}
            rows={shown}
            getRowId={(r) => r.id}
            onRowClick={(r) => setSelId(r.id)}
            rowClassName={(r) => (r.id === selId ? "bg-brand-50" : undefined)}
            state={{ loading: loading && !list, error: list ? undefined : error, onRetry: reload, empty: (list?.length ?? 0) === 0 ? { title: "Chưa có yêu cầu", description: canRequest ? "Bấm Yêu cầu mới để gửi." : undefined } : { title: "Không có yêu cầu khớp bộ lọc" } }}
            maxHeight="calc(100dvh - 300px)"
          />
        </Card>

        <div className="min-w-0">
          {!d ? (
            <Card>
              <EmptyState icon="thongTin" title="Chọn một yêu cầu" />
            </Card>
          ) : (
            <Card
              title={
                <span className="flex flex-wrap items-center gap-2">
                  {d.chuDe}
                  <Badge tone={STATUS[d.status].tone}>{STATUS[d.status].label}</Badge>
                </span>
              }
              description={`${HINH_THUC[d.hinhThuc].label}${d.requester ? ` · ${d.requester}` : ""} · gửi ${ngay(d.createdAt)}`}
            >
              <Stepper steps={STEPS} current={stepOf(d)} className="mb-4" />
              {d.status === "submitted" && (
                <p className={cn("mb-3 text-sm", conNgayLamViec(d.createdAt) < 0 ? "text-danger-700" : "text-fg-subtle")}>
                  {conNgayLamViec(d.createdAt) >= 0 ? `Hạn xử lý: còn ${conNgayLamViec(d.createdAt)} ngày.` : `Quá hạn xử lý ${-conNgayLamViec(d.createdAt)} ngày.`}
                </p>
              )}
              {d.status === "refused" && <Banner tone="danger" className="mb-3">Yêu cầu đã bị từ chối{d.reviewNote ? `: ${d.reviewNote}` : "."}</Banner>}
              <dl className="mb-3 grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-sm">
                <dt className="text-fg-subtle">Mục đích</dt>
                <dd>{d.mucDich}</dd>
                <dt className="text-fg-subtle">Phí</dt>
                <dd>
                  {tien(d.phi)}
                  {(d.phi ?? 0) > 0 && <span className={d.paidAt ? "text-success-700" : "text-warning-700"}> · {d.paidAt ? `đã thu ${ngay(d.paidAt)}` : "chưa thu"}</span>}
                </dd>
                {d.bienTap && <><dt className="text-fg-subtle">Biên tập</dt><dd>{d.bienTap}</dd></>}
                {d.reviewNote && d.status !== "refused" && <><dt className="text-fg-subtle">Ghi chú</dt><dd>{d.reviewNote}</dd></>}
                {d.fulfilledAt && <><dt className="text-fg-subtle">Trả kết quả</dt><dd>{ngay(d.fulfilledAt)}</dd></>}
              </dl>
              <Toolbar>
                {canReview && d.status === "submitted" && (
                  <>
                    <Tip text="Duyệt yêu cầu, phí tính tự động (Điều 46.1.c)">
                      <Button variant="primary" icon="duyet" disabled={busy} onClick={() => void act(d, "approve")}>
                        Duyệt yêu cầu
                      </Button>
                    </Tip>
                    <Button variant="dangerOutline" icon="tuChoi" disabled={busy} onClick={() => void act(d, "refuse")}>
                      Từ chối
                    </Button>
                  </>
                )}
                {canReview && d.status === "approved" && (d.phi ?? 0) > 0 && !d.paidAt && (
                  <Tip text="Theo Điều 46.4">
                    <Button variant="primary" icon="thanhToan" disabled={busy} onClick={() => void act(d, "pay")}>
                      Xác nhận đã thu phí
                    </Button>
                  </Tip>
                )}
                {canReview && d.status === "approved" && (
                  <Tip text="Hệ thống lập kết quả theo hình thức đã chọn (Điều 47.1)">
                    <Button variant={d.coKetQua ? "secondary" : "primary"} icon={d.coKetQua ? "lamMoi" : "ketXuat"} disabled={busy} onClick={() => void act(d, "generate")}>
                      {d.coKetQua ? "Kết xuất lại" : "Kết xuất"}
                    </Button>
                  </Tip>
                )}
                {canReview && d.status === "approved" && d.coKetQua && (
                  <Tip text="Theo Điều 47.3 và 47.4">
                    <Button variant="primary" icon="guiKetQua" disabled={busy} onClick={() => void act(d, "fulfill")}>
                      Trả kết quả
                    </Button>
                  </Tip>
                )}
                {(d.status === "fulfilled" || (canReview && d.coKetQua)) && (
                  <Button icon="taiVe" disabled={busy} onClick={() => void tai(d)}>
                    Tải kết quả
                  </Button>
                )}
              </Toolbar>
            </Card>
          )}
        </div>
      </div>

      <Help className="mt-4" title="Quy trình cung cấp thông tin">
        <p>Người dùng nêu chủ đề và mục đích (Điều 44–45) → xét duyệt trong 6 ngày làm việc, phí tính theo đơn giá (Điều 46) → thu phí nếu có → hệ thống kết xuất, người xét duyệt rà soát rồi trả kết quả (Điều 47).</p>
      </Help>

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

function YeuCauMoiDialog({ projectId, onClose, onDone }: { projectId: string | null; onClose: () => void; onDone: (r: V4us) => void }) {
  const [hinhThuc, setHinhThuc] = useState<V4us["hinhThuc"]>("danh_muc");
  const [chuDe, setChuDe] = useState("");
  const [mucDich, setMucDich] = useState("");
  const [busy, setBusy] = useState(false);
  const [touched, setTouched] = useState(false);
  const body = () => ({ projectId, hinhThuc, chuDe, mucDich, phuongThuc: "tai_khoan" });
  const errs = V4v9(body());
  const gui = async () => {
    setTouched(true);
    if (errs.length > 0 || busy) return;
    setBusy(true);
    try {
      const r = await V4uv(body());
      toast.success("Đã gửi yêu cầu. Hạn xét duyệt 6 ngày làm việc.");
      onDone(r);
    } catch (e) {
      notifyError(e, "gửi yêu cầu");
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog open onOpenChange={(o) => !o && onClose()} title="Yêu cầu cung cấp thông tin" size="md" footer={<><Button onClick={onClose}>Huỷ</Button><Button variant="primary" loading={busy} onClick={() => void gui()}>Gửi yêu cầu</Button></>}>
      <div className="space-y-4">
        <div className="grid gap-2 sm:grid-cols-3">
          {(Object.keys(HINH_THUC) as V4us["hinhThuc"][]).map((k) => (
            <button key={k} type="button" className={cn("flex flex-col items-start gap-1 rounded-md border p-3 text-left transition-colors", hinhThuc === k ? "border-brand-500 bg-brand-50" : "border-border hover:border-border-strong")} onClick={() => setHinhThuc(k)} aria-pressed={hinhThuc === k}>
              <span className={cn("grid size-7 place-items-center rounded-full", hinhThuc === k ? "bg-brand-500 text-white" : "bg-gray-100 text-fg-subtle")}>
                <Icon name={HINH_THUC[k].icon} size={14} />
              </span>
              <span className="font-medium text-fg-strong">{HINH_THUC[k].label}</span>
              <span className="text-xs text-fg-subtle">{HINH_THUC[k].desc}</span>
            </button>
          ))}
        </div>
        <div className="space-y-3">
          <Field label="Chủ đề cụ thể" required error={touched && !chuDe.trim() ? "Chưa nêu chủ đề." : undefined}>
            <Input value={chuDe} onChange={(e) => setChuDe(e.target.value)} autoFocus />
          </Field>
          <Field label="Mục đích sử dụng" required error={touched && !mucDich.trim() ? "Chưa khai mục đích." : undefined}>
            <Textarea value={mucDich} onChange={(e) => setMucDich(e.target.value)} rows={3} />
          </Field>
        </div>
      </div>
    </Dialog>
  );
}
