// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import { V460 } from "@/state";
import { isTauri } from "@/api/ipc";
import { V40w, type V40v } from "@/api/archive";
import { V4ny, V4o0, V4nz, V4nx } from "@/api/report";
import V4o9 from "./ReportView";
import ProjectProgressView from "./ProjectProgressView";
import StaffReportView from "./StaffReportView";
import { V4o4, V4o5, V4o6, V4o3 } from "./reportLogic";
import ThongKeView from "@/features/chinhly/ThongKeView";

function V4oa() {
  return (
    <>
      <ProjectProgressView />
      <details className="mt-3.5">
        <summary className="muted text-sm cursor-pointer">Chi tiết khổ giấy theo phông (dữ liệu trên máy này)</summary>
        <V4oaCu />
      </details>
    </>
  );
}

function V4oaCu() {
  return (
    <V4o9
      title="Tổng kết số hoá"
      subtitle="Số hồ sơ/PDF đã số hoá và số trang theo khổ giấy (quy đổi A4) — gom theo phông."
      columns={V4o3}
      fetchRows={V4nx}
      sheetName="TongKetSoHoa"
      fileBase="tong-ket-so-hoa"
    />
  );
}

function V4ob() {
  return (
    <>
      <StaffReportView />
      <details className="mt-3.5" open>
        <summary className="muted text-sm cursor-pointer">Chỉnh lý (máy chủ): hồ sơ, văn bản, số trường thêm mới / sửa theo lịch sử</summary>
        <ThongKeView kind="nhan-vien" />
      </details>
      <details className="mt-3.5">
        <summary className="muted text-sm cursor-pointer">Thao tác trên máy này (hồ sơ, văn bản, thêm/sửa)</summary>
        <V4obCu />
      </details>
    </>
  );
}

function V4obCu() {
  return (
    <V4o9
      title="Tổng kết theo nhân viên"
      subtitle="Khối lượng công việc quy về người đăng nhập: hồ sơ, văn bản, trang quét, thao tác."
      columns={V4o5}
      fetchRows={V4o0}
      sheetName="TongKetNhanVien"
      fileBase="tong-ket-nhan-vien"
    />
  );
}

function V4oc() {
  const [V22d, V22e] = useState<V40v[]>([]);
  const [maDa, V22f] = useState<string>("");
  const V22g = useCallback(() => {
    V40w()
      .then(V22e)
      .catch(() => {});
  }, []);
  V41p(V22g, { topics: ["catalog", "hoso"], enabled: isTauri() });
  return (
    <V4o9
      title="Tổng kết chỉnh lý"
      subtitle="Hộp và hồ sơ vĩnh viễn / có thời hạn, bó–tập hồ sơ loại — gom theo phòng ban."
      columns={V4o4}
      fetchRows={() => V4ny(maDa || undefined)}
      sheetName="TongKetChinhLy"
      fileBase="tong-ket-chinh-ly"
      filters={
        <select className="inp max-w-65" value={maDa} onChange={(V22h) => V22f(V22h.target.value)}>
          <option value="">— Tất cả phông —</option>
          {V22d.map((p) => (
            <option key={p.maDuAn} value={p.maDuAn}>
              {p.tenDuAn}
            </option>
          ))}
        </select>
      }
    />
  );
}

function V4od() {
  const [V22i, V22j] = useState<string>("");
  const [V22k, V22l] = useState<string>("");
  return (
    <V4o9
      title="Tổng kết phần mềm"
      subtitle="Hộp/hồ sơ vĩnh viễn & có thời hạn, bó–tập loại — gom theo phông, lọc theo ngày khởi tạo."
      columns={V4o6}
      fetchRows={() => V4nz(V22i || undefined, V22k || undefined)}
      sheetName="TongKetPhanMem"
      fileBase="tong-ket-phan-mem"
      filters={
        <>
          <label className="muted text-sm">
            Từ ngày
            <input
              className="inp ml-1.5"
              type="date"
              value={V22i}
              onChange={(V22m) => V22j(V22m.target.value)}
            />
          </label>
          <label className="muted text-sm ml-2.5">
            Đến ngày
            <input
              className="inp ml-1.5"
              type="date"
              value={V22k}
              onChange={(V22n) => V22l(V22n.target.value)}
            />
          </label>
        </>
      }
    />
  );
}

export default function V4oe() {
  const { reportTab: V22o } = V460();
  if (V22o === "chinhly")
    return (
      <>
        <ThongKeView kind="chinh-ly" />
        <details className="mt-3.5">
          <summary className="muted text-sm cursor-pointer">Số liệu chỉnh lý cũ trên máy này</summary>
          <V4oc />
        </details>
      </>
    );
  if (V22o === "nhanvien") return <V4ob />;
  if (V22o === "phanmem")
    return (
      <>
        <ThongKeView kind="phan-mem" />
        <details className="mt-3.5">
          <summary className="muted text-sm cursor-pointer">Số liệu cũ trên máy này</summary>
          <V4od />
        </details>
      </>
    );
  if (V22o === "muontra") return <ThongKeView kind="muon-tra" />;
  return <V4oa />;
}
