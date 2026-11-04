// Copyright (c) 2026 DigiText. All rights reserved.
// Biên bản theo Thông tư 05/2025: sổ biên bản (lọc theo loại, tải PDF hoặc Word) và hộp thoại lập biên bản hai bước (chọn mẫu bằng
// thẻ → điền nội dung, bảng dòng nhập tại chỗ). 3/11/2026: `SoBienBan` nhúng được vào màn khác (Giao nhận, Nhật ký,
// Nghiệm thu) với `kinds`, `projectId`, `prefill`; mẫu generic (Điều 9) gửi khối PDF dựng sẵn + khoá phụ `ref`.

import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { V41t } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { V4t0, V4t1, V4sz, type V4sy } from "@/api/records";
import { V460 } from "@/state";
import { taoPayloadGeneric, V4ti, V4tf, V4tg, type V4ta } from "./recordForms";
import { Icon } from "@/components/Icon";
import { Banner, Button, Card, Dialog, Field, FormGrid, IconButton, Input, notifyError, PageHeader, Select, Stepper, Table, Tip, toast, type Column } from "@/ui";

export const TEN_LOAI: Record<string, string> = Object.fromEntries(V4tf.map((t) => [t.kind, t.label]));
TEN_LOAI["thu_nop"] = "Biên bản thu nộp";
TEN_LOAI["giao_nhan"] = "Biên bản giao nhận vào lưu trữ lịch sử";
TEN_LOAI["tra_lai"] = "Biên bản hồ sơ, tài liệu trả lại";
TEN_LOAI["kiem_tra_san_pham"] = "Biên bản kiểm tra sản phẩm số hoá";
TEN_LOAI["nghiem_thu"] = "Biên bản nghiệm thu";
TEN_LOAI["ban_giao_lan"] = "Biên bản bàn giao sản phẩm";

export type NoiDungDien = { fields?: Record<string, string>; lists?: Record<string, Record<string, string>[]> };
/** Nguồn điền sẵn: màn gọi cung cấp các lựa chọn (phông, ngày…) và hàm trả nội dung cho mẫu đang chọn. */
export interface NguonDien {
  label: string;
  options: { value: string; label: string }[];
  dien: (kind: V4ta, value: string) => Promise<NoiDungDien>;
}

