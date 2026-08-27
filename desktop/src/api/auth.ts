// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";
import type { V3xs } from "@/domain/types";
import type { V3yx } from "@/domain/permissions";

export type V3zb = "online" | "offline";

export interface V3zc {
  userId: string;
  username: string;
  displayName: string;
  role: V3xs;
  projectId: string | null;
  projectCode: string | null;
  projectName: string | null;
  mode: V3zb;

  offlineExpiry: string;
  locked: boolean;

  permissions: V3yx[];
}

export interface V3zd {
  userId: string;
  username: string;
  displayName: string;
  role: V3xs;
  projectName: string | null;
  offlineExpiry: string;
  lastOnlineAt: string;
}

export interface V3ze {
  id: string;

  image: string;
}

export function V3zf(): Promise<V3ze> {
  return invoke<V3ze>("auth_captcha_new");
}

export function V3zg(Vo: string, Vp: string, Vq: string, Vr: string): Promise<V3zc> {
  return invoke<V3zc>("auth_login", {
    args: { username: Vo, password: Vp, captchaId: Vq, captchaAnswer: Vr },
  });
}

export function V3zh(Vs: string, Vt: string, Vu: string, Vv: string): Promise<V3zc> {
  return invoke<V3zc>("auth_switch_user", {
    args: { username: Vs, password: Vt, captchaId: Vu, captchaAnswer: Vv },
  });
}

export function V3zi(Vw: string): Promise<V3zc> {
  return invoke<V3zc>("auth_unlock", { args: { password: Vw } });
}

export function lock(): Promise<void> {
  return invoke<void>("auth_lock");
}

export function V3zj(): Promise<void> {
  return invoke<void>("auth_logout");
}

export function V3zk(): Promise<V3zc | null> {
  return invoke<V3zc | null>("auth_current");
}

export function V3zl(): Promise<V3zd[]> {
  return invoke<V3zd[]>("auth_list_accounts");
}

export function V3zm(): Promise<string | null> {
  return invoke<string | null>("auth_get_server_url");
}

export function V3zn(url: string | null): Promise<void> {
  return invoke<void>("auth_set_server_url", { args: { url } });
}

export function V3zo(): Promise<V3zc> {
  return invoke<V3zc>("auth_validate_online");
}
