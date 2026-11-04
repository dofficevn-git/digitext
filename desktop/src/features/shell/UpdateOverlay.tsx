// Copyright (c) 2026 DigiText. All rights reserved.
// Hộp thoại cập nhật phần mềm: không đóng được khi đang tải / cài; đóng được khi đã mới nhất hoặc lỗi.

import type { V42t } from "@/api/updater";
import { Button, describeError, Dialog } from "@/ui";
import { V3wu } from "@/config";

const mb = (n: number) => (n / 1_000_000).toFixed(1).replace(".", ",");

export default function V4x7({ status, onDismiss }: { status: V42t; onDismiss: () => void }) {
  if (status.phase === "idle") return null;
  if (status.phase === "checking" && !status.manual) return null;
  if (status.phase === "uptodate" && !status.manual) return null;

  if (status.phase === "uptodate") {
    return (
      <Dialog open onOpenChange={(o) => !o && onDismiss()} title="Đã là bản mới nhất" description={`${V3wu.appName} v${status.version ?? "?"}`} size="sm" footer={<Button variant="primary" onClick={onDismiss}>Đóng</Button>} />
    );
  }

  if (status.phase === "error") {
    const d = describeError(status.error, "kiểm tra cập nhật");
    return (
      <Dialog open onOpenChange={(o) => !o && onDismiss()} title="Không kiểm tra được cập nhật" size="sm" footer={<Button variant="primary" onClick={onDismiss}>Bỏ qua và tiếp tục</Button>}>
        <p className="text-sm">{d.message}</p>
        {d.detail && <pre className="mt-2 max-h-32 overflow-auto whitespace-pre-wrap break-all rounded bg-gray-50 p-2 font-mono text-xs text-fg-muted">{d.detail}</pre>}
      </Dialog>
    );
  }

  const dangTai = status.phase === "downloading";
  const pt = dangTai && status.total > 0 ? Math.round((status.downloaded / status.total) * 100) : null;
  const text = status.phase === "checking" ? "Đang kiểm tra cập nhật…" : status.phase === "available" ? "Đang chuẩn bị cập nhật…" : dangTai ? `Đang tải bản ${status.version}…${pt != null ? ` ${pt}%` : ""}` : status.phase === "installing" ? "Đang cài đặt bản mới…" : "Đang khởi động lại…";

  return (
    <Dialog open onOpenChange={() => {}} closable={false} title={`Cập nhật ${V3wu.appName}`} size="sm">
      <p className="text-sm text-fg-muted">{text}</p>
      <div className="mt-3 h-2 overflow-hidden rounded bg-border-subtle" role="progressbar" aria-label="Tiến độ cập nhật" aria-valuemin={0} aria-valuemax={100} aria-valuenow={pt ?? undefined}>
        <div className={`h-2 rounded bg-accent ${pt == null ? "animate-pulse" : "transition-[width]"}`} style={{ width: pt == null ? "100%" : `${pt}%` }} />
      </div>
      {dangTai && status.total > 0 && (
        <p className="mt-2 text-xs tabular-nums text-fg-subtle">
          {mb(status.downloaded)} / {mb(status.total)} MB
        </p>
      )}
    </Dialog>
  );
}
