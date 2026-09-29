// Copyright (c) 2026 DigiText. All rights reserved.
// Kho yêu cầu hộp thoại xác nhận / nhập nhanh, ở mức module để gọi được từ ngoài cây React
// (state/navigation, api/*). <ConfirmHost/> lắng nghe và hiển thị; mỗi lúc một hộp, hàng đợi tuần tự.

export interface ConfirmOptions {
  title: string;
  body?: string;
  confirmLabel?: string;
  cancelLabel?: string;
  /** Nút xác nhận màu đỏ (xoá, huỷ). */
  danger?: boolean;
  /** Bắt gõ đúng chuỗi này mới cho xác nhận (thao tác không hoàn tác được). */
  typeToConfirm?: string;
}

export interface PromptOptions {
  title: string;
  label?: string;
  body?: string;
  type?: "text" | "textarea" | "date" | "number" | "select";
  options?: { value: string; label: string }[];
  initial?: string;
  placeholder?: string;
  required?: boolean;
  /** Trả chuỗi lỗi nếu không hợp lệ. */
  validate?: (v: string) => string | null;
  confirmLabel?: string;
  cancelLabel?: string;
  danger?: boolean;
}

export type Request =
  | { kind: "confirm"; id: number; opts: ConfirmOptions; resolve: (ok: boolean) => void }
  | { kind: "prompt"; id: number; opts: PromptOptions; resolve: (v: string | null) => void };

type Listener = (current: Request | null) => void;

const queue: Request[] = [];
let listener: Listener | null = null;
let seq = 0;

function notify() {
  listener?.(queue[0] ?? null);
}

export function subscribe(l: Listener): () => void {
  listener = l;
  notify();
  return () => {
    if (listener === l) listener = null;
  };
}

export function finish(id: number, value: unknown) {
  const i = queue.findIndex((r) => r.id === id);
  if (i < 0) return;
  const [r] = queue.splice(i, 1);
  if (r.kind === "confirm") r.resolve(!!value);
  else r.resolve(value === null || value === undefined ? null : String(value));
  notify();
}

/** Hộp xác nhận. Khi chưa có <ConfirmHost/> (ví dụ lúc khởi động) thì lùi về window.confirm. */
export function confirm(opts: ConfirmOptions | string): Promise<boolean> {
  const o = typeof opts === "string" ? { title: opts } : opts;
  if (!listener) return Promise.resolve(window.confirm(o.body ? `${o.title}\n\n${o.body}` : o.title));
  return new Promise((resolve) => {
    queue.push({ kind: "confirm", id: ++seq, opts: o, resolve });
    if (queue.length === 1) notify();
  });
}

/** Hộp nhập một giá trị. Trả null khi huỷ. */
export function prompt(opts: PromptOptions | string): Promise<string | null> {
  const o = typeof opts === "string" ? { title: opts } : opts;
  if (!listener) return Promise.resolve(window.prompt(o.title, o.initial ?? ""));
  return new Promise((resolve) => {
    queue.push({ kind: "prompt", id: ++seq, opts: o, resolve });
    if (queue.length === 1) notify();
  });
}
