// Copyright (c) 2026 DigiText. All rights reserved.
// Dải dây chuyền: các bước xử lý của một tài liệu / hồ sơ, đọc từ trái sang phải (Quét → OCR → PDF/A → Ký số…).
// Bước xong: xanh ngọc; bước đang làm: hổ phách, tô theo phần trăm; bước chưa tới: xám; bước lỗi: đỏ.
// Dòng chữ dưới dải nói bước hiện tại — nhìn một dòng biết hồ sơ đang ở đâu, kẹt ở đâu.

import type { ReactNode } from "react";
import { V462 as cn } from "@/lib/utils";

export interface PipelineStep {
  label: string;
  /** Số đã xong / tổng. total = 0 → bước chưa áp dụng (xám). */
  done: number;
  total: number;
  /** Đơn vị cho dòng mô tả: "trang", "tài liệu"… */
  unit?: string;
  /** Bước có lỗi cần xử lý (đỏ), kèm câu hướng dẫn. */
  error?: string;
}

type State = "done" | "run" | "wait" | "err";

const fmt = (n: number) => n.toLocaleString("vi-VN");

function stateOf(s: PipelineStep, prevDone: boolean): State {
  if (s.error) return "err";
  if (s.total > 0 && s.done >= s.total) return "done";
  if (s.done > 0 || prevDone) return "run";
  return "wait";
}

export default function Pipeline({ steps, doneText = "Đã xong", className, extra }: { steps: PipelineStep[]; doneText?: string; className?: string; extra?: ReactNode }) {
  const states: State[] = [];
  steps.forEach((s, i) => states.push(stateOf(s, i === 0 || states[i - 1] === "done")));
  const cur = states.findIndex((st) => st !== "done");
  const s = cur >= 0 ? steps[cur] : null;
  const st = cur >= 0 ? states[cur] : "done";
  const note = !s
    ? doneText
    : st === "err"
      ? s.error!
      : s.done > 0
        ? `${s.label} ${fmt(s.done)}/${fmt(s.total)}${s.unit ? ` ${s.unit}` : ""}`
        : `Chờ ${/^[A-ZĐ][a-zà-ỹ]/.test(s.label) ? s.label[0].toLowerCase() + s.label.slice(1) : s.label}`;
  const label = steps.map((x, i) => `${x.label}: ${states[i] === "done" ? "xong" : states[i] === "err" ? "lỗi" : `${fmt(x.done)}/${fmt(x.total)}`}`).join("; ");

  return (
    <div className={cn("min-w-[12rem]", className)}>
      <div className="grid gap-[3px]" style={{ gridTemplateColumns: `repeat(${steps.length}, minmax(0, 1fr))` }} role="img" aria-label={label}>
        {steps.map((x, i) => {
          const p = x.total > 0 ? Math.min(100, Math.floor((x.done / x.total) * 100)) : 0;
          const k = states[i];
          return (
            <span
              key={x.label}
              title={`${x.label}: ${x.total > 0 ? `${fmt(x.done)}/${fmt(x.total)}${x.unit ? ` ${x.unit}` : ""}` : "—"}`}
              className={cn("h-1.5 rounded-sm", k === "done" ? "bg-brand-500" : k === "err" ? "bg-danger-600" : k === "run" ? "bg-run-200" : "bg-gray-200")}
              style={k === "run" && p > 0 ? { background: `linear-gradient(90deg, var(--color-run-solid) ${p}%, var(--color-run-200) ${p}%)` } : undefined}
            />
          );
        })}
      </div>
      <div className={cn("mt-1 text-xs", st === "done" ? "text-brand-700" : st === "err" ? "font-medium text-danger-700" : st === "run" ? "font-medium text-run" : "text-fg-subtle")}>{note}</div>
      {extra}
    </div>
  );
}
