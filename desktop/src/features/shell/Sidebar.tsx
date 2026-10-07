// Copyright (c) 2026 DigiText. All rights reserved.
// Thanh bên 2 cấp: nhóm (gập/mở, nhớ trạng thái) → mục. Bàn phím: ↑↓ di chuyển, → mở nhóm, ← đóng / về nhóm,
// Home/End. Mục đang mở có aria-current. Thu gọn (Ctrl+B hoặc cửa sổ hẹp) thành dải icon, bấm mở danh sách mục.

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { V460 } from "@/state";
import { V3z8 } from "@/domain/permissions";
import { V462 as cn } from "@/lib/utils";
import { Icon } from "@/components/Icon";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { Tip } from "@/ui";
import { gotoLeaf, leafActive, NAV_TREE, type NavBadge, type V46s } from "./navTree";
import { getOpenGroups, setOpenGroups, V46v, V46w } from "./sidebarPrefs";
import { TOGGLE_SIDEBAR_EVENT } from "./Topbar";
import { useNotifications } from "./useNotifications";
import { shouldUseRail, useNarrowViewport } from "./useNarrowViewport";
import BrandLogo from "./BrandLogo";

// Mỗi nhóm menu một màu (icon trên ô nền cùng màu) cho dễ nhận, bớt đơn điệu. Từ 0.18 thanh menu nền mực:
// màu icon sáng (300), nền ô trong suốt để hợp nền tối.
const MAU_NHOM: Record<string, string> = {
  home: "bg-sky-400/15 text-sky-300",
  sohoa: "bg-emerald-400/15 text-emerald-300",
  xuly: "bg-violet-400/15 text-violet-300",
  chinhly: "bg-amber-400/15 text-amber-300",
  luutruso: "bg-indigo-400/20 text-indigo-300",
  baocao: "bg-rose-400/15 text-rose-300",
  quantri: "bg-slate-400/20 text-slate-300",
};
function mauNhom(id: string): string {
  return MAU_NHOM[id] ?? "bg-white/10 text-[var(--side-text)]";
}

