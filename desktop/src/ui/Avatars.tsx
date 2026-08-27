// Copyright (c) 2026 DigiText. All rights reserved.

import { V462 } from "@/lib/utils";

const V47v = [
  "bg-brand-500",
  "bg-success-700",
  "bg-warning-700",
  "bg-info-700",
  "bg-gray-700",
] as const;

export default function V47w({ members: Vft = [] }: { members?: string[] }) {
  const Vfu = Vft.slice(0, 4);
  return (
    <span className="inline-flex" role="list">
      {Vfu.map((m, i) => (
        <span
          key={i}
          role="listitem"
          className={V462(
            "grid size-[22px] place-items-center rounded-full border-2 border-surface text-2xs font-semibold text-white -ml-1.5 first:ml-0",
            m.startsWith("+") ? "bg-gray-500" : V47v[i % V47v.length],
          )}
        >
          {m}
        </span>
      ))}
    </span>
  );
}
