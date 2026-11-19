// Copyright (c) 2026 DigiText. All rights reserved.
// Logo chữ DIGITEXT (0.25, 19/11/2026): in hoa, đậm, nghiêng, bè ngang như logo DIGIFILE — phông `--font-display`
// (theme.css). Một chỗ duy nhất vẽ chữ logo: thanh trên, màn đăng nhập, màn nạp. Bản Lưu trữ kèm chữ nhỏ "Lưu trữ".
// Màu theo `currentColor` của nơi đặt.

import { LUU_TRU, V3wu } from "@/config";
import { V462 as cn } from "@/lib/utils";

const CO = {
  md: { chu: "text-[21px]", phu: "text-xs" },
  lg: { chu: "text-[34px]", phu: "text-sm" },
} as const;

export default function Wordmark({ size = "md", className }: { size?: keyof typeof CO; className?: string }) {
  const c = CO[size];
  return (
    <span role="img" aria-label={V3wu.appName} className={cn("inline-flex select-none items-baseline gap-1.5 whitespace-nowrap", className)}>
      {/* Russo One không có kiểu nghiêng → nghiêng 12° bằng transform (đều nét hơn nghiêng giả của trình duyệt). */}
      <span aria-hidden className={cn("inline-block origin-bottom-left -skew-x-12 font-display uppercase leading-none tracking-[0.01em]", c.chu)}>
        DigiText
      </span>
      {LUU_TRU && (
        <span aria-hidden className={cn("font-sans font-semibold opacity-85", c.phu)}>
          Lưu trữ
        </span>
      )}
    </span>
  );
}
