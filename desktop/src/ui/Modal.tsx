// Copyright (c) 2026 DigiText. All rights reserved.

import type { ReactNode } from "react";
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Label } from "@/components/ui/label";

export default function Modal({
  title: Vln,
  onClose: Vlo,
  onSave: Vlp,
  children: Vlq,
}: {
  title: string;
  onClose: () => void;
  onSave: () => void;
  children: ReactNode;
}) {
  return (
    <Dialog open onOpenChange={(Vlr) => !Vlr && Vlo()}>
      {}
      <DialogContent className="flex max-h-[calc(100dvh-4rem)] flex-col gap-0 p-0 sm:max-w-[480px]">
        <DialogHeader className="flex-none border-b border-border-subtle px-6 py-4">
          <DialogTitle>{Vln}</DialogTitle>
        </DialogHeader>

        {}
        <div className="min-h-0 flex-1 overflow-y-auto px-6 py-4">{Vlq}</div>

        <DialogFooter className="flex-none border-t border-border-subtle px-6 py-4">
          <Button variant="outline" onClick={Vlo}>
            Hủy
          </Button>
          <Button onClick={Vlp}>Lưu</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export function V49j({ label: Vls, children: Vlt }: { label: string; children: ReactNode }) {
  return (
    <div className="mb-4 last:mb-0 space-y-1.5">
      <Label className="text-sm font-medium text-fg-muted">{Vls}</Label>
      {Vlt}
    </div>
  );
}
