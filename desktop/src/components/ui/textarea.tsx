import * as React from "react";

import { V462 } from "@/lib/utils";

function V4yz({ className, ...V3ep }: React.ComponentProps<"textarea">) {
  return (
    <textarea
      data-slot="textarea"
      className={V462(
        "flex field-sizing-content min-h-16 w-full rounded-md border border-input bg-transparent px-3 py-2 text-base shadow-xs transition-[color,box-shadow] outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/40 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/30 dark:bg-input/30 dark:aria-invalid:ring-destructive/40",
        className,
      )}
      {...V3ep}
    />
  );
}

export { V4yz as Textarea };
