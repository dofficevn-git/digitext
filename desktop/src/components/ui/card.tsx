import * as React from "react";

import { V462 } from "@/lib/utils";

function V4y0({ className, ...V3d8 }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card"
      className={V462(
        "flex flex-col gap-6 rounded-xl border bg-card py-6 text-card-foreground shadow-sm",
        className,
      )}
      {...V3d8}
    />
  );
}

function V4y1({ className, ...V3d9 }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-header"
      className={V462(
        "@container/card-header grid auto-rows-min grid-rows-[auto_auto] items-start gap-2 px-6 has-data-[slot=card-action]:grid-cols-[1fr_auto] [.border-b]:pb-6",
        className,
      )}
      {...V3d9}
    />
  );
}

function V4y2({ className, ...V3da }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-title"
      className={V462("leading-none font-semibold", className)}
      {...V3da}
    />
  );
}

function V4y3({ className, ...V3db }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-description"
      className={V462("text-sm text-muted-foreground", className)}
      {...V3db}
    />
  );
}

function V4y4({ className, ...V3dc }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-action"
      className={V462("col-start-2 row-span-2 row-start-1 self-start justify-self-end", className)}
      {...V3dc}
    />
  );
}

function V4y5({ className, ...V3dd }: React.ComponentProps<"div">) {
  return <div data-slot="card-content" className={V462("px-6", className)} {...V3dd} />;
}

function V4y6({ className, ...V3de }: React.ComponentProps<"div">) {
  return (
    <div
      data-slot="card-footer"
      className={V462("flex items-center px-6 [.border-t]:pt-6", className)}
      {...V3de}
    />
  );
}

export {
  V4y0 as Card,
  V4y1 as CardHeader,
  V4y6 as CardFooter,
  V4y2 as CardTitle,
  V4y4 as CardAction,
  V4y3 as CardDescription,
  V4y5 as CardContent,
};
