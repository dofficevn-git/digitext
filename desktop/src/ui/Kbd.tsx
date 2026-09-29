// Copyright (c) 2026 DigiText. All rights reserved.
// Gợi ý phím tắt: <Kbd keys={["Ctrl", "K"]} />

export default function Kbd({ keys, className }: { keys: string[]; className?: string }) {
  return (
    <span className={className} aria-label={keys.join(" + ")}>
      {keys.map((k, i) => (
        <span key={i}>
          <kbd className="inline-block rounded border border-border bg-gray-50 px-1 text-xs font-medium text-fg-muted leading-4 shadow-[inset_0_-1px_0_var(--color-border)]">{k}</kbd>
          {i < keys.length - 1 && <span className="mx-0.5 text-xs text-fg-subtle">+</span>}
        </span>
      ))}
    </span>
  );
}
