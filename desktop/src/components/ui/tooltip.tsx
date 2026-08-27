import * as React from "react";
import { Tooltip as TooltipPrimitive } from "radix-ui";

import { V462 } from "@/lib/utils";

function V4z0({
  delayDuration: V3eq = 0,
  ...V3er
}: React.ComponentProps<typeof TooltipPrimitive.Provider>) {
  return <TooltipPrimitive.Provider data-slot="tooltip-provider" delayDuration={V3eq} {...V3er} />;
}

function V4z1({ ...V3es }: React.ComponentProps<typeof TooltipPrimitive.Root>) {
  return <TooltipPrimitive.Root data-slot="tooltip" {...V3es} />;
}

function V4z2({ ...V3et }: React.ComponentProps<typeof TooltipPrimitive.Trigger>) {
  return <TooltipPrimitive.Trigger data-slot="tooltip-trigger" {...V3et} />;
}

function V4z3({
  className,
  sideOffset: V3eu = 0,
  children: V3ev,
  ...V3ew
}: React.ComponentProps<typeof TooltipPrimitive.Content>) {
  return (
    <TooltipPrimitive.Portal>
      <TooltipPrimitive.Content
        data-slot="tooltip-content"
        sideOffset={V3eu}
        className={V462(
          "z-50 w-fit origin-(--radix-tooltip-content-transform-origin) animate-in rounded-md bg-foreground px-3 py-1.5 text-xs text-balance text-background fade-in-0 zoom-in-95 data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95",
          className,
        )}
        {...V3ew}
      >
        {V3ev}
        <TooltipPrimitive.Arrow className="z-50 size-2.5 translate-y-[calc(-50%_-_2px)] rotate-45 rounded-[2px] bg-foreground fill-foreground" />
      </TooltipPrimitive.Content>
    </TooltipPrimitive.Portal>
  );
}

export { V4z1 as Tooltip, V4z2 as TooltipTrigger, V4z3 as TooltipContent, V4z0 as TooltipProvider };
