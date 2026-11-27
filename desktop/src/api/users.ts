// Copyright (c) 2026 DigiText. All rights reserved.

import { invoke } from "@/api/ipc";
import type { V3xs } from "@/domain/types";
import type { V3yx } from "@/domain/permissions";

export interface V4a8 {
  id: string;
  username: string;
  email: string | null;
  displayName: string;
  role: V3xs;
  roleCode?: string | null;
  projectId: string | null;
  phongBanId?: string | null;
  isActive: boolean;
  createdAt: string;
}

export interface V4a9 {
  id: string;
  code: string;
  name: string;

  adminIds: string[];
  isActive: boolean;
  createdAt: string;
}

export interface V4aa {
  username: string;
  password: string;
  displayName: string;
  email?: string | null;
  role: V3xs;
  /** Vai trò động (mã trong bảng roles); có thì vai trò gốc lấy theo vai trò đó. */
  roleCode?: string | null;
  projectId?: string | null;
  phongBanId?: string | null;
}

export interface V4ab {
  id: string;
  displayName?: string | null;
  email?: string | null;
  /** Đổi vai trò (mã vai trò); người này phải đăng nhập lại. */
  roleCode?: string | null;
  /** Gắn phòng ban (0031); `boPhongBan` gỡ về không phòng ban. */
  phongBanId?: string | null;
  boPhongBan?: boolean;
}

export function V4ac(Vpn?: string | null): Promise<V4a8[]> {
  return invoke<V4a8[]>("users_list", { args: { projectId: Vpn ?? null } });
}

export function V4ad(Vpo: V4aa): Promise<V4a8> {
  return invoke<V4a8>("user_create", { args: Vpo });
}

export function V4ae(Vpp: V4ab): Promise<V4a8> {
  return invoke<V4a8>("user_update", { args: Vpp });
}

export function V4af(Vpq: string, Vpr: boolean): Promise<void> {
  return invoke<void>("user_set_active", { args: { id: Vpq, active: Vpr } });
}

export function V4ag(Vps: string, Vpt: string, Vpu?: string | null): Promise<void> {
  return invoke<void>("user_reset_password", {
    args: { id: Vps, newPassword: Vpt, oldPassword: Vpu ?? null },
  });
}

export interface V4ah {
  effective: V3yx[];

  grants: V3yx[];
  denies: V3yx[];

  roleDefaults: V3yx[];

  grantable: V3yx[];
}

export function V4ai(Vpv: string): Promise<V4ah> {
  return invoke<V4ah>("user_permissions_get", { args: { id: Vpv } });
}

export function V4aj(Vpw: string, Vpx: V3yx[], Vpy: V3yx[]): Promise<V4ah> {
  return invoke<V4ah>("user_permissions_set", {
    args: { id: Vpw, grants: Vpx, denies: Vpy },
  });
}

export function V4ak(): Promise<V4a9[]> {
  return invoke<V4a9[]>("projects_list");
}

export function V4al(Vpz: string, name: string): Promise<V4a9> {
  return invoke<V4a9>("project_create", { args: { code: Vpz, name } });
}

export function V4am(Vq0: string, Vq1: string[]): Promise<V4a9> {
  return invoke<V4a9>("project_set_admins", { args: { projectId: Vq0, adminIds: Vq1 } });
}

export function V4an(Vq2: string, Vq3: string): Promise<V4a9> {
  return invoke<V4a9>("project_add_admin", { args: { projectId: Vq2, adminId: Vq3 } });
}

export function V4ao(Vq4: string, Vq5: string): Promise<V4a9> {
  return invoke<V4a9>("project_remove_admin", { args: { projectId: Vq4, adminId: Vq5 } });
}

// ---- phạm vi phông (0038, 27/11/2026) — qua Tauri bc_call (máy chủ /users/{id}/phong)
export interface PhamViPhongUser {
  projectId: string;
  /** Rỗng = không giới hạn. */
  phongIds: string[];
  tenPhong: [string, string][];
}
export const userPhongGet = (id: string, projectId?: string | null) =>
  invoke<PhamViPhongUser>("bc_call", { method: "GET", path: `/users/${id}/phong${projectId ? `?projectId=${projectId}` : ""}`, body: null });
export const userPhongSet = (id: string, projectId: string, phongIds: string[]) =>
  invoke<PhamViPhongUser>("bc_call", { method: "PUT", path: `/users/${id}/phong`, body: { projectId, phongIds } });
