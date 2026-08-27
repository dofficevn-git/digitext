// Copyright (c) 2026 DigiText. All rights reserved.

import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { V462 } from "@/lib/utils";

interface V4op<T extends string> {
  value: T;
  label: string;
}

export default function V4oq<T extends string>({
  options: V234,
  value: V235,
  onChange: V236,
  full,
}: {
  options: V4op<T>[];
  value: T;
  onChange: (v: T) => void;
  full?: boolean;
}) {
  return (
    <ToggleGroup
      type="single"
      value={V235}
      onValueChange={(v) => {
        if (v) V236(v as T);
      }}
      className={V462(
        "inline-flex divide-x divide-border-subtle overflow-hidden rounded border border-border bg-surface",
        full && "w-full",
      )}
    >
      {V234.map((o) => (
        <ToggleGroupItem
          key={o.value}
          value={o.value}
          aria-label={o.label}
          className={V462(
            "h-[30px] rounded-none px-3 text-sm font-medium text-fg-muted transition-colors",
            "hover:bg-gray-50 hover:text-fg",
            "data-[state=on]:bg-brand-500 data-[state=on]:text-white",
            full && "flex-1",
          )}
        >
          {o.label}
        </ToggleGroupItem>
      ))}
    </ToggleGroup>
  );
}
