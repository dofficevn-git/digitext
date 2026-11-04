// Copyright (c) 2026 DigiText. All rights reserved.
// Đóng gói theo tiêu chuẩn (bước 7 Điều 9 TT05): Từ hồ sơ (phông / hộp / hồ sơ của dự án đang chọn trên máy chủ →
// SIP / AIP / DIP) · Gói tin (theo dõi, tải về) · Nâng cao (form đầy đủ trường). 3/11/2026: Từ kho ngoài, Đóng gói nhanh,
// Kiểm PDF/A, Kiểm tra gói tin chuyển sang màn Tiện ích (shell/TienIchHub). 26/10: Từ hồ sơ thôi dùng danh mục cục bộ.

import { useState } from "react";
import PackFromServer from "./PackFromServer";
import { PackageList } from "./PackFromDossiers";
import V4sx from "./PackageView";
import { PageHeader, Tabs } from "@/ui";

type Tab = "hoso" | "goi" | "nangcao";

export default function DongGoiView() {
  const [tab, setTab] = useState<Tab>("hoso");
  return (
    <section className="view active">
      <PageHeader title="Đóng gói" description="Bước 7 Điều 9 TT 05/2025: tạo gói SIP, AIP, DIP theo bố cục E-ARK từ hồ sơ đã số hoá trên máy chủ. Kiểm PDF/A, kiểm gói, đóng gói từ kho ngoài nằm ở Tiện ích." />
      <Tabs<Tab>
        value={tab}
        onChange={setTab}
        items={[
          { value: "hoso", label: "Từ hồ sơ", icon: "hoSo" },
          { value: "goi", label: "Gói tin", icon: "goi" },
          { value: "nangcao", label: "Nâng cao", icon: "caiDat" },
        ]}
      />
      <div className="mt-3">
        {tab === "hoso" && <PackFromServer />}
        {tab === "goi" && <PackageList />}
        {tab === "nangcao" && <V4sx />}
      </div>
    </section>
  );
}
