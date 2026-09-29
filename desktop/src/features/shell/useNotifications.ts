// Copyright (c) 2026 DigiText. All rights reserved.
// Số đếm cho chuông ở Topbar, badge ở Sidebar và thẻ Tổng quan: thông báo thu nộp / khai thác chưa đọc,
// xung đột chỉnh lý, tệp gom bị từ chối, gói việc còn dở. Một hook dùng chung để không hỏi máy chủ nhiều lần.

import { useCallback, useMemo } from "react";
import { isTauri } from "@/api/ipc";
import { V3z9 } from "@/domain/permissions";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { V4sc, V4se, type V4sa } from "@/api/submissions";
import { clConflicts } from "@/api/chinhly";
import { V43i } from "@/api/lansync";
import { workMyPackages } from "@/api/work";

export interface NotificationCounts {
  unread: number;
  items: V4sa[];
  clConflicts: number;
  collectedRejected: number;
  myWork: number;
  total: number;
  loading: boolean;
  markAllRead: () => Promise<void>;
  reload: () => void;
}

interface Raw {
  unread: number;
  items: V4sa[];
  clConflicts: number;
  collectedRejected: number;
  myWork: number;
}

export function useNotifications(): NotificationCounts {
  const { session } = V460();
  const perms = session?.permissions;
  const canNotif = V3z9(perms, "submission.view") || V3z9(perms, "access.view");
  const canCl = V3z9(perms, "dossier.read");
  const canCollect = V3z9(perms, "upload.execute");
  const canWork = V3z9(perms, "work.execute");
  const enabled = isTauri() && !!session && !session.locked;

  const load = useCallback(async (): Promise<Raw> => {
    const [n, c, g, w] = await Promise.all([
      canNotif ? V4sc().catch(() => null) : null,
      canCl ? clConflicts().catch(() => []) : [],
      canCollect ? V43i(1).catch(() => null) : null,
      canWork ? workMyPackages().catch(() => []) : [],
    ]);
    return {
      unread: n?.unread ?? 0,
      items: n?.items.slice(0, 8) ?? [],
      clConflicts: c.length,
      collectedRejected: g?.rejected ?? 0,
      myWork: w.filter((p) => p.status !== "closed" && p.nPending > 0).length,
    };
  }, [canNotif, canCl, canCollect, canWork]);

  const { data, loading, reload } = V41t(load, { topics: ["submissions", "access", "info", "clSync", "collected", "work"], enabled, pollMs: 60_000 });

  const markAllRead = useCallback(async () => {
    await V4se();
    reload();
  }, [reload]);

  return useMemo(
    () => ({
      unread: data?.unread ?? 0,
      items: data?.items ?? [],
      clConflicts: data?.clConflicts ?? 0,
      collectedRejected: data?.collectedRejected ?? 0,
      myWork: data?.myWork ?? 0,
      total: (data?.unread ?? 0) + (data?.clConflicts ?? 0) + (data?.collectedRejected ?? 0),
      loading,
      markAllRead,
      reload,
    }),
    [data, loading, markAllRead, reload],
  );
}
