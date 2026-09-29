// Copyright (c) 2026 DigiText. All rights reserved.
// Mượn / trả hồ sơ giấy (chỉ online): lập phiếu (người mượn, đơn vị, điện thoại, email, nội dung, ngày mượn, hạn trả),
// ghi trả, quá hạn, in phiếu; một hồ sơ chỉ có một phiếu đang mượn.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { clHoSoList, clRemote, clRemoteSave, qs, str, type ClRow } from "@/api/chinhly";
import { CanMayChu, ChuaChonDuAn, fmtNgay, fmtTime, useClSync } from "./common";
import { esc, inHtml } from "./print";
import { Badge, Button, Card, Dialog, ErrorBanner, Field, FormGrid, Input, Menu, notifyError, PageHeader, prompt, Seg, Table, Textarea, toast, type Column } from "@/ui";

const TT: Record<string, [string, "info" | "ok"]> = { dang_muon: ["Đang mượn", "info"], da_tra: ["Đã trả", "ok"] };
type Loc = "dang_muon" | "qua_han" | "da_tra" | "tat_ca";
const homNay = () => new Date().toISOString().slice(0, 10);

function inPhieu(r: ClRow, duAn: string): void {
  const dong = (l: string, v: string) => `<tr><td style="width:180px;font-weight:600">${esc(l)}</td><td>${esc(v)}</td></tr>`;
  const body = `<h2 style="text-align:center">PHIẾU MƯỢN HỒ SƠ</h2><p style="text-align:center">${esc(duAn)}</p><table class="kv">${dong("Hồ sơ", `${str(r, "ma_ho_so") || `Số tạm ${str(r, "so_tam")}`} · ${str(r, "tieu_de")}`)}${dong("Hộp / hồ sơ số", `${str(r, "hop_so")} / ${str(r, "ho_so_so")}`)}${dong("Người mượn", str(r, "nguoi_muon"))}${dong("Đơn vị", str(r, "don_vi"))}${dong("Điện thoại / email", [str(r, "dien_thoai"), str(r, "email")].filter(Boolean).join(" · "))}${dong("Nội dung", str(r, "noi_dung"))}${dong("Ngày mượn", fmtNgay(str(r, "ngay_muon")))}${dong("Hạn trả", fmtNgay(str(r, "ngay_du_kien_tra")))}${dong("Ngày trả", fmtNgay(str(r, "ngay_thuc_te_tra")))}${dong("Ghi chú", str(r, "ghi_chu"))}</table><table style="width:100%;margin-top:32px;text-align:center"><tr><td>Người mượn<br/><br/><br/><br/>${esc(str(r, "nguoi_muon"))}</td><td>Người lập phiếu<br/><br/><br/><br/>${esc(str(r, "nguoi_lap_ten"))}</td></tr></table>`;
  inHtml("Phiếu mượn hồ sơ", body);
}

