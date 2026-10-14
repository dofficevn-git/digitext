// Copyright (c) 2026 DigiText. All rights reserved.
//
// "Kho trên máy này" (14/10/2026): quét thư mục Kho theo dự án → phông, thấy ngay phông nào còn tệp
// chưa ghi sổ (chép tay bằng Explorer) hoặc chưa lên máy chủ; ghi sổ / đẩy riêng từng phông.

import { useState } from "react";
import { khoPendingIds, khoScan, type KhoPhong } from "@/api/khoScan";
import type { V43b, V43d, V43e } from "@/api/lansync";
import { Badge, Button, Card, notifyError, Table, toast } from "@/ui";
import { V4ps } from "./addFilesMessage";
import { V4pu } from "./forwardMessage";

const fmtN = (n: number) => n.toLocaleString("vi-VN");
const fmtGb = (b: number) => (b >= 1e9 ? `${(b / 1e9).toLocaleString("vi-VN", { maximumFractionDigits: 1 })} GB` : `${Math.max(1, Math.round(b / 1e6)).toLocaleString("vi-VN")} MB`);

export default function KhoScanPanel({
  disabled,
  startAddFiles,
  startForward,
  onChanged,
}: {
  disabled: boolean;
  startAddFiles: (a: V43d) => Promise<V43e>;
  startForward: (ids?: number[]) => Promise<V43b>;
  onChanged: () => void | Promise<void>;
}) {
  const [rows, setRows] = useState<KhoPhong[] | null>(null);
  const [busy, setBusy] = useState<string | null>(null);

  const quet = async () => {
    setBusy("scan");
    try {
      setRows(await khoScan());
    } catch (e) {
      notifyError(e, "quét Kho");
    } finally {
      setBusy(null);
    }
  };

  const ghiSo = async (r: KhoPhong) => {
    if (!r.projectId) return;
    setBusy(`ghi:${r.path}`);
    try {
      const kq = await startAddFiles({ paths: [r.path], projectId: r.projectId, projectLabel: r.projectDir, direct: false });
      toast.success(V4ps(kq));
      await onChanged();
      setRows(await khoScan());
    } catch (e) {
      notifyError(e, `ghi sổ ${r.phong || r.projectDir}`);
    } finally {
      setBusy(null);
    }
  };

  const day = async (r: KhoPhong) => {
    setBusy(`day:${r.path}`);
    try {
      const ids = await khoPendingIds(r.path);
      if (ids.length === 0) {
        toast.info("Phông này không còn tệp nào chờ đẩy.");
      } else {
        const kq = await startForward(ids);
        toast.success(V4pu(kq));
      }
      await onChanged();
      setRows(await khoScan());
    } catch (e) {
      notifyError(e, `đẩy ${r.phong || r.projectDir}`);
    } finally {
      setBusy(null);
    }
  };

  const tong = (rows ?? []).reduce((s, r) => ({ chuaGhiSo: s.chuaGhiSo + r.chuaGhiSo, chuaDay: s.chuaDay + r.chuaDay }), { chuaGhiSo: 0, chuaDay: 0 });
  const khoa = disabled || busy !== null;

  return (
    <Card
      title="Kho trên máy này"
      description={
        rows === null
          ? "Quét thư mục Kho để xem từng phông còn tệp chưa ghi sổ (chép tay vào Kho) hoặc chưa lên máy chủ."
          : `${fmtN(rows.length)} phông · chưa ghi sổ ${fmtN(tong.chuaGhiSo)} · chưa lên máy chủ ${fmtN(tong.chuaDay)}`
      }
      actions={
        <Button size="sm" icon="lamMoi" loading={busy === "scan"} disabled={khoa} onClick={() => void quet()}>
          {rows === null ? "Quét Kho" : "Quét lại"}
        </Button>
      }
      padding={rows && rows.length > 0 ? "none" : undefined}
    >
      {rows !== null && rows.length === 0 && <p className="text-sm text-[var(--text-muted)]">Kho chưa có thư mục dự án nào có tệp PDF.</p>}
      {rows !== null && rows.length > 0 && (
        <Table<KhoPhong>
          dense
          maxHeight={360}
          rows={rows}
          getRowId={(r) => r.path}
          columns={[
            {
              key: "phong",
              header: "Phông",
              cell: (r) => (
                <div className="min-w-0">
                  <div className="truncate font-medium">{r.phong || "Tệp lẻ ở gốc dự án"}</div>
                  <div className="truncate text-xs text-[var(--text-muted)]">{r.projectDir}</div>
                </div>
              ),
            },
            { key: "total", header: "Tệp PDF", align: "right", cell: (r) => fmtN(r.total) },
            { key: "daDay", header: "Đã lên", align: "right", cell: (r) => fmtN(r.daDay) },
            {
              key: "chuaGhiSo",
              header: "Chưa ghi sổ",
              align: "right",
              cell: (r) => (r.chuaGhiSo > 0 ? <Badge tone="warn">{`${fmtN(r.chuaGhiSo)} · ${fmtGb(r.chuaGhiSoBytes)}`}</Badge> : "—"),
            },
            { key: "chuaDay", header: "Chưa lên", align: "right", cell: (r) => (r.chuaDay > 0 ? <Badge tone="info">{fmtN(r.chuaDay)}</Badge> : "—") },
            { key: "loi", header: "Lỗi", align: "right", cell: (r) => (r.loi > 0 ? <Badge tone="err">{fmtN(r.loi)}</Badge> : "—") },
            {
              key: "act",
              header: "",
              align: "right",
              cell: (r) => (
                <div className="flex justify-end gap-1.5">
                  {r.chuaGhiSo > 0 && (
                    <Button
                      size="sm"
                      loading={busy === `ghi:${r.path}`}
                      disabled={khoa || !r.projectId}
                      title={r.projectId ? "Ghi sổ tại chỗ (không chép), rồi bấm Đẩy" : "Thư mục dự án chưa gắn dự án máy chủ"}
                      onClick={() => void ghiSo(r)}
                    >
                      Ghi sổ {fmtN(r.chuaGhiSo)}
                    </Button>
                  )}
                  {r.chuaDay > 0 && (
                    <Button size="sm" variant="primary" icon="taiLen" loading={busy === `day:${r.path}`} disabled={khoa} onClick={() => void day(r)}>
                      Đẩy {fmtN(r.chuaDay)}
                    </Button>
                  )}
                </div>
              ),
            },
          ]}
        />
      )}
    </Card>
  );
}
