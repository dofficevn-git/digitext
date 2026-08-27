// Copyright (c) 2026 DigiText. All rights reserved.

const V4or = [0.2126, 0.7152, 0.0722] as const;

const V4os = 255;

export interface V4ot {
  brightness: number;

  contrast: number;

  gamma: number;
  colorMode: "color" | "gray" | "bw";

  threshold: number;
}

export interface V4ou {
  active: boolean;

  brightness: number;
  contrastSlope: number;
  contrastIntercept: number;

  gammaExponent: number;

  grayMatrix?: string;

  bwSlope?: number;
  bwIntercept?: number;
}

export function V4ov(r: V4ot): V4ou {
  const V237 = ((100 + r.contrast) / 100) ** 2;
  const V238 = r.gamma > 0 ? r.gamma : 100;
  const V239 = r.colorMode === "gray" || r.colorMode === "bw";
  const V23a: V4ou = {
    active: r.brightness !== 0 || r.contrast !== 0 || r.gamma !== 100 || r.colorMode !== "color",
    brightness: r.brightness / 255,
    contrastSlope: V237,
    contrastIntercept: 0.5 - 0.5 * V237,
    gammaExponent: 100 / V238,
  };
  if (V239) {
    const [V23b, V23c, V23d] = V4or;
    V23a.grayMatrix = [
      `${V23b} ${V23c} ${V23d} 0 0`,
      `${V23b} ${V23c} ${V23d} 0 0`,
      `${V23b} ${V23c} ${V23d} 0 0`,
      "0 0 0 1 0",
    ].join(" ");
  }
  if (r.colorMode === "bw") {
    V23a.bwSlope = V4os;

    V23a.bwIntercept = 0.5 - V4os * (r.threshold / 255);
  }
  return V23a;
}
