// Copyright (c) 2026 DigiText. All rights reserved.
// Hộp thoại chuẩn (Radix): tiêu đề, thân cuộn được, chân cố định. size: sm 420 · md 560 · lg 800 · xl 1040 · full.

import type { ReactNode } from "react";
import { Dialog as Root, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import { V462 as cn } from "@/lib/utils";

const SIZE = {
  sm: "sm:max-w-[420px]",
  md: "sm:max-w-[560px]",
  lg: "sm:max-w-[800px]",
  xl: "sm:max-w-[1040px]",
  full: "sm:max-w-[calc(100vw-3rem)] h-[calc(100dvh-3rem)]",
} as const;

export default function Dialog({
  open,
  onOpenChange,
  title,
  description,
  size = "md",
  footer,
  children,
  className,
  bodyClassName,
  closable = true,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: ReactNode;
  description?: ReactNode;
  size?: keyof typeof SIZE;
  footer?: ReactNode;
  children?: ReactNode;
  className?: string;
  bodyClassName?: string;
  /** false → không có nút X, không đóng khi bấm nền / Esc (đang chạy tác vụ). */
  closable?: boolean;
}) {
  return (
    <Root open={open} onOpenChange={(o) => (closable || o ? onOpenChange(o) : undefined)}>
      <DialogContent
        showCloseButton={closable}
        className={cn("flex max-h-[calc(100dvh-3rem)] flex-col gap-0 p-0 rounded-xl", SIZE[size], className)}
        onEscapeKeyDown={(e) => !closable && e.preventDefault()}
        onInteractOutside={(e) => !closable && e.preventDefault()}
      >
        <DialogHeader className="flex-none border-b border-border-subtle px-5 py-3.5">
          <DialogTitle className="text-lg">{title}</DialogTitle>
          <DialogDescription className={description ? "" : "sr-only"}>{description ?? (typeof title === "string" ? title : "")}</DialogDescription>
        </DialogHeader>
        <div className={cn("min-h-0 flex-1 overflow-y-auto px-5 py-4", bodyClassName)}>{children}</div>
        {footer && <DialogFooter className="flex-none border-t border-border-subtle px-5 py-3">{footer}</DialogFooter>}
      </DialogContent>
    </Root>
  );
}
