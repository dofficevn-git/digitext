// Copyright (c) 2026 DigiText. All rights reserved.
// Bảng chuẩn. Hai cách dùng:
//  (a) cấu hình: <Table columns rows getRowId sort selection state pagination …/> — cho bảng chỉ hiển thị.
//  (b) compound: <Table.Root><Table.Head><Table.Row><Table.HeadCell sortKey="x">…  — cho bảng có ô nhập / hàng mở rộng.
// Có sẵn: tiêu đề dính, sắp xếp bằng nút (aria-sort, bàn phím), cột số canh phải, chọn nhiều, loading / lỗi / rỗng, phân trang.

import * as React from "react";
import { Icon } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";
import { ErrorBanner } from "./Banner";
import EmptyState from "./EmptyState";
import { Checkbox } from "./Input";
import Pagination from "./Pagination";
import Skeleton from "./Skeleton";

// ---------- sắp xếp
export type SortDir = "asc" | "desc";
export interface SortState {
  key: string;
  dir: SortDir;
}

interface SortCtx {
  sort?: SortState | null;
  onSort?: (key: string) => void;
}
const SortContext = React.createContext<SortCtx>({});

export const collator = new Intl.Collator("vi", { numeric: true, sensitivity: "base" });

/** So sánh chung: số theo số, chuỗi theo tiếng Việt, null xuống cuối. */
export function compareValues(a: unknown, b: unknown): number {
  const an = a === null || a === undefined || a === "";
  const bn = b === null || b === undefined || b === "";
  if (an && bn) return 0;
  if (an) return 1;
  if (bn) return -1;
  if (typeof a === "number" && typeof b === "number") return a - b;
  if (typeof a === "boolean" && typeof b === "boolean") return Number(a) - Number(b);
  return collator.compare(String(a), String(b));
}

/** Quản lý trạng thái sắp xếp + trả mảng đã sắp (phía máy). */
export function useTableSort<T>(rows: T[], getValue: (row: T, key: string) => unknown, initial?: SortState | null) {
  const [sort, setSort] = React.useState<SortState | null>(initial ?? null);
  const onSort = React.useCallback((key: string) => {
    setSort((s) => (s?.key === key ? (s.dir === "asc" ? { key, dir: "desc" } : null) : { key, dir: "asc" }));
  }, []);
  const sorted = React.useMemo(() => {
    if (!sort) return rows;
    const dir = sort.dir === "asc" ? 1 : -1;
    return [...rows].sort((a, b) => dir * compareValues(getValue(a, sort.key), getValue(b, sort.key)));
  }, [rows, sort, getValue]);
  return { sort, setSort, onSort, sorted };
}

// ---------- compound
function Root({
  children,
  className,
  maxHeight,
  dense,
  stickyHeader = true,
  sort,
  onSort,
  ...rest
}: React.ComponentProps<"table"> & { maxHeight?: string | number; dense?: boolean; stickyHeader?: boolean; sort?: SortState | null; onSort?: (key: string) => void }) {
  return (
    <SortContext.Provider value={{ sort, onSort }}>
      <div className="relative w-full overflow-auto rounded-md border border-border-subtle bg-surface" style={maxHeight ? { maxHeight } : undefined}>
        <table
          className={cn("w-full border-collapse text-base", dense ? "[&_td]:py-1 [&_td]:text-sm" : "[&_td]:py-1.5", stickyHeader && "[&_thead_th]:sticky [&_thead_th]:top-0 [&_thead_th]:z-10", className)}
          {...rest}
        >
          {children}
        </table>
      </div>
    </SortContext.Provider>
  );
}

function Head({ children, className, ...rest }: React.ComponentProps<"thead">) {
  return (
    <thead className={cn("bg-gray-50 text-sm text-fg-muted [&_th]:border-b [&_th]:border-border-subtle", className)} {...rest}>
      {children}
    </thead>
  );
}

function Body({ children, className, ...rest }: React.ComponentProps<"tbody">) {
  return (
    <tbody className={cn("[&_tr:not(:last-child)_td]:border-b [&_td]:border-border-subtle", className)} {...rest}>
      {children}
    </tbody>
  );
}

function Row({ children, className, selected, clickable, onClick, ...rest }: React.ComponentProps<"tr"> & { selected?: boolean; clickable?: boolean }) {
  const canClick = clickable || !!onClick;
  return (
    <tr
      className={cn("transition-colors", canClick && "cursor-pointer hover:bg-surface-hover focus-visible:bg-surface-hover outline-none", selected && "bg-brand-50", className)}
      aria-selected={selected || undefined}
      tabIndex={canClick ? 0 : undefined}
      onClick={onClick}
      onKeyDown={
        canClick && onClick
          ? (e) => {
              if ((e.key === "Enter" || e.key === " ") && e.target === e.currentTarget) {
                e.preventDefault();
                onClick(e as unknown as React.MouseEvent<HTMLTableRowElement>);
              }
            }
          : undefined
      }
      {...rest}
    >
      {children}
    </tr>
  );
}

