"use client";

import * as React from "react";
import { Progress as ProgressPrimitive } from "radix-ui";

import { V462 } from "@/lib/utils";

function V4ye({
  className,
  value: V3do,
  ...V3dp
}: React.ComponentProps<typeof ProgressPrimitive.Root>) {
  return (
    <ProgressPrimitive.Root
      data-slot="progress"
      className={V462("relative h-2 w-full overflow-hidden rounded-full bg-primary/20", className)}
      {...V3dp}
    >
      <ProgressPrimitive.Indicator
        data-slot="progress-indicator"
        className="h-full w-full flex-1 bg-primary transition-all"
        style={{ transform: `translateX(-${100 - (V3do || 0)}%)` }}
      />
    </ProgressPrimitive.Root>
  );
}

export { V4ye as Progress };
