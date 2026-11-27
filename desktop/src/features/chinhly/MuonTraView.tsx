// Copyright (c) 2026 DigiText. All rights reserved.
// Mượn / trả hồ sơ giấy CÓ DUYỆT (0034, 27/11/2026): một phiếu nhiều hồ sơ (cùng số phiếu), chờ duyệt → duyệt (lẻ hoặc cả phiếu)
// → giao → trả; từ chối / huỷ có lý do; gia hạn tối đa 2 lần giữ hạn gốc; quá hạn máy chủ nhắc. Phiếu PDF và sổ Excel máy chủ dựng.
// Chỉ online. Bản cũ (một hồ sơ, giao luôn) vẫn đọc được: dòng không có số phiếu.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { V3z9 } from "@/domain/permissions";
import { clHoSoList, str, type ClRow } from "@/api/chinhly";
import { mtBuoc, mtGiaHan, mtLap, mtList, mtNhomBuoc, mtPhieuPdf, mtTongQuan, type DongMuon, type LocMuon } from "@/api/muonTra";
import { bcTai } from "@/api/baoCao";
import { CanMayChu, ChuaChonDuAn, fmtNgay, fmtTime, useClSync } from "./common";
import { congNgay, hanhDongMuon, homNay, TT_MUON } from "./muonTraFlow";
import { Badge, Button, Card, Checkbox, Dialog, ErrorBanner, Field, FormGrid, Input, KpiRow, KpiTile, Menu, notifyError, PageHeader, prompt, Seg, Table, Textarea, Tip, toast, type Column, type MenuEntry } from "@/ui";

type Loc = "cho_duyet" | "da_duyet" | "dang_muon" | "qua_han" | "da_tra" | "tat_ca";
const LOC_API: Record<Loc, LocMuon> = { cho_duyet: "cho_duyet", da_duyet: "da_duyet", dang_muon: "dang_muon", qua_han: "qua_han", da_tra: "da_tra", tat_ca: "" };

