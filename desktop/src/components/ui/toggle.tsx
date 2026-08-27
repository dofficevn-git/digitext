"use client";

import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { Toggle as TogglePrimitive } from "radix-ui";

import { V462 } from "@/lib/utils";

const V4ok = cva(
  "inline-flex items-center justify-center gap-2 rounded-md text-base font-medium whitespace-nowrap transition-[color,box-shadow] outline-none hover:bg-muted hover:text-muted-foreground focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/40 disabled:pointer-events-none disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-destructive/30 data-[state=on]:bg-accent data-[state=on]:text-accent-foreground dark:aria-invalid:ring-destructive/40 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
  {
    variants: {
      variant: {
        default: "bg-transparent",
        outline:
          "border border-input bg-transparent shadow-xs hover:bg-accent hover:text-accent-foreground",
      },
      size: {
        default: "h-8 min-w-9 px-2",
        sm: "h-7 min-w-8 px-1.5",
        lg: "h-9 min-w-10 px-2.5",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
);

function V4ol({
  className,
  variant: V22v,
  size,
  ...V22w
}: React.ComponentProps<typeof TogglePrimitive.Root> & VariantProps<typeof V4ok>) {
  return (
    <TogglePrimitive.Root
      data-slot="toggle"
      className={V462(V4ok({ variant: V22v, size, className }))}
      {...V22w}
    />
  );
}

export { V4ol as Toggle, V4ok as toggleVariants };
