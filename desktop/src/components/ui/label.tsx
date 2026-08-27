import * as React from "react";
import { Label as LabelPrimitive } from "radix-ui";

import { V462 } from "@/lib/utils";

function V49i({ className, ...Vlm }: React.ComponentProps<typeof LabelPrimitive.Root>) {
  return (
    <LabelPrimitive.Root
      data-slot="label"
      className={V462(
        "flex items-center gap-2 text-base leading-none font-medium select-none group-data-[disabled=true]:pointer-events-none group-data-[disabled=true]:opacity-50 peer-disabled:cursor-not-allowed peer-disabled:opacity-50",
        className,
      )}
      {...Vlm}
    />
  );
}

export { V49i as Label };
