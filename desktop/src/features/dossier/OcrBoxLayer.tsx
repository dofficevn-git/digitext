// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useEffect, useRef, useState } from "react";
import { V4gw, V4gu, V4gn, V4gs, V4gp, type V4gm, type V4gk } from "./ocrBoxes";

const V4gx = "var(--color-brand-500)";

const V4gy = "var(--color-gray-0)";

const V4gz: V4gm[] = ["nw", "n", "ne", "e", "se", "s", "sw", "w"];
const V4h0: Record<V4gm, string> = {
  nw: "nwse-resize",
  se: "nwse-resize",
  ne: "nesw-resize",
  sw: "nesw-resize",
  n: "ns-resize",
  s: "ns-resize",
  e: "ew-resize",
  w: "ew-resize",
  move: "move",
};

type V4h1 = { key: string; handle: V4gm; startX: number; startY: number; orig: V4gk };

type V4h2 = { x0: number; y0: number; x1: number; y1: number };

export default function V4h3({
  src,
  boxes: V1ad,
  pageW: V1ae,
  pageH: V1af,
  editable: V1ag = true,
  selectedKey: V1ah,
  onSelect: V1ai,
  onChange: V1aj,
  drawField: V1ak,
  onCreate: V1al,
  boxLabel: V1am,
  boxColorFn: V1an,
  onNatural: V1ao,
}: {
  src: string;
  boxes: V4gk[];

  pageW: number;
  pageH: number;
  editable?: boolean;
  selectedKey?: string | null;
  onSelect?: (V1ap: string | null) => void;
  onChange?: (V1aq: V4gk) => void;

  drawField?: string | null;
  onCreate?: (V1ar: V4gk) => void;

  boxLabel?: (V1as: V4gk) => string | null;

  boxColorFn?: (V1at: V4gk) => string;

  onNatural?: (w: number, h: number) => void;
}) {
  const V1au = useRef<HTMLImageElement>(null);
  const V1av = useRef<SVGSVGElement>(null);
  const [V1aw, V1ax] = useState(0);
  const V1ay = useRef<V4h1 | null>(null);
  const [V1az, V1b0] = useState<V4h2 | null>(null);
  const V1b1 = V1ak != null && V1ag && V1al != null;

  const V1b2 = useCallback(() => {
    const V1b3 = V1au.current;
    if (!V1b3) return;
    V1ax(V1b3.clientWidth);
    if (V1b3.naturalWidth > 0) V1ao?.(V1b3.naturalWidth, V1b3.naturalHeight);
  }, [V1ao]);
  useEffect(() => {
    V1b2();
    window.addEventListener("resize", V1b2);
    return () => window.removeEventListener("resize", V1b2);
  }, [V1b2]);

  const scale = V1ae > 0 && V1aw > 0 ? V1aw / V1ae : 1;

  const V1b4 = (V1b5: React.PointerEvent) => {
    const V1b6 = V1av.current?.getBoundingClientRect();
    const left = V1b6?.left ?? 0;
    const top = V1b6?.top ?? 0;
    return { px: (V1b5.clientX - left) / scale, py: (V1b5.clientY - top) / scale };
  };

  const V1b7 = (V1b8: React.PointerEvent, V1b9: V4gk, V1ba: V4gm) => {
    if (!V1ag) return;
    V1b8.preventDefault();
    V1b8.stopPropagation();
    (V1b8.target as Element).setPointerCapture?.(V1b8.pointerId);
    V1ay.current = {
      key: V1b9.key,
      handle: V1ba,
      startX: V1b8.clientX,
      startY: V1b8.clientY,
      orig: V1b9,
    };
    V1ai?.(V1b9.key);
  };

  const V1bb = (V1bc: React.PointerEvent) => {
    if (V1b1 && scale > 0) {
      V1bc.preventDefault();
      (V1bc.target as Element).setPointerCapture?.(V1bc.pointerId);
      const { px, py: V1bd } = V1b4(V1bc);
      V1b0({ x0: px, y0: V1bd, x1: px, y1: V1bd });
    } else {
      V1ai?.(null);
    }
  };
  const V1be = (V1bf: React.PointerEvent) => {
    if (V1az) {
      const { px, py: V1bg } = V1b4(V1bf);
      V1b0((d) => (d ? { ...d, x1: px, y1: V1bg } : d));
      return;
    }
    const d = V1ay.current;
    if (!d || scale <= 0) return;
    const V1bh = (V1bf.clientX - d.startX) / scale;
    const V1bi = (V1bf.clientY - d.startY) / scale;
    V1aj?.(V4gs(d.orig, d.handle, V1bh, V1bi, V1ae, V1af));
  };
  const V1bj = (V1bk: React.PointerEvent) => {
    if (V1az) {
      (V1bk.target as Element).releasePointerCapture?.(V1bk.pointerId);
      const { x0: V1bl, y0: V1bm, x1: V1bn, y1: V1bo } = V1az;
      V1b0(null);

      if (V1ak != null && (Math.abs(V1bn - V1bl) >= V4gn || Math.abs(V1bo - V1bm) >= V4gn)) {
        V1al?.(V4gu(`draw:${V1ak}`, V1ak, V1bl, V1bm, V1bn, V1bo, V1ae, V1af));
      }
      return;
    }
    if (V1ay.current) (V1bk.target as Element).releasePointerCapture?.(V1bk.pointerId);
    V1ay.current = null;
  };

  const V1bp = V1az
    ? V4gp(
        {
          key: "_",
          x: Math.min(V1az.x0, V1az.x1),
          y: Math.min(V1az.y0, V1az.y1),
          w: Math.abs(V1az.x1 - V1az.x0),
          h: Math.abs(V1az.y1 - V1az.y0),
        },
        scale,
      )
    : null;

  return (
    <div className="ocr-boxwrap relative leading-none">
      <img
        ref={V1au}
        src={src}
        alt="Trang"
        draggable={false}
        onLoad={V1b2}
        className="w-full h-auto block"
      />
      <svg
        ref={V1av}
        className={`ocr-boxsvg absolute inset-0 h-full w-full touch-none ${
          V1b1 ? "cursor-crosshair" : "cursor-default"
        }`}
        onPointerMove={V1be}
        onPointerUp={V1bj}
        onPointerLeave={V1bj}
        onPointerDown={V1bb}
      >
        {V1ad.map((V1bq) => {
          const r = V4gp(V1bq, scale);
          const V1br = V1bq.key === V1ah;
          const color = V1an ? V1an(V1bq) : V4gw(V1bq.conf);
          const V1bs = V1am?.(V1bq) ?? null;
          return (
            <g key={V1bq.key}>
              <rect
                x={r.left}
                y={r.top}
                width={r.width}
                height={r.height}
                fill={V1br ? "rgba(59,130,246,0.10)" : "transparent"}
                stroke={V1br ? V4gx : color}
                strokeWidth={V1br ? 2 : 1.5}
                style={{ cursor: V1ag && !V1b1 ? "move" : "pointer" }}
                onPointerDown={(V1bt) => V1b7(V1bt, V1bq, "move")}
              />
              {V1bs && (
                <text
                  x={r.left}
                  y={Math.max(10, r.top - 3)}
                  fontSize={11}
                  fontWeight={600}
                  fill={V4gy}
                  stroke={V1br ? V4gx : color}
                  strokeWidth={3}
                  paintOrder="stroke"
                  className="pointer-events-none select-none"
                >
                  {V1bs}
                </text>
              )}
              {V1ag &&
                !V1b1 &&
                V1br &&
                V4gz.map((h) => {
                  const V1bu =
                    r.left + (h.includes("w") ? 0 : h.includes("e") ? r.width : r.width / 2);
                  const V1bv =
                    r.top + (h.includes("n") ? 0 : h.includes("s") ? r.height : r.height / 2);
                  return (
                    <rect
                      key={h}
                      x={V1bu - 4}
                      y={V1bv - 4}
                      width={8}
                      height={8}
                      fill={V4gy}
                      stroke={V4gx}
                      strokeWidth={1.5}
                      style={{ cursor: V4h0[h] }}
                      onPointerDown={(V1bw) => V1b7(V1bw, V1bq, h)}
                    />
                  );
                })}
            </g>
          );
        })}
        {V1bp && (
          <rect
            x={V1bp.left}
            y={V1bp.top}
            width={V1bp.width}
            height={V1bp.height}
            fill="rgba(59,130,246,0.10)"
            stroke={V4gx}
            strokeWidth={1.5}
            strokeDasharray="4 3"
            className="pointer-events-none"
          />
        )}
      </svg>
    </div>
  );
}
