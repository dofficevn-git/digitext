// Copyright (c) 2026 DigiText. All rights reserved.
// Hub Báo cáo: chọn báo cáo theo tab đang mở (số liệu máy chủ). Tab Nhân viên gộp hai nguồn: số hoá và chỉnh lý.

import { useState } from "react";
import { V460 } from "@/state";
import ProjectProgressView from "./ProjectProgressView";
import StaffReportView from "./StaffReportView";
import ThongKeView from "@/features/chinhly/ThongKeView";
import { PageHeader, Tabs } from "@/ui";

type NhanVienTab = "sohoa" | "chinhly";

export default function V4oe() {
  const { reportTab } = V460();
  const [nv, setNv] = useState<NhanVienTab>("sohoa");
  if (reportTab === "chinhly") return <ThongKeView kind="chinh-ly" />;
  if (reportTab === "phanmem") return <ThongKeView kind="phan-mem" />;
  if (reportTab === "muontra") return <ThongKeView kind="muon-tra" />;
  if (reportTab === "nhanvien")
    return (
      <section className="view active">
        <PageHeader title="Báo cáo nhân viên" description="Sản lượng, năng suất, lỗi và KPI theo từng người, số liệu trên máy chủ." />
        <Tabs<NhanVienTab> value={nv} onChange={setNv} items={[{ value: "sohoa", label: "Số hoá", icon: "sohoa" }, { value: "chinhly", label: "Chỉnh lý", icon: "chinhly" }]} />
        <div className="mt-3">
          {nv === "sohoa" && <StaffReportView embedded />}
          {nv === "chinhly" && <ThongKeView kind="nhan-vien" embedded />}
        </div>
      </section>
    );
  return <ProjectProgressView />;
}
