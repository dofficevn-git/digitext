// Copyright (c) 2026 DigiText. All rights reserved.
// Trang Tổng quan: ba tầng — tiến độ số hoá (thanh) + việc cần xử lý; hàng thẻ số gọn (mỗi thẻ một con số,
// bấm cả thẻ để mở màn); sức khoẻ hệ thống và lối tắt báo cáo. Mỗi khối tự nạp, giữ số cũ khi lỗi.

import { useCallback, type ReactNode } from "react";
import { isTauri } from "@/api/ipc";
import { V3z9 } from "@/domain/permissions";
import { V41t } from "@/live/useLive";
import { V460, useCollects, useUploads } from "@/state";
import { V3zr } from "@/state/sessionCore";
import { Icon, type V41v as IconName } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";
import { Banner, Button, Card, ErrorBanner, Menu, Skeleton, Tip, describeError } from "@/ui";
import { V4ez, V4f3 } from "@/api/docs";
import { workListPackages, workMyPackages, workRejectedItems } from "@/api/work";
import { V4qt } from "@/api/sign";
import { V4ru } from "@/api/packaging";
import { clHoSoList } from "@/api/chinhly";
import { V4r1 } from "@/api/stats";
import { V431, V43i } from "@/api/lansync";
import { V4s2 } from "@/api/submissions";
import { V4to } from "@/api/access";
import { V4ut } from "@/api/infoops";
import { V4uc } from "@/api/disposal";
import { V4v1, V4v4 } from "@/api/infoops";
import { syncLabel, useClSync } from "@/features/chinhly/common";
import { useNotifications } from "@/features/shell/useNotifications";
import type { V3xa } from "@/live/topics";

const n = (v: number) => v.toLocaleString("vi-VN");
const pct = (a: number, b: number) => (b > 0 ? Math.min(100, Math.round((a / b) * 100)) : 0);
type Tone = "ok" | "warn" | "err" | "info";
const TONE_TEXT: Record<Tone, string> = { ok: "text-success-700", warn: "text-warning-700", err: "text-danger-700", info: "text-info-700" };
const TONE_BG: Record<Tone, string> = { ok: "bg-success-50 text-success-700", warn: "bg-warning-50 text-warning-700", err: "bg-danger-50 text-danger-700", info: "bg-info-50 text-info-700" };

/** Nạp số liệu cho một khối: chỉ chạy khi có quyền, không xoá số cũ khi lỗi. */
function useSo<T>(load: () => Promise<T>, topics: V3xa[], enabled: boolean) {
  return V41t(load, { topics, enabled: enabled && isTauri(), pollMs: 60_000, errorText: (e) => describeError(e).message });
}

/** Thẻ số gọn: một con số, một nhãn, một dòng phụ; cả thẻ là nút mở màn. */
function StatCard({ icon, label, value, sub, tone, onClick, loading, error }: { icon: IconName; label: string; value: number | string | undefined; sub?: ReactNode; tone?: Tone; onClick?: () => void; loading?: boolean; error?: unknown }) {
  const body = (
    <>
      <span className={cn("grid size-10 flex-none place-items-center rounded-md", tone ? TONE_BG[tone] : "bg-gray-100 text-fg-muted")}>
        <Icon name={icon} size={18} />
      </span>
      <span className="min-w-0 flex-1">
        <span className="block truncate text-sm text-fg-subtle">{label}</span>
        {loading && value === undefined ? (
          <Skeleton.Line className="mt-1 h-6 w-16" />
        ) : (
          <span className={cn("block text-2xl font-semibold leading-tight tabular-nums", tone ? TONE_TEXT[tone] : "text-fg-strong")}>{value === undefined ? "—" : typeof value === "number" ? n(value) : value}</span>
        )}
        <span className="block truncate text-xs text-fg-subtle">{error ? <span className="text-warning-700">Không cập nhật được</span> : (sub ?? " ")}</span>
      </span>
      {onClick && <Icon name="sau" size={14} className="flex-none text-fg-subtle" />}
    </>
  );
  const cls = "flex w-full items-center gap-3 rounded-md border border-border-subtle bg-surface p-3 text-left";
  return onClick ? (
    <button type="button" className={cn(cls, "transition-colors hover:border-border-strong hover:bg-surface-hover focus-visible:ring-2 focus-visible:ring-ring/40 outline-none")} onClick={onClick}>
      {body}
    </button>
  ) : (
    <div className={cls}>{body}</div>
  );
}

