import * as React from "react";
import { Separator as SeparatorPrimitive } from "radix-ui";

import { V462 } from "@/lib/utils";

function V4yr({
  className,
  orientation: V3e9 = "horizontal",
  decorative: V3ea = true,
  ...V3eb
}: React.ComponentProps<typeof SeparatorPrimitive.Root>) {
  return (
    <SeparatorPrimitive.Root
      data-slot="separator"
      decorative={V3ea}
      orientation={V3e9}
      className={V462(
        "shrink-0 bg-border data-[orientation=horizontal]:h-px data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full data-[orientation=vertical]:w-px",
        className,
      )}
      {...V3eb}
    />
  );
}

export { V4yr as Separator };
