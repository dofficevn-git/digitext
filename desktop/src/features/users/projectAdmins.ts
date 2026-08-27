// Copyright (c) 2026 DigiText. All rights reserved.

export interface V4r4 {
  currentUserId: string;

  isSuper: boolean;

  adminIds: string[];

  isProjectAdmin: boolean;
}

export function V4r5(r: V4r4): boolean {
  return r.isSuper || r.isProjectAdmin;
}

export function V4r6(r: V4r4, V2hs: string): string | null {
  if (r.isSuper) return null;
  if (!r.isProjectAdmin) return "QuG6oW4ga2jDtG5nIHBo4bulIHRyw6FjaCBk4buxIMOhbiBuw6B5Lg==";
  if (V2hs !== r.currentUserId) {
    return "Q2jhu4kgc3VwZXJfYWRtaW4gbeG7m2kgZ+G7oSDEkcaw4bujYyBuZ8aw4budaSBraMOhYyDigJQgYuG6oW4gY2jhu4kgdOG7sSByw7p0IG3DrG5oLg==";
  }

  if (r.adminIds.length <= 1) {
    return "QuG6oW4gbMOgIGFkbWluIGR1eSBuaOG6pXQg4oCUIG5o4budIHN1cGVyX2FkbWluIGfDoW4gbmfGsOG7nWkgdGhheSB0csaw4bubYyBraGkgcsO6dC4=";
  }
  return null;
}

export function V4r7(r: V4r4, V2ht: string): boolean {
  return V4r6(r, V2ht) === null;
}

export function V4r8<T extends { id: string }>(V2hu: T[], V2hv: string[]): T[] {
  const V2hw = new Set(V2hv);
  return V2hu.filter((a) => !V2hw.has(a.id));
}
