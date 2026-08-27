// Copyright (c) 2026 DigiText. All rights reserved.

export function chuaChonPhong(maDa: string | null | undefined): boolean {
  return (maDa ?? "").trim() === "";
}

export const V4bj =
  "SMOjeSBjaOG7jW4gUGjDtG5nIChjxqEgcXVhbikg4bufIGLhu5kgbOG7jWMgxJHhu4MgeGVtIGjhu5Mgc8ahIGPhu6dhIGPGoSBxdWFuIMSRw7Mu";

export function bangTrongText(maDa: string | null | undefined): string {
  return chuaChonPhong(maDa) ? V4bj : "S2jDtG5nIHTDrG0gdGjhuqV5IGThu68gbGnhu4d1Lg==";
}
