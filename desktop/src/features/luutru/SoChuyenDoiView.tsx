// Copyright (c) 2026 DigiText. All rights reserved.
// Sổ chuyển đổi tài liệu lưu trữ số ra giấy (0037, 27/11/2026 — Điều 13–16, Phụ lục VI TT05): mỗi lần in tài liệu số ra giấy từ
// màn Tra cứu ghi một dòng (số chuyển đổi, tài liệu gốc, chữ ký số lúc chuyển đổi, mục đích, người nhận, người thực hiện).
// Tải lại PDF đã ghép (trang thể thức ở cuối) hoặc xuất sổ Excel.

import { useCallback, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { V3z9 } from "@/domain/permissions";
import { cdSo, cdTai, cdXuatSo, type DongChuyenDoi } from "@/api/chuyenDoi";
import { CanDuAn, Badge, Button, Card, Field, FormGrid, Input, Menu, notifyError, PageHeader, Table, Tip, toast, type Column } from "@/ui";

const fmtLuc = (iso: string) => {
  const d = new Date(iso);
  return Number.isNaN(d.getTime()) ? iso : d.toLocaleString("vi-VN", { hour: "2-digit", minute: "2-digit", day: "2-digit", month: "2-digit", year: "numeric" });
};
const toneKy = (kl: string | null): "ok" | "warn" | "err" | "gray" => (kl === "Đạt" ? "ok" : kl?.startsWith("Đạt") ? "warn" : kl === "Chưa ký" ? "gray" : kl ? "err" : "gray");

export default function SoChuyenDoiView() {
  const { session, goto } = V460();
  const projectId = useProject().projectId ?? session?.projectId ?? "";
  const canBo = V3z9(session?.permissions, "access.review");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const loader = useCallback(async () => {
    if (!isTauri() || !projectId) return { rows: [] as DongChuyenDoi[], total: 0 };
    return cdSo(projectId, from || undefined, to || undefined);
  }, [projectId, from, to]);
  const { data, error, loading, reload } = V41t(loader, { topics: ["records"], errorText: () => "Không tải được sổ chuyển đổi." });
  const rows = useMemo(() => data?.rows ?? [], [data]);

  const tai = async (r: DongChuyenDoi) => {
    setBusy(r.id);
    try {
      const p = await cdTai(r);
      if (p) toast.success(`Đã lưu: ${p}`);
    } catch (e) {
      notifyError(e, "tải bản chuyển đổi");
    } finally {
      setBusy(null);
    }
  };
  const xuat = async () => {
    setBusy("xlsx");
    try {
      const p = await cdXuatSo(projectId, from || undefined, to || undefined);
      if (p) toast.success(`Đã xuất sổ: ${p}`);
    } catch (e) {
      notifyError(e, "xuất sổ");
    } finally {
      setBusy(null);
    }
  };

  const cols: Column<DongChuyenDoi>[] = [
    { key: "so_chuyen_doi", header: "Số chuyển đổi", width: 140, sortable: true, cell: (r) => <span className="font-medium">{r.so_chuyen_doi}</span> },
    { key: "created_at", header: "Thời điểm", width: 140, sortable: true, cell: (r) => fmtLuc(r.created_at) },
    {
      key: "ten_tai_lieu",
      header: "Tài liệu gốc",
      className: "max-w-[380px] whitespace-normal",
      cell: (r) => (
        <>
          <span className="block text-xs text-fg-subtle">{r.ma_tai_lieu}</span>
          {r.ten_tai_lieu}
          {r.ma_ho_so && <span className="block text-xs text-fg-subtle">Hồ sơ {r.ma_ho_so}{r.ten_phong ? ` · ${r.ten_phong}` : ""}</span>}
        </>
      ),
    },
    { key: "so_trang", header: "Trang", width: 70, align: "right", cell: (r) => r.so_trang ?? "—" },
    { key: "muc_dich", header: "Mục đích", className: "max-w-[220px] whitespace-normal text-sm", cell: (r) => r.muc_dich ?? "" },
    { key: "nguoi_nhan", header: "Người nhận", className: "text-sm", cell: (r) => r.nguoi_nhan ?? "", hideBelow: 1400 },
    {
      key: "ky_so_ket_luan",
      header: "Chữ ký số lúc chuyển đổi",
      width: 170,
      cell: (r) =>
        r.ky_so_chi_tiet ? (
          <Tip text={r.ky_so_chi_tiet}>
            <Badge tone={toneKy(r.ky_so_ket_luan)}>{r.ky_so_ket_luan ?? "—"}</Badge>
          </Tip>
        ) : (
          <Badge tone={toneKy(r.ky_so_ket_luan)}>{r.ky_so_ket_luan ?? "—"}</Badge>
        ),
    },
    { key: "nguoi_thuc_hien_ten", header: "Người thực hiện", className: "text-sm", cell: (r) => r.nguoi_thuc_hien_ten ?? "", hideBelow: 1500 },
  ];

  return (
    <section className="view active">
      <PageHeader
        title="Sổ chuyển đổi tài liệu số ra giấy"
        description="Điều 13–16, Phụ lục VI. Chuyển đổi từ màn Tra cứu."
        infoTitle="Cách làm"
        info={
          <>
            <p>Ở Tra cứu, cán bộ lưu trữ bấm "In ra giấy" trên văn bản cần cung cấp: hệ thống kiểm chữ ký số, ghép trang thể thức "TÀI LIỆU LƯU TRỮ CHUYỂN ĐỔI" vào cuối, đánh chân trang từng trang và cấp số chuyển đổi theo năm.</p>
            <p>Bản in có giá trị khi người có thẩm quyền ký và đóng dấu; hệ thống không ký số lại bản ghép. Mỗi lần chuyển đổi được ghi vào nhật ký khai thác.</p>
          </>
        }
        actions={
          <>
            {canBo && (
              <Button icon="tim" onClick={() => goto("tracuu")}>
                Tra cứu để chuyển đổi
              </Button>
            )}
            <Button icon="tepExcel" disabled={!projectId || rows.length === 0} loading={busy === "xlsx"} onClick={() => void xuat()}>
              Xuất sổ
            </Button>
          </>
        }
      />
      {!projectId ? (
        <Card padding="none">
          <CanDuAn />
        </Card>
      ) : (
        <>
          <Card className="mb-3" padding="sm">
            <FormGrid cols={4}>
              <Field label="Từ ngày">
                <Input size="sm" type="date" value={from} max={to || undefined} onChange={(e) => setFrom(e.target.value)} />
              </Field>
              <Field label="Đến ngày">
                <Input size="sm" type="date" value={to} min={from || undefined} onChange={(e) => setTo(e.target.value)} />
              </Field>
            </FormGrid>
          </Card>
          <Card padding="none" title={`${(data?.total ?? 0).toLocaleString("vi-VN")} lượt chuyển đổi`}>
            <Table
              columns={cols}
              rows={rows}
              getRowId={(r) => r.id}
              dense
              maxHeight={620}
              rowActions={(r) => <Menu size="sm" items={[{ label: "Tải PDF đã ghép", icon: "tepPdf", disabled: busy === r.id, onSelect: () => void tai(r) }]} />}
              state={{ loading: loading && !data, error, onRetry: reload, empty: { title: "Chưa có lượt chuyển đổi nào", description: "Chuyển đổi tài liệu số ra giấy từ màn Tra cứu.", icon: "inRa" } }}
            />
          </Card>
        </>
      )}
    </section>
  );
}
