// Copyright (c) 2026 DigiText. All rights reserved.
// Toast (sonner): kết quả của thao tác vừa bấm. Gọi được từ ngoài cây React.
//   toast.success("Đã lưu hồ sơ.")  ·  toast.error("Không lưu được.", { detail })  ·  notifyError(e, "lưu hồ sơ")
//   toast.promise(p, { loading, success, error })

import { Toaster as SonnerToaster, toast as sonner } from "sonner";
import { Icon } from "@/components/Icon";
import { describeError } from "@/lib/errors";

const SUCCESS_MS = 4000;
const ERROR_MS = 8000;

export const toast = {
  success(message: string, opts?: { description?: string; action?: { label: string; onClick: () => void } }) {
    return sonner.success(message, { description: opts?.description, duration: SUCCESS_MS, action: opts?.action });
  },
  info(message: string, opts?: { description?: string }) {
    return sonner(message, { description: opts?.description, duration: SUCCESS_MS });
  },
  warning(message: string, opts?: { description?: string }) {
    return sonner.warning(message, { description: opts?.description, duration: ERROR_MS });
  },
  error(message: string, opts?: { detail?: string; action?: { label: string; onClick: () => void } }) {
    return sonner.error(message, {
      description: opts?.detail,
      duration: ERROR_MS,
      action: opts?.action,
      classNames: { description: "font-mono text-xs break-all" },
    });
  },
  promise<T>(p: Promise<T>, msgs: { loading: string; success: string | ((v: T) => string); error?: string | ((e: unknown) => string) }) {
    return sonner.promise(p, {
      loading: msgs.loading,
      success: msgs.success,
      error: (e: unknown) => (typeof msgs.error === "function" ? msgs.error(e) : (msgs.error ?? describeError(e).message)),
      duration: SUCCESS_MS,
    });
  },
  dismiss: sonner.dismiss,
};

/** Báo lỗi từ ngoại lệ: câu người dùng + chi tiết kỹ thuật (mô tả nhỏ). `ctx` = "lưu hồ sơ" → "Không lưu hồ sơ được." */
export function notifyError(e: unknown, ctx?: string, retry?: () => void) {
  const d = describeError(e, ctx);
  return toast.error(d.message, { detail: d.detail, action: retry ? { label: "Thử lại", onClick: retry } : undefined });
}

/** Gắn một lần ở gốc ứng dụng. */
export function Toaster() {
  return (
    <SonnerToaster
      position="bottom-right"
      offset={{ bottom: "calc(var(--size-statusbar) + 12px)", right: 16 }}
      gap={8}
      visibleToasts={4}
      closeButton
      icons={{
        success: <Icon name="thanhCong" size={16} className="text-success-700" />,
        error: <Icon name="loi" size={16} className="text-danger-700" />,
        warning: <Icon name="canhBao" size={16} className="text-warning-700" />,
        info: <Icon name="thongTin" size={16} className="text-info-700" />,
      }}
      toastOptions={{
        unstyled: true,
        classNames: {
          toast: "flex w-[360px] items-start gap-2.5 rounded-md border border-border bg-surface px-3 py-2.5 text-base text-fg shadow-lg",
          title: "font-medium leading-snug",
          description: "text-sm text-fg-subtle mt-0.5",
          icon: "mt-0.5 flex-none",
          content: "min-w-0 flex-1",
          actionButton: "ml-auto flex-none rounded border border-border bg-surface px-2 py-1 text-sm font-medium hover:bg-surface-hover",
          closeButton: "absolute -top-2 -left-2 grid size-5 place-items-center rounded-full border border-border bg-surface text-fg-subtle hover:text-fg",
          error: "border-danger-200",
          success: "border-success-200",
          warning: "border-warning-200",
        },
      }}
    />
  );
}
