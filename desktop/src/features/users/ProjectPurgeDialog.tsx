// Copyright (c) 2026 DigiText. All rights reserved.
//
// "Xoá dữ liệu dự án" (15/10/2026): chạy thử → gõ lại mã dự án + lý do → máy chủ xoá nền, hiện tiến độ.
// Giữ lại vỏ dự án (tên đổi thành "[ĐÃ XOÁ] …"), quản trị, thành viên, tài khoản, nhật ký. Không hoàn tác được.

import { useEffect, useRef, useState } from "react";
import { projectPurgePreview, projectPurgeStart, projectPurgeStatus, type PurgePreview, type PurgeStatus } from "@/api/projectPurge";
import { Banner, Button, Checkbox, Dialog, Field, Input, KpiRow, KpiTile, notifyError, Textarea, toast } from "@/ui";

const fmtN = (n: number) => n.toLocaleString("vi-VN");

export default function ProjectPurgeDialog({ projectId, onClose, onDone }: { projectId: string; onClose: () => void; onDone: () => void }) {
  const [preview, setPreview] = useState<PurgePreview | null>(null);
  const [status, setStatus] = useState<PurgeStatus | null>(null);
  const [ma, setMa] = useState("");
  const [lyDo, setLyDo] = useState("");
  const [chuaBanGiao, setChuaBanGiao] = useState(false);
  const [busy, setBusy] = useState(false);
  const timer = useRef<number | null>(null);

  useEffect(() => {
    let huy = false;
    void (async () => {
      try {
        const [p, s] = await Promise.all([projectPurgePreview(projectId), projectPurgeStatus(projectId)]);
        if (huy) return;
        setPreview(p);
        if (s?.state === "running") theoDoi();
      } catch (e) {
        notifyError(e, "xem số liệu dự án");
      }
    })();
    return () => {
      huy = true;
      if (timer.current) window.clearInterval(timer.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [projectId]);

  const theoDoi = () => {
    if (timer.current) window.clearInterval(timer.current);
    const tick = async () => {
      try {
        const s = await projectPurgeStatus(projectId);
        setStatus(s);
        if (s && s.state !== "running") {
          if (timer.current) window.clearInterval(timer.current);
          timer.current = null;
          if (s.state === "done") toast.success(`Đã xoá dữ liệu dự án: ${fmtN(s.docsDone)} tài liệu, ${fmtN(s.objectsDeleted)} tệp trong kho.`);
          onDone();
        }
      } catch {
        /* mất mạng một nhịp — thử lại ở nhịp sau */
      }
    };
    void tick();
    timer.current = window.setInterval(() => void tick(), 2000);
  };

  const xoa = async () => {
    if (!preview) return;
    setBusy(true);
    try {
      await projectPurgeStart({ projectId, confirmCode: ma.trim(), reason: lyDo.trim(), allowUndelivered: chuaBanGiao });
      theoDoi();
    } catch (e) {
      notifyError(e, "xoá dữ liệu dự án");
    } finally {
      setBusy(false);
    }
  };

  const c = preview?.counts;
  const dangXoa = status?.state === "running";
  const xong = status?.state === "done" || status?.state === "failed";
  const duocXoa =
    !!preview &&
    preview.blockers.length === 0 &&
    ma.trim().toLowerCase() === preview.projectCode.trim().toLowerCase() &&
    lyDo.trim().length >= 5 &&
    (!preview.undelivered || chuaBanGiao) &&
    !dangXoa &&
    !xong;

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !dangXoa && onClose()}
      closable={!dangXoa}
      size="md"
      title={preview ? `Xoá dữ liệu dự án ${preview.projectCode}` : "Xoá dữ liệu dự án"}
      description={preview?.projectName}
      footer={
        xong ? (
          <Button variant="primary" onClick={onClose}>
            Đóng
          </Button>
        ) : (
          <>
            <Button onClick={onClose} disabled={dangXoa}>
              Huỷ
            </Button>
            <Button variant="danger" icon="xoa" loading={busy || dangXoa} disabled={!duocXoa} onClick={() => void xoa()}>
              Xoá vĩnh viễn
            </Button>
          </>
        )
      }
    >
      {!preview && <p className="text-sm text-fg-subtle">Đang đếm dữ liệu của dự án…</p>}
      {preview && c && (
        <div className="space-y-4">
          <KpiRow>
            <KpiTile label="Tài liệu" value={c.documents} />
            <KpiTile label="Đã ký số" value={c.signed} />
            <KpiTile label="Hồ sơ chỉnh lý" value={c.hoSo} />
            <KpiTile label="Gói bàn giao" value={c.packages} tone={c.packages === 0 ? "warn" : undefined} />
          </KpiRow>
          {status ? (
            <Banner tone={status.state === "failed" ? "danger" : status.state === "done" ? "success" : "info"} title={status.state === "running" ? "Đang xoá…" : status.state === "done" ? "Đã xoá xong" : "Xoá dở dang"}>
              {fmtN(status.docsDone)}/{fmtN(status.docsTotal)} tài liệu · {fmtN(status.objectsDeleted)} tệp trong kho
              {status.error ? ` — ${status.error}. Mở lại hộp này và xoá tiếp để chạy nốt phần còn lại.` : ""}
            </Banner>
          ) : (
            <>
              <Banner tone="danger" title="Không hoàn tác được">
                Xoá toàn bộ tài liệu, bản PDF/A, bản ký số, gói và dữ liệu chỉnh lý của dự án trong kho máy chủ. Giữ lại tài khoản, quản trị, thành viên và nhật ký; tên dự án đổi thành “[ĐÃ XOÁ] …”. Nên xuất gói bàn giao hoặc sao lưu trước.
              </Banner>
              {preview.blockers.length > 0 && (
                <Banner tone="warning" title="Chưa xoá được">
                  {preview.blockers.join("; ")}.
                </Banner>
              )}
              {preview.undelivered && <Checkbox checked={chuaBanGiao} onChange={setChuaBanGiao} text="Dự án chưa có gói bàn giao nào — vẫn xoá" />}
              <Field label={`Gõ lại mã dự án (${preview.projectCode}) để xác nhận`}>
                <Input value={ma} onChange={(e) => setMa(e.target.value)} autoComplete="off" spellCheck={false} />
              </Field>
              <Field label="Lý do xoá" hint="Ghi vào nhật ký hệ thống.">
                <Textarea value={lyDo} onChange={(e) => setLyDo(e.target.value)} rows={2} placeholder="Ví dụ: đã bàn giao gói SIP ngày …, hết hạn lưu tạm" />
              </Field>
            </>
          )}
        </div>
      )}
    </Dialog>
  );
}
