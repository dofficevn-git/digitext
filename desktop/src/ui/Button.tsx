// Copyright (c) 2026 DigiText. All rights reserved.
// Nút chuẩn DigiText. Màn nghiệp vụ dùng cái này (không dùng thẳng components/ui/button hay class .btn).
//   variant: secondary (mặc định, viền) · primary (xanh ngọc) · ghost · danger (đỏ đặc) · dangerOutline (chữ đỏ) · link
//   size: md 32px · sm 28px. `icon` là tên trong components/Icon. `loading` khoá nút và thay icon bằng vòng xoay.

import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { Slot } from "radix-ui";
import { Icon, type V41v as IconName } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";
import Spinner from "./Spinner";

export const buttonStyles = cva(
  "inline-flex shrink-0 items-center justify-center gap-1.5 whitespace-nowrap rounded-md border font-semibold transition-colors outline-none select-none focus-visible:ring-2 focus-visible:ring-ring/40 focus-visible:border-ring disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        // do365-tokens.css .do-btn-secondary: nền trong, viền + chữ tím, hover nền tím nhạt.
        secondary: "border-brand-500 bg-transparent text-brand-600 hover:bg-brand-50 active:bg-brand-100",
        // .do-btn-primary: tím → đậm khi rê → đậm hơn khi bấm.
        primary: "border-brand-500 bg-brand-500 text-fg-onbrand hover:bg-brand-600 hover:border-brand-600 active:bg-brand-700 active:border-brand-700",
        ghost: "border-transparent bg-transparent text-fg-muted hover:bg-surface-active hover:text-fg",
        danger: "border-danger-solid bg-danger-solid text-white hover:bg-danger-700 hover:border-danger-700",
        dangerOutline: "border-border bg-surface text-danger-700 hover:bg-danger-50 hover:border-danger-200",
        link: "border-transparent bg-transparent text-fg-link underline-offset-4 hover:underline px-0 h-auto",
      },
      size: {
        md: "h-8 px-3 text-base [&_svg]:size-4",
        sm: "h-7 px-2 text-sm [&_svg]:size-3.5",
      },
    },
    defaultVariants: { variant: "secondary", size: "md" },
  },
);

export interface ButtonProps extends React.ComponentProps<"button">, VariantProps<typeof buttonStyles> {
  icon?: IconName;
  iconRight?: IconName;
  loading?: boolean;
  asChild?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  { className, variant, size, icon, iconRight, loading, asChild, children, disabled, type, ...rest },
  ref,
) {
  const Comp = asChild ? Slot.Root : "button";
  const iconSize = size === "sm" ? 14 : 16;
  return (
    <Comp
      ref={ref}
      type={asChild ? undefined : (type ?? "button")}
      className={cn(buttonStyles({ variant, size }), className)}
      disabled={disabled || loading}
      aria-busy={loading || undefined}
      {...rest}
    >
      {loading ? <Spinner size={iconSize} /> : icon ? <Icon name={icon} size={iconSize} /> : null}
      {children}
      {iconRight && !loading ? <Icon name={iconRight} size={iconSize} /> : null}
    </Comp>
  );
});

export default Button;
