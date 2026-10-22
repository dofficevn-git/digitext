// Copyright (c) 2026 DigiText. All rights reserved.
// Dùng chung cho ba dashboard của Tổng quan: nạp số liệu giữ số cũ khi lỗi, thẻ tự nạp, huy hiệu tình trạng dự án
// (icon + chữ, không chỉ màu), chọn số ngày, câu "dự kiến xong".

import type { ReactNode } from "react";
import { isTauri } from "@/api/ipc";
import type { Health, Step } from "@/api/dashboard";
import { V41t } from "@/live/useLive";
import type { V3xa } from "@/live/topics";
import { Icon, type V41v as IconName } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";
import { Badge, Button, Card, ErrorBanner, Seg, Skeleton, describeError, fmtDayFull } from "@/ui";

export const n = (v: number) => v.toLocaleString("vi-VN");
export const pct = (a: number, b: number) => (b > 0 ? Math.min(100, Math.round((a / b) * 100)) : 0);
export const pct1 = (v: number | null | undefined) => (v == null ? "—" : `${Math.round(v)} %`);
export type Tone = "ok" | "warn" | "err" | "info";
export const TONE_TEXT: Record<Tone, string> = { ok: "text-success-700", warn: "text-warning-700", err: "text-danger-700", info: "text-info-700" };
export const TONE_BG: Record<Tone, string> = { ok: "bg-success-50 text-success-700", warn: "bg-warning-50 text-warning-700", err: "bg-danger-50 text-danger-700", info: "bg-info-50 text-info-700" };

/** Chạy được trong app lẫn bản xem trước dev (devMock). */
export const coNguon = () => isTauri() || import.meta.env.DEV;

/** Nạp số liệu cho một khối: chỉ chạy khi có quyền, không xoá số cũ khi lỗi. */
export function useSo<T>(load: () => Promise<T>, topics: V3xa[], enabled: boolean, pollMs = 60_000) {
  return V41t(load, { topics, enabled: enabled && coNguon(), pollMs, errorText: (e) => describeError(e).message });
}

/** Thẻ có tiêu đề, tự nạp: skeleton lần đầu, lỗi gọn, giữ dữ liệu cũ. */
export function DataCard<T>({ title, icon, data, error, loading, reload, open, openLabel = "Mở", actions, description, children, className, padding }: { title: ReactNode; icon?: IconName; data: T | undefined; error: unknown; loading: boolean; reload: () => void; open?: () => void; openLabel?: string; actions?: ReactNode; description?: ReactNode; children: (d: T) => ReactNode; className?: string; padding?: "md" | "sm" | "none" }) {
  return (
    <Card
      className={cn("flex flex-col", className)}
      padding={padding}
      description={description}
      title={
        <span className="flex items-center gap-2">
          {icon && <Icon name={icon} size={16} className="text-fg-subtle" />}
          {title}
        </span>
      }
      actions={
        <span className="flex items-center gap-2">
          {actions}
          {open && (
            <Button size="sm" variant="ghost" iconRight="sau" onClick={open}>
              {openLabel}
            </Button>
          )}
        </span>
      }
    >
      {loading && data === undefined ? <Skeleton.Rows rows={3} cols={1} /> : data === undefined ? <ErrorBanner error={error ?? "Không nạp được."} onRetry={reload} /> : children(data)}
      {data !== undefined && error ? <p className="mt-2 text-xs text-warning-700">Không cập nhật được, đang hiện số liệu cũ.</p> : null}
    </Card>
  );
}

/** Tình trạng dự án: màu + icon + chữ (đọc được cả khi in đen trắng). */
export function RagBadge({ rag, className }: { rag: Health["rag"]; className?: string }) {
  const m = rag === "green" ? { tone: "ok" as const, icon: "thanhCong" as const, text: "Đúng tiến độ" } : rag === "amber" ? { tone: "warn" as const, icon: "canhBao" as const, text: "Có rủi ro" } : rag === "red" ? { tone: "err" as const, icon: "loi" as const, text: "Chậm tiến độ" } : { tone: "gray" as const, icon: "thongTin" as const, text: "Chưa có hợp đồng" };
  return (
    <Badge tone={m.tone} className={className}>
      <Icon name={m.icon} size={12} className="mr-1" />
      {m.text}
    </Badge>
  );
}

