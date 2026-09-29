// Copyright (c) 2026 DigiText. All rights reserved.
// Hub Báo cáo: chọn báo cáo theo tab đang mở (số liệu máy chủ).

import { V460 } from "@/state";
import ProjectProgressView from "./ProjectProgressView";
import StaffReportView from "./StaffReportView";
import ThongKeView from "@/features/chinhly/ThongKeView";

export default function V4oe() {
  const { reportTab } = V460();
  if (reportTab === "chinhly") return <ThongKeView kind="chinh-ly" />;
  if (reportTab === "nhanvien")
    return (
      <>
        <StaffReportView />
        <ThongKeView kind="nhan-vien" />
      </>
    );
  if (reportTab === "phanmem") return <ThongKeView kind="phan-mem" />;
  if (reportTab === "muontra") return <ThongKeView kind="muon-tra" />;
  return <ProjectProgressView />;
}
