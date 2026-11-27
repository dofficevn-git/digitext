// Copyright (c) 2026 DigiText. All rights reserved.
// Màn "Cơ quan lưu trữ (TT05)" trong menu Quản trị của bản Lưu trữ (27/11/2026): xem cấu hình đang hiệu lực của kho lưu trữ
// (dự án) đang chọn và mở hộp thoại sửa (CoQuanDialog). Digifile đặt mục này ở Cấu hình › Cơ quan lưu trữ (TT05).

import { useCallback, useEffect, useState } from "react";
import { coQuanGet, LOAI_HINH, type CoQuanDto } from "@/api/coQuan";
import { V3z9 } from "@/domain/permissions";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { Badge, Button, EmptyState, ErrorBanner, errorText, PageHeader, Toolbar } from "@/ui";
import CoQuanDialog from "./CoQuanDialog";

export default function CoQuanView() {
  const { session } = V460();
  const has = (p: Parameters<typeof V3z9>[1]) => V3z9(session?.permissions, p);
  const { projectId, projectLabel } = useProject();
  const [dto, setDto] = useState<CoQuanDto | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const [sua, setSua] = useState(false);
  const canEdit = has("project.update");

  const tai = useCallback(() => {
    if (!projectId) return;
    setErr(null);
    coQuanGet(projectId)
      .then(setDto)
      .catch((e) => setErr(errorText(e)));
  }, [projectId]);
  useEffect(() => {
    tai();
  }, [tai]);

  if (!projectId) {
    return (
      <section className="view active">
        <PageHeader title="Cơ quan lưu trữ" />
        <EmptyState title="Chưa chọn kho lưu trữ" description="Chọn kho lưu trữ (dự án) ở thanh trên rồi quay lại." />
      </section>
    );
  }
  const c = dto?.cauHinh;
  const h = dto?.hieuDung;
  const loaiHinh = LOAI_HINH.find(([v]) => v === (c?.loaiHinh ?? h?.loaiHinh))?.[1] ?? "";
  const dong: Array<[string, string]> = h
    ? [
        ["Mã cơ quan lưu trữ", c?.maCoQuan ?? "—"],
        ["Tên cơ quan", h.ten],
        ["Cơ quan cấp trên", c?.tenCapTren ?? "—"],
        ["Loại hình", loaiHinh],
        ["Địa chỉ", c?.diaChi ?? "—"],
        ["Điện thoại / email", [c?.dienThoai, c?.email].filter(Boolean).join(" · ") || "—"],
        ["Người ký", c?.nguoiKy ?? "— (chừa chỗ ký)"],
        ["Chức vụ người ký", h.chucVuNguoiKy],
        ["Hạn nộp lưu", `${h.hanNopNgay} ngày kể từ ngày kết thúc hồ sơ`],
        ["Nhắc trước hạn", h.nhacTruocNgay.map((x) => `${x} ngày`).join(", ")],
        ["Quyền sử dụng sau nộp", `${h.quyenSuDungNgay} ngày`],
        ["Ghi chú", c?.ghiChu ?? "—"],
      ]
    : [];

  return (
    <section className="view active">
      <PageHeader
        title="Cơ quan lưu trữ"
        description="Thông tin cơ quan dùng cho báo cáo, chuyển đổi và nộp lưu"
        info="In lên đầu và khối ký của báo cáo TT05, trang thể thức chuyển đổi (Phụ lục VI); quyết định hạn nộp lưu và mốc nhắc theo Điều 19. Chưa khai thì dùng tên kho lưu trữ và tham số mặc định."
        actions={
          <Toolbar>
            {h && <Badge tone={h.daKhai ? "ok" : "warn"}>{h.daKhai ? "Đã khai" : "Chưa khai — đang dùng mặc định"}</Badge>}
            <Button onClick={tai}>Tải lại</Button>
            <Button variant="primary" onClick={() => setSua(true)} disabled={!dto}>
              {canEdit ? "Sửa" : "Xem chi tiết"}
            </Button>
          </Toolbar>
        }
      />
      {err ? (
        <ErrorBanner error={err} />
      ) : !h ? (
        <p className="text-sm text-fg-subtle">Đang tải…</p>
      ) : (
        <dl className="grid max-w-3xl grid-cols-[14rem_1fr] gap-x-6 gap-y-2 text-sm">
          {dong.map(([k, v]) => (
            <div key={k} className="contents">
              <dt className="text-fg-subtle">{k}</dt>
              <dd className="m-0 whitespace-pre-wrap">{v}</dd>
            </div>
          ))}
        </dl>
      )}
      {sua && (
        <CoQuanDialog
          projectId={projectId}
          projectLabel={projectLabel}
          canEdit={canEdit}
          onClose={() => setSua(false)}
          onSaved={tai}
        />
      )}
    </section>
  );
}
