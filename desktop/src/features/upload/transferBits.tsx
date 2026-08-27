// Copyright (c) 2026 DigiText. All rights reserved.

import { Icon } from "@/components/Icon";

export type V4wi = "done" | "skipped" | "error" | "cancelled";

export function V4wj({ pct: V35h, label: V35i }: { pct: number; label: string }) {
  const r = 11;
  const V35j = 2 * Math.PI * r;
  return (
    <svg
      className="uw-ring"
      width={28}
      height={28}
      viewBox="0 0 28 28"
      role="img"
      aria-label={`${V35i} ${V35h}%`}
    >
      <circle className="uw-ring-track" cx={14} cy={14} r={r} fill="none" strokeWidth={3} />
      <circle
        className="uw-ring-arc"
        cx={14}
        cy={14}
        r={r}
        fill="none"
        strokeWidth={3}
        strokeDasharray={V35j}
        strokeDashoffset={(V35j * (100 - V35h)) / 100}
      />
      <text className="uw-pct" x={14} y={17} textAnchor="middle">
        {V35h}
      </text>
    </svg>
  );
}

export function V4wk({ state }: { state: V4wi }) {
  if (state === "error") {
    return (
      <span className="uw-ico err" aria-label="VGjhuqV0IGLhuqFp" role="img">
        !
      </span>
    );
  }
  if (state === "cancelled") {
    return (
      <span className="uw-ico cancelled" aria-label="xJDDoyBo4buneQ==" role="img">
        <Icon name="daHuy" size={14} />
      </span>
    );
  }
  return (
    <span className="uw-ico done" aria-label="VGjDoG5oIGPDtG5n" role="img">
      <Icon name="duyet" size={14} />
    </span>
  );
}
