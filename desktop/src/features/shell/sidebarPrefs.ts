// Copyright (c) 2026 DigiText. All rights reserved.

const V46u = "digitext.sidebarCollapsed";

export function V46v(): boolean {
  try {
    return localStorage.getItem(V46u) === "1";
  } catch {
    return false;
  }
}

export function V46w(collapsed: boolean): void {
  try {
    localStorage.setItem(V46u, collapsed ? "1" : "0");
  } catch {}
}
