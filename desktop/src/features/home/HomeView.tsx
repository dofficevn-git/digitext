// Copyright (c) 2026 DigiText. All rights reserved.
// Trang Tổng quan: thẻ theo vai trò, mỗi thẻ tự nạp / tự lỗi / tự trống, có nút mở màn liên quan.

import { useCallback, type ReactNode } from "react";
import { isTauri } from "@/api/ipc";
import { V3z9 } from "@/domain/permissions";
import { V41t } from "@/live/useLive";
import { V460, useCollects, useUploads } from "@/state";
import { V3zr } from "@/state/sessionCore";
import { Icon, type V41v as IconName } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";
import { Button, Card, ErrorBanner, KpiRow, KpiTile, PageHeader, Skeleton, describeError } from "@/ui";
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

/** Thẻ tự nạp: skeleton lần đầu, lỗi hiện gọn, giữ dữ liệu cũ khi tải lại. */
function DataCard<T>({
  title,
  icon,
  load,
  topics,
  enabled = true,
  open,
  openLabel = "Mở",
  children,
  className,
}: {
  title: string;
  icon: IconName;
  load: () => Promise<T>;
  topics: V3xa[];
  enabled?: boolean;
  open?: () => void;
  openLabel?: string;
  children: (data: T) => ReactNode;
  className?: string;
}) {
  const { data, error, loading, reload } = V41t(load, { topics, enabled: enabled && isTauri(), pollMs: 60_000, errorText: (e) => describeError(e).message });
  if (!enabled) return null;
  return (
    <Card
      className={className}
      title={
        <span className="flex items-center gap-2">
          <Icon name={icon} size={16} className="text-fg-subtle" />
          {title}
        </span>
      }
      actions={
        open && (
          <Button size="sm" variant="ghost" iconRight="sau" onClick={open}>
            {openLabel}
          </Button>
        )
      }
    >
      {loading && data === undefined ? (
        <Skeleton.Rows rows={2} cols={3} />
      ) : data === undefined ? (
        <ErrorBanner error={error ?? "Không nạp được."} onRetry={reload} />
      ) : (
        <>
          {error && <p className="mb-2 text-xs text-warning-700">Không cập nhật được, đang hiện số liệu cũ.</p>}
          {children(data)}
        </>
      )}
    </Card>
  );
}

