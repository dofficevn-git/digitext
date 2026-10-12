// Copyright (c) 2026 DigiText. All rights reserved.
// Đóng gói theo tiêu chuẩn: Từ hồ sơ (chọn hồ sơ rồi đóng gói SIP / AIP / DIP) · Gói tin (theo dõi, tải về) ·
// Nâng cao (form đầy đủ trường) · Từ kho ngoài (thư mục PDF + Excel) · Kiểm PDF/A.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { V40w, type V40v } from "@/api/archive";
import SoHoaPane from "@/features/dossier/SoHoaPane";
import { PackageList } from "./PackFromDossiers";
import V4sx from "./PackageView";
import ExternalImportPanel from "./ExternalImportPanel";
import PdfaCheckPanel from "./PdfaCheckPanel";
import PackageCheckPanel from "./PackageCheckPanel";
import { PageHeader, Tabs } from "@/ui";

type Tab = "hoso" | "goi" | "nangcao" | "ngoai" | "nhanh" | "pdfa" | "kiemgoi";

export default function DongGoiView() {
  const [tab, setTab] = useState<Tab>("hoso");
  const [phongs, setPhongs] = useState<V40v[]>([]);
  const load = useCallback(() => {
    V40w()
      .then(setPhongs)
      .catch(() => {});
  }, []);
  V41p(load, { topics: ["catalog", "hoso"], enabled: isTauri() });
  return (
    <section className="view active">
      <PageHeader title="Đóng gói" description="Tạo gói SIP, AIP, DIP theo bố cục E-ARK từ hồ sơ đã số hoá hoặc từ kho tài liệu bên ngoài." />
      <Tabs<Tab>
        value={tab}
        onChange={setTab}
        items={[
          { value: "hoso", label: "Từ hồ sơ", icon: "hoSo" },
          { value: "goi", label: "Gói tin", icon: "goi" },
          { value: "nangcao", label: "Nâng cao", icon: "caiDat" },
          { value: "ngoai", label: "Từ kho ngoài", icon: "nhapKho" },
          { value: "nhanh", label: "Đóng gói nhanh", icon: "chay" },
          { value: "pdfa", label: "Kiểm PDF/A", icon: "kiemTraPdfa" },
          { value: "kiemgoi", label: "Kiểm tra gói tin", icon: "kiemTra" },
        ]}
      />
      <div className="mt-3">
        {tab === "hoso" && <SoHoaPane phongs={phongs} mode="donggoi" />}
        {tab === "goi" && <PackageList />}
        {tab === "nangcao" && <V4sx />}
        {tab === "ngoai" && <ExternalImportPanel />}
        {tab === "nhanh" && <ExternalImportPanel nhanh />}
        {tab === "pdfa" && <PdfaCheckPanel />}
        {tab === "kiemgoi" && <PackageCheckPanel />}
      </div>
    </section>
  );
}
