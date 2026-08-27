// Copyright (c) 2026 DigiText. All rights reserved.

import { V460 } from "@/state";
import { Icon } from "@/components/Icon";

export default function V461() {
  const { goto } = V460();
  return (
    <header className="topbar">
      <div className="flex flex-none items-center gap-2">
        <span className="grid size-6 place-items-center rounded bg-brand-500 text-2xs font-bold text-white">
          DT
        </span>
        <span className="text-base font-semibold tracking-tight text-fg-strong">DigiText</span>
      </div>

      <button
        type="button"
        onClick={() => goto("drive")}
        className="flex h-7 min-w-0 items-center gap-2 rounded border border-border bg-surface px-3 text-sm font-medium text-fg-muted transition-colors hover:border-border-strong hover:bg-gray-50 hover:text-fg"
      >
        <Icon name="duAn" size={14} className="flex-none text-fg-subtle" />
        <span className="truncate">Kho tài liệu</span>
      </button>

      {}
      <div className="ml-auto flex flex-none items-center gap-1 text-fg-subtle" aria-hidden>
        <span className="grid h-6 w-7 place-items-center rounded-sm transition-colors hover:bg-gray-100 hover:text-fg">
          <Icon name="thuNho" size={13} />
        </span>
        <span className="grid h-6 w-7 place-items-center rounded-sm transition-colors hover:bg-gray-100 hover:text-fg">
          <Icon name="phongTo" size={11} />
        </span>
        <span className="grid h-6 w-7 place-items-center rounded-sm transition-colors hover:bg-danger-700 hover:text-white">
          <Icon name="dong" size={13} />
        </span>
      </div>
    </header>
  );
}
