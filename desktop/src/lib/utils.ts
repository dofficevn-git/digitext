// Copyright (c) 2026 DigiText. All rights reserved.

import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function V462(...Vcl: ClassValue[]) {
  return twMerge(clsx(Vcl));
}