/** Thẻ có tiêu đề, tự nạp: skeleton lần đầu, lỗi gọn, giữ dữ liệu cũ. */
function DataCard<T>({ title, icon, data, error, loading, reload, open, children, className }: { title: string; icon: IconName; data: T | undefined; error: unknown; loading: boolean; reload: () => void; open?: () => void; children: (d: T) => ReactNode; className?: string }) {
  return (
    <Card
      className={cn("flex flex-col", className)}
      title={
        <span className="flex items-center gap-2">
          <Icon name={icon} size={16} className="text-fg-subtle" />
          {title}
        </span>
      }
      actions={
        open && (
          <Button size="sm" variant="ghost" iconRight="sau" onClick={open}>
            Mở
          </Button>
        )
      }
    >
      {loading && data === undefined ? <Skeleton.Rows rows={3} cols={1} /> : data === undefined ? <ErrorBanner error={error ?? "Không nạp được."} onRetry={reload} /> : children(data)}
      {data !== undefined && error ? <p className="mt-2 text-xs text-warning-700">Không cập nhật được, đang hiện số liệu cũ.</p> : null}
    </Card>
  );
}

function ThanhTienDo({ label, value, total, tone = "info" }: { label: string; value: number; total: number; tone?: Tone }) {
  const p = pct(value, total);
  const fill = tone === "ok" ? "bg-success-600" : tone === "warn" ? "bg-warning-600" : tone === "err" ? "bg-danger-600" : "bg-brand-500";
  return (
    <div>
      <div className="flex items-baseline justify-between gap-2 text-sm">
        <span className="text-fg-muted">{label}</span>
        <span className="tabular-nums">
          <b className="text-fg-strong">{n(value)}</b>
          <span className="text-fg-subtle"> / {n(total)} · {p}%</span>
        </span>
      </div>
      <div className="mt-1 h-1.5 overflow-hidden rounded bg-border-subtle" role="progressbar" aria-label={label} aria-valuemin={0} aria-valuemax={100} aria-valuenow={p}>
        <div className={cn("h-full rounded", fill)} style={{ width: `${p}%` }} />
      </div>
    </div>
  );
}

function DongViec({ label, value, tone = "warn", onClick }: { label: string; value: number; tone?: Tone; onClick: () => void }) {
  return (
    <li>
      <button type="button" className="flex w-full items-center gap-2 rounded px-1 py-1.5 text-left text-sm hover:bg-surface-hover" onClick={onClick}>
        <span className="flex-1 text-fg">{label}</span>
        <span className={cn("min-w-7 rounded-full px-2 text-center text-xs font-semibold tabular-nums leading-5", value > 0 ? TONE_BG[tone] : "bg-gray-100 text-fg-subtle")}>{n(value)}</span>
        <Icon name="sau" size={13} className="text-fg-subtle" />
      </button>
    </li>
  );
}

function DongTrangThai({ tone, label, value, sub }: { tone: Tone; label: string; value: ReactNode; sub?: ReactNode }) {
  const dot = tone === "ok" ? "bg-success-600" : tone === "warn" ? "bg-warning-600" : tone === "err" ? "bg-danger-600" : "bg-info-600";
  return (
    <li className="flex items-center gap-3 py-2">
      <span className={cn("size-2.5 flex-none rounded-full", dot)} aria-hidden />
      <span className="flex-1 text-sm text-fg">{label}</span>
      <span className={cn("text-sm font-semibold tabular-nums", TONE_TEXT[tone])}>{value}</span>
      {sub && <span className="text-xs text-fg-subtle">{sub}</span>}
    </li>
  );
}

