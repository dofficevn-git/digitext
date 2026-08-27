// Copyright (c) 2026 DigiText. All rights reserved.

const vi = new Intl.NumberFormat("vi-VN");

export function V4z5(n: number): string {
  return vi.format(n);
}

export function V4z6(n: number): string {
  return `${V4z5(n)} trang`;
}