export const STEP_LABEL: Record<Step, string> = { scan: "quét", pdfa: "PDF/A", sign: "ký số", review: "duyệt" };

/** "Dự kiến xong 14/03/2027 · sớm 5 ngày" / "· chậm 12 ngày so với hạn" / "Chưa đủ số liệu để dự kiến". */
export function duKienText(h: Health, deadline: string | null): string {
  if (!h.projectedFinishOn) return h.progressPct != null && h.progressPct >= 100 ? "Đã hoàn thành khối lượng hợp đồng" : "Chưa đủ số liệu để dự kiến";
  let s = `Dự kiến xong ${fmtDayFull(h.projectedFinishOn)}`;
  if (deadline) {
    const diff = Math.round((new Date(`${h.projectedFinishOn}T00:00:00`).getTime() - new Date(`${deadline}T00:00:00`).getTime()) / 86_400_000);
    s += diff > 0 ? ` · chậm ${n(diff)} ngày so với hạn` : diff < 0 ? ` · sớm ${n(-diff)} ngày` : " · đúng hạn";
  }
  if (h.bottleneckStep) s += ` · bước chậm nhất: ${STEP_LABEL[h.bottleneckStep]}`;
  return s;
}

export type Days = 14 | 30 | 90;
export function DaysSeg({ value, onChange }: { value: Days; onChange: (d: Days) => void }) {
  return (
    <Seg<`${Days}`>
      value={`${value}`}
      onChange={(v) => onChange(Number(v) as Days)}
      options={[
        { value: "14", label: "14 ngày" },
        { value: "30", label: "30 ngày" },
        { value: "90", label: "90 ngày" },
      ]}
    />
  );
}

/** Dòng "label · số" bấm được trong danh sách việc cần xử lý. */
export function DongViec({ label, value, tone = "warn", onClick }: { label: string; value: number; tone?: Tone; onClick: () => void }) {
  return (
    <li>
      <button type="button" className="flex w-full items-center gap-2 rounded px-1 py-1.5 text-left text-sm hover:bg-surface-hover" onClick={onClick}>
        <span className="flex-1 text-fg">{label}</span>
        <span className={cn("min-w-7 rounded-full px-2 text-center text-xs font-semibold tabular-nums leading-5", value > 0 ? TONE_BG[tone] : "bg-gray-100 text-fg-subtle")}>{n(value)}</span>
        <Icon name="sau" size={13} className="text-fg-subtle" />
      </button>
    </li>
  );
}

export function DongTrangThai({ tone, label, value, sub }: { tone: Tone; label: string; value: ReactNode; sub?: ReactNode }) {
  const dot = tone === "ok" ? "bg-success-600" : tone === "warn" ? "bg-warning-600" : tone === "err" ? "bg-danger-600" : "bg-info-600";
  return (
    <li className="flex items-center gap-3 py-2">
      <span className={cn("size-2.5 flex-none rounded-full", dot)} aria-hidden />
      <span className="flex-1 text-sm text-fg">{label}</span>
      <span className={cn("text-sm font-semibold tabular-nums", TONE_TEXT[tone])}>{value}</span>
      {sub && <span className="text-xs text-fg-subtle">{sub}</span>}
    </li>
  );
}

/** Phần trăm thay đổi so với kỳ trước: "+8 % so tuần trước". */
export function soVoiKyTruoc(cur: number, prev: number, ten = "tuần trước"): string {
  if (prev <= 0) return cur > 0 ? `mới có từ ${ten}` : `không đổi so ${ten}`;
  const d = Math.round(((cur - prev) / prev) * 100);
  return `${d > 0 ? "+" : ""}${d} % so ${ten}`;
}
