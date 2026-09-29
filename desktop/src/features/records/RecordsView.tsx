// Copyright (c) 2026 DigiText. All rights reserved.
// Biên bản theo Phụ lục VII Thông tư 05/2025: sổ biên bản (lọc theo loại, tải PDF) và hộp thoại lập biên bản
// hai bước (chọn mẫu bằng thẻ → điền nội dung, bảng dòng nhập tại chỗ).

import { useCallback, useMemo, useState } from "react";
import { V41t } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { V4t0, V4t1, V4sz, type V4sy } from "@/api/records";
import { V460 } from "@/state";
import { V4ti, V4tf, V4tg, type V4ta } from "./recordForms";
import { Icon } from "@/components/Icon";
import { Banner, Button, Card, Dialog, Field, FormGrid, IconButton, Input, notifyError, PageHeader, Select, Stepper, Table, Tip, toast, type Column } from "@/ui";

const TEN_LOAI: Record<string, string> = Object.fromEntries(V4tf.map((t) => [t.kind, t.label]));
TEN_LOAI["thu_nop"] = "Biên bản thu nộp";
TEN_LOAI["giao_nhan"] = "Biên bản giao nhận vào lưu trữ lịch sử";
TEN_LOAI["tra_lai"] = "Biên bản hồ sơ, tài liệu trả lại";

