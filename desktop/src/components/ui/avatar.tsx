import * as React from "react";
import { Avatar as AvatarPrimitive } from "radix-ui";

import { V462 } from "@/lib/utils";

function V4xs({
  className,
  size = "default",
  ...V3cy
}: React.ComponentProps<typeof AvatarPrimitive.Root> & {
  size?: "default" | "sm" | "lg";
}) {
  return (
    <AvatarPrimitive.Root
      data-slot="avatar"
      data-size={size}
      className={V462(
        "group/avatar relative flex size-8 shrink-0 overflow-hidden rounded-full select-none data-[size=lg]:size-10 data-[size=sm]:size-6",
        className,
      )}
      {...V3cy}
    />
  );
}

function V4xt({ className, ...V3cz }: React.ComponentProps<typeof AvatarPrimitive.Image>) {
  return (
    <AvatarPrimitive.Image
      data-slot="avatar-image"
      className={V462("aspect-square size-full", className)}
      {...V3cz}
    />
  );
}

function V4xu({ className, ...V3d0 }: React.ComponentProps<typeof AvatarPrimitive.Fallback>) {
  return (
    <AvatarPrimitive.Fallback
      data-slot="avatar-fallback"
      className={V462(
        "flex size-full items-center justify-center rounded-full bg-muted text-sm text-muted-foreground group-data-[size=sm]/avatar:text-xs",
        className,
      )}
      {...V3d0}
    />
  );
}

function V4xv({ className, ...V3d1 }: React.ComponentProps<"span">) {
  return (
    <span
      data-slot="avatar-badge"
      className={V462(
        "absolute right-0 bottom-0 z-10 inline-flex items-center justify-center rounded-full bg-primary text-primary-foreground ring-2 ring-background select-none",
        "group-data-[size=sm]/avatar:size-2 group-data-[size=sm]/avatar:[&>svg]:hidden",
        "group-data-[size=default]/avatar:size-2.5 group-data-[size=default]/avatar:[&>svg]:size-2",
        "group-data-[size=lg]/avatar:size-3 group-data-[size=lg]/avatar:[&>svg]:size-2",
        className,
      )}
      {...V3d1}
    />
  );
}

function V4xw({ className, ...V3d2 }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="avatar-group"
      className={V462(
        "group/avatar-group flex -space-x-2 *:data-[slot=avatar]:ring-2 *:data-[slot=avatar]:ring-background",
        className,
      )}
      {...V3d2}
    />
  );
}

function V4xx({ className, ...V3d3 }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="avatar-group-count"
      className={V462(
        "relative flex size-8 shrink-0 items-center justify-center rounded-full bg-muted text-sm text-muted-foreground ring-2 ring-background group-has-data-[size=lg]/avatar-group:size-10 group-has-data-[size=sm]/avatar-group:size-6 [&>svg]:size-4 group-has-data-[size=lg]/avatar-group:[&>svg]:size-5 group-has-data-[size=sm]/avatar-group:[&>svg]:size-3",
        className,
      )}
      {...V3d3}
    />
  );
}

export {
  V4xs as Avatar,
  V4xt as AvatarImage,
  V4xu as AvatarFallback,
  V4xv as AvatarBadge,
  V4xw as AvatarGroup,
  V4xx as AvatarGroupCount,
};
