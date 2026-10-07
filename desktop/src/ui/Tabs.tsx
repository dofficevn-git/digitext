// Copyright (c) 2026 DigiText. All rights reserved.
// Tab chuyển mục trong một màn (Radix). Bộ lọc nhiều lựa chọn thì dùng Seg.

import type { ReactNode } from "react";
import { Tabs as Root, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Icon, type V41v as IconName } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";

export interface TabItem<T extends string> {
  value: T;
  label: string;
  icon?: IconName;
  count?: number;
  disabled?: boolean;
}

export default function Tabs<T extends string>({
  items,
  value,
  onChange,
  children,
  className,
  variant = "line",
}: {
  items: TabItem<T>[];
  value: T;
  onChange: (v: T) => void;
  children?: ReactNode;
  className?: string;
  variant?: "line" | "default";
}) {
  return (
    <Root value={value} onValueChange={(v) => onChange(v as T)} className={cn("gap-3", className)}>
      <TabsList variant={variant} className={variant === "line" ? "w-full justify-start border-b border-border-subtle rounded-none p-0 h-9" : undefined}>
        {items.map((t) => (
          <TabsTrigger key={t.value} value={t.value} disabled={t.disabled} className={variant === "line" ? "flex-none px-3" : undefined}>
            {t.icon && <Icon name={t.icon} size={14} />}
            {t.label}
            {t.count !== undefined && <span className="ml-1 rounded-full bg-gray-100 px-1.5 text-xs text-fg-muted tabular-nums">{t.count.toLocaleString("vi-VN")}</span>}
          </TabsTrigger>
        ))}
      </TabsList>
      {children}
    </Root>
  );
}

export function TabPanel({ value, children, className }: { value: string; children: ReactNode; className?: string }) {
  return (
    <TabsContent value={value} className={className}>
      {children}
    </TabsContent>
  );
}
