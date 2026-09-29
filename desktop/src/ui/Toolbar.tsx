// Copyright (c) 2026 DigiText. All rights reserved.
// Thanh công cụ: các nhóm nút / ô lọc xếp ngang, xuống dòng khi hẹp.

import type { ReactNode } from "react";
import { V462 as cn } from "@/lib/utils";

function Toolbar({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cn("flex flex-wrap items-center gap-2", className)}>{children}</div>;
}

function Group({ children, className }: { children: ReactNode; className?: string }) {
  return <div className={cn("flex items-center gap-2", className)}>{children}</div>;
}

function Spacer() {
  return <div className="flex-1" />;
}

Toolbar.Group = Group;
Toolbar.Spacer = Spacer;

export default Toolbar;
