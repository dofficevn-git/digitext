// Copyright (c) 2026 DigiText. All rights reserved.

import { V460 } from "@/state";
import KhaiThacView from "@/features/chinhly/KhaiThacView";
import KhoGiaView from "@/features/chinhly/KhoGiaView";
import MuonTraView from "@/features/chinhly/MuonTraView";

export default function V4ns() {
  const { khoGiaTab: V21p } = V460();
  if (V21p === "kho") return <KhoGiaView tab="kho" />;
  if (V21p === "gia") return <KhoGiaView tab="gia" />;
  if (V21p === "muontra") return <MuonTraView />;
  return <KhaiThacView />;
}