export default function MuonTraView() {
  const { session } = V460();
  const projectId = useProject().projectId ?? ""; // dự án đang chọn (super admin không gắn dự án trong phiên)
  const perms = session?.permissions;
  const canManage = V3z9(perms, "loan.manage");
  const canApprove = V3z9(perms, "loan.approve");
  const canStats = V3z9(perms, "stats.view") || V3z9(perms, "stats.view_project");
  const sync = useClSync();
  const online = sync?.phase !== "offline";
  const [loc, setLoc] = useState<Loc>("cho_duyet");
  const [q, setQ] = useState("");
  const [qA, setQA] = useState("");
  const [lap, setLap] = useState(false);
  const [giaHan, setGiaHan] = useState<DongMuon | null>(null);
  const [busyXuat, setBusyXuat] = useState(false);

  useEffect(() => {
    const t = window.setTimeout(() => setQA(q.trim()), 400);
    return () => window.clearTimeout(t);
  }, [q]);

  const loader = useCallback(async () => {
    if (!isTauri() || !online || !projectId) return { rows: [] as DongMuon[], total: 0, tq: null };
    const [ds, tq] = await Promise.all([mtList(projectId, LOC_API[loc], qA), mtTongQuan(projectId).catch(() => null)]);
    return { ...ds, tq };
  }, [online, projectId, loc, qA]);
  const { data, error, loading, reload } = V41t(loader, { topics: ["chinhLy"], errorText: () => "Không tải được danh sách phiếu mượn." });
  const rows = useMemo(() => data?.rows ?? [], [data]);
  const tq = data?.tq ?? null;
  // Số dòng còn chờ duyệt theo phiếu (để hiện "Duyệt cả phiếu").
  const choTheoNhom = useMemo(() => {
    const m = new Map<string, number>();
    for (const r of rows) if (r.trang_thai === "cho_duyet" && r.nhom_id) m.set(r.nhom_id, (m.get(r.nhom_id) ?? 0) + 1);
    return m;
  }, [rows]);

  const chay = async (r: DongMuon, key: string) => {
    try {
      switch (key) {
        case "duyet":
          await mtBuoc(r.id, "duyet");
          toast.success("Đã duyệt.");
          break;
        case "tu-choi": {
          const ly = await prompt({ title: "Từ chối phiếu mượn", body: str(r, "tieu_de"), label: "Lý do", type: "textarea", required: true, confirmLabel: "Từ chối", danger: true });
          if (ly === null) return;
          await mtBuoc(r.id, "tu-choi", { lyDo: ly });
          toast.success("Đã từ chối.");
          break;
        }
        case "duyet-phieu":
          if (!r.nhom_id) return;
          await mtNhomBuoc(r.nhom_id, "duyet");
          toast.success(`Đã duyệt cả phiếu ${r.so_phieu ?? ""}.`);
          break;
        case "tu-choi-phieu": {
          if (!r.nhom_id) return;
          const ly = await prompt({ title: `Từ chối cả phiếu ${r.so_phieu ?? ""}`, label: "Lý do", type: "textarea", required: true, confirmLabel: "Từ chối", danger: true });
          if (ly === null) return;
          await mtNhomBuoc(r.nhom_id, "tu-choi", ly);
          toast.success("Đã từ chối cả phiếu.");
          break;
        }
        case "giao": {
          const ngay = await prompt({ title: "Giao hồ sơ cho người mượn", body: str(r, "tieu_de"), label: "Ngày giao", type: "date", initial: homNay(), required: true, confirmLabel: "Giao" });
          if (ngay === null) return;
          await mtBuoc(r.id, "giao", { ngay });
          toast.success("Đã giao hồ sơ, phiếu chuyển sang đang mượn.");
          break;
        }
        case "tra": {
          const ngay = await prompt({ title: "Ghi trả hồ sơ", body: str(r, "tieu_de"), label: "Ngày trả", type: "date", initial: homNay(), required: true, confirmLabel: "Ghi trả" });
          if (ngay === null) return;
          await mtBuoc(r.id, "tra", { ngay });
          toast.success("Đã ghi trả.");
          break;
        }
        case "huy": {
          const ly = await prompt({ title: "Huỷ phiếu mượn", body: str(r, "tieu_de"), label: "Lý do", type: "textarea", required: true, confirmLabel: "Huỷ phiếu", danger: true });
          if (ly === null) return;
          await mtBuoc(r.id, "huy", { lyDo: ly });
          toast.success("Đã huỷ phiếu.");
          break;
        }
        case "gia-han":
          setGiaHan(r);
          return;
        default:
          return;
      }
      reload();
    } catch (e) {
      notifyError(e, "xử lý phiếu mượn");
    }
  };
  const inPdf = async (r: DongMuon) => {
    try {
      const p = await mtPhieuPdf(r.id, r.so_phieu);
      if (p) toast.success(`Đã lưu phiếu: ${p}`);
    } catch (e) {
      notifyError(e, "in phiếu mượn");
    }
  };
  const xuatSo = async () => {
    setBusyXuat(true);
    try {
      const p = await bcTai("muon_tra", { projectId }, "xlsx");
      if (p) toast.success(`Đã xuất sổ: ${p}`);
    } catch (e) {
      notifyError(e, "xuất sổ mượn trả");
    } finally {
      setBusyXuat(false);
    }
  };

  const cols: Column<DongMuon>[] = [
    { key: "so_phieu", header: "Số phiếu", width: 120, cell: (r) => <span className="font-medium">{r.so_phieu ?? <span className="text-fg-subtle">—</span>}</span> },
    {
      key: "tieu_de",
      header: "Hồ sơ",
      className: "max-w-[340px] whitespace-normal",
      cell: (r) => (
        <>
          <span className="block text-xs text-fg-subtle">{str(r, "ma_ho_so") || `Số tạm ${str(r, "so_tam")}`}{str(r, "hop_so") ? ` · hộp ${str(r, "hop_so")}` : ""}</span>
          {str(r, "tieu_de")}
        </>
      ),
    },
    { key: "nguoi_muon", header: "Người mượn", sortable: true, cell: (r) => <>{str(r, "nguoi_muon")}{str(r, "don_vi") && <span className="block text-xs text-fg-subtle">{str(r, "don_vi")}</span>}</> },
    { key: "noi_dung", header: "Nội dung", className: "max-w-[240px] whitespace-normal text-sm", cell: (r) => str(r, "noi_dung"), hideBelow: 1500 },
    { key: "ngay_muon", header: "Ngày mượn", sortable: true, width: 100, cell: (r) => fmtNgay(str(r, "ngay_muon")) },
    {
      key: "ngay_du_kien_tra",
      header: "Hạn trả",
      sortable: true,
      width: 130,
      cell: (r) => (
        <span className={r.qua_han === true ? "font-medium text-danger-700" : undefined}>
          {fmtNgay(str(r, "ngay_du_kien_tra"))}
          {(r.so_lan_gia_han ?? 0) > 0 && (
            <Tip text={`Gia hạn ${r.so_lan_gia_han} lần, hạn gốc ${fmtNgay(r.han_tra_goc ?? "")}`}>
              <Badge tone="info" className="ml-1">+{r.so_lan_gia_han}</Badge>
            </Tip>
          )}
        </span>
      ),
    },
    { key: "ngay_thuc_te_tra", header: "Ngày trả", width: 100, cell: (r) => fmtNgay(str(r, "ngay_thuc_te_tra")), hideBelow: 1300 },
    {
      key: "trang_thai",
      header: "Trạng thái",
      width: 120,
      cell: (r) => {
        const t = TT_MUON[r.trang_thai] ?? { label: r.trang_thai, tone: "gray" as const };
        if (r.qua_han === true) return <Badge tone="err">Quá hạn</Badge>;
        const ly = str(r, "ly_do_tu_choi");
        return ly && (r.trang_thai === "tu_choi" || r.trang_thai === "huy") ? (
          <Tip text={ly}>
            <Badge tone={t.tone}>{t.label}</Badge>
          </Tip>
        ) : (
          <Badge tone={t.tone}>{t.label}</Badge>
        );
      },
    },
    {
      key: "nguoi",
      header: "Lập / duyệt / giao",
      className: "text-sm",
      hideBelow: 1600,
      cell: (r) => (
        <>
          {str(r, "nguoi_lap_ten")} <span className="text-xs text-fg-subtle">{fmtTime(str(r, "created_at"))}</span>
          {str(r, "nguoi_duyet_ten") && <span className="block text-xs text-fg-subtle">Duyệt: {str(r, "nguoi_duyet_ten")}</span>}
          {str(r, "giao_boi_ten") && <span className="block text-xs text-fg-subtle">Giao: {str(r, "giao_boi_ten")}</span>}
        </>
      ),
    },
  ];

  return (
    <section className="view active">
      <PageHeader
        title="Mượn và trả hồ sơ giấy"
        description="Lập phiếu, duyệt, giao, gia hạn, trả. Cần kết nối máy chủ."
        infoTitle="Luồng phiếu"
        info={
          <>
            <p>Phiếu lập xong ở trạng thái chờ duyệt; người có quyền duyệt mượn duyệt từng hồ sơ hoặc cả phiếu, có thể từ chối kèm lý do. Hồ sơ đã duyệt được giao (chuyển sang đang mượn), gia hạn tối đa 2 lần (giữ hạn gốc), ghi trả khi nhận lại.</p>
            <p>Máy chủ nhắc người lập phiếu và quản trị dự án khi còn 3 ngày đến hạn và mỗi tuần khi quá hạn. Số phiếu dạng MT-năm-số thứ tự; phiếu bản cũ không có số.</p>
          </>
        }
        actions={
          <>
            {canStats && (
              <Button icon="tepExcel" disabled={!online || !projectId} loading={busyXuat} onClick={() => void xuatSo()}>
                Sổ mượn trả
              </Button>
            )}
            {canManage && (
              <Button variant="primary" icon="them" disabled={!online} onClick={() => setLap(true)}>
                Lập phiếu mượn
              </Button>
            )}
          </>
        }
      />
      <ChuaChonDuAn />
      <CanMayChu sync={sync} viec="Mượn và trả cần máy chủ." />
      {tq && (
        <KpiRow className="mb-3 grid grid-cols-2 gap-3 md:grid-cols-4">
          <KpiTile icon="dangCho" label="Chờ duyệt" value={tq.choDuyet} sub={tq.phieuChoDuyet ? `${tq.phieuChoDuyet} phiếu` : undefined} tone={tq.choDuyet ? "warn" : undefined} onClick={() => setLoc("cho_duyet")} />
          <KpiTile icon="xuatTep" label="Chờ giao" value={tq.choGiao} tone={tq.choGiao ? "info" : undefined} onClick={() => setLoc("da_duyet")} />
          <KpiTile icon="hoSo" label="Đang mượn" value={tq.dangMuon} sub={tq.sapHan ? `${tq.sapHan} sắp đến hạn` : undefined} onClick={() => setLoc("dang_muon")} />
          <KpiTile icon="canhBao" label="Quá hạn" value={tq.quaHan} tone={tq.quaHan ? "err" : "ok"} onClick={() => setLoc("qua_han")} />
        </KpiRow>
      )}
      <Card
        padding="none"
        title={
          <Seg<Loc>
            value={loc}
            onChange={setLoc}
            options={[
              { value: "cho_duyet", label: "Chờ duyệt" },
              { value: "da_duyet", label: "Chờ giao" },
              { value: "dang_muon", label: "Đang mượn" },
              { value: "qua_han", label: "Quá hạn" },
              { value: "da_tra", label: "Đã trả" },
              { value: "tat_ca", label: "Tất cả" },
            ]}
          />
        }
        actions={<Input size="sm" prefix="tim" className="w-72" placeholder="Tìm người mượn, đơn vị, hồ sơ" value={q} onChange={(e) => setQ(e.target.value)} aria-label="Tìm phiếu" />}
      >
        <Table
          columns={cols}
          rows={rows}
          getRowId={(r) => r.id}
          dense
          rowClassName={(r) => (r.qua_han === true ? "bg-danger-50/50" : undefined)}
          rowActions={(r) => {
            const hd = hanhDongMuon(r, { manage: canManage, approve: canApprove }, (choTheoNhom.get(r.nhom_id ?? "") ?? 0) > 1);
            const items: MenuEntry[] = hd.map((h) => ({ label: h.label, icon: h.icon, danger: h.danger, disabled: !online, onSelect: () => void chay(r, h.key) }));
            if (items.length > 0) items.push("-");
            items.push({ label: "Phiếu mượn (PDF)", icon: "tepPdf", disabled: !online, onSelect: () => void inPdf(r) });
            return <Menu size="sm" items={items} />;
          }}
          state={{
            loading: loading && !data,
            error,
            onRetry: reload,
            empty: {
              title: qA ? "Không có phiếu nào khớp từ khoá" : loc === "cho_duyet" ? "Không có phiếu chờ duyệt" : loc === "da_duyet" ? "Không có hồ sơ chờ giao" : loc === "dang_muon" ? "Không có hồ sơ nào đang mượn" : loc === "qua_han" ? "Không có phiếu nào quá hạn" : "Chưa có phiếu mượn nào",
              icon: "bienBan",
            },
          }}
        />
      </Card>

      {lap && (
        <LapPhieuDialog
          projectId={projectId}
          canApprove={canApprove}
          onClose={() => setLap(false)}
          onDone={(so, n) => {
            setLap(false);
            toast.success(`Đã lập phiếu ${so} (${n} hồ sơ).`);
            setLoc(canApprove ? "cho_duyet" : "cho_duyet");
            reload();
          }}
        />
      )}
      {giaHan && (
        <GiaHanDialog
          row={giaHan}
          onClose={() => setGiaHan(null)}
          onDone={() => {
            setGiaHan(null);
            toast.success("Đã gia hạn.");
            reload();
          }}
        />
      )}
    </section>
  );
}

