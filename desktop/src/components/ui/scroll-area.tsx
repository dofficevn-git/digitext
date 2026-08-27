"use client";

import * as React from "react";
import { ScrollArea as ScrollAreaPrimitive } from "radix-ui";

import { V462 } from "@/lib/utils";

function V4yf({
  className,
  children: V3dq,
  ...V3dr
}: React.ComponentProps<typeof ScrollAreaPrimitive.Root>) {
  return (
    <ScrollAreaPrimitive.Root
      data-slot="scroll-area"
      className={V462("relative", className)}
      {...V3dr}
    >
      <ScrollAreaPrimitive.Viewport
        data-slot="scroll-area-viewport"
        className="size-full rounded-[inherit] transition-[color,box-shadow] outline-none focus-visible:ring-2 focus-visible:ring-ring/40 focus-visible:outline-1"
      >
        {V3dq}
      </ScrollAreaPrimitive.Viewport>
      <V4yg />
      <ScrollAreaPrimitive.Corner />
    </ScrollAreaPrimitive.Root>
  );
}

function V4yg({
  className,
  orientation: V3ds = "vertical",
  ...V3dt
}: React.ComponentProps<typeof ScrollAreaPrimitive.ScrollAreaScrollbar>) {
  return (
    <ScrollAreaPrimitive.ScrollAreaScrollbar
      data-slot="scroll-area-scrollbar"
      orientation={V3ds}
      className={V462(
        "flex touch-none p-px transition-colors select-none",
        V3ds === "vertical" && "h-full w-2.5 border-l border-l-transparent",
        V3ds === "horizontal" && "h-2.5 flex-col border-t border-t-transparent",
        className,
      )}
      {...V3dt}
    >
      <ScrollAreaPrimitive.ScrollAreaThumb
        data-slot="scroll-area-thumb"
        className="relative flex-1 rounded-full bg-border"
      />
    </ScrollAreaPrimitive.ScrollAreaScrollbar>
  );
}

export { V4yf as ScrollArea, V4yg as ScrollBar };
