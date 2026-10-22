// Copyright (c) 2026 DigiText. All rights reserved.
// Định mức và ngưỡng KPI theo dự án (Quản lý / Quản trị dự án): định mức/người/ngày từng chỉ số, ngưỡng Xuất sắc / Đạt,
// ngưỡng tiến độ (lệch điểm → vàng / đỏ), SLA gói việc, số người tối thiểu để hiện trung bình nhóm. Dùng chung cho
// Tổng quan (dự án) và Báo cáo nhân viên.

import { useEffect, useState } from "react";
import { kpiSettingsGet, kpiSettingsSet, QUOTA_METRICS, type KpiSettings, type KpiSettingsBody } from "@/api/dashboard";
import { Button, Dialog, ErrorBanner, Field, FormGrid, Help, Input, Skeleton, describeError, notifyError, toast } from "@/ui";

const n = (v: number) => v.toLocaleString("vi-VN");

function toBody(s: KpiSettings): KpiSettingsBody {
  return { quotas: { ...s.quotas }, excellentPct: s.excellentPct, passPct: s.passPct, excellentErrPct: s.excellentErrPct, passErrPct: s.passErrPct, ragAmberLech: s.ragAmberLech, ragRedLech: s.ragRedLech, reviewBacklogWarnDays: s.reviewBacklogWarnDays, workItemSlaHours: s.workItemSlaHours, groupMinUsers: s.groupMinUsers };
}

export default function KpiSettingsDialog({ projectId, projectLabel, onClose, onSaved }: { projectId: string; projectLabel?: string; onClose: () => void; onSaved?: (s: KpiSettings) => void }) {
  const [form, setForm] = useState<KpiSettingsBody | null>(null);
  const [quotaText, setQuotaText] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    let alive = true;
    kpiSettingsGet(projectId)
      .then((s) => {
        if (!alive) return;
        setForm(toBody(s));
        setQuotaText(Object.fromEntries(QUOTA_METRICS.map((m) => [m.key, s.quotas[m.key] != null ? String(s.quotas[m.key]) : ""])));
      })
      .catch((e) => alive && setError(describeError(e).message));
    return () => {
      alive = false;
    };
  }, [projectId]);

  const num = (k: keyof Omit<KpiSettingsBody, "quotas">) => (e: React.ChangeEvent<HTMLInputElement>) => setForm((f) => (f ? { ...f, [k]: Number(e.target.value) } : f));

  const luu = async () => {
    if (!form) return;
    const quotas: Record<string, number> = {};
    for (const m of QUOTA_METRICS) {
      const t = (quotaText[m.key] ?? "").trim();
      if (t === "") continue;
      const v = Number(t);
      if (!Number.isFinite(v) || v < 0) {
        notifyError(new Error(`Định mức «${m.label}» phải là số không âm`), "lưu định mức");
        return;
      }
      if (v > 0) quotas[m.key] = v;
    }
    setBusy(true);
    try {
      const saved = await kpiSettingsSet(projectId, { ...form, quotas });
      toast.success("Đã lưu định mức và ngưỡng KPI.");
      onSaved?.(saved);
      onClose();
    } catch (e) {
      notifyError(e, "lưu định mức");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      closable={!busy}
      title="Định mức và ngưỡng KPI"
      description={projectLabel ?? "Áp dụng cho dự án đang chọn"}
      size="lg"
      footer={
        <>
          <Button onClick={onClose} disabled={busy}>
            Huỷ
          </Button>
          <Button variant="primary" icon="luu" loading={busy} onClick={() => void luu()} disabled={!form}>
            Lưu
          </Button>
        </>
      }
    >
      {error && <ErrorBanner error={error} className="mb-3" />}
      {!form ? (
        <Skeleton.Rows rows={4} />
      ) : (
        <div className="space-y-5">
          <section>
            <h3 className="mb-2 text-sm font-semibold">Định mức mỗi người mỗi ngày</h3>
            <p className="mb-2 text-xs text-fg-subtle">Để trống chỉ số không áp định mức. Dashboard so sản lượng trung bình mỗi ngày có làm với số này.</p>
            <FormGrid cols={3}>
              {QUOTA_METRICS.map((m) => (
                <Field key={m.key} label={`${m.label} (${m.unit})`}>
                  <Input type="number" min={0} inputMode="numeric" value={quotaText[m.key] ?? ""} onChange={(e) => setQuotaText((t) => ({ ...t, [m.key]: e.target.value }))} placeholder="—" />
                </Field>
              ))}
            </FormGrid>
          </section>
          <section>
            <h3 className="mb-2 text-sm font-semibold">Xếp loại KPI</h3>
            <FormGrid cols={4}>
              <Field label="Xuất sắc: hoàn thành từ (%)">
                <Input type="number" min={0} value={form.excellentPct} onChange={num("excellentPct")} />
              </Field>
              <Field label="Xuất sắc: lỗi tối đa (%)">
                <Input type="number" min={0} step={0.1} value={form.excellentErrPct} onChange={num("excellentErrPct")} />
              </Field>
              <Field label="Đạt: hoàn thành từ (%)">
                <Input type="number" min={0} value={form.passPct} onChange={num("passPct")} />
              </Field>
              <Field label="Đạt: lỗi tối đa (%)">
                <Input type="number" min={0} step={0.1} value={form.passErrPct} onChange={num("passErrPct")} />
              </Field>
            </FormGrid>
          </section>
          <section>
            <h3 className="mb-2 text-sm font-semibold">Tiến độ dự án và gói việc</h3>
            <FormGrid cols={4}>
              <Field label="Có rủi ro khi lệch ≤ (điểm)" hint="lệch = % hoàn thành − % thời gian đã qua">
                <Input type="number" max={0} value={form.ragAmberLech} onChange={num("ragAmberLech")} />
              </Field>
              <Field label="Chậm tiến độ khi lệch ≤ (điểm)">
                <Input type="number" max={0} value={form.ragRedLech} onChange={num("ragRedLech")} />
              </Field>
              <Field label="SLA mục gói việc (giờ)">
                <Input type="number" min={1} value={form.workItemSlaHours} onChange={num("workItemSlaHours")} />
              </Field>
              <Field label="Nhóm tối thiểu để so sánh (người)" hint="Ẩn trung bình nhóm khi ít hơn">
                <Input type="number" min={2} value={form.groupMinUsers} onChange={num("groupMinUsers")} />
              </Field>
            </FormGrid>
          </section>
          <Help title="Cách tính">
            <p>
              Mức đạt = sản lượng trung bình mỗi ngày có làm ÷ định mức × 100. Xuất sắc khi đạt từ {n(form.excellentPct)} % và lỗi tối đa {form.excellentErrPct} %; Đạt khi từ {n(form.passPct)} % và lỗi tối đa {form.passErrPct} %; còn lại là Cảnh báo lỗi. Tình trạng dự án: vàng khi lệch ≤ {form.ragAmberLech} điểm hoặc dự kiến xong sau hạn; đỏ khi lệch ≤ {form.ragRedLech} điểm hoặc quá hạn mà chưa xong.
            </p>
          </Help>
        </div>
      )}
    </Dialog>
  );
}
