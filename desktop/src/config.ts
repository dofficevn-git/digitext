// Copyright (c) 2026 DigiText. All rights reserved.

export const V3wu = {
  appName: "DigiText",

  repo: { owner: "dangdat1111", name: "DigiText" },

  windowsDownloadUrl:
    "https://github.com/dangdat1111/DigiText/releases/latest/download/DigiText-Setup.exe",

  defaults: {
    driveMode: "grid" as const,
    catalog: "phong" as const,
  },
} as const;

export type V3wv = typeof V3wu;
