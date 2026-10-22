// Copyright (c) 2026 DigiText. All rights reserved.
// Trang Tổng quan: một màn, ba góc nhìn theo vai trò (22/10/2026).
//   - Nhân viên dự án → StaffDashboard (số của mình + trung bình nhóm ẩn danh).
//   - Quản trị dự án → ProjectDashboard của dự án được gán.
//   - Quản lý / Quản trị hệ thống → LeaderDashboard (mọi dự án); đã chọn dự án ở Topbar thì đổi được sang dự án đó.
// Đầu trang giữ lời chào, lối tắt theo vòng đời và cảnh báo phiên ngoại tuyến như trước.

import { useEffect, useState } from "react";
import { V3z9 } from "@/domain/permissions";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { V3zr } from "@/state/sessionCore";
import { type V41v as IconName } from "@/components/Icon";
import { Banner, Button, EmptyState, Seg, Tip } from "@/ui";
import LeaderDashboard from "./LeaderDashboard";
import ProjectDashboard from "./ProjectDashboard";
import StaffDashboard from "./StaffDashboard";

type Mode = "leader" | "project";

export default function HomeView() {
  const nav = V460();
  const { session, goto, gotoHoSo, gotoReport } = nav;
  const proj = useProject();
  const has = (p: Parameters<typeof V3z9>[1]) => V3z9(session?.permissions, p);
  const role = session?.role ?? "user";
  const laLanhDao = role === "manager" || role === "super_admin";
  const projectId = session?.projectId ?? proj.projectId ?? null;

  const [mode, setMode] = useState<Mode>(projectId ? "project" : "leader");
  // Đổi dự án ở Topbar → xem ngay dự án đó; bỏ chọn → về toàn bộ.
  useEffect(() => {
    setMode(proj.projectId ? "project" : "leader");
  }, [proj.projectId, proj.version]);

  const gio = new Date().getHours();
  const chao = gio < 11 ? "Chào buổi sáng" : gio < 14 ? "Chào buổi trưa" : gio < 18 ? "Chào buổi chiều" : "Chào buổi tối";
  const offlineDays = session ? V3zr(session.offlineExpiry) : 0;
  const moTa = [session?.mode === "offline" ? `Ngoại tuyến, còn ${offlineDays} ngày` : "Đã kết nối máy chủ", proj.projectLabel ? `Dự án: ${proj.projectLabel}` : null].filter(Boolean).join(" · ");

  type LoiTat = { label: string; icon: IconName; run: () => void; show: boolean };
  const nhomGoc: { nhom: string; items: LoiTat[] }[] = [
    { nhom: "Chỉnh lý", items: [{ label: "Lập hồ sơ", icon: "hoSo", run: () => gotoHoSo("laphoso"), show: has("dossier.read") }] },
    {
      nhom: "Số hoá",
      items: [
        { label: "Quét", icon: "quet", run: () => goto("scan"), show: has("scan.view") },
        { label: "Đưa lên máy chủ", icon: "taiLen", run: () => goto("upload"), show: has("upload.execute") },
      ],
    },
    {
      nhom: "Xử lý",
      items: [
        { label: "OCR và bóc tách", icon: "xuLy", run: () => gotoHoSo("ocr"), show: has("dossier.read") },
        { label: "Ký số", icon: "kySo", run: () => goto("sign"), show: has("sign.view") },
        { label: "Đóng gói", icon: "goi", run: () => goto("package"), show: has("package.view") },
      ],
    },
    {
      nhom: "Lưu trữ số",
      items: [
        { label: "Thu nộp", icon: "nhapKho", run: () => goto("submission"), show: has("submission.view") },
        { label: "Khai thác", icon: "tim", run: () => goto("access"), show: has("access.view") },
      ],
    },
    { nhom: "Báo cáo", items: [{ label: "Tiến độ dự án", icon: "baoCao", run: () => gotoReport("sohoa"), show: has("stats.view_project") }] },
  ];
  const nhomLoiTat = nhomGoc.map((g) => ({ ...g, items: g.items.filter((i) => i.show) })).filter((g) => g.items.length > 0);

  const moDuAn = (id: string) => {
    void proj.choose(id).then(() => setMode("project"));
  };

  let body: React.ReactNode;
  if (!has("stats.view_project")) {
    body = <StaffDashboard />;
  } else if (!laLanhDao) {
    body = projectId ? <ProjectDashboard projectId={projectId} /> : <EmptyState icon="duAn" title="Chưa chọn dự án" description="Chọn dự án ở ô cạnh tên bạn trên thanh trên cùng để xem tổng quan." />;
  } else {
    body = (
      <>
        <div className="mb-3 flex items-center gap-3">
          <Seg<Mode> value={mode} onChange={(m) => (m === "project" && !projectId ? undefined : setMode(m))} options={[{ value: "leader", label: "Toàn bộ dự án" }, { value: "project", label: projectId ? "Dự án đang chọn" : "Dự án đang chọn (chưa chọn)" }]} />
          {!projectId && (
            <Tip text="Chọn dự án ở thanh trên cùng để xem riêng">
              <span className="text-xs text-fg-subtle">Chưa chọn dự án</span>
            </Tip>
          )}
        </div>
        {mode === "project" && projectId ? <ProjectDashboard projectId={projectId} /> : <LeaderDashboard onOpenProject={moDuAn} />}
      </>
    );
  }

  return (
    <section className="view active">
      <header className="mb-4">
        <h1 className="m-0 text-xl font-semibold text-fg-strong">
          {chao}, {session?.displayName ?? ""}
        </h1>
        <p className="mt-0.5 text-sm text-fg-subtle">{moTa}</p>
        {nhomLoiTat.length > 0 && (
          <nav aria-label="Lối tắt" className="mt-3 flex flex-wrap gap-x-5 gap-y-2">
            {nhomLoiTat.map((g) => (
              <div key={g.nhom} className="flex items-center gap-1.5">
                <span className="mr-0.5 text-sm text-fg-subtle">{g.nhom}</span>
                {g.items.map((i) => (
                  <Button key={i.label} size="sm" icon={i.icon} onClick={i.run}>
                    {i.label}
                  </Button>
                ))}
              </div>
            ))}
          </nav>
        )}
      </header>
      {session?.mode === "offline" && offlineDays < 7 && (
        <Banner tone="warning" className="mb-4">
          Phiên ngoại tuyến sắp hết hạn, còn {offlineDays} ngày. Kết nối mạng và đăng nhập lại để không gián đoạn.
        </Banner>
      )}
      {body}
    </section>
  );
}
