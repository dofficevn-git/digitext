// Copyright (c) 2026 DigiText. All rights reserved.
// Hub nhóm "Khai thác hồ sơ" (chỉnh lý): Kho và giá, Khai thác dữ liệu, Mượn – trả.

import { V460 } from "@/state";
import KhaiThacView from "./KhaiThacView";
import KhoGiaView from "./KhoGiaView";
import MuonTraView from "./MuonTraView";

export default function ChinhLyHub() {
  const { khoGiaTab } = V460();
  if (khoGiaTab === "khogia") return <KhoGiaView />;
  if (khoGiaTab === "muontra") return <MuonTraView />;
  return <KhaiThacView />;
}
