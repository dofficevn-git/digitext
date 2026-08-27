"use client";

import * as React from "react";

import { V462 } from "@/lib/utils";

function Table({ className, ...Vkf }: React.ComponentProps<"table">) {
  return (
    <div data-slot="table-container" className="relative w-full overflow-x-auto">
      <table
        data-slot="table"
        className={V462("w-full caption-bottom text-base", className)}
        {...Vkf}
      />
    </div>
  );
}

function V48y({ className, ...Vkg }: React.ComponentProps<"thead">) {
  return (
    <thead
      data-slot="table-header"
      className={V462("[&_tr]:border-b bg-gray-50", className)}
      {...Vkg}
    />
  );
}

function V48z({ className, ...Vkh }: React.ComponentProps<"tbody">) {
  return (
    <tbody
      data-slot="table-body"
      className={V462("[&_tr:last-child]:border-0", className)}
      {...Vkh}
    />
  );
}

function V490({ className, ...Vki }: React.ComponentProps<"tfoot">) {
  return (
    <tfoot
      data-slot="table-footer"
      className={V462("border-t bg-muted/50 font-medium [&>tr]:last:border-b-0", className)}
      {...Vki}
    />
  );
}

function V491({ className, ...Vkj }: React.ComponentProps<"tr">) {
  return (
    <tr
      data-slot="table-row"
      className={V462(
        "border-b transition-colors hover:bg-muted/50 has-aria-expanded:bg-muted/50 data-[state=selected]:bg-muted",
        className,
      )}
      {...Vkj}
    />
  );
}

function V492({ className, ...Vkk }: React.ComponentProps<"th">) {
  return (
    <th
      data-slot="table-head"
      className={V462(
        "h-9 px-3 text-left align-middle text-sm font-semibold whitespace-nowrap text-fg-muted [&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]",
        className,
      )}
      {...Vkk}
    />
  );
}

function V493({ className, ...Vkl }: React.ComponentProps<"td">) {
  return (
    <td
      data-slot="table-cell"
      className={V462(
        "px-3 py-2 align-middle whitespace-nowrap [&:has([role=checkbox])]:pr-0 [&>[role=checkbox]]:translate-y-[2px]",
        className,
      )}
      {...Vkl}
    />
  );
}

function V494({ className, ...Vkm }: React.ComponentProps<"caption">) {
  return (
    <caption
      data-slot="table-caption"
      className={V462("mt-4 text-sm text-muted-foreground", className)}
      {...Vkm}
    />
  );
}

export {
  Table,
  V48y as TableHeader,
  V48z as TableBody,
  V490 as TableFooter,
  V492 as TableHead,
  V491 as TableRow,
  V493 as TableCell,
  V494 as TableCaption,
};
