// Copyright (c) 2026 DigiText. All rights reserved.

export const V4wy = 5;

export function V4wz(V383: string): string {
  return V383.replace(/\s+/g, "").toUpperCase().slice(0, V4wy);
}

export function V4x0(V384: string): boolean {
  return V4wz(V384).length === V4wy;
}
