import * as React from "react";
import { Slider as SliderPrimitive } from "radix-ui";

import { V462 } from "@/lib/utils";

function V4ys({
  className,
  defaultValue: V3ec,
  value: V3ed,
  min = 0,
  max = 100,
  ...V3ee
}: React.ComponentProps<typeof SliderPrimitive.Root>) {
  const _V3ef = React.useMemo(
    () => (Array.isArray(V3ed) ? V3ed : Array.isArray(V3ec) ? V3ec : [min, max]),
    [V3ed, V3ec, min, max],
  );

  return (
    <SliderPrimitive.Root
      data-slot="slider"
      defaultValue={V3ec}
      value={V3ed}
      min={min}
      max={max}
      className={V462(
        "relative flex w-full touch-none items-center select-none data-[disabled]:opacity-50 data-[orientation=vertical]:h-full data-[orientation=vertical]:min-h-44 data-[orientation=vertical]:w-auto data-[orientation=vertical]:flex-col",
        className,
      )}
      {...V3ee}
    >
      <SliderPrimitive.Track
        data-slot="slider-track"
        className={V462(
          "relative grow overflow-hidden rounded-full bg-muted data-[orientation=horizontal]:h-1.5 data-[orientation=horizontal]:w-full data-[orientation=vertical]:h-full data-[orientation=vertical]:w-1.5",
        )}
      >
        <SliderPrimitive.Range
          data-slot="slider-range"
          className={V462(
            "absolute bg-primary data-[orientation=horizontal]:h-full data-[orientation=vertical]:w-full",
          )}
        />
      </SliderPrimitive.Track>
      {Array.from({ length: _V3ef.length }, (_V3eg, V3eh) => (
        <SliderPrimitive.Thumb
          data-slot="slider-thumb"
          key={V3eh}
          className="block size-4 shrink-0 rounded-full border border-primary bg-white shadow-sm ring-ring/40 transition-[color,box-shadow] hover:ring-4 focus-visible:ring-4 focus-visible:outline-hidden disabled:pointer-events-none disabled:opacity-50"
        />
      ))}
    </SliderPrimitive.Root>
  );
}

export { V4ys as Slider };
