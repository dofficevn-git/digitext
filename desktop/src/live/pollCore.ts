// Copyright (c) 2026 DigiText. All rights reserved.

import { isTopic, V3x9, type V3xa } from "./topics";

export const V45l = 2_500;

export const V45m = 30_000;

export interface V45n {
  seq: number;

  topics: string[];

  full: boolean;
}

export interface V45o {
  since: number | null;

  errors: number;
}

export const V45p: V45o = { since: null, errors: 0 };

export function V45q(Vbx: number): number {
  if (Vbx <= 0) return V45l;
  return Math.min(V45l * 2 ** Vbx, V45m);
}

export function V45r(state: V45o, Vby: V45n): { state: V45o; publish: readonly V3xa[] } {
  const Vbz = state.since === null;
  const Vc0 = state.since !== null && Vby.seq < state.since;
  const Vc1: V45o = { since: Vby.seq, errors: 0 };
  if (Vbz) return { state: Vc1, publish: [] };
  if (Vby.full || Vc0) return { state: Vc1, publish: V3x9 };
  return { state: Vc1, publish: Vby.topics.filter(isTopic) };
}

export function V45s(state: V45o): V45o {
  return { since: state.since, errors: Math.min(state.errors + 1, 8) };
}
