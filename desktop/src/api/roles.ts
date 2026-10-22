// Copyright (c) 2026 DigiText. All rights reserved.
// Vai trò và bộ quyền (máy chủ /roles, migration 0022): 5 vai trò hệ thống sửa được bộ quyền; vai trò tuỳ chỉnh =
// tên + vai trò gốc + bộ quyền. Tạo người dùng chỉ việc chọn vai trò.

import { invoke } from "@/api/ipc";
import type { V3yx } from "@/domain/permissions";
import type { V3xs } from "@/domain/types";

export interface RoleInfo {
  code: string;
  name: string;
  description: string | null;
  /** Vai trò gốc quyết định tầng và phạm vi dự án. */
  baseRole: V3xs;
  baseRoleName: string;
  isSystem: boolean;
  sortOrder: number;
  permissions: V3yx[];
  /** Quyền được phép có với vai trò gốc này (không vượt tầng). */
  allowed: V3yx[];
  userCount: number;
  updatedAt: string;
}

export interface RoleCreateBody {
  code: string;
  name: string;
  description?: string | null;
  baseRole: V3xs;
  permissions: V3yx[];
}
export interface RoleUpdateBody {
  name: string;
  description?: string | null;
  permissions: V3yx[];
}

export const rolesList = () => invoke<RoleInfo[]>("roles_list");
export const roleCreate = (body: RoleCreateBody) => invoke<RoleInfo>("role_create", { body });
export const roleUpdate = (code: string, body: RoleUpdateBody) => invoke<RoleInfo>("role_update", { code, body });
export const roleReset = (code: string) => invoke<RoleInfo>("role_reset", { code });
export const roleDelete = (code: string) => invoke<void>("role_delete", { code });

/** Mã vai trò gợi ý từ tên: "Ký số viên" → "ky_so_vien". */
export function slugRole(name: string): string {
  const s = name
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/đ/g, "d")
    .replace(/Đ/g, "D")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "_")
    .replace(/^_+|_+$/g, "")
    .slice(0, 40);
  return /^[a-z]/.test(s) ? s : s ? `vt_${s}`.slice(0, 40) : "";
}
