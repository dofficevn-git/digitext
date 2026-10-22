// Copyright (c) 2026 DigiText. All rights reserved.
// "Dự án đang làm" dùng chung toàn ứng dụng. Tài khoản gắn dự án → lấy từ phiên (không đổi được);
// tài khoản quản trị không gắn dự án → chọn ở Topbar (lưu trên máy qua cl_project_set).
// `version` tăng mỗi lần đổi để các màn có bộ lọc dự án riêng đặt lại mặc định.

import { createContext, useCallback, useContext, useEffect, useMemo, useState, type ReactNode } from "react";
import { isTauri } from "@/api/ipc";
import { clProjectGet, clProjectSet, type ClProjectInfo } from "@/api/chinhly";
import { V407 } from "./session";

export interface ProjectState {
  /** Dự án hiệu lực (null = chưa chọn). */
  projectId: string | null;
  /** Mã · tên để hiển thị. */
  projectLabel: string;
  /** true = lấy từ tài khoản, không đổi được. */
  fromAccount: boolean;
  projects: ClProjectInfo["projects"];
  cached: boolean;
  loading: boolean;
  version: number;
  choose: (id: string) => Promise<void>;
  refresh: () => void;
}

const Ctx = createContext<ProjectState | null>(null);

export function ProjectProvider({ children }: { children: ReactNode }) {
  const { session } = V407();
  const active = !!session && !session.locked;
  const [info, setInfo] = useState<ClProjectInfo | null>(null);
  const [loading, setLoading] = useState(false);
  const [version, setVersion] = useState(0);

  const refresh = useCallback(() => {
    if (!(isTauri() || import.meta.env.DEV) || !active) {
      setInfo(null);
      return;
    }
    setLoading(true);
    clProjectGet()
      .then(setInfo)
      .catch(() => setInfo(null))
      .finally(() => setLoading(false));
  }, [active]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const choose = useCallback(
    async (id: string) => {
      await clProjectSet(id);
      setVersion((v) => v + 1);
      refresh();
    },
    [refresh],
  );

  const value = useMemo<ProjectState>(() => {
    const fromAccount = !!session?.projectId || (info?.fromAccount ?? false);
    const projectId = session?.projectId ?? info?.projectId ?? null;
    const p = info?.projects.find((x) => x.id === projectId);
    const projectLabel = session?.projectId
      ? [session.projectCode, session.projectName].filter(Boolean).join(" · ")
      : p
        ? `${p.code} · ${p.name}`
        : "";
    return {
      projectId,
      projectLabel,
      fromAccount,
      projects: info?.projects ?? [],
      cached: info?.cached ?? false,
      loading,
      version,
      choose,
      refresh,
    };
  }, [session, info, loading, version, choose, refresh]);

  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}

export function useProject(): ProjectState {
  const c = useContext(Ctx);
  if (!c) throw new Error("useProject must be used within <ProjectProvider>");
  return c;
}
