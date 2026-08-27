// Copyright (c) 2026 DigiText. All rights reserved.

import type { ReactNode } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import type { V3y0 } from "@/domain/types";
import { V462 } from "@/lib/utils";

const V47x = cva(
  "inline-flex w-fit shrink-0 items-center gap-1 whitespace-nowrap rounded-sm border px-1.5 text-sm font-medium leading-[18px]",
  {
    variants: {
      tone: {
        ok: "border-success-200 bg-success-50 text-success-700",
        info: "border-info-200 bg-info-50 text-info-700",
        warn: "border-warning-200 bg-warning-50 text-warning-700",
        err: "border-danger-200 bg-danger-50 text-danger-700",
        gray: "border-border-subtle bg-gray-100 text-fg-muted",
      } satisfies Record<V3y0, string>,
    },
    defaultVariants: { tone: "gray" },
  },
);

export default function V47y({
  tone: Vfv,
  children: Vfw,
  className,
}: {
  tone: V3y0;
  children: ReactNode;
  className?: string;
}) {
  return <span className={V462(V47x({ tone: Vfv }), className)}>{Vfw}</span>;
}

export type V47z = VariantProps<typeof V47x>;
