// Copyright (c) 2026 DigiText. All rights reserved.
// Ô chọn NHIỀU giá trị cho bộ lọc (0.25 "ít chữ"): một nút gọn "Nhãn" + số đang chọn, bấm mở danh sách đánh dấu —
// thay cho hàng chục chip dàn ngang (THBQ ở Lập hồ sơ có 16+ chip). Danh sách giữ mở khi đánh dấu để chọn liền tay.

import { Icon } from "@/components/Icon";
import { DropdownMenu, DropdownMenuCheckboxItem, DropdownMenuContent, DropdownMenuItem, DropdownMenuLabel, DropdownMenuSeparator, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { V462 as cn } from "@/lib/utils";

export interface MultiOption {
  value: string;
  label: string;
  /** Tên nhóm; các mục cùng nhóm đứng liền nhau dưới một tiêu đề. */
  group?: string;
}

export default function MultiSelect({
  label,
  options,
  value,
  onChange,
  size = "sm",
  disabled,
  className,
}: {
  label: string;
  options: MultiOption[];
  value: readonly string[];
  onChange: (next: string[]) => void;
  size?: "md" | "sm";
  disabled?: boolean;
  className?: string;
}) {
  const chon = new Set(value);
  const doi = (v: string, bat: boolean) => {
    const n = new Set(chon);
    if (bat) n.add(v);
    else n.delete(v);
    onChange(options.filter((o) => n.has(o.value)).map((o) => o.value));
  };
  const nhom: { ten?: string; muc: MultiOption[] }[] = [];
  for (const o of options) {
    const cuoi = nhom[nhom.length - 1];
    if (cuoi && cuoi.ten === o.group) cuoi.muc.push(o);
    else nhom.push({ ten: o.group, muc: [o] });
  }
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <button
          type="button"
          disabled={disabled}
          aria-label={chon.size ? `${label}: đang chọn ${chon.size}` : label}
          className={cn(
            "inline-flex flex-none items-center gap-1.5 rounded-md border bg-surface font-medium outline-none transition-colors focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/40 disabled:opacity-50",
            size === "sm" ? "h-7 px-2 text-sm" : "h-8 px-2.5 text-base",
            chon.size ? "border-brand-500 text-brand-700" : "border-border text-fg hover:border-border-strong",
            className,
          )}
        >
          {label}
          {chon.size > 0 && <span className="min-w-5 rounded-full bg-brand-500 px-1.5 text-center text-xs font-semibold leading-5 text-white tabular-nums">{chon.size}</span>}
          <Icon name="xuong" size={14} className="text-fg-subtle" />
        </button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start" className="max-h-80 w-60 overflow-y-auto">
        {nhom.map((g, i) => (
          <div key={g.ten ?? `n${i}`}>
            {i > 0 && <DropdownMenuSeparator />}
            {g.ten && <DropdownMenuLabel className="text-xs font-medium text-fg-subtle">{g.ten}</DropdownMenuLabel>}
            {g.muc.map((o) => (
              <DropdownMenuCheckboxItem key={o.value} checked={chon.has(o.value)} onCheckedChange={(b) => doi(o.value, !!b)} onSelect={(e) => e.preventDefault()}>
                {o.label}
              </DropdownMenuCheckboxItem>
            ))}
          </div>
        ))}
        {chon.size > 0 && (
          <>
            <DropdownMenuSeparator />
            <DropdownMenuItem onSelect={() => onChange([])}>Bỏ chọn tất cả</DropdownMenuItem>
          </>
        )}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
