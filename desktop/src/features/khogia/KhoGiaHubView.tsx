// Copyright (c) 2026 DigiText. All rights reserved.

import { V460 } from "@/state";
import KhaiThacView from "./KhaiThacView";
import V4no from "./KhoView";
import V4nr from "./GiaView";

export default function V4ns() {
  const { khoGiaTab: V21p } = V460();
  if (V21p === "kho") return <V4no />;
  if (V21p === "gia") return <V4nr />;
  return <KhaiThacView />;
}
