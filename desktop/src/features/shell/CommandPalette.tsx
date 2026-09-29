// Copyright (c) 2026 DigiText. All rights reserved.
// Tìm nhanh (Ctrl+K): gõ tên màn hoặc hành động, ↑↓ chọn, Enter mở. So khớp không dấu.

import { useEffect, useMemo, useRef, useState } from "react";
import { Dialog as Root, DialogContent, DialogDescription, DialogTitle } from "@/components/ui/dialog";
import { Icon, type V41v as IconName } from "@/components/Icon";
import { V3z8, V3z9 } from "@/domain/permissions";
import { V460 } from "@/state";
import { V462 as cn } from "@/lib/utils";
import { clSyncNow } from "@/api/chinhly";
import { isTauri } from "@/api/ipc";
import { Kbd, notifyError, toast } from "@/ui";
import { gotoLeaf, khongDau, NAV_TREE, type V46s } from "./navTree";

interface Entry {
  id: string;
  label: string;
  group: string;
  icon?: IconName;
  keywords: string;
  run: () => void;
}

export default function CommandPalette({ open, onOpenChange }: { open: boolean; onOpenChange: (o: boolean) => void }) {
  const nav = V460();
  const { session } = nav;
  const [q, setQ] = useState("");
  const [idx, setIdx] = useState(0);
  const listRef = useRef<HTMLUListElement>(null);

  const entries = useMemo<Entry[]>(() => {
    const can = (v: V46s["gate"]) => !session || V3z8(session.permissions, v);
    const out: Entry[] = [];
    for (const g of NAV_TREE) {
      if (g.leaf && can(g.gate)) out.push({ id: g.id, label: g.label, group: "Màn hình", icon: g.icon, keywords: khongDau(`${g.label} ${g.keywords ?? ""}`), run: () => gotoLeaf(g.leaf!, nav) });
      for (const c of g.children ?? []) {
        if (!c.leaf || !can(c.gate)) continue;
        out.push({ id: c.id, label: c.label, group: g.label, icon: g.icon, keywords: khongDau(`${g.label} ${c.label} ${c.keywords ?? ""}`), run: () => gotoLeaf(c.leaf!, nav) });
      }
    }
    const perms = session?.permissions;
    if (V3z9(perms, "scan.view")) out.push({ id: "act:scan", label: "Quét tài liệu mới", group: "Hành động", icon: "quet", keywords: "quet tai lieu moi scan", run: () => nav.goto("scan") });
    if (V3z9(perms, "upload.execute")) out.push({ id: "act:upload", label: "Đưa tệp lên máy chủ", group: "Hành động", icon: "taiLen", keywords: "dua tep len may chu tai len upload", run: () => nav.goto("upload") });
    if (V3z9(perms, "dossier.read") && isTauri())
      out.push({
        id: "act:sync",
        label: "Đồng bộ chỉnh lý ngay",
        group: "Hành động",
        icon: "lamMoi",
        keywords: "dong bo chinh ly ngay sync",
        run: () => {
          clSyncNow()
            .then(() => toast.success("Đã đồng bộ chỉnh lý."))
            .catch((e) => notifyError(e, "đồng bộ"));
        },
      });
    if (V3z9(perms, "settings.view")) out.push({ id: "act:settings", label: "Mở Cài đặt", group: "Hành động", icon: "caiDat", keywords: "cai dat settings", run: () => nav.goto("settings") });
    return out;
  }, [nav, session]);

  const hits = useMemo(() => {
    const k = khongDau(q.trim());
    if (!k) return entries.slice(0, 12);
    const words = k.split(/\s+/);
    return entries
      .map((e) => ({ e, score: words.every((w) => e.keywords.includes(w)) ? (e.keywords.startsWith(k) ? 2 : 1) : 0 }))
      .filter((x) => x.score > 0)
      .sort((a, b) => b.score - a.score)
      .slice(0, 12)
      .map((x) => x.e);
  }, [entries, q]);

  useEffect(() => setIdx(0), [q, open]);
  useEffect(() => {
    if (!open) setQ("");
  }, [open]);
  useEffect(() => {
    listRef.current?.querySelector<HTMLElement>(`[data-i="${idx}"]`)?.scrollIntoView({ block: "nearest" });
  }, [idx]);

  const run = (e: Entry) => {
    onOpenChange(false);
    e.run();
  };

  return (
    <Root open={open} onOpenChange={onOpenChange}>
      <DialogContent showCloseButton={false} className="top-[12vh] translate-y-0 gap-0 overflow-hidden p-0 sm:max-w-[560px]">
        <DialogTitle className="sr-only">Tìm nhanh</DialogTitle>
        <DialogDescription className="sr-only">Gõ tên màn hình hoặc hành động, dùng phím mũi tên để chọn, Enter để mở.</DialogDescription>
        <div className="flex items-center gap-2 border-b border-border-subtle px-3">
          <Icon name="tim" size={16} className="flex-none text-fg-subtle" />
          <input
            className="h-11 w-full bg-transparent text-base outline-none placeholder:text-fg-placeholder"
            placeholder="Đi tới màn hình hoặc hành động…"
            value={q}
            autoFocus
            aria-label="Tìm nhanh"
            aria-controls="cmdk-list"
            aria-activedescendant={hits[idx] ? `cmdk-${hits[idx].id}` : undefined}
            onChange={(e) => setQ(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "ArrowDown") {
                e.preventDefault();
                setIdx((i) => Math.min(hits.length - 1, i + 1));
              } else if (e.key === "ArrowUp") {
                e.preventDefault();
                setIdx((i) => Math.max(0, i - 1));
              } else if (e.key === "Enter" && hits[idx]) {
                e.preventDefault();
                run(hits[idx]);
              }
            }}
          />
          <Kbd keys={["Esc"]} />
        </div>
        <ul ref={listRef} id="cmdk-list" role="listbox" className="max-h-[50vh] overflow-y-auto py-1">
          {hits.length === 0 && <li className="px-3 py-6 text-center text-sm text-fg-subtle">Không tìm thấy màn hình hay hành động nào.</li>}
          {hits.map((e, i) => (
            <li
              key={e.id}
              id={`cmdk-${e.id}`}
              data-i={i}
              role="option"
              aria-selected={i === idx}
              className={cn("flex cursor-pointer items-center gap-2.5 px-3 py-2 text-base", i === idx ? "bg-brand-50 text-brand-700" : "hover:bg-surface-hover")}
              onMouseEnter={() => setIdx(i)}
              onClick={() => run(e)}
            >
              <Icon name={e.icon ?? "moNhanh"} size={15} className="flex-none text-fg-subtle" />
              <span className="min-w-0 flex-1 truncate">{e.label}</span>
              <span className="flex-none text-xs text-fg-subtle">{e.group}</span>
            </li>
          ))}
        </ul>
      </DialogContent>
    </Root>
  );
}
