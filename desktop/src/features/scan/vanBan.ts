// Copyright (c) 2026 DigiText. All rights reserved.

export function V4oj(V22u: string): string | null {
  const base = V22u.replace(/\\/g, "/").split("/").pop() ?? V22u;
  const m = base.match(/\.(\d{7})\.pdf$/i);
  return m ? m[1] : null;
}