function HeadCell({
  children,
  className,
  sortKey,
  align,
  width,
  ...rest
}: React.ComponentProps<"th"> & { sortKey?: string; align?: "left" | "right" | "center"; width?: string | number }) {
  const { sort, onSort } = React.useContext(SortContext);
  const active = sortKey && sort?.key === sortKey ? sort.dir : undefined;
  const alignCls = align === "right" ? "text-right" : align === "center" ? "text-center" : "text-left";
  return (
    <th
      scope="col"
      className={cn("h-9 whitespace-nowrap px-3 font-semibold", alignCls, className)}
      style={width ? { width } : undefined}
      aria-sort={sortKey ? (active === "asc" ? "ascending" : active === "desc" ? "descending" : "none") : undefined}
      {...rest}
    >
      {sortKey && onSort ? (
        <button
          type="button"
          className={cn("inline-flex h-7 max-w-full items-center gap-1 rounded px-1 -mx-1 hover:bg-gray-100 hover:text-fg outline-none focus-visible:ring-2 focus-visible:ring-ring/40", align === "right" && "flex-row-reverse", active && "text-fg-strong")}
          onClick={() => onSort(sortKey)}
        >
          <span className="truncate">{children}</span>
          <Icon name={active === "asc" ? "tangDan" : active === "desc" ? "giamDan" : "sapXep"} size={12} className={cn("flex-none", !active && "opacity-40")} />
        </button>
      ) : (
        children
      )}
    </th>
  );
}

function Cell({ children, className, align, ...rest }: React.ComponentProps<"td"> & { align?: "left" | "right" | "center" }) {
  return (
    <td className={cn("px-3 align-middle", align === "right" && "text-right tabular-nums", align === "center" && "text-center", className)} {...rest}>
      {children}
    </td>
  );
}

/** Hàng chiếm toàn bảng cho trạng thái rỗng / lỗi / tải. */
function FullRow({ colSpan, children, className }: { colSpan: number; children: React.ReactNode; className?: string }) {
  return (
    <tr>
      <td colSpan={colSpan} className={cn("p-0", className)}>
        {children}
      </td>
    </tr>
  );
}

// ---------- cấu hình
export interface Column<T> {
  key: string;
  header: React.ReactNode;
  cell?: (row: T, index: number) => React.ReactNode;
  align?: "left" | "right" | "center";
  width?: string | number;
  /** true → sắp theo `sortValue` hoặc `row[key]`. */
  sortable?: boolean;
  sortValue?: (row: T) => unknown;
  className?: string;
  /** Ẩn cột khi cửa sổ hẹp hơn (px). */
  hideBelow?: number;
}

export interface TableState {
  loading?: boolean;
  error?: unknown;
  onRetry?: () => void;
  empty?: { title?: React.ReactNode; description?: React.ReactNode; action?: React.ReactNode; icon?: React.ComponentProps<typeof EmptyState>["icon"] };
}

export interface TableProps<T> {
  columns: Column<T>[];
  rows: T[];
  getRowId: (row: T) => string;
  /** Sắp xếp do ngoài quản (máy chủ) hoặc để trống dùng useTableSort bên trong. */
  sort?: SortState | null;
  onSortChange?: (s: SortState | null) => void;
  clientSort?: boolean;
  selection?: { selected: ReadonlySet<string>; onChange: (next: Set<string>) => void; canSelect?: (row: T) => boolean };
  onRowClick?: (row: T) => void;
  rowClassName?: (row: T) => string | undefined;
  rowActions?: (row: T) => React.ReactNode;
  state?: TableState;
  pagination?: { page: number; pageSize: number; total: number; onChange: (p: number) => void; onPageSizeChange?: (s: number) => void };
  stickyHeader?: boolean;
  maxHeight?: string | number;
  dense?: boolean;
  className?: string;
  /** Hàng mở rộng dưới mỗi hàng chính. */
  renderExpanded?: (row: T) => React.ReactNode;
  caption?: string;
}

function useViewportWidth() {
  const [w, setW] = React.useState(() => (typeof window === "undefined" ? 1920 : window.innerWidth));
  React.useEffect(() => {
    const on = () => setW(window.innerWidth);
    window.addEventListener("resize", on);
    return () => window.removeEventListener("resize", on);
  }, []);
  return w;
}