function Stat({ label, value, tone }: { label: string; value: number | string; tone?: "ok" | "warn" | "err" | "info" }) {
  return <KpiTile label={label} value={value} tone={tone} className="min-w-[6.5rem] flex-1" />;
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
    const last = bk[0]?.createdAt ? new Date(bk[0].createdAt) : null;
    const hours = last ? (Date.now() - last.getTime()) / 3_600_000 : null;
    return { loi: ir.soLoi, daKiem: ir.daKiem3Nam, tong: ir.tongTaiLieu, backupHours: hours };
  }, []);

  const allShortcuts: { label: string; icon: IconName; run: () => void; show: boolean }[] = [
    { label: "Quét tài liệu", icon: "quet", run: () => goto("scan"), show: has("scan.view") },
    { label: "Đưa lên máy chủ", icon: "taiLen", run: () => goto("upload"), show: has("upload.execute") },
    { label: "Lập hồ sơ", icon: "hoSo", run: () => gotoHoSo("laphoso"), show: has("dossier.read") },
    { label: "Ký số", icon: "kySo", run: () => goto("sign"), show: has("sign.view") },
    { label: "Đăng ký thu nộp", icon: "nhapKho", run: () => goto("submission"), show: has("submission.view") },
    { label: "Yêu cầu khai thác", icon: "tim", run: () => goto("access"), show: has("access.view") },
  ];
  const shortcuts = allShortcuts.filter((s) => s.show);

  const syncL = syncLabel(cl);

  return (
    <section className="view active">
      <PageHeader
        title={`${chao}, ${session?.displayName ?? ""}`}
        description={session?.mode === "offline" ? `Đang làm việc ngoại tuyến, còn ${offlineDays} ngày. Dữ liệu sẽ tự đồng bộ khi có mạng.` : nav.project.projectLabel ? `Dự án đang làm: ${nav.project.projectLabel}` : undefined}
        actions={
          shortcuts.length > 0 && (
            <div className="flex flex-wrap gap-2">
              {shortcuts.map((s) => (
                <Button key={s.label} icon={s.icon} onClick={s.run}>
                  {s.label}
                </Button>
              ))}
            </div>
          )
        }
      />

      {session?.mode === "offline" && offlineDays < 7 && (
        <Card tone="warning" className="mb-4" padding="sm">
          <p className="text-sm text-warning-800">Phiên ngoại tuyến sắp hết hạn (còn {offlineDays} ngày). Kết nối mạng và đăng nhập lại để không gián đoạn.</p>
        </Card>
      )}

      <div className="grid gap-4" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(320px, 1fr))" }}>
        {has("work.execute") && (
          <DataCard title="Việc của tôi" icon="kiemTra" load={loadWork} topics={["work"]} open={() => goto("work")}>
            {(d) => (
              <KpiRow>
                <Stat label="Mục còn làm" value={d.pending} tone={d.pending > 0 ? "info" : undefined} />
                <Stat label="Gói đang mở" value={d.packages} />
                <Stat label="Bị trả lại" value={d.rejected} tone={d.rejected > 0 ? "err" : undefined} />
                {d.overdue > 0 && <Stat label="Gói quá hạn" value={d.overdue} tone="warn" />}
              </KpiRow>
            )}
          </DataCard>
        )}

        {has("work.manage") && (
          <DataCard title="Gói việc" icon="thanhVien" load={loadReview} topics={["work"]} open={() => goto("workManage")}>
            {(d) => (
              <KpiRow>
                <Stat label="Chờ duyệt" value={d.review} tone={d.review > 0 ? "warn" : undefined} />
                <Stat label="Mục đã xác nhận" value={d.done} />
                <Stat label="Quá hạn" value={d.overdue} tone={d.overdue > 0 ? "err" : undefined} />
              </KpiRow>
            )}
          </DataCard>
        )}

        {has("document.list") && (
          <DataCard title="Tiến độ số hoá" icon="sohoa" load={loadDocs} topics={["docs", "ocr"]} open={() => gotoHoSo("ocr")} className="xl:col-span-2">
            {(d) => (
              <KpiRow>
                <Stat label="Tệp" value={d.t.docs} />
                <Stat label="Đã OCR" value={d.t.ocrDone} tone="ok" />
                <Stat label="Đã bóc trường" value={d.t.fieldDocs ?? 0} />
                <Stat label="Đã duyệt" value={d.t.approved} tone="ok" />
                <Stat label="Chờ duyệt" value={d.t.pending} tone={d.t.pending > 0 ? "warn" : undefined} />
                {d.queued > 0 && <Stat label="Trang đang OCR" value={d.queued} tone="info" />}
                {d.failed > 0 && <Stat label="Trang lỗi" value={d.failed} tone="err" />}
              </KpiRow>
            )}
          </DataCard>
        )}

        {has("sign.view") && (
          <DataCard title="Ký số" icon="kySo" load={loadSign} topics={["sign"]} open={() => goto("sign")}>
            {(d) => (
              <KpiRow>
                <Stat label="Chờ ký" value={d} tone={d > 0 ? "warn" : undefined} />
              </KpiRow>
            )}
          </DataCard>
        )}

        {has("package.view") && (
          <DataCard title="Gói tin" icon="goi" load={loadPkg} topics={["packages"]} open={() => goto("package")}>
            {(d) => (
              <KpiRow>
                <Stat label="Đang tạo" value={d.queued} tone={d.queued > 0 ? "info" : undefined} />
                <Stat label="Lỗi" value={d.failed} tone={d.failed > 0 ? "err" : undefined} />
                <Stat label="Tổng gói" value={d.total} />
              </KpiRow>
            )}
          </DataCard>
        )}

        {has("dossier.read") && (
          <DataCard title="Chỉnh lý" icon="chinhly" load={loadCl} topics={["chinhLy", "clSync"]} enabled={!!projectId} open={() => gotoHoSo("laphoso")}>
            {(total) => (
              <>
                <KpiRow>
                  <Stat label="Hồ sơ" value={total} />
                  <Stat label="Xung đột" value={notif.clConflicts} tone={notif.clConflicts > 0 ? "err" : undefined} />
                  {cl && cl.pending > 0 && <Stat label="Chờ đẩy" value={cl.pending} tone="warn" />}
                </KpiRow>
                <p className={cn("mt-2 flex items-center gap-1.5 text-sm", syncL.dot === "err" ? "text-danger-700" : syncL.dot === "warn" ? "text-warning-700" : "text-fg-subtle")}>
                  <span className={cn("dot", syncL.dot !== "unknown" && syncL.dot)} />
                  {syncL.text}
                </p>
              </>
            )}
          </DataCard>
        )}

        {!isAdmin && (
          <DataCard title="Máy này" icon="mayTram" load={loadLocal} topics={["docs", "collected"]} open={() => goto("drive")} openLabel="Kho tài liệu">
            {(d) => (
              <>
                <KpiRow>
                  <Stat label="Trang đã quét" value={d.pagesScanned} />
                  <Stat label="Hồ sơ" value={d.hoSoCount} />
                  <Stat label="Tệp PDF" value={d.pdfCount} />
                </KpiRow>
                {d.sharing && <p className="mt-2 text-sm text-fg-subtle">Đang chia sẻ {n(d.shared)} tệp cho máy quản trị gom.</p>}
              </>
            )}
          </DataCard>
        )}

        {isAdmin && has("upload.execute") && (
          <DataCard title="Gom và đẩy lên máy chủ" icon="taiLen" load={loadCollect} topics={["collected", "uploadBatches"]} open={() => goto("upload")}>
            {(d) => (
              <>
                <KpiRow>
                  <Stat label="Chờ đẩy" value={d.pending} tone={d.pending > 0 ? "warn" : undefined} />
                  <Stat label="Đã đẩy" value={d.forwarded} tone="ok" />
                  <Stat label="Bị từ chối" value={d.rejected} tone={d.rejected > 0 ? "err" : undefined} />
                </KpiRow>
                {(up.running || col.running) && <p className="mt-2 text-sm text-info-700">Đang truyền tệp…</p>}
              </>
            )}
          </DataCard>
        )}

        {(has("submission.review") || has("access.review") || has("disposal.view")) && (
          <DataCard title="Lưu trữ số cần xử lý" icon="luuTruSo" load={loadLuuTru} topics={["submissions", "access", "info", "disposal"]}>
            {(d) => (
              <ul className="divide-y divide-border-subtle text-sm">
                {has("submission.view") && <Row label="Lần nộp chờ xử lý" value={d.sub} onClick={() => goto("submission")} />}
                {has("access.view") && <Row label="Yêu cầu khai thác chờ duyệt" value={d.acc} onClick={() => goto("access")} />}
                {has("access.view") && <Row label="Yêu cầu thông tin chờ duyệt" value={d.info} onClick={() => goto("info")} />}
                {has("disposal.view") && <Row label="Tài liệu chờ xét huỷ" value={d.dis} onClick={() => goto("disposal")} />}
              </ul>
            )}
          </DataCard>
        )}

        {has("bk.view") && (
          <DataCard title="Sức khoẻ hệ thống" icon="baoquan" load={loadOps} topics={["ops"]} open={() => goto("bk")}>
            {(d) => (
              <KpiRow>
                <Stat label="Lỗi toàn vẹn" value={d.loi} tone={d.loi > 0 ? "err" : "ok"} />
                <Stat label="Đã kiểm 3 năm" value={`${n(d.daKiem)}/${n(d.tong)}`} />
                <Stat label="Sao lưu gần nhất" value={d.backupHours === null ? "Chưa có" : d.backupHours < 26 ? "Trong ngày" : `${Math.round(d.backupHours / 24)} ngày trước`} tone={d.backupHours === null || d.backupHours >= 26 ? "warn" : "ok"} />
              </KpiRow>
            )}
          </DataCard>
        )}

        {has("stats.view_project") && (
          <Card title="Báo cáo" actions={<Button size="sm" variant="ghost" iconRight="sau" onClick={() => gotoReport("sohoa")}>Mở</Button>}>
            <p className="text-sm text-fg-subtle">Tiến độ dự án, sản lượng nhân viên, chỉnh lý, mượn trả.</p>
          </Card>
        )}
      </div>
    </section>
  );
}

function Row({ label, value, onClick }: { label: string; value: number; onClick: () => void }) {
  return (
    <li>
      <button type="button" className="flex w-full items-center gap-2 py-1.5 text-left hover:text-fg-strong" onClick={onClick}>
        <span className="flex-1">{label}</span>
        <span className={cn("rounded-full px-2 text-xs font-semibold tabular-nums", value > 0 ? "bg-warning-100 text-warning-800" : "bg-gray-100 text-fg-subtle")}>{n(value)}</span>
        <Icon name="sau" size={13} className="text-fg-subtle" />
      </button>
    </li>
  );
}
