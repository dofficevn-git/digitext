// Copyright (c) 2026 DigiText. All rights reserved.
// Nút "Giao diện" ở đáy thanh bên: bấm ra bảng chọn bộ màu (DO365 · Microsoft Teams · Zalo) + công tắc nền tối, như DO365.

import { useState } from "react";
import { Icon } from "@/components/Icon";
import { DropdownMenu, DropdownMenuContent, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { V462 as cn } from "@/lib/utils";
import { Tip } from "@/ui";
import { BO_MAU, boMauDaChon, datBoMau, datNenToi, nenToiDaChon, type BoMau } from "./boMau";

export default function ChonBoMau() {
  const [bo, setBo] = useState<BoMau>(boMauDaChon);
  const chon = (b: BoMau) => {
    setBo(b);
    datBoMau(b);
  };
  const [toi, setToi] = useState<boolean>(nenToiDaChon);
  const gatToi = () => {
    setToi(!toi);
    datNenToi(!toi);
  };

  return (
    <DropdownMenu>
      <Tip text="Giao diện — bộ màu và nền tối" side="right">
        <DropdownMenuTrigger asChild>
          <button
            type="button"
            className="grid size-8 place-items-center rounded-md text-[var(--side-faint)] outline-none transition-colors hover:bg-[var(--side-bg-2)] hover:text-[var(--side-strong)] focus-visible:ring-2 focus-visible:ring-white/70 data-[state=open]:bg-[var(--side-bg-3)] data-[state=open]:text-[var(--side-strong)]"
            aria-label="Giao diện — bộ màu và nền tối"
          >
            <Icon name="bangMau" size={16} />
          </button>
        </DropdownMenuTrigger>
      </Tip>
      <DropdownMenuContent side="right" align="end" sideOffset={10} className="w-72 p-2">
        <p className="m-0 px-0.5 pb-2 text-xs font-bold uppercase tracking-[0.04em] text-fg">Bộ màu</p>
        <div role="radiogroup" aria-label="Bộ màu" className="flex flex-col gap-1">
          {BO_MAU.map((x) => {
            const dang = x.k === bo;
            return (
              <button
                key={x.k}
                type="button"
                role="radio"
                aria-checked={dang}
                onClick={() => chon(x.k)}
                className={cn(
                  "flex w-full items-center gap-2.5 rounded-md border px-2 py-1.5 text-left outline-none transition-colors focus-visible:ring-2 focus-visible:ring-ring",
                  dang ? "border-brand-500 bg-brand-50" : "border-border-subtle bg-surface hover:bg-surface-hover",
                )}
              >
                <span aria-hidden className="flex h-7 w-[42px] flex-none overflow-hidden rounded border border-border-subtle">
                  {x.mau.map((m) => (
                    <i key={m} className="block h-full flex-1" style={{ background: m }} />
                  ))}
                </span>
                <span className="min-w-0 flex-1">
                  <b className="block text-sm font-semibold text-fg">{x.ten}</b>
                  <span className="block text-xs text-fg-subtle">{x.phu}</span>
                </span>
                {dang && <Icon name="duyet" size={16} className="flex-none text-brand-600" />}
              </button>
            );
          })}
        </div>
        <div aria-hidden className="my-2 h-px bg-border-subtle" />
        <button
          type="button"
          role="switch"
          aria-checked={toi}
          onClick={gatToi}
          className="flex w-full items-center gap-2 rounded-md px-1 py-1 text-left text-sm text-fg outline-none hover:bg-surface-hover focus-visible:ring-2 focus-visible:ring-ring"
        >
          <Icon name="nenToi" size={16} className="flex-none text-fg-subtle" />
          <span className="flex-1">Nền tối</span>
          <span aria-hidden className={cn("relative h-5 w-9 flex-none rounded-full transition-colors", toi ? "bg-brand-500" : "bg-gray-300")}>
            <i className={cn("absolute top-0.5 block size-4 rounded-full bg-white transition-[left]", toi ? "left-[18px]" : "left-0.5")} />
          </span>
        </button>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
