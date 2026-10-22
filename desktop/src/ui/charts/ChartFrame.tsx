// Copyright (c) 2026 DigiText. All rights reserved.
// Khung chung của mọi biểu đồ: đo bề rộng, legend (tự hiện khi ≥ 2 chuỗi), nút "Xem bảng" (mọi biểu đồ đều có bảng
// số), trạng thái trống, và vùng bắt bàn phím ←/→ cho lớp hover. Thành phần con chỉ lo vẽ SVG theo bề rộng được cấp.

import { useState, type KeyboardEvent, type ReactNode } from "react";
import { V462 as cn } from "@/lib/utils";
import Button from "../Button";
import EmptyState from "../EmptyState";
import Table, { type Column } from "../Table";
import ChartLegend, { type LegendItem } from "./ChartLegend";
import { useWidth } from "./hooks";

export interface ChartTable<T> {
  columns: Column<T>[];
  rows: T[];
  getRowId: (row: T) => string;
}

export interface ChartFrameProps<T> {
  /** Mô tả ngắn cho máy đọc màn hình: tên + kết luận chính ("Trang quét 14 ngày, cao nhất 1.200 ngày 21/10"). */
  ariaLabel: string;
  legend?: LegendItem[];
  table?: ChartTable<T>;
  empty?: boolean;
  emptyText?: string;
  className?: string;
  /** Hành động thêm ở góc phải (Seg chọn chỉ số, Seg số ngày…). */
  actions?: ReactNode;
  /** ←/→ khi vùng biểu đồ có focus. */
  onKeyStep?: (dir: -1 | 1) => void;
  onLeave?: () => void;
  children: (width: number) => ReactNode;
}

export default function ChartFrame<T>({ ariaLabel, legend, table, empty, emptyText = "Chưa có số liệu", className, actions, onKeyStep, onLeave, children }: ChartFrameProps<T>) {
  const [ref, width] = useWidth<HTMLDivElement>();
  const [showTable, setShowTable] = useState(false);
  const onKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (!onKeyStep) return;
    if (e.key === "ArrowRight") {
      e.preventDefault();
      onKeyStep(1);
    } else if (e.key === "ArrowLeft") {
      e.preventDefault();
      onKeyStep(-1);
    } else if (e.key === "Escape") {
      onLeave?.();
    }
  };
  const hasHeader = (legend && legend.length >= 2) || table || actions;
  return (
    <div className={cn("min-w-0", className)}>
      {hasHeader && (
        <div className="mb-2 flex flex-wrap items-center gap-x-4 gap-y-1">
          {legend && legend.length >= 2 && <ChartLegend items={legend} />}
          <div className="ml-auto flex items-center gap-2">
            {actions}
            {table && !empty && (
              <Button size="sm" variant="ghost" icon={showTable ? "thongke" : "danhSach"} onClick={() => setShowTable((v) => !v)} aria-pressed={showTable}>
                {showTable ? "Xem biểu đồ" : "Xem bảng"}
              </Button>
            )}
          </div>
        </div>
      )}
      {empty ? (
        <EmptyState compact icon="thongke" title={emptyText} />
      ) : showTable && table ? (
        <Table columns={table.columns} rows={table.rows} getRowId={table.getRowId} dense maxHeight={320} caption={ariaLabel} />
      ) : (
        // eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions, jsx-a11y/no-noninteractive-tabindex -- vùng nhận ←/→ để di chuyển điểm hover (roving theo chỉ số); các điểm không phải nút riêng
        <div ref={ref} role="group" tabIndex={0} aria-label={ariaLabel} className="relative w-full outline-none focus-visible:ring-2 focus-visible:ring-brand-300 rounded" onKeyDown={onKeyDown} onMouseLeave={onLeave} onBlur={onLeave}>
          {width > 0 && children(width)}
        </div>
      )}
    </div>
  );
}
