// Copyright (c) 2026 DigiText. All rights reserved.
// Nút (i) mở khung giải thích (0.25 "ít chữ"): nơi đặt căn cứ pháp lý, cách dùng, giải thích kỹ thuật — thay cho mô tả
// dài dưới tiêu đề màn và khối Help chiếm chỗ trong trang. Nội dung chỉ hiện khi người dùng bấm.

import type { ReactNode } from "react";
import { Popover } from "radix-ui";
import { Icon } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";

export default function InfoButton({
  title,
  children,
  label = "Giải thích",
  side = "bottom",
  className,
}: {
  title?: ReactNode;
  children: ReactNode;
  /** Tên cho đọc màn hình. */
  label?: string;
  side?: "top" | "bottom" | "left" | "right";
  className?: string;
}) {
  return (
    <Popover.Root>
      <Popover.Trigger asChild>
        <button
          type="button"
          aria-label={label}
          className={cn(
            "grid size-6 flex-none place-items-center rounded-full text-fg-subtle outline-none transition-colors hover:bg-surface-active hover:text-fg focus-visible:ring-2 focus-visible:ring-ring/40 data-[state=open]:bg-brand-50 data-[state=open]:text-brand-600",
            className,
          )}
        >
          <Icon name="thongTin" size={15} />
        </button>
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Content
          side={side}
          align="start"
          sideOffset={6}
          collisionPadding={12}
          className="z-50 w-[min(28rem,calc(100vw-2rem))] rounded-lg border border-border bg-surface p-3 text-sm text-fg-muted shadow-lg outline-none [&_p+p]:mt-1.5 [&_ul]:list-disc [&_ul]:pl-4"
        >
          {title && <p className="mb-1 text-base font-semibold text-fg">{title}</p>}
          {children}
        </Popover.Content>
      </Popover.Portal>
    </Popover.Root>
  );
}