export default function HomeView() {
  const nav = V460();
  const { session, goto, gotoHoSo, gotoReport } = nav;
  const perms = session?.permissions;
  const has = (p: Parameters<typeof V3z9>[1]) => V3z9(perms, p);
  const isAdmin = session?.role === "admin" || session?.role === "super_admin";
  const up = useUploads();
  const col = useCollects();
  const notif = useNotifications();
  const cl = useClSync();
  const projectId = nav.project.projectId;

  const gio = new Date().getHours();
  const chao = gio < 11 ? "Chào buổi sáng" : gio < 14 ? "Chào buổi trưa" : gio < 18 ? "Chào buổi chiều" : "Chào buổi tối";
  const offlineDays = session ? V3zr(session.offlineExpiry) : 0;
  const moTa = [session?.mode === "offline" ? `Ngoại tuyến, còn ${offlineDays} ngày` : "Đã kết nối máy chủ", nav.project.projectLabel ? `Dự án: ${nav.project.projectLabel}` : null].filter(Boolean).join(" · ");

  const loadWork = useCallback(async () => {
    const [mine, rejected] = await Promise.all([workMyPackages(), workRejectedItems()]);
    const open = mine.filter((p) => p.status !== "closed");
    return { pending: open.reduce((s, p) => s + p.nPending, 0), packages: open.length, rejected: rejected.length, overdue: open.filter((p) => p.dueAt && new Date(p.dueAt) < new Date() && p.nPending > 0).length };
  }, []);
  const loadReview = useCallback(async () => {
    const all = await workListPackages(projectId ?? undefined);
    return { review: all.filter((p) => p.status === "under_review").length, done: all.reduce((s, p) => s + p.nDone, 0), overdue: all.filter((p) => p.status !== "closed" && p.dueAt && new Date(p.dueAt) < new Date()).length };
  }, [projectId]);
  const loadDocs = useCallback(async () => {
    const [r, p] = await Promise.all([V4f3(), V4ez().catch(() => null)]);
    return { t: r.totals, queued: (p?.pagesOcrQueued ?? 0) + (p?.pagesOcrRunning ?? 0), failed: p?.pagesOcrFailed ?? 0 };
  }, []);
  const loadSign = useCallback(async () => (await V4qt(projectId ?? undefined)).filter((j) => j.status !== "signed" && j.status !== "failed").length, [projectId]);
  const loadPkg = useCallback(async () => {
    const all = await V4ru();
    return { queued: all.filter((p) => p.status === "queued" || p.status === "building").length, failed: all.filter((p) => p.status === "failed").length, total: all.length };
  }, []);
  const loadCl = useCallback(async () => (await clHoSoList({ limit: 1 })).total, []);
  const loadLocal = useCallback(async () => {
    const [s, sh] = await Promise.all([V4r1(), V431().catch(() => null)]);
    return { ...s, sharing: sh?.sharing ?? false, shared: sh?.fileCount ?? 0 };
  }, []);
  const loadCollect = useCallback(() => V43i(1), []);
  const loadLuuTru = useCallback(async () => {
    const [sub, acc, info, dis] = await Promise.all([
      has("submission.view") ? V4s2().catch(() => []) : [],
      has("access.view") ? V4to().catch(() => []) : [],
      has("access.view") ? V4ut().catch(() => []) : [],
      has("disposal.view") ? V4uc().catch(() => []) : [],
    ]);
    return {
      sub: sub.filter((s) => ["registered", "awaiting_reception", "received"].includes(s.status)).length,
      acc: acc.filter((a) => a.status === "submitted").length,
      info: info.filter((a) => a.status === "submitted").length,
      dis: dis.filter((d) => d.status === "xem_xet").length,
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [perms]);
  const loadOps = useCallback(async () => {
    const [ir, bk] = await Promise.all([V4v1(), V4v4().catch(() => [])]);
    const last = bk.filter((b) => b.ketQua === "dat").map((b) => new Date(b.createdAt).getTime()).reduce((a, b) => Math.max(a, b), 0);
    return { loi: ir.soLoi, daKiem: ir.daKiem3Nam, tong: ir.tongTaiLieu, backupHours: last ? (Date.now() - last) / 3_600_000 : null };
  }, []);

  const coLuuTru = has("submission.review") || has("access.review") || has("disposal.view");
  const work = useSo(loadWork, ["work"], has("work.execute"));
  const review = useSo(loadReview, ["work"], has("work.manage"));
  const docs = useSo(loadDocs, ["docs", "ocr"], has("document.list"));
  const sign = useSo(loadSign, ["sign"], has("sign.view"));
  const pkg = useSo(loadPkg, ["packages"], has("package.view"));
  const clTotal = useSo(loadCl, ["chinhLy", "clSync"], has("dossier.read") && !!projectId);
  const local = useSo(loadLocal, ["docs", "collected"], !isAdmin);
  const collect = useSo(loadCollect, ["collected", "uploadBatches"], isAdmin && has("upload.execute"));
  const luuTru = useSo(loadLuuTru, ["submissions", "access", "info", "disposal"], coLuuTru);
  const ops = useSo(loadOps, ["ops"], has("bk.view"));

  const allShortcuts: { label: string; icon: IconName; run: () => void; show: boolean }[] = [
    { label: "Quét tài liệu", icon: "quet", run: () => goto("scan"), show: has("scan.view") },
    { label: "Đưa lên máy chủ", icon: "taiLen", run: () => goto("upload"), show: has("upload.execute") },
    { label: "Lập hồ sơ", icon: "hoSo", run: () => gotoHoSo("laphoso"), show: has("dossier.read") },
    { label: "Ký số", icon: "kySo", run: () => goto("sign"), show: has("sign.view") },
    { label: "Đăng ký thu nộp", icon: "nhapKho", run: () => goto("submission"), show: has("submission.view") },
    { label: "Yêu cầu khai thác", icon: "tim", run: () => goto("access"), show: has("access.view") },
    { label: "Báo cáo", icon: "baoCao", run: () => gotoReport("sohoa"), show: has("stats.view_project") },
  ];
  const shortcuts = allShortcuts.filter((s) => s.show);
  const hien = shortcuts.slice(0, 3);
  const them = shortcuts.slice(3);
  const syncL = syncLabel(cl);
  const syncTone: Tone = syncL.dot === "err" ? "err" : syncL.dot === "warn" ? "warn" : "ok";

  // Việc cần xử lý gộp từ nhiều nguồn; chỉ hiện dòng có quyền.
  const viec: { label: string; value: number; tone?: Tone; run: () => void; show: boolean }[] = [
    { label: "Mục bị trả lại cho tôi", value: work.data?.rejected ?? 0, tone: "err" as Tone, run: () => goto("work"), show: has("work.execute") },
    { label: "Gói việc chờ duyệt", value: review.data?.review ?? 0, run: () => goto("workManage"), show: has("work.manage") },
    { label: "Gói việc quá hạn", value: review.data?.overdue ?? 0, tone: "err" as Tone, run: () => goto("workManage"), show: has("work.manage") },
    { label: "Tài liệu chờ duyệt bóc tách", value: docs.data?.t.pending ?? 0, run: () => gotoHoSo("ocr"), show: has("document.review") },
    { label: "Tệp bị máy chủ từ chối", value: collect.data?.rejected ?? 0, tone: "err" as Tone, run: () => goto("upload"), show: isAdmin && has("upload.execute") },
    { label: "Xung đột chỉnh lý", value: notif.clConflicts, tone: "err" as Tone, run: () => gotoHoSo("dongbo"), show: has("dossier.read") && !!projectId },
    { label: "Lần nộp chờ xử lý", value: luuTru.data?.sub ?? 0, run: () => goto("submission"), show: has("submission.view") },
    { label: "Yêu cầu khai thác chờ duyệt", value: luuTru.data?.acc ?? 0, run: () => goto("access"), show: has("access.view") },
    { label: "Yêu cầu thông tin chờ duyệt", value: luuTru.data?.info ?? 0, run: () => goto("info"), show: has("access.view") },
    { label: "Tài liệu chờ xét huỷ", value: luuTru.data?.dis ?? 0, run: () => goto("disposal"), show: has("disposal.view") },
  ].filter((v) => v.show);
  const tongViec = viec.reduce((s, v) => s + v.value, 0);

  return (
    <section className="view active">
      <header className="mb-5 flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <h1 className="m-0 text-xl font-semibold text-fg-strong">
            {chao}, {session?.displayName ?? ""}
          </h1>
          <p className="mt-0.5 text-sm text-fg-subtle">{moTa}</p>
        </div>
        {shortcuts.length > 0 && (
          <div className="flex flex-wrap items-center gap-2">
            {hien.map((s) => (
              <Button key={s.label} size="sm" icon={s.icon} onClick={s.run}>
                {s.label}
              </Button>
            ))}
            {them.length > 0 && <Menu size="sm" label="Lối tắt khác" items={them.map((s) => ({ label: s.label, icon: s.icon, onSelect: s.run }))} />}
          </div>
        )}
      </header>

      {session?.mode === "offline" && offlineDays < 7 && (
        <Banner tone="warning" className="mb-4">
          Phiên ngoại tuyến sắp hết hạn, còn {offlineDays} ngày. Kết nối mạng và đăng nhập lại để không gián đoạn.
        </Banner>
      )}

      <div className="grid gap-4 xl:grid-cols-12">
        {has("document.list") && (
          <DataCard title="Tiến độ số hoá" icon="sohoa" {...docs} open={() => gotoHoSo("ocr")} className="xl:col-span-8">
            {(d) => (
              <div className="space-y-3">
                <div className="flex flex-wrap items-baseline gap-x-4 gap-y-1">
                  <span className="text-2xl font-semibold tabular-nums text-fg-strong">{n(d.t.docs)}</span>
                  <span className="text-sm text-fg-subtle">tệp trên máy chủ · {n(d.t.pages)} trang</span>
                </div>
                <div className="grid gap-x-6 gap-y-3 sm:grid-cols-2">
                  <ThanhTienDo label="Đã OCR" value={d.t.ocrDone} total={d.t.docs} />
                  <ThanhTienDo label="Đã bóc 16 trường" value={d.t.fieldDocs ?? 0} total={d.t.docs} />
                  <ThanhTienDo label="Đã duyệt" value={d.t.approved} total={d.t.docs} tone="ok" />
                  <ThanhTienDo label="Chờ duyệt" value={d.t.pending} total={d.t.docs} tone="warn" />
                </div>
                <p className="flex flex-wrap gap-x-4 gap-y-1 text-sm text-fg-subtle">
                  {d.queued > 0 && (
                    <span className="text-info-700">
                      <Icon name="dangCho" size={13} /> {n(d.queued)} trang đang OCR
                    </span>
                  )}
                  {d.failed > 0 && (
                    <span className="text-danger-700">
                      <Icon name="canhBao" size={13} /> {n(d.failed)} trang OCR lỗi
                    </span>
                  )}
                  {d.t.rejected > 0 && <span>{n(d.t.rejected)} tài liệu bị từ chối</span>}
                  {d.queued === 0 && d.failed === 0 && d.t.rejected === 0 && <span>Không có việc OCR nào đang chờ.</span>}
                </p>
              </div>
            )}
          </DataCard>
        )}

        {viec.length > 0 && (
          <Card
            className={cn("flex flex-col", has("document.list") ? "xl:col-span-4" : "xl:col-span-12")}
            title={
              <span className="flex items-center gap-2">
                <Icon name="thongBao" size={16} className="text-fg-subtle" />
                Cần xử lý
              </span>
            }
            actions={tongViec > 0 ? <span className="rounded-full bg-warning-50 px-2 text-xs font-semibold leading-5 text-warning-700 tabular-nums">{n(tongViec)}</span> : <span className="text-xs text-fg-subtle">Không có gì chờ</span>}
            padding="sm"
          >
            <ul className="divide-y divide-border-subtle">
              {viec.map((v) => (
                <DongViec key={v.label} label={v.label} value={v.value} tone={v.tone} onClick={v.run} />
              ))}
            </ul>
          </Card>
        )}
      </div>

      <div className="mt-4 grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
        {has("work.execute") && <StatCard icon="kiemTra" label="Việc của tôi" value={work.data?.pending} loading={work.loading} error={work.error} tone={work.data && work.data.pending > 0 ? "info" : undefined} sub={work.data ? `${n(work.data.packages)} gói đang mở${work.data.overdue ? ` · ${n(work.data.overdue)} quá hạn` : ""}` : undefined} onClick={() => goto("work")} />}
        {has("work.manage") && <StatCard icon="thanhVien" label="Mục đã xác nhận" value={review.data?.done} loading={review.loading} error={review.error} sub={review.data ? `${n(review.data.review)} gói chờ duyệt` : undefined} onClick={() => goto("workManage")} />}
        {has("sign.view") && <StatCard icon="kySo" label="Chờ ký số" value={sign.data} loading={sign.loading} error={sign.error} tone={sign.data ? "warn" : undefined} sub="Tài liệu đã có PDF/A" onClick={() => goto("sign")} />}
        {has("package.view") && <StatCard icon="goi" label="Gói tin" value={pkg.data?.total} loading={pkg.loading} error={pkg.error} tone={pkg.data?.failed ? "err" : pkg.data?.queued ? "info" : undefined} sub={pkg.data ? (pkg.data.queued || pkg.data.failed ? `${n(pkg.data.queued)} đang tạo · ${n(pkg.data.failed)} lỗi` : "Không có gói đang tạo") : undefined} onClick={() => goto("package")} />}
        {has("dossier.read") && !!projectId && <StatCard icon="chinhly" label="Hồ sơ chỉnh lý" value={clTotal.data} loading={clTotal.loading} error={clTotal.error} tone={syncTone === "ok" ? undefined : syncTone} sub={syncL.text} onClick={() => gotoHoSo("laphoso")} />}
        {isAdmin && has("upload.execute") && <StatCard icon="taiLen" label="Đã đẩy lên máy chủ" value={collect.data?.forwarded} loading={collect.loading} error={collect.error} tone={collect.data?.pending ? "warn" : undefined} sub={up.running || col.running ? "Đang truyền tệp…" : collect.data ? `${n(collect.data.pending)} chờ đẩy · ${n(collect.data.rejected)} từ chối` : undefined} onClick={() => goto("upload")} />}
        {!isAdmin && <StatCard icon="mayTram" label="Trang đã quét trên máy này" value={local.data?.pagesScanned} loading={local.loading} error={local.error} sub={local.data ? `${n(local.data.hoSoCount)} hồ sơ · ${n(local.data.pdfCount)} PDF${local.data.sharing ? ` · đang chia sẻ ${n(local.data.shared)} tệp` : ""}` : undefined} onClick={() => goto("drive")} />}
      </div>

      {(has("bk.view") || has("stats.view_project")) && (
        <div className="mt-4 grid gap-4 xl:grid-cols-12">
          {has("bk.view") && (
            <DataCard title="Sức khoẻ hệ thống" icon="baoquan" {...ops} open={() => goto("bk")} className="xl:col-span-7">
              {(d) => (
                <ul className="divide-y divide-border-subtle">
                  <DongTrangThai tone={d.loi > 0 ? "err" : "ok"} label="Lỗi toàn vẹn tài liệu" value={n(d.loi)} sub={d.loi > 0 ? "cần lập biên bản kiểm tra" : undefined} />
                  <DongTrangThai tone={pct(d.daKiem, d.tong) >= 33 ? "ok" : "info"} label="Đã kiểm tra trong 3 năm" value={`${pct(d.daKiem, d.tong)}%`} sub={`${n(d.daKiem)} / ${n(d.tong)} tài liệu`} />
                  <DongTrangThai tone={d.backupHours === null || d.backupHours >= 26 ? "err" : "ok"} label="Sao lưu đạt gần nhất" value={d.backupHours === null ? "Chưa có" : d.backupHours < 26 ? "Trong ngày" : `${Math.round(d.backupHours / 24)} ngày trước`} sub={d.backupHours === null || d.backupHours >= 26 ? "cần sao lưu ít nhất mỗi ngày" : undefined} />
                </ul>
              )}
            </DataCard>
          )}
          {has("stats.view_project") && (
            <Card
              className={cn("flex flex-col", has("bk.view") ? "xl:col-span-5" : "xl:col-span-12")}
              padding="sm"
              title={
                <span className="flex items-center gap-2">
                  <Icon name="baoCao" size={16} className="text-fg-subtle" />
                  Báo cáo
                </span>
              }
            >
              <ul className="divide-y divide-border-subtle">
                {(
                  [
                    ["Tiến độ dự án", "sohoa", "So tiến độ công việc với thời gian hợp đồng"],
                    ["Nhân viên", "nhanvien", "Sản lượng, năng suất, KPI"],
                    ["Tổng kết chỉnh lý", "chinhly", "Hộp, hồ sơ, văn bản theo phòng ban"],
                    ["Mượn trả", "muontra", "Phiếu mượn, quá hạn"],
                  ] as const
                ).map(([label, tab, mota]) => (
                  <li key={tab}>
                    <Tip text={mota}>
                      <button type="button" className="flex w-full items-center gap-2 rounded px-1 py-1.5 text-left text-sm hover:bg-surface-hover" onClick={() => gotoReport(tab)}>
                        <span className="flex-1">{label}</span>
                        <Icon name="sau" size={13} className="text-fg-subtle" />
                      </button>
                    </Tip>
                  </li>
                ))}
              </ul>
            </Card>
          )}
        </div>
      )}
    </section>
  );
}
