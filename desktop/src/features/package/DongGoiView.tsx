// Copyright (c) 2026 DigiText. All rights reserved.
//
// Màn "Đóng gói theo tiêu chuẩn" (đề nghị chỉnh sửa mục 8): bảng hồ sơ giống Tạo lập CSDL (bỏ tìm
// kiếm / nhập / xuất / cột Mã hồ sơ, File đính kèm) + nút Đóng gói (SIP/AIP hồ sơ – tài liệu, DIP),
// danh sách gói tin để tải về; form đóng gói đầy đủ cũ để ở mục "Nâng cao".

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { V40w, type V40v } from "@/api/archive";
import SoHoaPane from "@/features/dossier/SoHoaPane";
import { PackageList } from "./PackFromDossiers";
import V4sx from "./PackageView";

export default function DongGoiView() {
  const [phongs, setPhongs] = useState<V40v[]>([]);
  const load = useCallback(() => {
    V40w()
      .then(setPhongs)
      .catch(() => {});
  }, []);
  V41p(load, { topics: ["catalog", "hoso"], enabled: isTauri() });
  return (
    <section className="view active">
      <SoHoaPane phongs={phongs} mode="donggoi" />
      <div className="lhs">
        <PackageList />
      </div>
      <details className="mt-3.5">
        <summary className="muted text-sm cursor-pointer">
          Nâng cao: tạo gói với đầy đủ trường thông tin, nhập gói từ ngoài, kiểm tra PDF/A
        </summary>
        <V4sx />
      </details>
    </section>
  );
}
