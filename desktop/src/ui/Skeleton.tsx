// Copyright (c) 2026 DigiText. All rights reserved.
// Khung xám nhấp nháy thay chỗ nội dung đang tải lần đầu.

import type React from "react";
import { V462 as cn } from "@/lib/utils";

function Skeleton({ className, style }: { className?: string; style?: React.CSSProperties }) {
  return <span className={cn("block animate-pulse rounded bg-gray-150", className)} style={style} aria-hidden />;
}

function Line({ w = "100%", className }: { w?: string; className?: string }) {
  return <Skeleton className={cn("h-3.5", className)} style={{ width: w }} />;
}

/** Vài hàng bảng giả. */
function Rows({ rows = 5, cols = 4 }: { rows?: number; cols?: number }) {
  return (
    <div className="space-y-2 p-3" role="status" aria-label="Đang tải">
      {Array.from({ length: rows }, (_, r) => (
        <div key={r} className="flex gap-3">
          {Array.from({ length: cols }, (_, c) => (
            <Skeleton key={c} className={cn("h-3.5 flex-1", c === 0 && "max-w-16", c === cols - 1 && "max-w-24")} />
          ))}
        </div>
      ))}
    </div>
  );
}

Skeleton.Line = Line;
Skeleton.Rows = Rows;
export default Skeleton;
