// Copyright (c) 2026 DigiText. All rights reserved.
// Ghi nhớ thanh bên trên máy: thu gọn hay không, nhóm nào đang mở.

const KEY_COLLAPSED = "digitext.sidebarCollapsed";
const KEY_OPEN = "digitext.sidebarOpen";

export function V46v(): boolean {
  try {
    return localStorage.getItem(KEY_COLLAPSED) === "1";
  } catch {
    return false;
  }
}

export function V46w(collapsed: boolean): void {
  try {
    localStorage.setItem(KEY_COLLAPSED, collapsed ? "1" : "0");
  } catch {}
}

export function getOpenGroups(): string[] {
  try {
    const v = JSON.parse(localStorage.getItem(KEY_OPEN) ?? "[]");
    return Array.isArray(v) ? v.filter((x) => typeof x === "string") : [];
  } catch {
    return [];
  }
}

export function setOpenGroups(ids: string[]): void {
  try {
    localStorage.setItem(KEY_OPEN, JSON.stringify(ids));
  } catch {}
}
