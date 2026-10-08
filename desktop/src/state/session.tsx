// Copyright (c) 2026 DigiText. All rights reserved.

import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import * as authApi from "@/api/auth";
import type { V3zd, V3zc } from "@/api/auth";
import { isTauri, V3xq } from "@/api/ipc";
import { V3zu, V3zq } from "./sessionCore";
import type { V3zp } from "./sessionCore";

export type { V3zc as AuthSession } from "@/api/auth";

interface V3zv {
  status: V3zp;
  session: V3zc | null;

  accounts: V3zd[];
  login: (V3zw: string, V3zx: string, V3zy: string, V3zz: string) => Promise<void>;
  unlock: (V400: string) => Promise<void>;
  switchUser: (V401: string, V402: string, V403: string, V404: string) => Promise<void>;
  lock: () => Promise<void>;
  logout: () => Promise<void>;

  applySession: (s: V3zc) => void;
}

const V405 = createContext<V3zv | null>(null);

export function V406({ children: V16 }: { children: ReactNode }) {
  const [V17, V18] = useState<V3zc | null>(null);
  const [V19, V1a] = useState(true);
  const [V1b, V1c] = useState<V3zd[]>([]);

  const V1d = useCallback(async () => {
    if (!isTauri()) return;
    try {
      V1c(await authApi.V3zl());
    } catch {}
  }, []);

  useEffect(() => {
    if (!isTauri()) {
      // Bản dev trên trình duyệt: ?login=1 mở màn đăng nhập (xem trước giao diện), mặc định vào phiên dev.
      V18(import.meta.env.DEV && new URLSearchParams(window.location.search).has("login") ? null : V3zu());
      V1a(false);
      return;
    }
    let V1e = false;
    (async () => {
      try {
        const s = await authApi.V3zk();
        if (!V1e) V18(s);
      } catch {
        if (!V1e) V18(null);
      } finally {
        if (!V1e) V1a(false);
      }
      void V1d();
    })();
    return () => {
      V1e = true;
    };
  }, [V1d]);

  const V1f = useCallback(
    async (V1g: string, V1h: string, V1i: string, V1j: string) => {
      V18(await authApi.V3zg(V1g, V1h, V1i, V1j));
      void V1d();
    },
    [V1d],
  );

  const V1k = useCallback(async (V1l: string) => {
    V18(await authApi.V3zi(V1l));
  }, []);

  const V1m = useCallback(
    async (V1n: string, V1o: string, V1p: string, V1q: string) => {
      V18(await authApi.V3zh(V1n, V1o, V1p, V1q));
      void V1d();
    },
    [V1d],
  );

  const lock = useCallback(async () => {
    if (isTauri()) await authApi.lock();
    V18((s) => (s ? { ...s, locked: true } : s));
  }, []);

  const V1r = useCallback(async () => {
    if (isTauri()) await authApi.V3zj();
    V18(null);
    void V1d();
  }, [V1d]);

  const V1s = useCallback((s: V3zc) => V18(s), []);

  // Tự làm mới phiên với máy chủ: khi mở, 10 phút một lần, và khi quay lại cửa sổ (cách lần trước ≥ 1 phút).
  // Trước 0.18.2 chỉ nút "Kiểm tra kết nối" ở Cài đặt mới làm việc này → tên / vai trò / dự án sửa trên máy
  // chủ không về máy khác cho tới khi đăng nhập lại, và máy lỡ vào ngoại tuyến thì ngoại tuyến mãi dù có mạng.
  // Mất mạng: im lặng giữ phiên ngoại tuyến. Tài khoản bị thu hồi: Rust đã xoá phiên → đăng xuất.
  const V1u = V17 && !V17.locked ? V17.userId : null;
  useEffect(() => {
    if (!isTauri() || !V1u) return;
    let busy = false;
    let last = 0;
    const lamMoi = async (force: boolean) => {
      if (busy || (!force && Date.now() - last < 60_000)) return;
      busy = true;
      last = Date.now();
      try {
        const s = await authApi.V3zo();
        V18((cur) => (cur && !cur.locked && cur.userId === s.userId ? s : cur));
      } catch (e) {
        if (e instanceof V3xq && e.code === "revoked") V18(null);
      } finally {
        busy = false;
      }
    };
    void lamMoi(true);
    const timer = window.setInterval(() => void lamMoi(true), 10 * 60_000);
    const onFocus = () => void lamMoi(false);
    window.addEventListener("focus", onFocus);
    return () => {
      window.clearInterval(timer);
      window.removeEventListener("focus", onFocus);
    };
  }, [V1u]);

  const V1t = useMemo<V3zv>(
    () => ({
      status: V3zq(V17, V19),
      session: V17,
      accounts: V1b,
      login: V1f,
      unlock: V1k,
      switchUser: V1m,
      lock,
      logout: V1r,
      applySession: V1s,
    }),
    [V17, V19, V1b, V1f, V1k, V1m, lock, V1r, V1s],
  );
  return <V405.Provider value={V1t}>{V16}</V405.Provider>;
}

export function V407(): V3zv {
  const c = useContext(V405);
  if (!c) throw new Error("useSession must be used within <SessionProvider>");
  return c;
}
