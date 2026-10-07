// Copyright (c) 2026 DigiText. All rights reserved.

import type { ReactNode } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import type { V3y0 } from "@/domain/types";
import { V462 } from "@/lib/utils";

const V47x = cva(
  // 0.18: viên thuốc có chấm màu đầu nhãn (chấm theo màu chữ) — đọc trạng thái nhờ cả màu lẫn chữ.
  "inline-flex w-fit shrink-0 items-center gap-1.5 whitespace-nowrap rounded-full border border-transparent px-2 text-sm font-medium leading-[22px] before:size-1.5 before:flex-none before:rounded-full before:bg-current before:opacity-80 before:content-['']",
  {
    variants: {
      tone: {
        ok: "bg-success-50 text-success-700",
        info: "bg-info-50 text-info-700",
        warn: "bg-warning-50 text-warning-700",
        err: "bg-danger-50 text-danger-700",
        gray: "bg-gray-100 text-fg-muted",
        // Đang chạy (OCR, ghép PDF/A, ký…): hổ phách — màu "đang làm" của cả hệ thống.
        run: "bg-run-bg text-run",
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
