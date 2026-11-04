// Copyright (c) 2026 DigiText. All rights reserved.

/** Bản Lưu trữ (cài tại máy chủ chủ đầu tư) — build với `VITE_EDITION=luu_tru`; xem trước bằng `?edition=luu_tru` khi DEV. */
export const LUU_TRU =
  import.meta.env.VITE_EDITION === "luu_tru" ||
  (import.meta.env.DEV && typeof location !== "undefined" && new URLSearchParams(location.search).get("edition") === "luu_tru");

export const V3wu = {
  appName: LUU_TRU ? "DigiText Lưu trữ" : "DigiText",

  defaults: {
    driveMode: "grid" as const,
    catalog: "phong" as const,
  },
} as const;

export type V3wv = typeof V3wu;
