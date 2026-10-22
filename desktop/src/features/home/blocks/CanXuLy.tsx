// Copyright (c) 2026 DigiText. All rights reserved.
// "Cần xử lý": gộp mọi việc chờ từ nhiều nguồn, chỉ hiện dòng có quyền, dòng khác 0 lên trước. Tách nguyên từ Tổng quan
// cũ để ba dashboard dùng chung.

import { useCallback } from "react";
import { V3z9 } from "@/domain/permissions";
import { V460 } from "@/state";
import { Icon } from "@/components/Icon";
import { Card } from "@/ui";
import { V4ez, V4f3 } from "@/api/docs";
import { workListPackages, workMyPackages, workRejectedItems } from "@/api/work";
import { V43i } from "@/api/lansync";
import { V4s2 } from "@/api/submissions";
import { V4to } from "@/api/access";
import { V4ut } from "@/api/infoops";
import { V4uc } from "@/api/disposal";
import { useNotifications } from "@/features/shell/useNotifications";
import { DongViec, n, useSo, type Tone } from "./common";

export interface ViecRow {
  label: string;
  value: number;
  tone?: Tone;
  run: () => void;
}

/** Nạp và gộp các việc cần xử lý theo quyền của người đang đăng nhập. */
export function useCanXuLy() {
  const nav = V460();
  const { session, goto, gotoHoSo } = nav;
  const perms = session?.permissions;
  const has = (p: Parameters<typeof V3z9>[1]) => V3z9(perms, p);
  const isAdmin = session?.role === "admin" || session?.role === "manager" || session?.role === "super_admin";
  const projectId = nav.project.projectId;
  const notif = useNotifications();

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

  const coLuuTru = has("submission.review") || has("access.review") || has("disposal.view");
  const work = useSo(loadWork, ["work"], has("work.execute"));
  const review = useSo(loadReview, ["work"], has("work.manage"));
  // Thống kê nặng (quét toàn bộ tài liệu trên máy chủ): chỉ nạp theo đồng hồ 60 s, không theo từng sự kiện OCR.
  const docs = useSo(loadDocs, [], has("document.list"));
  const collect = useSo(loadCollect, ["collected", "uploadBatches"], isAdmin && has("upload.execute"));
  const luuTru = useSo(loadLuuTru, ["submissions", "access", "info", "disposal"], coLuuTru);

  const viec: (ViecRow & { show: boolean })[] = [
    { label: "Tệp bị máy chủ từ chối", value: collect.data?.rejected ?? 0, tone: "err", run: () => goto("upload"), show: isAdmin && has("upload.execute") },
    { label: "Mục bị trả lại cho tôi", value: work.data?.rejected ?? 0, tone: "err", run: () => goto("work"), show: has("work.execute") },
    { label: "Gói việc chờ duyệt", value: review.data?.review ?? 0, run: () => goto("workManage"), show: has("work.manage") },
    { label: "Gói việc quá hạn", value: review.data?.overdue ?? 0, tone: "err", run: () => goto("workManage"), show: has("work.manage") },
    { label: "Tài liệu chờ duyệt bóc tách", value: docs.data?.t.pending ?? 0, run: () => gotoHoSo("ocr"), show: has("document.review") },
    { label: "Trang OCR lỗi", value: docs.data?.failed ?? 0, tone: "err", run: () => gotoHoSo("ocr"), show: has("document.list") },
    { label: "Xung đột chỉnh lý", value: notif.clConflicts, tone: "err", run: () => gotoHoSo("dongbo"), show: has("dossier.read") && !!projectId },
    { label: "Lần nộp chờ xử lý", value: luuTru.data?.sub ?? 0, run: () => goto("submission"), show: has("submission.view") },
    { label: "Yêu cầu khai thác chờ duyệt", value: luuTru.data?.acc ?? 0, run: () => goto("access"), show: has("access.view") },
    { label: "Yêu cầu thông tin chờ duyệt", value: luuTru.data?.info ?? 0, run: () => goto("info"), show: has("access.view") },
    { label: "Tài liệu chờ xét huỷ", value: luuTru.data?.dis ?? 0, run: () => goto("disposal"), show: has("disposal.view") },
  ];
  const rows = viec.filter((v) => v.show).sort((x, y) => Number(y.value > 0) - Number(x.value > 0));
  return { rows, tong: rows.reduce((s, v) => s + v.value, 0), work: work.data, review: review.data, docs: docs.data, collect: collect.data };
}

export default function CanXuLyCard({ className }: { className?: string }) {
  const { rows, tong } = useCanXuLy();
  if (rows.length === 0) return null;
  return (
    <Card
      className={className}
      title={
        <span className="flex items-center gap-2">
          <Icon name="thongBao" size={16} className="text-fg-subtle" />
          Cần xử lý
        </span>
      }
      actions={tong > 0 ? <span className="rounded-full bg-warning-50 px-2 text-xs font-semibold leading-5 text-warning-700 tabular-nums">{n(tong)}</span> : <span className="text-xs text-fg-subtle">Không có gì chờ</span>}
      padding="sm"
    >
      <ul className="divide-y divide-border-subtle">
        {rows.map((v) => (
          <DongViec key={v.label} label={v.label} value={v.value} tone={v.tone} onClick={v.run} />
        ))}
      </ul>
    </Card>
  );
}