function ConfigTable<T>(props: TableProps<T>) {
  const { columns, rows, getRowId, selection, onRowClick, rowClassName, rowActions, state, pagination, stickyHeader, maxHeight, dense, className, renderExpanded, caption } = props;
  const vw = useViewportWidth();
  const cols = React.useMemo(() => columns.filter((c) => !c.hideBelow || vw >= c.hideBelow), [columns, vw]);
  const getValue = React.useCallback(
    (row: T, key: string) => {
      const c = columns.find((x) => x.key === key);
      if (c?.sortValue) return c.sortValue(row);
      return (row as Record<string, unknown>)[key];
    },
    [columns],
  );
  const clientSort = props.clientSort ?? !props.onSortChange;
  const internal = useTableSort(rows, getValue, props.sort ?? null);
  const sort = props.onSortChange ? (props.sort ?? null) : internal.sort;
  const onSort = props.onSortChange
    ? (key: string) => props.onSortChange!(sort?.key === key ? (sort.dir === "asc" ? { key, dir: "desc" } : null) : { key, dir: "asc" })
    : internal.onSort;
  const shown = clientSort && !props.onSortChange ? internal.sorted : rows;
  const span = cols.length + (selection ? 1 : 0) + (rowActions ? 1 : 0);
  const selectable = selection ? shown.filter((r) => selection.canSelect?.(r) ?? true) : [];
  const allSelected = selectable.length > 0 && selectable.every((r) => selection!.selected.has(getRowId(r)));
  const someSelected = selectable.some((r) => selection!.selected.has(getRowId(r)));
  const firstLoad = state?.loading && rows.length === 0;

  return (
    <div className={cn("space-y-2", className)}>
      {state?.error !== undefined && state.error !== null && rows.length > 0 && <ErrorBanner error={state.error} onRetry={state.onRetry} />}
      <Root sort={sort} onSort={onSort} stickyHeader={stickyHeader} maxHeight={maxHeight} dense={dense} aria-busy={state?.loading || undefined}>
        {caption && <caption className="sr-only">{caption}</caption>}
        <Head>
          <tr>
            {selection && (
              <th scope="col" className="h-9 w-9 px-2">
                <Checkbox
                  checked={allSelected}
                  onChange={(v) => {
                    const next = new Set(selection.selected);
                    for (const r of selectable) {
                      if (v) next.add(getRowId(r));
                      else next.delete(getRowId(r));
                    }
                    selection.onChange(next);
                  }}
                  label={allSelected || someSelected ? "Bỏ chọn tất cả" : "Chọn tất cả"}
                />
              </th>
            )}
            {cols.map((c) => (
              <HeadCell key={c.key} sortKey={c.sortable ? c.key : undefined} align={c.align} width={c.width} className={c.className}>
                {c.header}
              </HeadCell>
            ))}
            {rowActions && <th scope="col" className="h-9 w-px px-2" />}
          </tr>
        </Head>
        <Body>
          {firstLoad ? (
            <FullRow colSpan={span}>
              <Skeleton.Rows rows={5} cols={Math.min(cols.length, 5)} />
            </FullRow>
          ) : state?.error !== undefined && state.error !== null && rows.length === 0 ? (
            <FullRow colSpan={span} className="p-3">
              <ErrorBanner error={state.error} onRetry={state.onRetry} />
            </FullRow>
          ) : shown.length === 0 ? (
            <FullRow colSpan={span}>
              <EmptyState compact icon={state?.empty?.icon} title={state?.empty?.title} description={state?.empty?.description} action={state?.empty?.action} />
            </FullRow>
          ) : (
            shown.map((row, i) => {
              const id = getRowId(row);
              const sel = selection?.selected.has(id) ?? false;
              return (
                <React.Fragment key={id}>
                  <Row selected={sel} onClick={onRowClick ? () => onRowClick(row) : undefined} className={rowClassName?.(row)}>
                    {selection && (
                      <Cell className="w-9 px-2" onClick={(e) => e.stopPropagation()}>
                        {(selection.canSelect?.(row) ?? true) && (
                          <Checkbox
                            checked={sel}
                            onChange={(v) => {
                              const next = new Set(selection.selected);
                              if (v) next.add(id);
                              else next.delete(id);
                              selection.onChange(next);
                            }}
                            label="Chọn dòng"
                          />
                        )}
                      </Cell>
                    )}
                    {cols.map((c) => (
                      <Cell key={c.key} align={c.align} className={c.className}>
                        {c.cell ? c.cell(row, i) : ((row as Record<string, unknown>)[c.key] as React.ReactNode)}
                      </Cell>
                    ))}
                    {rowActions && (
                      <Cell className="w-px whitespace-nowrap px-2" onClick={(e) => e.stopPropagation()}>
                        <div className="flex items-center justify-end gap-0.5">{rowActions(row)}</div>
                      </Cell>
                    )}
                  </Row>
                  {renderExpanded && (
                    <tr>
                      <td colSpan={span} className="bg-gray-25 p-0">
                        {renderExpanded(row)}
                      </td>
                    </tr>
                  )}
                </React.Fragment>
              );
            })
          )}
        </Body>
      </Root>
      {pagination && <Pagination {...pagination} />}
    </div>
  );
}

type TableComponent = (<T>(props: TableProps<T>) => React.ReactElement) & {
  Root: typeof Root;
  Head: typeof Head;
  Body: typeof Body;
  Row: typeof Row;
  HeadCell: typeof HeadCell;
  Cell: typeof Cell;
  FullRow: typeof FullRow;
};

const Table = ConfigTable as TableComponent;
Table.Root = Root;
Table.Head = Head;
Table.Body = Body;
Table.Row = Row;
Table.HeadCell = HeadCell;
Table.Cell = Cell;
Table.FullRow = FullRow;

export default Table;
