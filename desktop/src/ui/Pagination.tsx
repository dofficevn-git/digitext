// Copyright (c) 2026 DigiText. All rights reserved.
// Phân trang chuẩn: "1–50 trong 1.234", nút trước/sau, ô nhập trang, cỡ trang.

import { useEffect, useMemo, useState } from "react";
import { V462 as cn } from "@/lib/utils";
import IconButton from "./IconButton";
import { Select } from "./Input";

const n = (v: number) => v.toLocaleString("vi-VN");

export interface PaginationState {
  page: number; // 1-based
  pageSize: number;
}

export function usePagination(initialPageSize = 50): PaginationState & { setPage: (p: number) => void; setPageSize: (s: number) => void; reset: () => void } {
  const [page, setPage] = useState(1);
  const [pageSize, setPageSizeRaw] = useState(initialPageSize);
  return {
    page,
    pageSize,
    setPage,
    setPageSize: (s) => {
      setPageSizeRaw(s);
      setPage(1);
    },
    reset: () => setPage(1),
  };
}

/** Cắt mảng theo trang (phân trang phía máy). */
export function pageSlice<T>(rows: T[], p: PaginationState): T[] {
  const start = (p.page - 1) * p.pageSize;
  return rows.slice(start, start + p.pageSize);
}

export default function Pagination({
  page,
  pageSize,
  total,
  onChange,
  onPageSizeChange,
  pageSizeOptions = [25, 50, 100, 200],
  className,
}: {
  page: number;
  pageSize: number;
  total: number;
  onChange: (page: number) => void;
  onPageSizeChange?: (size: number) => void;
  pageSizeOptions?: number[];
  className?: string;
}) {
  const pages = Math.max(1, Math.ceil(total / Math.max(1, pageSize)));
  const cur = Math.min(Math.max(1, page), pages);
  const from = total === 0 ? 0 : (cur - 1) * pageSize + 1;
  const to = Math.min(total, cur * pageSize);
  const [draft, setDraft] = useState(String(cur));
  useEffect(() => setDraft(String(cur)), [cur]);
  const opts = useMemo(() => pageSizeOptions.map((s) => ({ value: String(s), label: `${s} / trang` })), [pageSizeOptions]);
  if (total <= 0) return null;
  return (
    <nav className={cn("flex flex-wrap items-center gap-2 text-sm text-fg-muted", className)} aria-label="Phân trang">
      <span className="tabular-nums">
        {n(from)}–{n(to)} trong {n(total)}
      </span>
      <div className="ml-auto flex items-center gap-1">
        {onPageSizeChange && <Select size="sm" value={String(pageSize)} onChange={(v) => onPageSizeChange(Number(v))} options={opts} className="w-auto" aria-label="Số dòng mỗi trang" />}
        <IconButton label="Trang trước" icon="truoc" size="sm" variant="secondary" disabled={cur <= 1} onClick={() => onChange(cur - 1)} />
        <span className="flex items-center gap-1">
          <input
            className="h-7 w-12 rounded-md border border-border bg-surface text-center text-sm tabular-nums outline-none focus-visible:border-ring focus-visible:ring-2 focus-visible:ring-ring/40"
            value={draft}
            aria-label="Trang"
            inputMode="numeric"
            onChange={(e) => setDraft(e.target.value)}
            onBlur={() => {
              const v = Number(draft);
              if (Number.isFinite(v) && v >= 1 && v <= pages && v !== cur) onChange(v);
              else setDraft(String(cur));
            }}
            onKeyDown={(e) => e.key === "Enter" && (e.target as HTMLInputElement).blur()}
          />
          <span className="tabular-nums">/ {n(pages)}</span>
        </span>
        <IconButton label="Trang sau" icon="sau" size="sm" variant="secondary" disabled={cur >= pages} onClick={() => onChange(cur + 1)} />
      </div>
    </nav>
  );
}
