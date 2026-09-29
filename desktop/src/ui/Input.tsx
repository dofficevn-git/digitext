// Copyright (c) 2026 DigiText. All rights reserved.
// Ô nhập / ô nhiều dòng / ô chọn (native) / hộp kiểm / công tắc — đọc FieldContext để gắn id và aria-*.

import * as React from "react";
import { Checkbox as CheckboxPrimitive } from "@/components/ui/checkbox";
import { Switch as SwitchPrimitive } from "@/components/ui/switch";
import { Icon, type V41v as IconName } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";
import { useFieldProps } from "./Field";

const base =
  "w-full min-w-0 rounded-md border border-border bg-surface text-fg placeholder:text-fg-placeholder outline-none transition-[border-color,box-shadow] focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/40 disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-danger-500 aria-invalid:ring-2 aria-invalid:ring-danger-200";

export interface InputProps extends Omit<React.ComponentProps<"input">, "size" | "prefix"> {
  size?: "md" | "sm";
  prefix?: IconName | string;
  suffix?: IconName | string;
}

function isIcon(v: unknown): v is IconName {
  return typeof v === "string" && v.length < 24 && !/\s/.test(v) && /^[a-zA-Z0-9]+$/.test(v);
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(function Input({ className, size = "md", prefix, suffix, ...rest }, ref) {
  const fp = useFieldProps();
  const h = size === "sm" ? "h-7 text-sm" : "h-8 text-base";
  const el = (
    <input
      ref={ref}
      className={cn(base, h, "px-2.5", prefix && "pl-8", suffix && "pr-8", className)}
      {...fp}
      {...rest}
    />
  );
  if (!prefix && !suffix) return el;
  const aff = (v: IconName | string, side: "l" | "r") => (
    <span className={cn("pointer-events-none absolute inset-y-0 flex items-center text-fg-subtle text-sm", side === "l" ? "left-2.5" : "right-2.5")}>
      {isIcon(v) ? <Icon name={v} size={14} /> : v}
    </span>
  );
  return (
    <span className="relative block w-full">
      {prefix && aff(prefix, "l")}
      {el}
      {suffix && aff(suffix, "r")}
    </span>
  );
});

export const Textarea = React.forwardRef<HTMLTextAreaElement, React.ComponentProps<"textarea">>(function Textarea({ className, rows = 3, ...rest }, ref) {
  const fp = useFieldProps();
  return <textarea ref={ref} rows={rows} className={cn(base, "px-2.5 py-1.5 text-base leading-snug", className)} {...fp} {...rest} />;
});

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectProps extends Omit<React.ComponentProps<"select">, "size" | "onChange" | "value"> {
  value: string;
  onChange: (value: string) => void;
  options?: SelectOption[];
  /** Dòng đầu (giá trị ""). */
  placeholder?: string;
  size?: "md" | "sm";
}

/** Ô chọn native: nhanh với danh mục dài, bàn phím và đọc màn hình tốt. */
export const Select = React.forwardRef<HTMLSelectElement, SelectProps>(function Select(
  { className, size = "md", value, onChange, options, placeholder, children, ...rest },
  ref,
) {
  const fp = useFieldProps();
  const h = size === "sm" ? "h-7 text-sm" : "h-8 text-base";
  return (
    <select
      ref={ref}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      className={cn(base, h, "appearance-none pl-2.5 pr-7 bg-[image:var(--select-chevron)] bg-no-repeat bg-[right_6px_center] bg-[length:14px]", className)}
      {...fp}
      {...rest}
    >
      {placeholder !== undefined && <option value="">{placeholder}</option>}
      {options?.map((o) => (
        <option key={o.value} value={o.value} disabled={o.disabled}>
          {o.label}
        </option>
      ))}
      {children}
    </select>
  );
});

export interface CheckboxProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  className?: string;
  /** Tên cho đọc màn hình khi không nằm trong Field inline. */
  label?: string;
}

/** Hộp kiểm; vùng bấm 28px dù ô 16px. */
export function Checkbox({ checked, onChange, disabled, className, label }: CheckboxProps) {
  const fp = useFieldProps();
  return (
    <span className={cn("inline-grid size-7 place-items-center -m-1.5", className)}>
      <CheckboxPrimitive checked={checked} onCheckedChange={(v) => onChange(v === true)} disabled={disabled} aria-label={label} {...fp} />
    </span>
  );
}

export function Switch({ checked, onChange, disabled, className, label }: CheckboxProps) {
  const fp = useFieldProps();
  return <SwitchPrimitive checked={checked} onCheckedChange={onChange} disabled={disabled} aria-label={label} className={className} {...fp} />;
}

export default Input;
