// Copyright (c) 2026 DigiText. All rights reserved.
//
// "Kho trên máy này" (14/10/2026): quét thư mục Kho theo dự án → phông, thấy ngay phông nào còn tệp
// chưa ghi sổ (chép tay bằng Explorer) hoặc chưa lên máy chủ. 19/10/2026: một nút "Đẩy" cho mỗi phông —
// tệp chép tay được ghi sổ tại chỗ (không chép) và gửi lên máy chủ ngay trong cùng lượt (`direct`).

import { useState } from "react";
import { khoPendingIds, khoScan, type KhoPhong } from "@/api/khoScan";
import type { V43b, V43d, V43e } from "@/api/lansync";
import { Badge, Button, Card, notifyError, Table, toast } from "@/ui";
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

  const choDay = (r: KhoPhong) => r.chuaGhiSo + r.chuaDay;

  // Đẩy một phông: còn tệp chép tay chưa ghi sổ thì ghi sổ tại chỗ + gửi luôn (lượt `direct` cũng gửi
  // các tệp đã ghi sổ mà chưa lên); chỉ còn tệp đã ghi sổ thì đẩy theo id như cũ.
  const dayPhong = async (r: KhoPhong): Promise<string> => {
    const ten = r.phong || r.projectDir;
    if (r.chuaGhiSo > 0 && r.projectId) {
      const kq: V43e = await startAddFiles({ paths: [r.path], projectId: r.projectId, projectLabel: r.projectDir, direct: true });
      const phan = [`${fmtN(kq.pushed)} tệp lên máy chủ`];
      if (kq.added > 0) phan.push(`${fmtN(kq.added)} tệp mới ghi sổ`);
      if (kq.pushSkipped > 0) phan.push(`${fmtN(kq.pushSkipped)} máy chủ đã có`);
      const loi = kq.failed + kq.pushFailed + kq.pushRejected;
      if (loi > 0) phan.push(`${fmtN(loi)} lỗi`);
      const dung = kq.cancelled + kq.pushCancelled;
      if (dung > 0) phan.push(`${fmtN(dung)} đã dừng — bấm Đẩy để chạy tiếp`);
      return `${ten}: ${phan.join(" · ")}.`;
    }
    const ids = await khoPendingIds(r.path);
    if (ids.length === 0) return `${ten}: không còn tệp nào chờ đẩy.`;
    return `${ten}: ${V4pu(await startForward(ids))}`;
  };

  const lamMoi = async () => {
    await onChanged();
    try {
      setRows(await khoScan());
    } catch {
      /* giữ bảng cũ */
    }
  };

  const day = async (r: KhoPhong) => {
    setBusy(`day:${r.path}`);
    try {
      toast.success(await dayPhong(r));
    } catch (e) {
      notifyError(e, `đẩy ${r.phong || r.projectDir}`);
    } finally {
      await lamMoi();
      setBusy(null);
    }
  };

  const dayHet = async () => {
    const ds = (rows ?? []).filter((r) => choDay(r) > 0 && (r.chuaGhiSo === 0 || r.projectId));
    try {
      for (const r of ds) {
        setBusy(`day:${r.path}`);
        try {
          toast.success(await dayPhong(r));
        } catch (e) {
          notifyError(e, `đẩy ${r.phong || r.projectDir}`);
          break;
        }
      }
    } finally {
      await lamMoi();
      setBusy(null);
    }
  };

  const tong = (rows ?? []).reduce((s, r) => ({ chuaGhiSo: s.chuaGhiSo + r.chuaGhiSo, chuaDay: s.chuaDay + r.chuaDay }), { chuaGhiSo: 0, chuaDay: 0 });
  const tongCho = tong.chuaGhiSo + tong.chuaDay;
  const khoa = disabled || busy !== null;

  return (
    <Card
      title="Kho trên máy này"
      description={
        rows === null
          ? "Quét thư mục Kho để xem từng phông còn tệp chưa ghi sổ (chép tay vào Kho) hoặc chưa lên máy chủ."
          : tongCho > 0
            ? `${fmtN(rows.length)} phông · ${fmtN(tongCho)} tệp chưa lên máy chủ. Bấm Đẩy: tệp chép tay vào Kho được ghi sổ tại chỗ (không chép lại) rồi gửi lên luôn.`
            : `${fmtN(rows.length)} phông · tất cả đã lên máy chủ.`
      }
      actions={
        <div className="flex gap-1.5">
          <Button size="sm" icon="lamMoi" loading={busy === "scan"} disabled={khoa} onClick={() => void quet()}>
            {rows === null ? "Quét Kho" : "Quét lại"}
          </Button>
          {tongCho > 0 && (
            <Button size="sm" variant="primary" icon="taiLen" disabled={khoa} onClick={() => void dayHet()}>
              Đẩy tất cả {fmtN(tongCho)}
            </Button>
          )}
        </div>
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
              key: "choDay",
              header: "Chưa lên",
              align: "right",
              cell: (r) =>
                choDay(r) > 0 ? (
                  <div>
                    <Badge tone="info">{fmtN(choDay(r))}</Badge>
                    {r.chuaGhiSo > 0 && (
                      <div className="text-xs text-[var(--text-muted)]">{`${fmtN(r.chuaGhiSo)} chép tay · ${fmtGb(r.chuaGhiSoBytes)}`}</div>
                    )}
                  </div>
                ) : (
                  "—"
                ),
            },
            { key: "loi", header: "Lỗi", align: "right", cell: (r) => (r.loi > 0 ? <Badge tone="err">{fmtN(r.loi)}</Badge> : "—") },
            {
              key: "act",
              header: "",
              align: "right",
              cell: (r) => (
                <div className="flex justify-end">
                  {choDay(r) > 0 && (
                    <Button
                      size="sm"
                      variant="primary"
                      icon="taiLen"
                      loading={busy === `day:${r.path}`}
                      disabled={khoa || (r.chuaGhiSo > 0 && !r.projectId)}
                      title={r.chuaGhiSo > 0 && !r.projectId ? "Thư mục dự án chưa gắn dự án máy chủ" : undefined}
                      onClick={() => void day(r)}
                    >
                      Đẩy {fmtN(choDay(r))}
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