export default function MuonTraView() {
  const { session } = V460();
  const projectId = session?.projectId ?? "";
  const canManage = (session?.permissions ?? []).includes("loan.manage");
  const sync = useClSync();
  const online = sync?.phase !== "offline";
  const [loc, setLoc] = useState<Loc>("dang_muon");
  const [q, setQ] = useState("");
  const [qA, setQA] = useState("");
  const [form, setForm] = useState<{ row: ClRow | null } | null>(null);
  const [d, setD] = useState<Record<string, unknown>>({});
  const [formErr, setFormErr] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [timHs, setTimHs] = useState("");
  const [hsGoiY, setHsGoiY] = useState<ClRow[]>([]);
  const [hsChon, setHsChon] = useState<ClRow | null>(null);

  useEffect(() => {
    const t = window.setTimeout(() => setQA(q.trim()), 400);
    return () => window.clearTimeout(t);
  }, [q]);

  const loader = useCallback(async () => {
    if (!isTauri() || !online) return { rows: [] as ClRow[], total: 0 };
    return clRemote<{ rows: ClRow[]; total: number }>("GET", `/cl/muon-tra${qs({ projectId, trangThai: loc === "tat_ca" ? "" : loc, q: qA, limit: 500 })}`);
  }, [online, projectId, loc, qA]);
  const { data, error, loading, reload } = V41t(loader, { topics: ["chinhLy"], errorText: () => "Không tải được danh sách phiếu mượn." });
  const rows = useMemo(() => data?.rows ?? [], [data]);

  const g = (k: string) => (d[k] === null || d[k] === undefined ? "" : String(d[k]));
  const set = (k: string, v: unknown) => setD((p) => ({ ...p, [k]: v }));
  const mo = (row: ClRow | null) => {
    setD(row ? { ...row } : { ngay_muon: homNay(), ngay_du_kien_tra: new Date(Date.now() + 7 * 86400000).toISOString().slice(0, 10) });
    setHsChon(null);
    setTimHs("");
    setHsGoiY([]);
    setForm({ row });
    setFormErr(null);
  };
  const timHoSo = async (t: string) => {
    setTimHs(t);
    setHsChon(null);
    if (t.trim().length < 2) {
      setHsGoiY([]);
      return;
    }
    try {
      const r = await clHoSoList({ q: t.trim(), limit: 10 });
      setHsGoiY(r.rows);
    } catch {
      setHsGoiY([]);
    }
  };
  const luu = async () => {
    setBusy(true);
    setFormErr(null);
    try {
      const body: Record<string, unknown> = {};
      for (const k of ["nguoi_muon", "don_vi", "dien_thoai", "email", "noi_dung", "ngay_muon", "ngay_du_kien_tra", "ghi_chu"]) body[k] = d[k] === "" ? null : d[k];
      if (!form?.row) {
        if (!hsChon) {
          setFormErr("Chọn hồ sơ cho mượn.");
          return;
        }
        body.ho_so_id = hsChon.id;
        body.trang_thai = "dang_muon";
      }
      await clRemoteSave("muon_tra", projectId, form?.row?.id ?? null, body, form?.row ? Number(form.row.ver) : undefined);
      setForm(null);
      toast.success(form?.row ? "Đã lưu phiếu mượn." : "Đã lập phiếu mượn.");
      reload();
    } catch (e) {
      setFormErr(e);
    } finally {
      setBusy(false);
    }
  };
  const tra = async (r: ClRow) => {
    const ngay = await prompt({ title: "Ghi trả hồ sơ", body: str(r, "tieu_de"), label: "Ngày trả", type: "date", initial: homNay(), required: true, confirmLabel: "Ghi trả" });
    if (ngay === null) return;
    try {
      await clRemoteSave("muon_tra", projectId, r.id, { trang_thai: "da_tra", ngay_thuc_te_tra: ngay }, Number(r.ver));
      toast.success("Đã ghi trả.");
      reload();
    } catch (e) {
      notifyError(e, "ghi trả");
    }
  };

  const cols: Column<ClRow>[] = [
    {
      key: "tieu_de",
      header: "Hồ sơ",
      className: "max-w-[360px] whitespace-normal",
      cell: (r) => (
        <>
          <span className="block text-xs text-fg-subtle">{str(r, "ma_ho_so") || `Số tạm ${str(r, "so_tam")}`}</span>
          {str(r, "tieu_de")}
        </>
      ),
    },
    { key: "hop", header: "Hộp / hồ sơ số", width: 110, cell: (r) => `${str(r, "hop_so")} / ${str(r, "ho_so_so")}` },
    { key: "nguoi_muon", header: "Người mượn", sortable: true, cell: (r) => str(r, "nguoi_muon") },
    { key: "don_vi", header: "Đơn vị", cell: (r) => str(r, "don_vi"), hideBelow: 1400 },
    { key: "dien_thoai", header: "Điện thoại", width: 110, cell: (r) => str(r, "dien_thoai"), hideBelow: 1400 },
    { key: "noi_dung", header: "Nội dung", className: "max-w-[260px] whitespace-normal text-sm", cell: (r) => str(r, "noi_dung") },
    { key: "ngay_muon", header: "Ngày mượn", sortable: true, width: 100, cell: (r) => fmtNgay(str(r, "ngay_muon")) },
    { key: "ngay_du_kien_tra", header: "Hạn trả", sortable: true, width: 100, cell: (r) => <span className={r.qua_han === true ? "text-danger-700 font-medium" : undefined}>{fmtNgay(str(r, "ngay_du_kien_tra"))}</span> },
    { key: "ngay_thuc_te_tra", header: "Ngày trả", width: 100, cell: (r) => fmtNgay(str(r, "ngay_thuc_te_tra")) },
    {
      key: "trang_thai",
      header: "Trạng thái",
      width: 110,
      cell: (r) => {
        const [l, tone] = TT[str(r, "trang_thai")] ?? [str(r, "trang_thai"), "info"];
        return r.qua_han === true ? <Badge tone="err">Quá hạn</Badge> : <Badge tone={tone}>{l}</Badge>;
      },
    },
    { key: "nguoi_lap_ten", header: "Người lập", className: "text-sm", cell: (r) => <>{str(r, "nguoi_lap_ten")} <span className="text-xs text-fg-subtle">{fmtTime(str(r, "created_at"))}</span></>, hideBelow: 1600 },
  ];

  return (
    <section className="view active">
      <PageHeader
        title="Mượn và trả hồ sơ giấy"
        description="Phiếu mượn, hạn trả, quá hạn. Cần kết nối máy chủ."
        actions={
          canManage && (
            <Button variant="primary" icon="them" disabled={!online} onClick={() => mo(null)}>
              Lập phiếu mượn
            </Button>
          )
        }
      />
      <ChuaChonDuAn />
      <CanMayChu sync={sync} viec="Mượn và trả cần máy chủ." />
      <Card padding="none" title={<Seg<Loc> value={loc} onChange={setLoc} options={[{ value: "dang_muon", label: "Đang mượn" }, { value: "qua_han", label: "Quá hạn" }, { value: "da_tra", label: "Đã trả" }, { value: "tat_ca", label: "Tất cả" }]} />} actions={<Input size="sm" prefix="tim" className="w-72" placeholder="Tìm người mượn, đơn vị, hồ sơ" value={q} onChange={(e) => setQ(e.target.value)} aria-label="Tìm phiếu" />}>
        <Table
          columns={cols}
          rows={rows}
          getRowId={(r) => r.id}
          dense
          rowClassName={(r) => (r.qua_han === true ? "bg-danger-50/50" : undefined)}
          rowActions={(r) => (
            <Menu
              size="sm"
              items={[
                ...(canManage && str(r, "trang_thai") === "dang_muon" ? [{ label: "Ghi trả", icon: "duyet" as const, disabled: !online, onSelect: () => void tra(r) }] : []),
                ...(canManage ? [{ label: "Sửa phiếu", icon: "sua" as const, disabled: !online, onSelect: () => mo(r) }] : []),
                { label: "In phiếu mượn", icon: "inRa" as const, onSelect: () => inPhieu(r, session?.projectName ?? "") },
              ]}
            />
          )}
          state={{ loading: loading && !data, error, onRetry: reload, empty: { title: qA ? "Không có phiếu nào khớp từ khoá" : loc === "dang_muon" ? "Không có hồ sơ nào đang mượn" : loc === "qua_han" ? "Không có phiếu nào quá hạn" : "Chưa có phiếu mượn nào", icon: "bienBan" } }}
        />
      </Card>

      {form && (
        <Dialog
          open
          onOpenChange={(o) => !o && !busy && setForm(null)}
          closable={!busy}
          title={form.row ? "Sửa phiếu mượn" : "Lập phiếu mượn"}
          size="md"
          footer={
            <>
              <Button onClick={() => setForm(null)} disabled={busy}>
                Huỷ
              </Button>
              <Button variant="primary" icon="luu" loading={busy} onClick={() => void luu()}>
                {form.row ? "Lưu" : "Lập phiếu"}
              </Button>
            </>
          }
        >
          <div className="space-y-3">
            {formErr !== null && <ErrorBanner error={formErr} />}
            {!form.row && (
              <Field label="Hồ sơ cho mượn" required hint={hsChon ? undefined : "Gõ tiêu đề, số tạm hoặc mã hồ sơ"}>
                <Input value={timHs} onChange={(e) => void timHoSo(e.target.value)} prefix="tim" autoFocus />
                {hsGoiY.length > 0 && !hsChon && (
                  <div className="mt-1 max-h-48 overflow-auto rounded-md border border-border-subtle" role="listbox" aria-label="Hồ sơ gợi ý">
                    {hsGoiY.map((h) => (
                      <div key={h.id}>
                        <button
                          type="button"
                          role="option"
                          aria-selected={false}
                          className="block w-full px-2 py-1.5 text-left text-sm hover:bg-surface-hover"
                          onClick={() => {
                            setHsChon(h);
                            setTimHs(str(h, "tieu_de"));
                          }}
                        >
                          <span className="text-fg-subtle">{str(h, "ma_ho_so") || `Số tạm ${str(h, "so_tam")}`}</span> · {str(h, "tieu_de")}
                          {h.dang_muon === true && (
                            <Badge tone="warn" className="ml-1">
                              đang mượn
                            </Badge>
                          )}
                        </button>
                      </div>
                    ))}
                  </div>
                )}
                {hsChon && (
                  <p className="mt-1 text-sm">
                    Đã chọn: <b>{str(hsChon, "tieu_de")}</b>
                  </p>
                )}
              </Field>
            )}
            <FormGrid cols={2}>
              <Field label="Người mượn" required>
                <Input value={g("nguoi_muon")} onChange={(e) => set("nguoi_muon", e.target.value)} />
              </Field>
              <Field label="Đơn vị công tác">
                <Input value={g("don_vi")} onChange={(e) => set("don_vi", e.target.value)} />
              </Field>
              <Field label="Điện thoại">
                <Input type="tel" value={g("dien_thoai")} onChange={(e) => set("dien_thoai", e.target.value)} />
              </Field>
              <Field label="Email">
                <Input type="email" value={g("email")} onChange={(e) => set("email", e.target.value)} />
              </Field>
              <Field label="Ngày mượn">
                <Input type="date" value={g("ngay_muon")} onChange={(e) => set("ngay_muon", e.target.value)} />
              </Field>
              <Field label="Hạn trả">
                <Input type="date" value={g("ngay_du_kien_tra")} onChange={(e) => set("ngay_du_kien_tra", e.target.value)} />
              </Field>
            </FormGrid>
            <Field label="Nội dung, mục đích">
              <Textarea rows={2} value={g("noi_dung")} onChange={(e) => set("noi_dung", e.target.value)} />
            </Field>
            <Field label="Ghi chú">
              <Input value={g("ghi_chu")} onChange={(e) => set("ghi_chu", e.target.value)} />
            </Field>
          </div>
        </Dialog>
      )}
    </section>
  );
}
