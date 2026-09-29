// Copyright (c) 2026 DigiText. All rights reserved.
// Menu "…" gom hành động phụ. items: {label, icon?, onSelect, danger?, disabled?} hoặc "-" (vạch ngăn).

import type { ReactNode } from "react";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Icon, type V41v as IconName } from "@/components/Icon";
import Button from "./Button";
import IconButton from "./IconButton";

export interface MenuItem {
  label: string;
  icon?: IconName;
  onSelect: () => void;
  danger?: boolean;
  disabled?: boolean;
  hint?: string;
}

export type MenuEntry = MenuItem | "-";

export default function Menu({
  items,
  label = "Thêm hành động",
  trigger,
  align = "end",
  size = "md",
}: {
  items: MenuEntry[];
  /** Nhãn nút "…" (aria-label) hoặc chữ hiện trên nút khi `trigger="button"`. */
  label?: string;
  trigger?: "dots" | "button" | ReactNode;
  align?: "start" | "end";
  size?: "md" | "sm";
}) {
  const t =
    trigger === undefined || trigger === "dots" ? (
      <IconButton label={label} icon="moNhanh" variant="secondary" size={size} className="[&_svg]:rotate-90" noTooltip />
    ) : trigger === "button" ? (
      <Button size={size} iconRight="xuong">
        {label}
      </Button>
    ) : (
      trigger
    );
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>{t}</DropdownMenuTrigger>
      <DropdownMenuContent align={align} className="min-w-44">
        {items.map((it, i) =>
          it === "-" ? (
            <DropdownMenuSeparator key={`sep-${i}`} />
          ) : (
            <DropdownMenuItem key={it.label} onSelect={it.onSelect} disabled={it.disabled} variant={it.danger ? "destructive" : "default"} title={it.hint}>
              {it.icon && <Icon name={it.icon} size={14} />}
              {it.label}
            </DropdownMenuItem>
          ),
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
