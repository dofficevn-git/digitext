"use client";

import * as React from "react";
import { Popover as PopoverPrimitive } from "radix-ui";

import { V462 } from "@/lib/utils";

function V4y7({ ...V3df }: React.ComponentProps<typeof PopoverPrimitive.Root>) {
  return <PopoverPrimitive.Root data-slot="popover" {...V3df} />;
}

function V4y8({ ...V3dg }: React.ComponentProps<typeof PopoverPrimitive.Trigger>) {
  return <PopoverPrimitive.Trigger data-slot="popover-trigger" {...V3dg} />;
}

function V4y9({
  className,
  align: V3dh = "center",
  sideOffset: V3di = 4,
  ...V3dj
}: React.ComponentProps<typeof PopoverPrimitive.Content>) {
  return (
    <PopoverPrimitive.Portal>
      <PopoverPrimitive.Content
        data-slot="popover-content"
        align={V3dh}
        sideOffset={V3di}
        className={V462(
          "z-50 w-72 origin-(--radix-popover-content-transform-origin) rounded-md border bg-popover p-4 text-popover-foreground shadow-md outline-hidden data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
          className,
        )}
        {...V3dj}
      />
    </PopoverPrimitive.Portal>
  );
}

function V4ya({ ...V3dk }: React.ComponentProps<typeof PopoverPrimitive.Anchor>) {
  return <PopoverPrimitive.Anchor data-slot="popover-anchor" {...V3dk} />;
}

function V4yb({ className, ...V3dl }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="popover-header"
      className={V462("flex flex-col gap-1 text-base", className)}
      {...V3dl}
    />
  );
}

function V4yc({ className, ...V3dm }: React.ComponentProps<"h2">) {
  return <div data-slot="popover-title" className={V462("font-medium", className)} {...V3dm} />;
}

function V4yd({ className, ...V3dn }: React.ComponentProps<"p">) {
  return (
    <p
      data-slot="popover-description"
      className={V462("text-muted-foreground", className)}
      {...V3dn}
    />
  );
}

export {
  V4y7 as Popover,
  V4y8 as PopoverTrigger,
  V4y9 as PopoverContent,
  V4ya as PopoverAnchor,
  V4yb as PopoverHeader,
  V4yc as PopoverTitle,
  V4yd as PopoverDescription,
};
