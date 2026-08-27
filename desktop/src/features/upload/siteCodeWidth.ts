// Copyright (c) 2026 DigiText. All rights reserved.

export const V4q7 = 30;

const V4q8 = 22;

const V4q9 = 60;

export function V4qa(V2cl: string): string {
  const n = Math.min(V2cl.length || V4q7, V4q9);
  return `calc(${n}ch + ${V4q8}px)`;
}
