import * as React from "react";
import { type VariantProps } from "class-variance-authority";
import { ToggleGroup as ToggleGroupPrimitive } from "radix-ui";

import { V462 } from "@/lib/utils";
import { toggleVariants } from "@/components/ui/toggle";

const V4om = React.createContext<
  VariantProps<typeof toggleVariants> & {
    spacing?: number;
  }
>({
  size: "default",
  variant: "default",
  spacing: 0,
});

function V4on({
  className,
  variant: V22x,
  size,
  spacing = 0,
  children: V22y,
  ...V22z
}: React.ComponentProps<typeof ToggleGroupPrimitive.Root> &
  VariantProps<typeof toggleVariants> & {
    spacing?: number;
  }) {
  return (
    <ToggleGroupPrimitive.Root
      data-slot="toggle-group"
      data-variant={V22x}
      data-size={size}
      data-spacing={spacing}
      style={{ "--gap": spacing } as React.CSSProperties}
      className={V462(
        "group/toggle-group flex w-fit items-center gap-[--spacing(var(--gap))] rounded-md data-[spacing=default]:data-[variant=outline]:shadow-xs",
        className,
      )}
      {...V22z}
    >
      <V4om.Provider value={{ variant: V22x, size, spacing }}>{V22y}</V4om.Provider>
    </ToggleGroupPrimitive.Root>
  );
}

function V4oo({
  className,
  children: V230,
  variant: V231,
  size,
  ...V232
}: React.ComponentProps<typeof ToggleGroupPrimitive.Item> & VariantProps<typeof toggleVariants>) {
  const V233 = React.useContext(V4om);

  return (
    <ToggleGroupPrimitive.Item
      data-slot="toggle-group-item"
      data-variant={V233.variant || V231}
      data-size={V233.size || size}
      data-spacing={V233.spacing}
      className={V462(
        toggleVariants({
          variant: V233.variant || V231,
          size: V233.size || size,
        }),
        "w-auto min-w-0 shrink-0 px-3 focus:z-10 focus-visible:z-10",
        "data-[spacing=0]:rounded-none data-[spacing=0]:shadow-none data-[spacing=0]:first:rounded-l-md data-[spacing=0]:last:rounded-r-md data-[spacing=0]:data-[variant=outline]:border-l-0 data-[spacing=0]:data-[variant=outline]:first:border-l",
        className,
      )}
      {...V232}
    >
      {V230}
    </ToggleGroupPrimitive.Item>
  );
}

export { V4on as ToggleGroup, V4oo as ToggleGroupItem };
