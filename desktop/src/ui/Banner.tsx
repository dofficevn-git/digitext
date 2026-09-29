// Copyright (c) 2026 DigiText. All rights reserved.
// Dải thông báo trong màn (trạng thái kéo dài: lỗi nạp, ngoại tuyến, tóm tắt lỗi form). Có phần "Chi tiết" gập.

import { useState, type ReactNode } from "react";
import { Icon, type V41v as IconName } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";
import { describeError } from "@/lib/errors";
import IconButton from "./IconButton";

export type BannerTone = "info" | "success" | "warning" | "danger";

const STYLE: Record<BannerTone, { box: string; icon: IconName }> = {
  info: { box: "border-info-200 bg-info-50 text-info-800", icon: "thongTin" },
  success: { box: "border-success-200 bg-success-50 text-success-800", icon: "thanhCong" },
  warning: { box: "border-warning-200 bg-warning-50 text-warning-800", icon: "canhBao" },
  danger: { box: "border-danger-200 bg-danger-50 text-danger-800", icon: "loi" },
};

export interface BannerProps {
  tone?: BannerTone;
  title?: ReactNode;
  children?: ReactNode;
  actions?: ReactNode;
  onClose?: () => void;
  /** Phần kỹ thuật, ẩn sau nút "Chi tiết". */
  detail?: string;
  className?: string;
  compact?: boolean;
}

export default function Banner({ tone = "info", title, children, actions, onClose, detail, className, compact }: BannerProps) {
  const [showDetail, setShowDetail] = useState(false);
  const st = STYLE[tone];
  return (
    <div
      className={cn("flex items-start gap-2.5 rounded-md border text-sm", compact ? "px-2.5 py-1.5" : "px-3 py-2.5", st.box, className)}
      role={tone === "danger" ? "alert" : "status"}
    >
      <Icon name={st.icon} size={16} className="mt-0.5 flex-none" />
      <div className="min-w-0 flex-1">
        {title && <p className="font-semibold">{title}</p>}
        {children && <div className={title ? "mt-0.5" : undefined}>{children}</div>}
        {detail && (
          <div className="mt-1">
            <button type="button" className="text-xs underline underline-offset-2 opacity-80 hover:opacity-100" onClick={() => setShowDetail((v) => !v)}>
              {showDetail ? "Ẩn chi tiết" : "Chi tiết"}
            </button>
            {showDetail && <pre className="mt-1 max-h-40 overflow-auto whitespace-pre-wrap break-all rounded bg-white/60 p-2 font-mono text-xs text-fg">{detail}</pre>}
          </div>
        )}
      </div>
      {actions && <div className="flex flex-none items-center gap-1.5">{actions}</div>}
      {onClose && <IconButton label="Đóng" icon="dong" size="sm" onClick={onClose} className="-mr-1 -mt-0.5 text-current hover:bg-black/5" noTooltip />}
    </div>
  );
}

/** Banner lỗi từ một ngoại lệ bất kỳ: câu người dùng + chi tiết kỹ thuật gập. */
export function ErrorBanner({ error, ctx, onRetry, onClose, className }: { error: unknown; ctx?: string; onRetry?: () => void; onClose?: () => void; className?: string }) {
  if (error === null || error === undefined || error === "") return null;
  const d = describeError(error, ctx);
  return (
    <Banner
      tone="danger"
      detail={d.detail}
      onClose={onClose}
      className={className}
      actions={
        onRetry ? (
          <button type="button" className="rounded border border-current/30 px-2 py-0.5 text-xs font-medium hover:bg-white/50" onClick={onRetry}>
            Thử lại
          </button>
        ) : undefined
      }
    >
      {d.message}
    </Banner>
  );
}