export default function Sidebar() {
  const nav = V460();
  const { session } = nav;
  const notif = useNotifications();
  const can = useCallback((n: V46s) => !session || V3z8(session.permissions, n.gate), [session]);
  const visible = useMemo(() => NAV_TREE.filter((g) => (g.leaf ? can(g) : (g.children ?? []).some(can))), [can]);
  const isActive = useCallback((n: V46s): boolean => (n.leaf ? leafActive(n.leaf, nav) : (n.children ?? []).some((c) => c.leaf && leafActive(c.leaf, nav))), [nav]);
  const activeGroup = visible.find(isActive)?.id;

  const [open, setOpen] = useState<Set<string>>(() => new Set([...getOpenGroups(), ...(activeGroup ? [activeGroup] : [])]));
  useEffect(() => {
    if (activeGroup) setOpen((s) => (s.has(activeGroup) ? s : new Set([...s, activeGroup])));
  }, [activeGroup]);
  useEffect(() => setOpenGroups([...open]), [open]);

  const [collapsed, setCollapsed] = useState<boolean>(V46v);
  useEffect(() => V46w(collapsed), [collapsed]);
  useEffect(() => {
    const on = () => setCollapsed((c) => !c);
    window.addEventListener(TOGGLE_SIDEBAR_EVENT, on);
    return () => window.removeEventListener(TOGGLE_SIDEBAR_EVENT, on);
  }, []);
  const narrow = useNarrowViewport();
  const rail = shouldUseRail(collapsed, narrow);

  const badgeOf = (b?: NavBadge): number => (b === "clConflicts" ? notif.clConflicts : b === "myWork" ? notif.myWork : b === "notifications" ? notif.unread : 0);

  const toggle = (id: string) =>
    setOpen((s) => {
      const n = new Set(s);
      if (n.has(id)) n.delete(id);
      else n.add(id);
      return n;
    });

  // Bàn phím trong nav: di chuyển giữa các nút [data-nav].
  const navRef = useRef<HTMLElement>(null);
  const onKeyDown = (e: React.KeyboardEvent) => {
    const root = navRef.current;
    if (!root) return;
    const items = Array.from(root.querySelectorAll<HTMLButtonElement>("[data-nav]"));
    const cur = document.activeElement as HTMLButtonElement | null;
    const i = cur ? items.indexOf(cur) : -1;
    const focus = (j: number) => items[Math.max(0, Math.min(items.length - 1, j))]?.focus();
    switch (e.key) {
      case "ArrowDown":
        e.preventDefault();
        focus(i + 1);
        break;
      case "ArrowUp":
        e.preventDefault();
        focus(i - 1);
        break;
      case "Home":
        e.preventDefault();
        focus(0);
        break;
      case "End":
        e.preventDefault();
        focus(items.length - 1);
        break;
      case "ArrowRight": {
        const g = cur?.dataset.group;
        if (g && !open.has(g)) {
          e.preventDefault();
          setOpen((s) => new Set([...s, g]));
        } else if (g) {
          e.preventDefault();
          focus(i + 1);
        }
        break;
      }
      case "ArrowLeft": {
        const g = cur?.dataset.group;
        const parent = cur?.dataset.parent;
        if (g && open.has(g)) {
          e.preventDefault();
          toggle(g);
        } else if (parent) {
          e.preventDefault();
          root.querySelector<HTMLButtonElement>(`[data-group="${parent}"]`)?.focus();
        }
        break;
      }
    }
  };

  const btn = "group flex w-full items-center gap-2 rounded-md text-left outline-none transition-colors focus-visible:ring-2 focus-visible:ring-brand-300";
  // Mục đang mở: nền sáng hơn một bậc + vạch hổ phách bên trái (màu "đang làm" của cả hệ thống).
  const tone = (active: boolean) =>
    active
      ? "bg-[var(--side-bg-2)] text-[var(--side-strong)] font-semibold shadow-[inset_3px_0_0_var(--side-active)]"
      : "text-[var(--side-text)] font-medium hover:bg-[var(--side-bg-2)] hover:text-[var(--side-strong)]";

  const Badge = ({ n }: { n: number }) =>
    n > 0 ? <span className="ml-auto min-w-5 rounded-full bg-danger-600 px-1.5 text-center text-xs font-semibold leading-5 text-white tabular-nums">{n > 99 ? "99+" : n}</span> : null;

  return (
    <aside className={cn("sidebar", rail && "collapsed")}>
      <div className="side-brand">
        <BrandLogo className="size-7 flex-none" />
        {!rail && <span className="text-md font-semibold tracking-wide">DIGITEXT</span>}
      </div>
      {/* eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions -- roving tabindex: nav bắt mũi tên, các mục bên trong là nút thật */}
      <nav ref={navRef} className="nav-scroll" aria-label="Điều hướng chính" onKeyDown={onKeyDown}>
        {rail ? (
          <ul className="flex flex-col items-center gap-1">
            {visible.map((g) => {
              const active = isActive(g);
              const kids = (g.children ?? []).filter(can);
              const cnt = kids.reduce((s, c) => s + badgeOf(c.badge), 0) + badgeOf(g.badge);
              const iconBtn = (
                <button
                  type="button"
                  data-nav
                  className={cn("relative grid size-9 place-items-center rounded-md transition-colors outline-none focus-visible:ring-2 focus-visible:ring-brand-300", active ? "bg-[var(--side-bg-2)] shadow-[inset_3px_0_0_var(--side-active)]" : "hover:bg-[var(--side-bg-2)]")}
                  aria-label={g.label}
                  aria-current={g.leaf && active ? "page" : undefined}
                  onClick={g.leaf ? () => gotoLeaf(g.leaf!, nav) : undefined}
                >
                  {g.icon && (
                    <span className={cn("grid size-7 place-items-center rounded-md", mauNhom(g.id))}>
                      <Icon name={g.icon} size={17} />
                    </span>
                  )}
                  {cnt > 0 && <span className="absolute right-0.5 top-0.5 size-2 rounded-full bg-danger-600" aria-hidden />}
                </button>
              );
              return (
                <li key={g.id}>
                  {g.leaf ? (
                    <Tip text={g.label} side="right">
                      {iconBtn}
                    </Tip>
                  ) : (
                    <DropdownMenu>
                      <DropdownMenuTrigger asChild>{iconBtn}</DropdownMenuTrigger>
                      <DropdownMenuContent side="right" align="start" className="w-56">
                        <DropdownMenuLabel>{g.label}</DropdownMenuLabel>
                        <DropdownMenuSeparator />
                        {kids.map((c) => (
                          <DropdownMenuItem key={c.id} onSelect={() => c.leaf && gotoLeaf(c.leaf, nav)} className={cn(c.leaf && leafActive(c.leaf, nav) && "font-semibold text-brand-700")}>
                            {c.label}
                            <Badge n={badgeOf(c.badge)} />
                          </DropdownMenuItem>
                        ))}
                      </DropdownMenuContent>
                    </DropdownMenu>
                  )}
                </li>
              );
            })}
          </ul>
        ) : (
          <ul className="flex flex-col gap-0.5">
            {visible.map((g) => {
              const active = isActive(g);
              const isOpen = open.has(g.id);
              const kids = (g.children ?? []).filter(can);
              return (
                <li key={g.id}>
                  <button
                    type="button"
                    data-nav
                    data-group={g.leaf ? undefined : g.id}
                    className={cn(btn, tone(active && !!g.leaf), "h-8 px-2 text-base", !g.leaf && active && "text-[var(--side-strong)]")}
                    onClick={g.leaf ? () => gotoLeaf(g.leaf!, nav) : () => toggle(g.id)}
                    aria-expanded={g.leaf ? undefined : isOpen}
                    aria-current={g.leaf && active ? "page" : undefined}
                  >
                    {g.icon && (
                      <span className={cn("grid size-6 flex-none place-items-center rounded-md", mauNhom(g.id))}>
                        <Icon name={g.icon} size={15} />
                      </span>
                    )}
                    <span className="min-w-0 flex-1 truncate">{g.label}</span>
                    {g.leaf ? <Badge n={badgeOf(g.badge)} /> : <Icon name="moNhanh" size={14} className={cn("flex-none text-[var(--side-faint)] transition-transform", isOpen && "rotate-90")} />}
                  </button>
                  {!g.leaf && isOpen && (
                    <ul className="my-0.5 ml-[19px] border-l border-[var(--side-line)] pl-2">
                      {kids.map((c) => {
                        const a = !!c.leaf && leafActive(c.leaf, nav);
                        return (
                          <li key={c.id}>
                            <button
                              type="button"
                              data-nav
                              data-parent={g.id}
                              className={cn(btn, tone(a), !a && "font-normal", "min-h-7 px-2 py-1 text-sm")}
                              onClick={() => c.leaf && gotoLeaf(c.leaf, nav)}
                              aria-current={a ? "page" : undefined}
                            >
                              <span className="min-w-0 flex-1 truncate">{c.label}</span>
                              <Badge n={badgeOf(c.badge)} />
                            </button>
                          </li>
                        );
                      })}
                    </ul>
                  )}
                </li>
              );
            })}
          </ul>
        )}
      </nav>

      <div className="side-foot">
        {!narrow && (
          <Tip text={collapsed ? "Mở rộng thanh bên (Ctrl+B)" : "Thu gọn thanh bên (Ctrl+B)"} side="right">
            <button
              type="button"
              className="grid size-8 place-items-center rounded-md text-[var(--side-faint)] transition-colors hover:bg-[var(--side-bg-2)] hover:text-[var(--side-strong)]"
              onClick={() => setCollapsed((c) => !c)}
              aria-label={collapsed ? "Mở rộng thanh bên" : "Thu gọn thanh bên"}
            >
              <Icon name={collapsed ? "moRong" : "thuGon"} size={16} />
            </button>
          </Tip>
        )}
      </div>
    </aside>
  );
}
