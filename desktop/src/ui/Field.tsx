// Copyright (c) 2026 DigiText. All rights reserved.
// Ô nhập có nhãn: sinh id, nối nhãn ↔ control, gợi ý và lỗi qua aria-describedby / aria-invalid.
// Con (Input / Select / Textarea / Checkbox của ui/*) tự đọc FieldContext; control lạ thì dùng `useFieldProps()`.

import { createContext, useContext, useId, type ReactNode } from "react";
import { V462 as cn } from "@/lib/utils";

interface FieldCtx {
  id: string;
  describedBy?: string;
  invalid: boolean;
  required: boolean;
}

const Ctx = createContext<FieldCtx | null>(null);

/** Thuộc tính cần gắn vào control bên trong Field (id, aria-*). */
export function useFieldProps(): { id?: string; "aria-describedby"?: string; "aria-invalid"?: true; "aria-required"?: true } {
  const c = useContext(Ctx);
  if (!c) return {};
  return {
    id: c.id,
    "aria-describedby": c.describedBy,
    "aria-invalid": c.invalid || undefined,
    "aria-required": c.required || undefined,
  };
}

export interface FieldProps {
  label: ReactNode;
  children: ReactNode;
  hint?: ReactNode;
  error?: ReactNode;
  required?: boolean;
  /** Nhãn nằm bên phải control (checkbox / switch). */
  inline?: boolean;
  className?: string;
  /** id riêng (mặc định tự sinh). */
  id?: string;
}

export default function Field({ label, children, hint, error, required, inline, className, id: idProp }: FieldProps) {
  const gen = useId();
  const id = idProp ?? `f-${gen}`;
  const hintId = hint ? `${id}-hint` : undefined;
  const errId = error ? `${id}-err` : undefined;
  const describedBy = [errId, hintId].filter(Boolean).join(" ") || undefined;
  const ctx: FieldCtx = { id, describedBy, invalid: !!error, required: !!required };
  const labelEl = (
    <label htmlFor={id} className={cn("text-sm font-medium text-fg-muted", inline ? "cursor-pointer select-none" : "block mb-1")}>
      {label}
      {required && (
        <span className="ml-0.5 text-danger-700" aria-hidden>
          *
        </span>
      )}
    </label>
  );
  return (
    <Ctx.Provider value={ctx}>
      <div className={cn("min-w-0", className)}>
        {inline ? (
          <div className="flex items-center gap-2 min-h-7">
            {children}
            {labelEl}
          </div>
        ) : (
          <>
            {labelEl}
            {children}
          </>
        )}
        {error && (
          <p id={errId} className="mt-1 text-sm text-danger-700" role="alert">
            {error}
          </p>
        )}
        {hint && !error && (
          <p id={hintId} className="mt-1 text-sm text-fg-subtle">
            {hint}
          </p>
        )}
      </div>
    </Ctx.Provider>
  );
}

/** Lưới form: cột tự xếp ≥ 220px. */
export function FormGrid({ children, className, cols }: { children: ReactNode; className?: string; cols?: 2 | 3 | 4 }) {
  return (
    <div
      className={cn("grid gap-3", className)}
      style={{ gridTemplateColumns: cols ? `repeat(${cols}, minmax(0, 1fr))` : "repeat(auto-fit, minmax(220px, 1fr))" }}
    >
      {children}
    </div>
  );
}
