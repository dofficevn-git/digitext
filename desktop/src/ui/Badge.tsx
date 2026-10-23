// Copyright (c) 2026 DigiText. All rights reserved.

import type { ReactNode } from "react";
import { cva, type VariantProps } from "class-variance-authority";
import type { V3y0 } from "@/domain/types";
import { V462 } from "@/lib/utils";

const V47x = cva(
  // do365-tokens.css .do-badge: viên thuốc, chữ nhỏ 600, nền nhạt + chữ màu đặc; giữ chấm màu đầu nhãn (0.18) để đọc
  // trạng thái nhờ cả màu lẫn chữ.
  "inline-flex w-fit shrink-0 items-center gap-1.5 whitespace-nowrap rounded-full border border-transparent px-2 text-xs font-semibold leading-[20px] before:size-1.5 before:flex-none before:rounded-full before:bg-current before:opacity-80 before:content-['']",
  {
    variants: {
      tone: {
        ok: "bg-success-100 text-success-600",
        info: "bg-info-50 text-info-600",
        warn: "bg-warning-50 text-warning-600",
        err: "bg-danger-50 text-danger-600",
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
