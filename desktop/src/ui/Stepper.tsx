// Copyright (c) 2026 DigiText. All rights reserved.
// Dãy bước ngang: xong (✓) · đang làm · chưa tới. Bước có `onClick` thì bấm được (quay lại bước đã xong).

import { Icon } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";

export interface StepItem {
  key: string;
  label: string;
  /** Chú thích ngắn dưới nhãn. */
  hint?: string;
}

export default function Stepper({
  steps,
  current,
  onSelect,
  className,
}: {
  steps: StepItem[];
  /** Chỉ số bước đang làm (0-based). Bước < current coi là xong. */
  current: number;
  onSelect?: (index: number) => void;
  className?: string;
}) {
  return (
    <ol className={cn("flex flex-wrap items-start gap-x-2 gap-y-2", className)} aria-label="Các bước">
      {steps.map((s, i) => {
        const done = i < current;
        const active = i === current;
        const clickable = !!onSelect && (done || active);
        const inner = (
          <>
            <span
              className={cn(
                "grid size-6 flex-none place-items-center rounded-full border text-xs font-semibold tabular-nums",
                done && "border-brand-500 bg-brand-500 text-white",
                active && "border-brand-500 text-brand-600",
                !done && !active && "border-border text-fg-subtle",
              )}
              aria-hidden
            >
              {done ? <Icon name="duyet" size={13} /> : i + 1}
            </span>
            <span className="min-w-0">
              <span className={cn("block text-sm font-medium", active ? "text-fg-strong" : done ? "text-fg" : "text-fg-subtle")}>{s.label}</span>
              {s.hint && <span className="block text-xs text-fg-subtle">{s.hint}</span>}
            </span>
          </>
        );
        return (
          <li key={s.key} className="flex items-center gap-2" aria-current={active ? "step" : undefined}>
            {clickable ? (
              <button type="button" className="flex items-center gap-2 rounded-md px-1 py-0.5 text-left hover:bg-surface-hover" onClick={() => onSelect(i)}>
                {inner}
              </button>
            ) : (
              <span className="flex items-center gap-2 px-1 py-0.5">{inner}</span>
            )}
            {i < steps.length - 1 && <span className="mx-1 h-px w-6 flex-none bg-border" aria-hidden />}
          </li>
        );
      })}
    </ol>
  );
}
