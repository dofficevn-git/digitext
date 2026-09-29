// Copyright (c) 2026 DigiText. All rights reserved.
// Nút chỉ có biểu tượng. `label` bắt buộc: là tên đọc màn hình (aria-label) và tooltip.

import * as React from "react";
import { Icon, type V41v as IconName } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";
import { buttonStyles } from "./Button";
import Spinner from "./Spinner";
import Tip from "./Tooltip";

export interface IconButtonProps extends Omit<React.ComponentProps<"button">, "children"> {
  label: string;
  icon: IconName;
  size?: "md" | "sm";
  variant?: "secondary" | "primary" | "ghost" | "danger" | "dangerOutline";
  loading?: boolean;
  /** Tắt tooltip (ví dụ trong bảng dày). aria-label vẫn giữ. */
  noTooltip?: boolean;
}

const IconButton = React.forwardRef<HTMLButtonElement, IconButtonProps>(function IconButton(
  { label, icon, size = "md", variant = "ghost", loading, noTooltip, className, disabled, type, ...rest },
  ref,
) {
  const px = size === "sm" ? 14 : 16;
  const btn = (
    <button
      ref={ref}
      type={type ?? "button"}
      aria-label={label}
      className={cn(buttonStyles({ variant, size }), size === "sm" ? "w-7 px-0" : "w-8 px-0", className)}
      disabled={disabled || loading}
      aria-busy={loading || undefined}
      {...rest}
    >
      {loading ? <Spinner size={px} /> : <Icon name={icon} size={px} />}
    </button>
  );
  return noTooltip ? btn : <Tip text={label}>{btn}</Tip>;
});

export default IconButton;
