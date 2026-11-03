// Copyright (c) 2026 DigiText. All rights reserved.
// Nhóm "Tiện ích" (3/11/2026): mọi công cụ nằm ngoài trục thi công Điều 9 TT 05/2025/TT-BNV gom về một màn, mỗi
// công cụ một tab (?view=tienich&tab=… khi xem trước). Panel bên trong là các component đã có, chỉ dời chỗ:
// Kiểm PDF/A, Kiểm tra gói tin, Đóng gói từ kho ngoài, Đóng gói nhanh (trước ở Đóng gói), Ví dụ dạy máy (trước ở OCR),
// Chuyển dữ liệu cũ (vẫn còn ở Cài đặt › Dữ liệu). Tách TIFF, Kho và giá, Mượn – trả giữ màn riêng, chỉ đổi mục menu.

import { useEffect, useMemo, useState } from "react";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { isTauri } from "@/api/ipc";
import { V4ak, type V4a9 } from "@/api/users";
import { V3z9 } from "@/domain/permissions";
import type { V3xTienIch } from "@/domain/types";
import PdfaCheckPanel from "@/features/package/PdfaCheckPanel";
import PackageCheckPanel from "@/features/package/PackageCheckPanel";
import ExternalImportPanel from "@/features/package/ExternalImportPanel";
import KieExamplesPanel from "@/features/dossier/KieExamplesPanel";
import ChuyenDuLieuCu from "@/features/chinhly/ChuyenDuLieuCu";
import { Card, EmptyState, PageHeader, Tabs } from "@/ui";

export default function TienIchHub() {
  const { session, tienIchTab, gotoTienIch } = V460();
  const duAnTopbar = useProject();
  const perms = session?.permissions;
  const coGoi = V3z9(perms, "package.view");
  const coDuyet = V3z9(perms, "document.review");
  const coHoSo = V3z9(perms, "dossier.read");

  const tabs = useMemo(
    () =>
      [
        { value: "pdfa" as const, label: "Kiểm PDF/A", icon: "kiemTraPdfa" as const, show: coGoi },
        { value: "kiemgoi" as const, label: "Kiểm tra gói tin", icon: "kiemTra" as const, show: coGoi },
        { value: "khongoai" as const, label: "Từ kho ngoài", icon: "nhapKho" as const, show: coGoi },
        { value: "nhanh" as const, label: "Đóng gói nhanh", icon: "chay" as const, show: coGoi },
        { value: "vidu" as const, label: "Ví dụ dạy máy", icon: "danhSach" as const, show: coDuyet },
        { value: "chuyendulieu" as const, label: "Chuyển dữ liệu cũ", icon: "chuyenDoi" as const, show: coHoSo },
      ].filter((t) => t.show),
    [coGoi, coDuyet, coHoSo],
  );
  const tab: V3xTienIch | null = tabs.some((t) => t.value === tienIchTab) ? tienIchTab : (tabs[0]?.value ?? null);

  // Ví dụ dạy máy cần dự án: tài khoản gắn dự án thì theo đó, không thì theo dự án đang chọn trên thanh trên.
  const [projects, setProjects] = useState<V4a9[]>([]);
  useEffect(() => {
    if (!isTauri() || tab !== "vidu") return;
    V4ak()
      .then(setProjects)
      .catch(() => setProjects([]));
  }, [tab]);
  const duAn = session?.projectId ?? duAnTopbar.projectId ?? "";

  return (
    <section className="view active">
      <PageHeader title="Tiện ích" description="Công cụ ngoài trục thi công TT 05/2025: kiểm tra tệp và gói, đóng gói từ kho bên ngoài, dạy máy bóc trường, chuyển dữ liệu cũ." />
      {tabs.length === 0 || !tab ? (
        <Card padding="md">
          <EmptyState icon="caiDat" title="Tài khoản chưa có quyền dùng tiện ích nào" description="Cần quyền đóng gói, duyệt OCR hoặc hồ sơ chỉnh lý." />
        </Card>
      ) : (
        <>
          <Tabs<V3xTienIch> value={tab} onChange={gotoTienIch} items={tabs.map(({ value, label, icon }) => ({ value, label, icon }))} />
          <div className="mt-3">
            {tab === "pdfa" && <PdfaCheckPanel />}
            {tab === "kiemgoi" && <PackageCheckPanel />}
            {tab === "khongoai" && <ExternalImportPanel />}
            {tab === "nhanh" && <ExternalImportPanel nhanh />}
            {tab === "vidu" && (duAn ? <KieExamplesPanel projectId={duAn} projects={projects} /> : <Card padding="md"><EmptyState icon="duAn" title="Chọn dự án đang làm ở thanh trên" description="Ví dụ dạy máy xếp theo dự án." /></Card>)}
            {tab === "chuyendulieu" && (
              <Card title="Chuyển dữ liệu chỉnh lý cũ" description="Đưa dữ liệu SQLite của bản DigiText cũ vào máy chủ. Cũng có ở Cài đặt › Dữ liệu.">
                <ChuyenDuLieuCu />
              </Card>
            )}
          </div>
        </>
      )}
    </section>
  );
}
