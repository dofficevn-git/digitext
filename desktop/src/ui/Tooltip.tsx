// Copyright (c) 2026 DigiText. All rights reserved.
// Tooltip ngắn (tên, phím tắt). Nội dung dài đưa vào <Help>. `text` rỗng → không bọc.

import type { ReactNode } from "react";
import { Tooltip as Root, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

export default function Tip({ text, children, side }: { text?: string; children: ReactNode; side?: "top" | "bottom" | "left" | "right" }) {
  if (!text) return <>{children}</>;
  if (import.meta.env.DEV && text.length > 80) console.warn(`Tooltip quá dài (${text.length} ký tự): "${text.slice(0, 40)}…" — chuyển sang Help.`);
  return (
    <Root>
      <TooltipTrigger asChild>{children}</TooltipTrigger>
      <TooltipContent side={side} sideOffset={4}>
        {text}
      </TooltipContent>
    </Root>
  );
}
