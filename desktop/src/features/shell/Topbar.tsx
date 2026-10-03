// Copyright (c) 2026 DigiText. All rights reserved.
// Thanh trên cùng: logo · đường dẫn "Nhóm › Mục" · dự án đang làm · tìm nhanh (Ctrl+K) · chuông · tài khoản.

import { useEffect, useState } from "react";
import { V460, useCollects, useUploads } from "@/state";
import { V3z8, V3za } from "@/domain/permissions";
import { Icon } from "@/components/Icon";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { V462 as cn } from "@/lib/utils";
import { IconButton, Kbd, Tip } from "@/ui";
import BrandLogo from "./BrandLogo";
import CommandPalette from "./CommandPalette";
import ProjectSwitch from "./ProjectSwitch";
import { navPath } from "./navTree";
import { useNotifications } from "./useNotifications";

export const TOGGLE_SIDEBAR_EVENT = "digitext:toggle-sidebar";
export const OPEN_TRANSFER_DOCK_EVENT = "digitext:open-transfer-dock";

export default function Topbar() {
  const nav = V460();
  const { session, goto, gotoHoSo, lock, logout } = nav;
  const [palette, setPalette] = useState(false);
  const notif = useNotifications();
  const up = useUploads();
  const col = useCollects();
  const path = navPath(nav);

  useEffect(() => {
    const on = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && !e.altKey && e.key.toLowerCase() === "k") {
        e.preventDefault();
        setPalette((v) => !v);
      } else if ((e.ctrlKey || e.metaKey) && !e.altKey && e.key.toLowerCase() === "b") {
        e.preventDefault();
        window.dispatchEvent(new CustomEvent(TOGGLE_SIDEBAR_EVENT));
      }
    };
    window.addEventListener("keydown", on);
    return () => window.removeEventListener("keydown", on);
  }, []);

  const user = session?.displayName ?? "";
  const initial = user ? user[0].toUpperCase() : "?";
  const transferring = up.running || col.running;

  return (
    <header className="topbar">
      <div className="flex flex-none items-center gap-2">
        <BrandLogo className="size-6" />
        <span className="text-base font-semibold tracking-tight text-fg-strong">DigiText</span>
      </div>

      <nav aria-label="Vị trí" className="hidden min-w-0 items-center gap-1 text-sm text-fg-subtle md:flex">
        {path.map((n, i) => (
          <span key={n.id} className="flex min-w-0 items-center gap-1">
            {i > 0 && <Icon name="moNhanh" size={12} className="flex-none" />}
            <span className={cn("truncate", i === path.length - 1 && "font-medium text-fg")}>{n.label}</span>
          </span>
        ))}
      </nav>

      <div className="ml-auto flex flex-none items-center gap-1.5">
        <ProjectSwitch />

        {transferring && (
          <Tip text="Đang truyền tệp — bấm để xem">
            <button
              type="button"
              className="flex h-7 items-center gap-1.5 rounded border border-info-200 bg-info-50 px-2 text-sm text-info-800"
              onClick={() => window.dispatchEvent(new CustomEvent(OPEN_TRANSFER_DOCK_EVENT))}
            >
              <Icon name="upload" size={14} className="animate-pulse" />
              Đang truyền
            </button>
          </Tip>
        )}

        <Tip text="Tìm nhanh (Ctrl+K)">
          <button
            type="button"
            className="flex h-7 items-center gap-2 rounded border border-border bg-surface px-2 text-sm text-fg-subtle hover:border-border-strong hover:text-fg"
            onClick={() => setPalette(true)}
            aria-label="Tìm nhanh"
          >
            <Icon name="tim" size={14} />
            <span className="hidden lg:inline">Tìm nhanh</span>
            <Kbd keys={["Ctrl", "K"]} className="hidden lg:inline" />
          </button>
        </Tip>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <button type="button" className="relative grid size-8 place-items-center rounded text-fg-subtle hover:bg-surface-active hover:text-fg" aria-label={`Thông báo${notif.total ? ` (${notif.total})` : ""}`}>
              <Icon name="thongBao" size={16} />
              {notif.total > 0 && (
                <span className="absolute -right-0.5 -top-0.5 min-w-4 rounded-full bg-danger-600 px-1 text-center text-[10px] font-semibold leading-4 text-white" aria-hidden>
                  {notif.total > 99 ? "99+" : notif.total}
                </span>
              )}
            </button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-80">
            <DropdownMenuLabel className="flex items-center">
              Thông báo
              {notif.unread > 0 && (
                <button type="button" className="ml-auto text-xs font-normal text-fg-link hover:underline" onClick={() => void notif.markAllRead()}>
                  Đánh dấu đã đọc
                </button>
              )}
            </DropdownMenuLabel>
            <DropdownMenuSeparator />
            {notif.clConflicts > 0 && (
              <DropdownMenuItem onSelect={() => gotoHoSo("dongbo")}>
                <Icon name="canhBao" size={14} className="text-warning-700" />
                {notif.clConflicts} xung đột chỉnh lý cần chọn
              </DropdownMenuItem>
            )}
            {notif.collectedRejected > 0 && (
              <DropdownMenuItem onSelect={() => goto("upload")}>
                <Icon name="loi" size={14} className="text-danger-700" />
                {notif.collectedRejected} tệp gom bị từ chối
              </DropdownMenuItem>
            )}
            {notif.myWork > 0 && (
              <DropdownMenuItem onSelect={() => goto("work")}>
                <Icon name="kiemTra" size={14} className="text-info-700" />
                {notif.myWork} gói việc đang chờ bạn
              </DropdownMenuItem>
            )}
            {notif.items.map((it) => (
              <DropdownMenuItem key={it.id} onSelect={() => goto("submission")} className={cn("items-start", !it.readAt && "font-medium")}>
                <Icon name="thongBao" size={14} className="mt-0.5 flex-none text-fg-subtle" />
                <span className="line-clamp-2 text-sm">{it.message}</span>
              </DropdownMenuItem>
            ))}
            {notif.total === 0 && notif.items.length === 0 && notif.myWork === 0 && <div className="px-2 py-4 text-center text-sm text-fg-subtle">Không có gì mới.</div>}
          </DropdownMenuContent>
        </DropdownMenu>

        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <button type="button" className="flex h-8 items-center gap-2 rounded pl-1 pr-2 text-left hover:bg-surface-active data-[state=open]:bg-surface-active" aria-label="Tài khoản">
              <span className="grid size-6 flex-none place-items-center rounded-full bg-brand-500 text-xs font-semibold text-white">{initial}</span>
              <span className="hidden max-w-40 truncate text-sm font-medium text-fg xl:inline">{user || "Chưa đăng nhập"}</span>
              <Icon name="xuong" size={13} className="text-fg-subtle" />
            </button>
          </DropdownMenuTrigger>
          <DropdownMenuContent align="end" className="w-60">
            <DropdownMenuLabel>
              <span className="block truncate">{user || "Chưa đăng nhập"}</span>
              <span className="block text-xs font-normal text-fg-subtle">
                {session ? V3za(session.role) : ""}
                {session?.mode === "offline" ? " · ngoại tuyến" : ""}
              </span>
            </DropdownMenuLabel>
            <DropdownMenuSeparator />
            {session && V3z8(session.permissions, "settings") && (
              <DropdownMenuItem onSelect={() => goto("settings")}>
                <Icon name="caiDat" size={15} />
                Cài đặt
              </DropdownMenuItem>
            )}
            <DropdownMenuItem onSelect={() => window.dispatchEvent(new CustomEvent(TOGGLE_SIDEBAR_EVENT))}>
              <Icon name="thuGon" size={15} />
              Thu / mở thanh bên
              <Kbd keys={["Ctrl", "B"]} className="ml-auto" />
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            <DropdownMenuItem onSelect={() => void lock()}>
              <Icon name="khoa" size={15} />
              Khoá màn hình
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={() => void logout()}>
              <Icon name="dangXuat" size={15} />
              Đăng xuất
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
        <IconButton label="Thu / mở thanh bên (Ctrl+B)" icon="thuGon" className="hidden" onClick={() => window.dispatchEvent(new CustomEvent(TOGGLE_SIDEBAR_EVENT))} />
      </div>

      <CommandPalette open={palette} onOpenChange={setPalette} />
    </header>
  );
}
