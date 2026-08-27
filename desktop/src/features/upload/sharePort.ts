// Copyright (c) 2026 DigiText. All rights reserved.

export const V4qb = 45817;

export function V4qc(V2cm: string): string | null {
  const t = V2cm.trim();
  if (t === "") return null;
  if (!/^\d+$/.test(t))
    return `Q+G7lW5nIGNoaWEgc+G6uyBwaOG6o2kgbMOgIHPhu5EgMTAyNOKAkzY1NTM1IChi4buPIHRy4buRbmcgPSBj4buVbmcgY2h1bmcg${V4qb}).`;
  const n = Number(t);
  if (n < 1024 || n > 65535)
    return "Q+G7lW5nIGNoaWEgc+G6uyBwaOG6o2kgdHJvbmcga2hv4bqjbmcgMTAyNOKAkzY1NTM1Lg==";
  return null;
}
