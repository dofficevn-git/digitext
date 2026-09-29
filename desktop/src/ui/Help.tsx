// Copyright (c) 2026 DigiText. All rights reserved.
// Khối "Trợ giúp" gập được: nơi đặt giải thích kỹ thuật, tên chuẩn, điều khoản — thay cho tooltip dài.

import { useState, type ReactNode } from "react";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { Icon } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";

export default function Help({ title = "Trợ giúp", children, defaultOpen, className }: { title?: string; children: ReactNode; defaultOpen?: boolean; className?: string }) {
  const [open, setOpen] = useState(!!defaultOpen);
  return (
    <Collapsible open={open} onOpenChange={setOpen} className={cn("rounded-md border border-border-subtle bg-gray-25", className)}>
      <CollapsibleTrigger className="flex w-full items-center gap-2 px-3 py-2 text-left text-sm font-medium text-fg-muted hover:text-fg">
        <Icon name="thongTin" size={14} />
        {title}
        <Icon name={open ? "len" : "xuong"} size={14} className="ml-auto" />
      </CollapsibleTrigger>
      <CollapsibleContent className="px-3 pb-3 text-sm text-fg-muted [&_p+p]:mt-1.5 [&_ul]:list-disc [&_ul]:pl-4">{children}</CollapsibleContent>
    </Collapsible>
  );
}