export function LapBienBanDialog({ onClose, onCreated, kinds, projectId, prefill, nguonDien }: { onClose: () => void; onCreated: () => void; kinds?: V4ta[]; projectId?: string | null; prefill?: NoiDungDien; nguonDien?: NguonDien }) {
  const { session } = V460();
  const mau = useMemo(() => (kinds?.length ? V4tf.filter((t) => kinds.includes(t.kind)) : V4tf), [kinds]);
  const [buoc, setBuoc] = useState(mau.length === 1 ? 1 : 0);
  const [kind, setKind] = useState<V4ta>(mau[0]?.kind ?? "kiem_tra");
  const [fields, setFields] = useState<Record<string, string>>({});
  const [lists, setLists] = useState<Record<string, Record<string, string>[]>>({});
  const [errs, setErrs] = useState<string[]>([]);
  const [fieldErrs, setFieldErrs] = useState<Record<string, string>>({});
  const [busy, setBusy] = useState(false);
  const [nguon, setNguon] = useState(nguonDien?.options[0]?.value ?? "");
  const [dangDien, setDangDien] = useState(false);
  const template = V4tg(kind);

  const apDung = useCallback(
    (k: V4ta, nd?: NoiDungDien) => {
      const t = V4tg(k);
      setFields({ ...(nd?.fields ?? {}) });
      const l: Record<string, Record<string, string>[]> = {};
      for (const x of t.lists) l[x.key] = nd?.lists?.[x.key] ?? (x.minRows > 0 ? [{}] : []);
      setLists(l);
      setErrs([]);
      setFieldErrs({});
    },
    [],
  );
  const chonMau = (k: V4ta) => {
    setKind(k);
    apDung(k, prefill);
  };
  // Mẫu duy nhất → vào thẳng bước điền với nội dung điền sẵn.
  useEffect(() => {
    if (mau.length === 1) apDung(mau[0].kind, prefill);
    // eslint-disable-next-line react-hooks/exhaustive-deps -- chỉ lúc mở
  }, []);
  const dienTuNguon = async () => {
    if (!nguonDien || !nguon) return;
    setDangDien(true);
    try {
      const nd = await nguonDien.dien(kind, nguon);
      setFields((f) => ({ ...f, ...(nd.fields ?? {}) }));
      setLists((l) => ({ ...l, ...(nd.lists ?? {}) }));
      toast.success("Đã điền từ dữ liệu, kiểm tra lại rồi lập.");
    } catch (e) {
      notifyError(e, "điền từ dữ liệu");
    } finally {
      setDangDien(false);
    }
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
      const pid = projectId ?? session?.projectId ?? null;
      const row = template.generic
        ? await (() => {
            const g = taoPayloadGeneric(template, fields, lists);
            return V4t0({ projectId: pid, kind, payload: g.payload, ref: g.ref });
          })()
        : await V4t0({ projectId: pid, kind, payload });
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
      title={mau.length === 1 ? mau[0].label : "Lập biên bản"}
      footer={
        <>
          <Button onClick={onClose} disabled={busy}>
            Huỷ
          </Button>
          {buoc === 1 && mau.length > 1 && (
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
      {mau.length > 1 && <Stepper className="mb-4" current={buoc} steps={[{ key: "mau", label: "Chọn mẫu" }, { key: "dien", label: "Điền nội dung" }]} onSelect={(i) => i === 0 && setBuoc(0)} />}
      {buoc === 0 ? (
        <div className="grid gap-3 sm:grid-cols-2" role="radiogroup" aria-label="Mẫu biên bản">
          {mau.map((t) => (
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
          {nguonDien && (
            <div className="flex flex-wrap items-end gap-2 rounded-md border border-border-subtle bg-surface-sunken p-3">
              <Field label={nguonDien.label}>
                <Select size="sm" value={nguon} onChange={setNguon} options={nguonDien.options} placeholder="— Chọn —" className="w-80" />
              </Field>
              <Button size="sm" icon="chuyenDoi" loading={dangDien} disabled={!nguon} onClick={() => void dienTuNguon()}>
                Điền từ dữ liệu
              </Button>
              <span className="text-xs text-fg-subtle">Số liệu điền vào các ô và bảng bên dưới; sửa tay được trước khi lập.</span>
            </div>
          )}
          {errs.length > 0 && <Banner tone="danger" title={`${errs.length} chỗ chưa hợp lệ`} detail={errs.join("\n")}>Sửa các ô đánh dấu đỏ rồi lập lại.</Banner>}
          <FormGrid cols={3}>
            {template.fields.map((f) => (
              <Field key={f.key} label={f.legal ? <Tip text={`Theo ${f.legal}`}><span>{f.label}</span></Tip> : f.label} required={!f.optional} hint={f.hint} error={fieldErrs[f.key]} className={f.multiline ? "sm:col-span-3" : undefined}>
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
                <span className="ml-auto text-xs text-fg-subtle">{(lists[l.key] ?? []).length} dòng</span>
              </div>
              <div className="max-h-72 overflow-auto rounded-md border border-border-subtle">
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
              </div>
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

/** Sổ biên bản dùng chung: toàn bộ (màn Sổ biên bản) hay một vài loại nhúng trong màn công đoạn. */
export function SoBienBan({ kinds, projectId, title = "Sổ biên bản", description, prefill, nguonDien, nutLap = "Lập biên bản", extraActions, embedded }: { kinds?: V4ta[]; projectId?: string | null; title?: string; description?: string; prefill?: NoiDungDien; nguonDien?: NguonDien; nutLap?: string; extraActions?: ReactNode; embedded?: boolean }) {
  const { session } = V460();
  const perms = session?.permissions ?? [];
  const pid = projectId ?? session?.projectId ?? null;
  // Khoá theo nội dung: màn cha hay truyền mảng `kinds` viết tại chỗ → danh tính đổi mỗi lần vẽ, hàm tải không được đổi theo.
  const kindsKey = kinds?.join(",") ?? "";
  const loader = useCallback(
    () => (isTauri() || import.meta.env.DEV ? V4sz({ kinds: kindsKey ? kindsKey.split(",") : undefined, projectId: pid }) : Promise.resolve([] as V4sy[])),
    [kindsKey, pid],
  );
  const { data, error, loading, reload } = V41t(loader, { topics: ["records"], errorText: () => "Không tải được sổ biên bản." });
  const rows = useMemo(() => {
    const all = data ?? [];
    // Máy chủ cũ không lọc → lọc lại ở máy trạm.
    return kinds?.length ? all.filter((r) => kinds.includes(r.kind as V4ta)) : all;
  }, [data, kinds]);
  const [loai, setLoai] = useState("");
  const [moLap, setMoLap] = useState(false);
  const [dangTai, setDangTai] = useState<string | null>(null);

  const taiPdf = async (row: V4sy, format: "pdf" | "docx" = "pdf") => {
    setDangTai(row.id);
    try {
      const path = await V4t1(row.id, `${row.soBienBan.split("/").join("-")}.${format}`, undefined, format);
      toast.success(format === "docx" ? "Đã lưu bản Word của biên bản." : "Đã lưu biên bản.", { description: path });
    } catch (e) {
      notifyError(e, "tải biên bản");
    } finally {
      setDangTai(null);
    }
  };

  const cacLoai = useMemo(() => [...new Set(rows.map((r) => r.kind))].map((k) => ({ value: k, label: TEN_LOAI[k] ?? k })), [rows]);
  const rowsHien = loai ? rows.filter((r) => r.kind === loai) : rows;
  const cols: Column<V4sy>[] = [
    { key: "soBienBan", header: "Số biên bản", width: 150, sortable: true },
    ...(cacLoai.length > 1 || !kinds?.length ? [{ key: "kind", header: "Loại", sortable: true, sortValue: (r: V4sy) => TEN_LOAI[r.kind] ?? r.kind, cell: (r: V4sy) => TEN_LOAI[r.kind] ?? r.kind } as Column<V4sy>] : []),
    { key: "title", header: "Tiêu đề", className: "whitespace-normal" },
    { key: "ref", header: "Phông / ngày", width: 160, cell: (r) => r.ref ?? "" },
    { key: "createdAt", header: "Ngày lập", width: 110, sortable: true, cell: (r) => new Date(r.createdAt).toLocaleDateString("vi-VN") },
  ];
  const coTao = perms.includes("record.create");

  return (
    <>
      <Card
        padding="none"
        title={title}
        description={description}
        actions={
          <span className="flex items-center gap-2">
            {extraActions}
            {cacLoai.length > 1 && <Select size="sm" className="w-64" value={loai} onChange={setLoai} placeholder="Mọi loại" options={cacLoai} aria-label="Lọc loại biên bản" />}
            {coTao && (
              <Button variant={embedded ? "secondary" : "primary"} size="sm" icon="bienBan" onClick={() => setMoLap(true)}>
                {nutLap}
              </Button>
            )}
          </span>
        }
      >
        <Table
          columns={cols}
          rows={rowsHien}
          getRowId={(r) => r.id}
          dense
          maxHeight={embedded ? 360 : undefined}
          rowActions={(r) => (
            <span className="flex items-center gap-1">
              <Button size="sm" icon="tepPdf" loading={dangTai === r.id} disabled={dangTai !== null} onClick={() => void taiPdf(r)}>
                PDF
              </Button>
              <Button size="sm" variant="ghost" icon="tepWord" disabled={dangTai !== null} onClick={() => void taiPdf(r, "docx")} title="Bản Word để chỉnh sửa, in ký tay">
                Word
              </Button>
            </span>
          )}
          state={{ loading: loading && !data, error, onRetry: reload, empty: { title: loai ? "Không có biên bản loại này" : "Chưa có biên bản nào", icon: "bienBan", action: coTao ? <Button size="sm" icon="bienBan" onClick={() => setMoLap(true)}>{nutLap}</Button> : undefined } }}
        />
      </Card>
      {moLap && <LapBienBanDialog kinds={kinds} projectId={pid} prefill={prefill} nguonDien={nguonDien} onClose={() => setMoLap(false)} onCreated={reload} />}
    </>
  );
}

export default function V4tk() {
  return (
    <section className="view active">
      <PageHeader title="Sổ biên bản" description="Bước 8 Điều 9 TT 05/2025: toàn bộ biên bản của dự án — giao nhận, nhật ký, kiểm tra, nghiệm thu, thu nộp, sao lưu, sự cố — theo mẫu Phụ lục VII và Điều 9." />
      <SoBienBan />
    </section>
  );
}
