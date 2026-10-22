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

// Bốn nhóm Chỉnh lý → Số hoá → Xử lý → Lưu trữ số là vòng đời một hồ sơ (luồng chuẩn: chỉnh lý xong mới số hoá)
// nên xếp trên một "trục quy trình" (đường dọc nối các nút tròn), mỗi nhóm ghi "Bước n" dưới tên; nhóm đang làm
// sáng xanh ngọc. Báo cáo / Quản trị nằm dưới đường chia, không thuộc trục. Mục đang mở giữ vạch hổ phách.
const QUY_TRINH = new Set(["chinhly", "sohoa", "xuly", "luutruso"]);

/** Nút tròn mang icon nhóm — đặc (không trong suốt) để che đường trục chạy phía sau. */
function NutNhom({ icon, lit, size = "md" }: { icon: NonNullable<V46s["icon"]>; lit: boolean; size?: "md" | "lg" }) {
  return (
    <span
      className={cn(
        "relative z-10 grid flex-none place-items-center rounded-full ring-1 transition-colors",
        size === "lg" ? "size-9" : "size-7",
        lit ? "bg-brand-500 text-white ring-brand-300/50" : "bg-[var(--side-node)] text-[var(--side-faint)] ring-[var(--side-line)] group-hover:text-[var(--side-strong)] group-hover:ring-[var(--side-line-2)]",
      )}
    >
      <Icon name={icon} size={size === "lg" ? 17 : 15} />
    </span>
  );
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

  const btn = "group flex w-full items-center rounded-lg text-left outline-none transition-colors focus-visible:ring-2 focus-visible:ring-brand-300";
  // Mục đang mở: nền sáng hơn một bậc + vạch hổ phách bên trái (màu "đang làm" của cả hệ thống).
  const tone = (active: boolean) =>
    active
      ? "bg-[var(--side-bg-2)] text-[var(--side-strong)] font-semibold shadow-[inset_2px_0_0_var(--side-active)]"
      : "text-[var(--side-text)] hover:bg-[var(--side-bg-2)]/70 hover:text-[var(--side-strong)]";

  const Badge = ({ n }: { n: number }) =>
    n > 0 ? <span className="ml-auto min-w-5 rounded-full bg-danger-600 px-1.5 text-center text-xs font-semibold leading-5 text-white tabular-nums">{n > 99 ? "99+" : n}</span> : null;

  // Chia danh sách: phần đầu (Tổng quan), trục quy trình (4 nhóm vòng đời hồ sơ), phần cuối (Báo cáo, Quản trị).
  const iDau = visible.findIndex((g) => QUY_TRINH.has(g.id));
  const dau = iDau < 0 ? visible : visible.slice(0, iDau);
  const truc = iDau < 0 ? [] : visible.filter((g) => QUY_TRINH.has(g.id));
  const cuoi = iDau < 0 ? [] : visible.slice(iDau).filter((g) => !QUY_TRINH.has(g.id));

  /** Một nhóm ở chế độ đầy đủ: dòng nhóm (nút tròn + tên + mũi tên) và các mục con thẳng hàng với tên nhóm. */
  const dongNhom = (g: V46s) => {
    const active = isActive(g);
    const isOpen = open.has(g.id);
    const kids = (g.children ?? []).filter(can);
    const soChuaDoc = kids.reduce((s, c) => s + badgeOf(c.badge), 0);
    return (
      <li key={g.id}>
        <button
          type="button"
          data-nav
          data-group={g.leaf ? undefined : g.id}
          className={cn(btn, g.step ? "min-h-12 gap-2.5 px-2 py-1 text-[14px]" : "h-10 gap-2.5 px-2 text-[14px]", g.leaf ? tone(active) : active ? "font-semibold text-[var(--side-strong)]" : "font-medium text-[var(--side-text)] hover:text-[var(--side-strong)]")}
          onClick={g.leaf ? () => gotoLeaf(g.leaf!, nav) : () => toggle(g.id)}
          aria-expanded={g.leaf ? undefined : isOpen}
          aria-current={g.leaf && active ? "page" : undefined}
        >
          {g.icon && <NutNhom icon={g.icon} lit={active} />}
          {g.step ? (
            <span className="flex min-w-0 flex-1 flex-col leading-tight">
              <span className="truncate">{g.label}</span>
              <span className={cn("truncate text-[11.5px] font-normal", active ? "text-brand-200" : "text-[var(--side-faint)]")}>{g.step}</span>
            </span>
          ) : (
            <span className="min-w-0 flex-1 truncate">{g.label}</span>
          )}
          {g.leaf ? (
            <Badge n={badgeOf(g.badge)} />
          ) : (
            <>
              {!isOpen && soChuaDoc > 0 && <span className="size-2 flex-none rounded-full bg-danger-500" aria-label={`${soChuaDoc} việc cần xem`} />}
              <Icon name="moNhanh" size={14} className={cn("flex-none text-[var(--side-faint)] transition-transform duration-200", isOpen && "rotate-90")} />
            </>
          )}
        </button>
        {!g.leaf && isOpen && (
          <ul className="mb-1.5 ml-[38px] flex flex-col gap-px">
            {kids.map((c) => {
              const a = !!c.leaf && leafActive(c.leaf, nav);
              return (
                <li key={c.id}>
                  <button
                    type="button"
                    data-nav
                    data-parent={g.id}
                    className={cn(btn, tone(a), "min-h-8 px-2.5 py-1 text-[13.5px]")}
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
  };

  /** Một nhóm ở chế độ thu gọn (dải icon): nút tròn; nhóm có mục con mở danh sách bên phải. */
  const nutRail = (g: V46s) => {
    const active = isActive(g);
    const kids = (g.children ?? []).filter(can);
    const cnt = kids.reduce((s, c) => s + badgeOf(c.badge), 0) + badgeOf(g.badge);
    const iconBtn = (
      <button
        type="button"
        data-nav
        className="group relative grid size-11 place-items-center rounded-lg outline-none focus-visible:ring-2 focus-visible:ring-brand-300"
        aria-label={g.step ? `${g.label} — ${g.step}` : g.label}
        aria-current={g.leaf && active ? "page" : undefined}
        onClick={g.leaf ? () => gotoLeaf(g.leaf!, nav) : undefined}
      >
        {g.icon && <NutNhom icon={g.icon} lit={active} size="lg" />}
        {cnt > 0 && <span className="absolute right-1 top-1 z-20 size-2 rounded-full bg-danger-500" aria-hidden />}
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
              <DropdownMenuLabel>
                {g.label}
                {g.step && <span className="block text-xs font-normal text-fg-subtle">{g.step}</span>}
              </DropdownMenuLabel>
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
  };

  const ve = rail ? nutRail : dongNhom;
  // Đường trục: chạy qua tâm các nút tròn của nhóm quy trình (tâm = lề nút 8px + nửa nút 14px; dòng nhóm cao 48px
  // nên tâm cách mép 24px; dải icon: giữa cột).
  const duongTruc = rail ? "left-1/2 top-[22px] bottom-[22px]" : "left-[22px] top-6 bottom-6";

  return (
    <aside className={cn("sidebar", rail && "collapsed")}>
      <div className="side-brand">
        <BrandLogo className="size-8 flex-none" />
        {!rail && <span className="text-[17px] font-semibold tracking-tight">DigiText</span>}
      </div>
      {/* eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions -- roving tabindex: nav bắt mũi tên, các mục bên trong là nút thật */}
      <nav ref={navRef} className="nav-scroll" aria-label="Điều hướng chính" onKeyDown={onKeyDown}>
        <ul className={cn("flex flex-col", rail && "items-center")}>{dau.map(ve)}</ul>
        {truc.length > 0 && (
          <div className="relative mt-1">
            <span aria-hidden className={cn("pointer-events-none absolute w-0.5 -translate-x-1/2 rounded-full bg-[var(--side-line-2)]", duongTruc)} />
            <ul className={cn("flex flex-col", rail && "items-center")} aria-label="Quy trình xử lý hồ sơ">
              {truc.map(ve)}
            </ul>
          </div>
        )}
        {cuoi.length > 0 && (
          <>
            <div aria-hidden className={cn("my-2 h-px bg-[var(--side-line)]", rail ? "mx-2" : "mx-2")} />
            <ul className={cn("flex flex-col", rail && "items-center")}>{cuoi.map(ve)}</ul>
          </>
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
