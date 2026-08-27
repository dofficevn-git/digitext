// Copyright (c) 2026 DigiText. All rights reserved.

import { useState } from "react";
import type { ReactNode } from "react";
import { Icon } from "@/components/Icon";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Checkbox } from "@/components/ui/checkbox";
import { V462 } from "@/lib/utils";

export interface V499<T> {
  head: ReactNode;
  cell: (r: T) => ReactNode;
  cls?: string;
}

export default function V49a<T>(Vku: {
  addLabel: string;

  onAdd?: () => void;
  columns: V499<T>[];
  rows: T[];
  getId: (r: T) => string | number;
  matches: (r: T, q: string) => boolean;
  rowActions: (r: T) => ReactNode;

  onDeleteMany?: (Vkv: (string | number)[]) => void;
  extraFilters?: ReactNode;
  extraActions?: ReactNode;
}) {
  const [q, Vkw] = useState("");
  const [Vkx, Vky] = useState<Set<string | number>>(new Set());

  const Vkz = q.trim().toLowerCase();
  const Vl0 = Vkz ? Vku.rows.filter((r) => Vku.matches(r, Vkz)) : Vku.rows;
  const Vl1 = Vl0.length > 0 && Vl0.every((r) => Vkx.has(Vku.getId(r)));
  const Vl2 = () => Vky(() => (Vl1 ? new Set() : new Set(Vl0.map(Vku.getId))));
  const Vl3 = (Vl4: string | number) =>
    Vky((s) => {
      const n = new Set(s);
      n.has(Vl4) ? n.delete(Vl4) : n.add(Vl4);
      return n;
    });

  return (
    <>
      <div className="mb-3 flex flex-wrap items-center gap-2">
        {Vku.extraFilters}
        <div className="relative w-60 max-w-full flex-none">
          <Icon
            name="tim"
            className="pointer-events-none absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-fg-subtle"
          />
          <Input
            placeholder="VMOsbSBraeG6v23igKY="
            value={q}
            onChange={(Vl5) => Vkw(Vl5.target.value)}
            aria-label="VMOsbSBraeG6v20gdHJvbmcgYuG6o25n"
            className="pl-8"
          />
        </div>
        <div className="flex-1" />
        {Vkx.size > 0 && Vku.onDeleteMany && (
          <Button
            variant="outline"
            className="border-danger-200 text-danger-700 hover:bg-danger-50"
            onClick={() => {
              if (
                !window.confirm(
                  `WMOzYSA=${Vkx.size}IG3hu6VjIMSRw6MgY2jhu41uPyBLaMO0bmcgdGjhu4MgaG/DoG4gdMOhYy4=`,
                )
              )
                return;
              Vku.onDeleteMany?.([...Vkx]);
              Vky(new Set());
            }}
          >
            <Icon name="xoa" /> Xóa {Vkx.size} mục đã chọn
          </Button>
        )}
        {Vku.extraActions}
        {Vku.onAdd && (
          <Button onClick={Vku.onAdd}>
            <Icon name="them" /> {Vku.addLabel}
          </Button>
        )}
      </div>

      <div className="overflow-hidden rounded border border-border-subtle">
        <Table>
          <TableHeader>
            <TableRow className="hover:bg-transparent">
              <TableHead className="w-9 text-center">
                <Checkbox
                  checked={Vl1}
                  onCheckedChange={Vl2}
                  aria-label="Q2jhu41uIHThuqV0IGPhuqMgY8OhYyBkw7JuZyDEkWFuZyBoaeG7g24gdGjhu4s="
                />
              </TableHead>
              {Vku.columns.map((c, i) => (
                <TableHead key={i} className={c.cls}>
                  {c.head}
                </TableHead>
              ))}
              <TableHead className="w-px text-right whitespace-nowrap">Thao tác</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {Vl0.length === 0 ? (
              <TableRow className="hover:bg-transparent">
                <TableCell
                  colSpan={Vku.columns.length + 2}
                  className="py-8 text-center text-fg-subtle"
                >
                  {Vkz
                    ? `S2jDtG5nIGPDsyBi4bqjbiBnaGkgbsOgbyBraOG7m3Ag4oCc${q.trim()}”.`
                    : "Q2jGsGEgY8OzIGThu68gbGnhu4d1Lg=="}
                </TableCell>
              </TableRow>
            ) : (
              Vl0.map((r) => {
                const Vl6 = Vku.getId(r);
                const Vl7 = Vkx.has(Vl6);
                return (
                  <TableRow key={Vl6} data-state={Vl7 ? "selected" : undefined}>
                    <TableCell className="text-center">
                      <Checkbox
                        checked={Vl7}
                        onCheckedChange={() => Vl3(Vl6)}
                        aria-label="Q2jhu41uIGTDsm5nIG7DoHk="
                      />
                    </TableCell>
                    {Vku.columns.map((c, i) => (
                      <TableCell key={i} className={V462("whitespace-normal", c.cls)}>
                        {c.cell(r)}
                      </TableCell>
                    ))}
                    <TableCell className="text-right whitespace-nowrap">
                      {Vku.rowActions(r)}
                    </TableCell>
                  </TableRow>
                );
              })
            )}
          </TableBody>
        </Table>
      </div>

      <div className="mt-3 flex items-center justify-between gap-3 text-sm text-fg-subtle">
        <span>
          Hiển thị {Vl0.length} / {Vku.rows.length} bản ghi
        </span>
      </div>
    </>
  );
}