function LapBienBanDialog({ onClose, onCreated }: { onClose: () => void; onCreated: () => void }) {
  const { session } = V460();
  const [buoc, setBuoc] = useState(0);
  const [kind, setKind] = useState<V4ta>("kiem_tra");
  const [fields, setFields] = useState<Record<string, string>>({});
  const [lists, setLists] = useState<Record<string, Record<string, string>[]>>({});
  const [errs, setErrs] = useState<string[]>([]);
  const [fieldErrs, setFieldErrs] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState(false);
  const template = V4tg(kind);

  const chonMau = (k: V4ta) => {
    setKind(k);
    setFields({});
    const l: Record<string, Record<string, string>[]> = {};
    for (const x of V4tg(k).lists) l[x.key] = x.minRows > 0 ? [{}] : [];
    setLists(l);
    setErrs([]);
    setFieldErrs({});
  };
  const setCell = (lk: string, row: number, ck: string, v: string) =>
    setLists((p) => {
      const rows = [...(p[lk] ?? [])];
      rows[row] = { ...rows[row], [ck]: v };
      return { ...p, [lk]: rows };
    });

  const lap = async () => {
    const { payload, errors, fieldErrors } = V4ti(template, fields, lists);
    setErrs(errors);
    setFieldErrs(fieldErrors);
    if (!payload) return;
    setBusy(true);
    try {
      const row = await V4t0({ projectId: session?.projectId ?? null, kind, payload });
      toast.success(`Đã lập biên bản số ${row.soBienBan}.`);
      onCreated();
      onClose();
    } catch (e) {
      notifyError(e, "lập biên bản");
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      closable={!busy}
      size="xl"
      title="Lập biên bản"
      footer={
        <>
          <Button onClick={onClose} disabled={busy}>
            Huỷ
          </Button>
          {buoc === 1 && (
            <Button icon="truoc" onClick={() => setBuoc(0)} disabled={busy}>
              Chọn mẫu khác
            </Button>
          )}
          {buoc === 0 ? (
            <Button variant="primary" iconRight="sau" onClick={() => { chonMau(kind); setBuoc(1); }}>
              Điền nội dung
            </Button>
          ) : (
            <Button variant="primary" icon="bienBan" loading={busy} onClick={() => void lap()}>
              Lập biên bản
            </Button>
          )}
        </>
      }
    >
      <Stepper className="mb-4" current={buoc} steps={[{ key: "mau", label: "Chọn mẫu" }, { key: "dien", label: "Điền nội dung" }]} onSelect={(i) => i === 0 && setBuoc(0)} />
      {buoc === 0 ? (
        <div className="grid gap-3 sm:grid-cols-2" role="radiogroup" aria-label="Mẫu biên bản">
          {V4tf.map((t) => (
            <button
              key={t.kind}
              type="button"
              role="radio"
              aria-checked={kind === t.kind}
              onClick={() => setKind(t.kind)}
              onDoubleClick={() => { chonMau(t.kind); setBuoc(1); }}
              className={`rounded-md border p-3 text-left transition-colors ${kind === t.kind ? "border-brand-500 bg-brand-50 ring-2 ring-brand-500/30" : "border-border-subtle hover:bg-surface-hover"}`}
            >
              <span className="flex items-center gap-2 font-semibold">
                <Icon name="bienBan" size={16} className="text-fg-subtle" />
                {t.label}
              </span>
              <span className="mt-1 block text-sm text-fg-muted">{t.description}</span>
              <span className="mt-1 block text-xs text-fg-subtle">{t.legal}</span>
            </button>
          ))}
        </div>
      ) : (
        <div className="space-y-4">
          <p className="text-sm text-fg-subtle">
            <b className="text-fg">{template.label}</b> · {template.legal}
          </p>
          {errs.length > 0 && <Banner tone="danger" title={`${errs.length} chỗ chưa hợp lệ`} detail={errs.join("\n")}>Sửa các ô đánh dấu đỏ rồi lập lại.</Banner>}
          <FormGrid cols={3}>
            {template.fields.map((f) => (
              <Field key={f.key} label={f.legal ? <Tip text={`Theo ${f.legal}`}><span>{f.label}</span></Tip> : f.label} required={!f.optional} hint={f.hint} error={fieldErrs[f.key]}>
                <Input type={f.kind === "number" ? "number" : "text"} value={fields[f.key] ?? ""} onChange={(e) => setFields((v) => ({ ...v, [f.key]: e.target.value }))} />
              </Field>
            ))}
          </FormGrid>
          {template.lists.map((l) => (
            <div key={l.key}>
              <div className="mb-1.5 flex items-center gap-2">
                <h4 className="m-0 text-sm font-semibold">
                  {l.label}
                  {l.minRows > 0 && <span className="ml-0.5 text-danger-700">*</span>}
                </h4>
                {l.legal && <span className="text-xs text-fg-subtle">{l.legal}</span>}
              </div>
              <Table.Root dense>
                <Table.Head>
                  <tr>
                    <Table.HeadCell width={40}>#</Table.HeadCell>
                    {l.simple ? <Table.HeadCell>Nội dung</Table.HeadCell> : l.columns.map((c) => <Table.HeadCell key={c.key}>{c.label}{c.optional ? <span className="ml-1 font-normal text-fg-subtle">(tuỳ chọn)</span> : null}</Table.HeadCell>)}
                    <Table.HeadCell width={44}>
                      <span className="sr-only">Xoá dòng</span>
                    </Table.HeadCell>
                  </tr>
                </Table.Head>
                <Table.Body>
                  {(lists[l.key] ?? []).map((row, i) => (
                    <Table.Row key={i}>
                      <Table.Cell>{i + 1}</Table.Cell>
                      {l.simple ? (
                        <Table.Cell>
                          <Input size="sm" aria-label={`${l.label} dòng ${i + 1}`} value={row.value ?? ""} onChange={(e) => setCell(l.key, i, "value", e.target.value)} />
                        </Table.Cell>
                      ) : (
                        l.columns.map((c) => (
                          <Table.Cell key={c.key}>
                            <Input size="sm" aria-label={`${c.label} dòng ${i + 1}`} value={row[c.key] ?? ""} onChange={(e) => setCell(l.key, i, c.key, e.target.value)} />
                          </Table.Cell>
                        ))
                      )}
                      <Table.Cell>
                        <IconButton label="Xoá dòng" icon="xoa" size="sm" onClick={() => setLists((p) => ({ ...p, [l.key]: (p[l.key] ?? []).filter((_r, j) => j !== i) }))} />
                      </Table.Cell>
                    </Table.Row>
                  ))}
                  {(lists[l.key] ?? []).length === 0 && (
                    <Table.FullRow colSpan={(l.simple ? 1 : l.columns.length) + 2}>
                      <p className="px-3 py-2 text-sm text-fg-subtle">Chưa có dòng nào.</p>
                    </Table.FullRow>
                  )}
                </Table.Body>
              </Table.Root>
              <Button size="sm" icon="them" className="mt-1.5" onClick={() => setLists((p) => ({ ...p, [l.key]: [...(p[l.key] ?? []), {}] }))}>
                Thêm dòng
              </Button>
            </div>
          ))}
        </div>
      )}
    </Dialog>
  );
}

export default function V4tk() {
  const { session } = V460();
  const perms = session?.permissions ?? [];
  const loader = useCallback(() => (isTauri() ? V4sz() : Promise.resolve([] as V4sy[])), []);
  const { data, error, loading, reload } = V41t(loader, { topics: ["records"], errorText: () => "Không tải được sổ biên bản." });
  const rows = useMemo(() => data ?? [], [data]);
  const [loai, setLoai] = useState("");
  const [moLap, setMoLap] = useState(false);
  const [dangTai, setDangTai] = useState<string | null>(null);

  const taiPdf = async (row: V4sy) => {
    setDangTai(row.id);
    try {
      const path = await V4t1(row.id, `${row.soBienBan.split("/").join("-")}.pdf`);
      toast.success("Đã lưu biên bản.", { description: path });
    } catch (e) {
      notifyError(e, "tải biên bản");
    } finally {
      setDangTai(null);
    }
  };

  const cacLoai = useMemo(() => [...new Set(rows.map((r) => r.kind))].map((k) => ({ value: k, label: TEN_LOAI[k] ?? k })), [rows]);
  const rowsHien = loai ? rows.filter((r) => r.kind === loai) : rows;
  const cols: Column<V4sy>[] = [
    { key: "soBienBan", header: "Số biên bản", width: 160, sortable: true },
    { key: "kind", header: "Loại", sortable: true, sortValue: (r) => TEN_LOAI[r.kind] ?? r.kind, cell: (r) => TEN_LOAI[r.kind] ?? r.kind },
    { key: "title", header: "Tiêu đề", className: "whitespace-normal" },
    { key: "createdAt", header: "Ngày lập", width: 110, sortable: true, cell: (r) => new Date(r.createdAt).toLocaleDateString("vi-VN") },
  ];

  return (
    <section className="view active">
      <PageHeader
        title="Biên bản"
        description="Sổ biên bản kiểm tra, sao lưu, sự cố, thu nộp theo mẫu Phụ lục VII Thông tư 05/2025."
        actions={
          perms.includes("record.create") && (
            <Button variant="primary" icon="bienBan" onClick={() => setMoLap(true)}>
              Lập biên bản
            </Button>
          )
        }
      />
      <Card padding="none" title="Sổ biên bản" actions={cacLoai.length > 1 && <Select size="sm" className="w-72" value={loai} onChange={setLoai} placeholder="Mọi loại" options={cacLoai} aria-label="Lọc theo loại" />}>
        <Table
          columns={cols}
          rows={rowsHien}
          getRowId={(r) => r.id}
          dense
          rowActions={(r) => (
            <Button size="sm" icon="taiVe" loading={dangTai === r.id} disabled={dangTai !== null} onClick={() => void taiPdf(r)}>
              Tải PDF
            </Button>
          )}
          state={{ loading: loading && !data, error, onRetry: reload, empty: { title: loai ? "Không có biên bản loại này" : "Chưa có biên bản nào", icon: "bienBan", action: perms.includes("record.create") ? <Button icon="bienBan" onClick={() => setMoLap(true)}>Lập biên bản</Button> : undefined } }}
        />
      </Card>
      {moLap && <LapBienBanDialog onClose={() => setMoLap(false)} onCreated={reload} />}
    </section>
  );
}