function LapPhieuDialog({ projectId, canApprove, onClose, onDone }: { projectId: string; canApprove: boolean; onClose: () => void; onDone: (soPhieu: string, n: number) => void }) {
  const [d, setD] = useState<Record<string, string>>({ ngayMuon: homNay(), ngayDuKienTra: congNgay(homNay(), 7) });
  const [giaoNgay, setGiaoNgay] = useState(false);
  const [err, setErr] = useState<unknown>(null);
  const [busy, setBusy] = useState(false);
  const [timHs, setTimHs] = useState("");
  const [goiY, setGoiY] = useState<ClRow[]>([]);
  const [chon, setChon] = useState<ClRow[]>([]);
  const g = (k: string) => d[k] ?? "";
  const set = (k: string, v: string) => setD((p) => ({ ...p, [k]: v }));

  useEffect(() => {
    const t = timHs.trim();
    if (t.length < 2) {
      setGoiY([]);
      return;
    }
    const h = window.setTimeout(() => {
      clHoSoList({ q: t, limit: 12 })
        .then((r) => setGoiY(r.rows.filter((x) => !chon.some((c) => c.id === x.id))))
        .catch(() => setGoiY([]));
    }, 300);
    return () => window.clearTimeout(h);
  }, [timHs, chon]);

  const luu = async () => {
    setBusy(true);
    setErr(null);
    try {
      if (chon.length === 0) {
        setErr("Chọn ít nhất một hồ sơ.");
        return;
      }
      if (!g("nguoiMuon").trim()) {
        setErr("Thiếu tên người mượn.");
        return;
      }
      const r = await mtLap({
        projectId,
        hoSoIds: chon.map((c) => c.id),
        nguoiMuon: g("nguoiMuon"),
        donVi: g("donVi") || undefined,
        dienThoai: g("dienThoai") || undefined,
        email: g("email") || undefined,
        noiDung: g("noiDung") || undefined,
        ngayMuon: g("ngayMuon") || undefined,
        ngayDuKienTra: g("ngayDuKienTra") || undefined,
        ghiChu: g("ghiChu") || undefined,
        giaoNgay: canApprove && giaoNgay,
      });
      onDone(r.soPhieu, r.rows.length);
    } catch (e) {
      setErr(e);
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      closable={!busy}
      title="Lập phiếu mượn"
      description="Một phiếu có thể gồm nhiều hồ sơ; từng hồ sơ được duyệt riêng."
      size="lg"
      footer={
        <>
          <Button onClick={onClose} disabled={busy}>
            Huỷ
          </Button>
          <Button variant="primary" icon="luu" loading={busy} onClick={() => void luu()}>
            {giaoNgay && canApprove ? "Lập và giao luôn" : "Lập phiếu"}
          </Button>
        </>
      }
    >
      <div className="space-y-3">
        {err !== null && <ErrorBanner error={err} />}
        <Field label="Hồ sơ cho mượn" required hint="Gõ tiêu đề, số tạm hoặc mã hồ sơ rồi chọn; chọn được nhiều hồ sơ">
          <Input value={timHs} onChange={(e) => setTimHs(e.target.value)} prefix="tim" autoFocus placeholder="Tìm hồ sơ" />
          {goiY.length > 0 && (
            <div className="mt-1 max-h-44 overflow-auto rounded-md border border-border-subtle" role="listbox" aria-label="Hồ sơ gợi ý">
              {goiY.map((h) => (
                <button
                  key={h.id}
                  type="button"
                  role="option"
                  aria-selected={false}
                  className="block w-full px-2 py-1.5 text-left text-sm hover:bg-surface-hover"
                  onClick={() => {
                    setChon((p) => [...p, h]);
                    setTimHs("");
                  }}
                >
                  <span className="text-fg-subtle">{str(h, "ma_ho_so") || `Số tạm ${str(h, "so_tam")}`}</span> · {str(h, "tieu_de")}
                  {h.dang_muon === true && (
                    <Badge tone="warn" className="ml-1">
                      đang mượn
                    </Badge>
                  )}
                </button>
              ))}
            </div>
          )}
          {chon.length > 0 && (
            <ul className="mt-2 divide-y divide-border-subtle rounded-md border border-border-subtle">
              {chon.map((h) => (
                <li key={h.id} className="flex items-center gap-2 px-2 py-1 text-sm">
                  <span className="min-w-0 flex-1 truncate">
                    <span className="text-fg-subtle">{str(h, "ma_ho_so") || `Số tạm ${str(h, "so_tam")}`}</span> · {str(h, "tieu_de")}
                  </span>
                  <Button size="sm" variant="ghost" icon="dong" aria-label="Bỏ hồ sơ" onClick={() => setChon((p) => p.filter((x) => x.id !== h.id))} />
                </li>
              ))}
            </ul>
          )}
        </Field>
        <FormGrid cols={2}>
          <Field label="Người mượn" required>
            <Input value={g("nguoiMuon")} onChange={(e) => set("nguoiMuon", e.target.value)} />
          </Field>
          <Field label="Đơn vị công tác">
            <Input value={g("donVi")} onChange={(e) => set("donVi", e.target.value)} />
          </Field>
          <Field label="Điện thoại">
            <Input type="tel" value={g("dienThoai")} onChange={(e) => set("dienThoai", e.target.value)} />
          </Field>
          <Field label="Email">
            <Input type="email" value={g("email")} onChange={(e) => set("email", e.target.value)} />
          </Field>
          <Field label="Ngày mượn">
            <Input type="date" value={g("ngayMuon")} onChange={(e) => set("ngayMuon", e.target.value)} />
          </Field>
          <Field label="Hạn trả">
            <Input type="date" value={g("ngayDuKienTra")} min={g("ngayMuon") || undefined} onChange={(e) => set("ngayDuKienTra", e.target.value)} />
          </Field>
        </FormGrid>
        <Field label="Nội dung, mục đích">
          <Textarea rows={2} value={g("noiDung")} onChange={(e) => set("noiDung", e.target.value)} />
        </Field>
        <Field label="Ghi chú">
          <Input value={g("ghiChu")} onChange={(e) => set("ghiChu", e.target.value)} />
        </Field>
        {canApprove && <Checkbox checked={giaoNgay} onChange={setGiaoNgay} text="Giao hồ sơ luôn, không qua bước duyệt (người lập có quyền duyệt)" />}
      </div>
    </Dialog>
  );
}

function GiaHanDialog({ row, onClose, onDone }: { row: DongMuon; onClose: () => void; onDone: () => void }) {
  const hanCu = str(row, "ngay_du_kien_tra");
  const [han, setHan] = useState(hanCu ? congNgay(hanCu, 7) : congNgay(homNay(), 7));
  const [lyDo, setLyDo] = useState("");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<unknown>(null);
  const luu = async () => {
    setBusy(true);
    setErr(null);
    try {
      await mtGiaHan(row.id, han, lyDo || undefined);
      onDone();
    } catch (e) {
      setErr(e);
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      title="Gia hạn mượn"
      description={`${str(row, "tieu_de")} — hạn hiện tại ${fmtNgay(hanCu)}${(row.so_lan_gia_han ?? 0) > 0 ? `, đã gia hạn ${row.so_lan_gia_han} lần` : ""}`}
      size="sm"
      footer={
        <>
          <Button onClick={onClose} disabled={busy}>
            Huỷ
          </Button>
          <Button variant="primary" icon="nhacHan" loading={busy} onClick={() => void luu()}>
            Gia hạn
          </Button>
        </>
      }
    >
      <div className="space-y-3">
        {err !== null && <ErrorBanner error={err} />}
        <Field label="Hạn trả mới" required>
          <Input type="date" value={han} min={hanCu || undefined} onChange={(e) => setHan(e.target.value)} autoFocus />
        </Field>
        <Field label="Lý do">
          <Textarea rows={2} value={lyDo} onChange={(e) => setLyDo(e.target.value)} />
        </Field>
      </div>
    </Dialog>
  );
}
