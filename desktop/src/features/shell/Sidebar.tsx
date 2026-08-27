// Copyright (c) 2026 DigiText. All rights reserved.

import { useEffect, useState } from "react";
import { V460 } from "@/state";
import type { V3xt } from "@/domain/types";
import { V3z8, V3za } from "@/domain/permissions";
import { V462 } from "@/lib/utils";
import { Icon } from "@/components/Icon";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { NAV_TREE, type V46r, type V46s } from "./navTree";
import { V46v, V46w } from "./sidebarPrefs";
import { shouldUseRail, useNarrowViewport } from "./useNarrowViewport";

export default function Sidebar() {
  const {
    view,
    goto,
    hoSoPane: Vdn,
    gotoHoSo: Vdo,
    catalogTab: Vdp,
    gotoCatalog: Vdq,
    khoGiaTab: Vdr,
    gotoKhoGia: Vds,
    reportTab: Vdt,
    gotoReport: Vdu,
    session: Vdv,
    lock,
    logout: Vdw,
  } = V460();

  const Vdx = (v: V3xt) => !Vdv || V3z8(Vdv.permissions, v);

  const Vdy = (Vdz: V46r): boolean => {
    switch (Vdz.kind) {
      case "view":
        return view === Vdz.view;
      case "hoso":
        return view === "dossier" && Vdn === Vdz.pane;
      case "catalog":
        return view === "catalog" && Vdp === Vdz.tab;
      case "khogia":
        return view === "khogia" && Vdr === Vdz.tab;
      case "report":
        return view === "report" && Vdt === Vdz.tab;
    }
  };
  const Ve0 = (Ve1: V46r) => {
    switch (Ve1.kind) {
      case "view":
        return goto(Ve1.view);
      case "hoso":
        return Vdo(Ve1.pane);
      case "catalog":
        return Vdq(Ve1.tab);
      case "khogia":
        return Vds(Ve1.tab);
      case "report":
        return Vdu(Ve1.tab);
    }
  };
  const Ve2 = (n: V46s): boolean => (n.leaf ? Vdx(n.gate) : (n.children ?? []).some(Ve2));
  const Ve3 = (n: V46s): boolean => (n.leaf ? Vdy(n.leaf) : (n.children ?? []).some(Ve3));

  const Ve4 = (): string[] => {
    const Ve5: string[] = [];
    const Ve6 = (Ve7: V46s[]) => {
      for (const n of Ve7) {
        if (n.children && Ve3(n)) {
          Ve5.push(n.id);
          Ve6(n.children);
        }
      }
    };
    Ve6(NAV_TREE);
    return Ve5;
  };

  const [open, Ve8] = useState<Set<string>>(() => new Set(Ve4()));

  useEffect(() => {
    Ve8((s) => new Set([...s, ...Ve4()]));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [view, Vdn, Vdp, Vdr, Vdt]);
  const Ve9 = (Vea: string) =>
    Ve8((s) => {
      const n = new Set(s);
      n.has(Vea) ? n.delete(Vea) : n.add(Vea);
      return n;
    });

  const [collapsed, Veb] = useState<boolean>(V46v);
  useEffect(() => V46w(collapsed), [collapsed]);

  const Vec = useNarrowViewport();
  const rail = shouldUseRail(collapsed, Vec);

  const Ved = (n: V46s) => {
    Veb(false);
    if (n.leaf) Ve0(n.leaf);
    else Ve8((s) => new Set([...s, n.id]));
  };

  const Vee =
    "group flex w-full items-center gap-2 rounded text-left transition-colors " +
    "focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-ring";
  const Vef = (Veg: boolean) =>
    Veg
      ? "bg-brand-50 text-brand-700 font-semibold"
      : "text-fg-muted font-medium hover:bg-gray-100 hover:text-fg";

  const Veh = (n: V46s, Vei: number) => {
    if (!Ve2(n)) return null;
    const Vej = Ve3(n);
    const Vek = open.has(n.id);
    const Vel = n.children ? n.children.map((c) => Veh(c, Vei + 1)) : null;

    if (Vei === 0) {
      const Vem = () => (n.leaf ? Ve0(n.leaf) : Ve9(n.id));
      return (
        <div key={n.id}>
          <button
            type="button"
            className={V462(Vee, Vef(Vej), "h-8 px-2 text-base")}
            onClick={Vem}
            aria-expanded={n.leaf ? undefined : Vek}
          >
            {n.icon && (
              <Icon
                name={n.icon}
                className={V462(
                  "flex-none transition-colors",
                  Vej ? "text-brand-600" : "text-fg-subtle group-hover:text-fg-muted",
                )}
              />
            )}
            <span className="min-w-0 flex-1 truncate">{n.label}</span>
            {!n.leaf && (
              <Icon
                name="moNhanh"
                size={14}
                className={V462(
                  "flex-none text-fg-subtle transition-transform",
                  Vek && "rotate-90",
                )}
              />
            )}
          </button>
          {!n.leaf && Vek && (
            <div className="my-0.5 ml-4 border-l border-border-subtle pl-2">{Vel}</div>
          )}
        </div>
      );
    }

    const Ven = () => (n.leaf ? Ve0(n.leaf) : Ve9(n.id));
    return (
      <div key={n.id}>
        <button
          type="button"
          className={V462(Vee, Vef(Vej), "min-h-7 px-2 py-1 text-sm")}
          onClick={Ven}
          aria-expanded={n.leaf ? undefined : Vek}
        >
          <span className="min-w-0 flex-1 truncate">{n.label}</span>
          {!n.leaf && (
            <Icon
              name="moNhanh"
              size={13}
              className={V462("flex-none text-fg-subtle transition-transform", Vek && "rotate-90")}
            />
          )}
        </button>
        {!n.leaf && Vek && (
          <div className="my-0.5 ml-2 border-l border-border-subtle pl-2">{Vel}</div>
        )}
      </div>
    );
  };

  const user = Vdv?.displayName ?? "";
  const Veo = user ? user[0].toUpperCase() : "?";

  return (
    <aside className={V462("sidebar", rail && "collapsed")}>
      <div className="side-head">
        {}
        {!Vec && (
          <button
            type="button"
            className="grid size-7 place-items-center rounded border border-border bg-surface text-fg-muted transition-colors hover:border-border-strong hover:bg-gray-100 hover:text-fg"
            onClick={() => Veb((c) => !c)}
            title={collapsed ? "Mở rộng thanh bên" : "Thu gọn thanh bên"}
            aria-label={
              collapsed ? "Mở rộng thanh bên" : "Thu gọn thanh bên"
            }
          >
            <Icon name={collapsed ? "moRong" : "thuGon"} size={15} />
          </button>
        )}
      </div>

      {}
      <nav className="nav-scroll" aria-label="Điều hướng chính">
        {rail ? (
          <div className="flex flex-col items-center gap-1">
            {NAV_TREE.filter(Ve2).map((n) => (
              <button
                type="button"
                key={n.id}
                className={V462(
                  "grid size-8 place-items-center rounded transition-colors",
                  Ve3(n)
                    ? "bg-brand-50 text-brand-600"
                    : "text-fg-subtle hover:bg-gray-100 hover:text-fg",
                )}
                title={n.label}
                aria-label={n.label}
                onClick={() => Ved(n)}
              >
                {n.icon && <Icon name={n.icon} size={17} />}
              </button>
            ))}
          </div>
        ) : (
          NAV_TREE.map((n) => Veh(n, 0))
        )}
      </nav>

      <div className="side-user">
        <DropdownMenu>
          <DropdownMenuTrigger asChild>
            <button
              type="button"
              className="flex min-w-0 flex-1 items-center gap-2 rounded p-1 text-left transition-colors hover:bg-gray-100 data-[state=open]:bg-gray-100"
              aria-label="Trình đơn tài khoản"
            >
              <span className="grid size-7 flex-none place-items-center rounded-full bg-brand-500 text-xs font-semibold text-white">
                {Veo}
              </span>
              {!rail && (
                <>
                  <span className="min-w-0 flex-1">
                    <span className="block truncate text-sm font-semibold text-fg">
                      {user || "Chưa đăng nhập"}
                    </span>
                    <span className="block truncate text-xs text-fg-subtle">
                      {Vdv ? V3za(Vdv.role) : ""}
                      {Vdv?.mode === "offline" ? " · ngoại tuyến" : ""}
                    </span>
                  </span>
                  <Icon name="moNhanh" size={14} className="flex-none text-fg-subtle" />
                </>
              )}
            </button>
          </DropdownMenuTrigger>
          <DropdownMenuContent side="top" align="start" className="w-56">
            <DropdownMenuLabel className="truncate">
              {user || "Chưa đăng nhập"}
            </DropdownMenuLabel>
            <DropdownMenuSeparator />
            {Vdx("settings") && (
              <DropdownMenuItem onSelect={() => goto("settings")}>
                <Icon name="caiDat" size={15} />
                Cài đặt
              </DropdownMenuItem>
            )}
            <DropdownMenuItem onSelect={() => void lock()}>
              <Icon name="khoa" size={15} />
              Khoá màn hình
            </DropdownMenuItem>
            <DropdownMenuItem onSelect={() => void Vdw()}>
              <Icon name="dangXuat" size={15} />
              Đăng xuất
            </DropdownMenuItem>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </aside>
  );
}
