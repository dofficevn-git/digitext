// Copyright (c) 2026 DigiText. All rights reserved.
// Danh sách PDF đã quét trong một hồ sơ; mở từng tệp để soát OCR và nhập văn bản.

import { useCallback, useState } from "react";
import { V41p } from "@/live/useLive";
import { V40x } from "@/api/archive";
import { V4cb, type V4bm, type V4ca } from "@/api/dossier";
import V4h7 from "./OcrReviewPanel";
import { Button, Card, ErrorBanner, IconButton, notifyError, PageHeader, Table, type Column } from "@/ui";

export default function V4h8({ dossier, onBack }: { dossier: V4bm; onBack: () => void }) {
  const [rows, setRows] = useState<V4ca[]>([]);
  const [loading, setLoading] = useState(true);
  const [err, setErr] = useState<unknown>(null);
  const [open, setOpen] = useState<V4ca | null>(null);

  const reload = useCallback(() => {
    V4cb(dossier.id)
      .then((rs) => {
        setRows(rs);
        setErr(null);
      })
      .catch(setErr)
      .finally(() => setLoading(false));
  }, [dossier.id]);
  V41p(reload, { topics: ["hoso", "dossier", "scan"] });

  const moPdf = (r: V4ca) => V40x(r.filePath).catch((e) => notifyError(e, "mở PDF"));

  if (open) {
    return (
      <V4h7
        dossier={dossier}
        file={open}
        onBack={() => {
          setOpen(null);
          reload();
        }}
      />
    );
  }

  const cols: Column<V4ca>[] = [
    { key: "stt", header: "#", align: "right", width: 50, cell: (_r, i) => i + 1 },
    { key: "fileName", header: "Tệp PDF", cell: (r) => <button type="button" className="text-left hover:text-fg-link" onClick={() => void moPdf(r)}>{r.fileName}</button> },
    { key: "loaiVanBan", header: "Loại VB", width: 120, cell: (r) => r.loaiVanBan ?? "" },
    { key: "soKyHieu", header: "Ký hiệu", width: 130, cell: (r) => r.soKyHieu ?? "" },
    { key: "ngayThang", header: "Ngày", width: 100, cell: (r) => r.ngayThang ?? "" },
    { key: "tacGia", header: "Tác giả", className: "whitespace-normal", cell: (r) => r.tacGia ?? "", hideBelow: 1366 },
    { key: "trichYeu", header: "Trích yếu", className: "whitespace-normal min-w-56", cell: (r) => r.trichYeu ?? "" },
    { key: "pageCount", header: "Trang", align: "right", width: 70 },
    { key: "tinhTrangVatLy", header: "TT vật lý", width: 100, cell: (r) => r.tinhTrangVatLy ?? "", hideBelow: 1440 },
  ];

  return (
    <>
      <PageHeader onBack={onBack} title={`Hồ sơ ${dossier.soTam}`} description={dossier.tieuDe || undefined} meta={<span className="text-sm text-fg-subtle">{rows.length.toLocaleString("vi-VN")} tệp PDF</span>} />
      {err && rows.length > 0 && <ErrorBanner error={err} onRetry={reload} className="mb-3" />}
      <Card padding="none">
        <Table
          columns={cols}
          rows={rows}
          getRowId={(r) => r.filePath}
          onRowClick={(r) => setOpen(r)}
          rowActions={(r) => (
            <>
              <Button size="sm" icon="tuDong" onClick={() => setOpen(r)}>
                Soát OCR
              </Button>
              <IconButton label="Mở PDF" icon="tepPdf" size="sm" onClick={() => void moPdf(r)} />
            </>
          )}
          state={{ loading, error: rows.length ? undefined : err, onRetry: reload, empty: { title: "Hồ sơ chưa có tệp PDF nào", description: "Quét vào hồ sơ này hoặc đưa tệp từ ổ đĩa vào Kho." } }}
        />
      </Card>
    </>
  );
}
