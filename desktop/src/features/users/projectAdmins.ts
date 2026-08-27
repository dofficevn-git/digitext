// Copyright (c) 2026 DigiText. All rights reserved.

export interface V4r4 {
  currentUserId: string;

  isSuper: boolean;

  adminIds: string[];

  isProjectAdmin: boolean;
}

export function V4r5(r: V4r4): boolean {
  return r.isSuper || r.isProjectAdmin;
}

export function V4r6(r: V4r4, V2hs: string): string | null {
  if (r.isSuper) return null;
  if (!r.isProjectAdmin) return "Bạn không phụ trách dự án này.";
  if (V2hs !== r.currentUserId) {
    return "Chỉ super_admin mới gỡ được người khác — bạn chỉ tự rút mình.";
  }

  if (r.adminIds.length <= 1) {
    return "Bạn là admin duy nhất — nhờ super_admin gán người thay trước khi rút.";
  }
  return null;
}

export function V4r7(r: V4r4, V2ht: string): boolean {
  return V4r6(r, V2ht) === null;
}

export function V4r8<T extends { id: string }>(V2hu: T[], V2hv: string[]): T[] {
  const V2hw = new Set(V2hv);
  return V2hu.filter((a) => !V2hw.has(a.id));
}
