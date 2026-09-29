// Copyright (c) 2026 DigiText. All rights reserved.
// Cây thư mục quét: danh mục hộp / hồ sơ của một phông trên máy này → tạo thư mục quét, phát cho máy nhân viên
// (qua máy chủ hoặc LAN); máy nhân viên nhận về. Nguồn hồ sơ: mục lục chỉnh lý, Excel, hoặc nhập tay.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41p } from "@/live/useLive";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { V40f, V40h, V40t, V40u, V40w, V40y, listHoSo, type V40v } from "@/api/archive";
import { V49w, V49x, type V49v } from "@/api/sync";
import { V4a0, V4a1, V4a2, V4a4, V4a5, type V49y, type V49z } from "@/api/lan";
import { V432, V433 } from "@/api/lansync";
import { V4ak, type V4a9 } from "@/api/users";
import { treeSeedFromCl } from "@/api/tree";
import { V4at, V4aw, V4b0 } from "@/domain/hoSoTime";
import type { V3yk, V3yr } from "@/domain/types";
import { Icon } from "@/components/Icon";
import { Banner, Button, Card, confirm, Dialog, EmptyState, Field, FormGrid, Help, IconButton, Input, Menu, notifyError, PageHeader, Select, Table, toast, Toolbar, type Column } from "@/ui";
import { chuaChonPhong } from "@/features/dossier/phongScope";
import { useProject } from "@/state/project";
import ImportExcelDialog from "./ImportExcelDialog";
import { V48w } from "./hoSoMapping";
import { V4a6, V4a7 } from "./treePeerAddr";

const n = (v: number) => v.toLocaleString("vi-VN");

type Draft = { mode: "add" | "edit"; draft: V3yr; orig?: number };

export default function TreeView() {
  const { session, setScanTarget, goto, resetToRoot } = V460();
  const project = useProject();
  const perms = session?.permissions;
  const canAssign = V3z9(perms, "tree.assign");
  const canPull = V3z9(perms, "tree.view") && !canAssign;
  const canScan = V3z9(perms, "scan.execute");
  const canCreate = V3z9(perms, "dossier.create");
  const canUpdate = V3z9(perms, "dossier.update");
  const canDelete = V3z9(perms, "dossier.delete");

  const [phongs, setPhongs] = useState<V40v[]>([]);
  const [maDa, setMaDa] = useState("");
  const [rows, setRows] = useState<V3yr[]>([]);
  const [pdfOf, setPdfOf] = useState<Record<string, { pdfPath: string; status: string }>>({});
  const [nguon, setNguon] = useState("all");
  const [q, setQ] = useState("");
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState(false);
  const [loaded, setLoaded] = useState(false);
  const [lan, setLan] = useState<V49y | null>(null);
  const [draft, setDraft] = useState<Draft | null>(null);
  const [importOpen, setImportOpen] = useState(false);
  const [publish, setPublish] = useState<{ projects: V4a9[]; projectId: string } | null>(null);
  const [lanRecv, setLanRecv] = useState<{ peers: V49z[]; peer: string; addr: string; code: string; scanning: boolean } | null>(null);

  const loadPhongs = useCallback(() => {
    if (!isTauri()) return;
    V40w()
      .then((ps) => {
        setPhongs(ps);
        setMaDa((cur) => cur || ps[0]?.maDuAn || "");
      })
      .catch(() => {});
  }, []);
  V41p(loadPhongs, { topics: ["catalog", "tree", "chinhLy"], enabled: isTauri() });

  const reload = useCallback(() => {
    if (!isTauri()) return;
    if (chuaChonPhong(maDa)) {
      setRows([]);
      setPdfOf({});
      setLoaded(true);
      return;
    }
    listHoSo(maDa)
      .then((rs) => {
        setRows(rs.map(V48w));
        const m: Record<string, { pdfPath: string; status: string }> = {};
        for (const r of rs) if (r.pdfPath) m[`${r.hopSo}|${r.hoSoSo}`] = { pdfPath: r.pdfPath, status: r.status };
        setPdfOf(m);
        setLoaded(true);
      })
      .catch(() => setLoaded(true));
  }, [maDa]);
  V41p(reload, { topics: ["hoso", "tree"], enabled: isTauri() });
  useEffect(() => setSel(new Set()), [maDa]);

  const phong = phongs.find((p) => p.maDuAn === maDa);
  const nguons = useMemo(() => Array.from(new Set(rows.map((h) => h.nguon).filter((x): x is string => !!x))), [rows]);
  const shown = useMemo(() => {
    const k = q.trim().toLowerCase();
    return V4b0(rows.filter((h) => (nguon === "all" || h.nguon === nguon) && (!k || h.tieuDe.toLowerCase().includes(k) || h.hoSoSo.toLowerCase().includes(k) || h.hopSo.toLowerCase().includes(k))));
  }, [rows, nguon, q]);

  const phongIn = (p: V40v) => ({ maDuAn: p.maDuAn, tenDuAn: p.tenDuAn, maKhoi: (p as V3yk).maKhoi as string | undefined, phongSo: p.phongSo, mucLucSo: p.mucLucSo });
  const hoSoIn = (h: V3yr) => ({ hopSo: h.hopSo, hoSoSo: h.hoSoSo, tieuDe: h.tieuDe, thoiGian: h.thoiGian, soTo: h.soTo, thbq: h.thbq, ghiChu: h.ghiChu, nhom: h.nhom, nam: h.nam, ngayBatDau: h.ngayBatDau, ngayKetThuc: h.ngayKetThuc });

  const run = async (ctx: string, f: () => Promise<void>) => {
    setBusy(true);
    try {
      await f();
    } catch (e) {
      notifyError(e, ctx);
    } finally {
      setBusy(false);
    }
  };

  // ---- hành động quản trị
  const taoCay = () =>
    run("tạo cây thư mục", async () => {
      if (!phong) return;
      await V40h(phongIn(phong), shown.map(hoSoIn));
      resetToRoot();
      toast.success(`Đã tạo cây thư mục cho ${n(shown.length)} hồ sơ.`);
      goto("drive");
    });
  const moPhat = () =>
    run("lấy danh sách dự án", async () => {
      const ps = await V4ak();
      setPublish({ projects: ps, projectId: ps.length === 1 ? ps[0].id : (project.projectId ?? "") });
    });
  const phatChoDuAn = () =>
    run("phát cây thư mục", async () => {
      if (!phong || !publish?.projectId) return;
      const ver = await V49w(publish.projectId, phongIn(phong), shown.map(hoSoIn));
      setPublish(null);
      toast.success(`Đã phát cây thư mục (phiên bản ${ver}). Máy nhân viên bấm "Nhận cây thư mục" để lấy về.`);
    });
  const phatLan = () =>
    run("phát qua mạng nội bộ", async () => {
      if (lan?.publishing) {
        await V4a2();
        setLan(null);
        toast.info("Đã dừng phát qua mạng nội bộ.");
      } else if (phong) {
        setLan(await V4a1(phong.maDuAn, 1, phongIn(phong), shown.map(hoSoIn)));
      }
    });
  const layTuChinhLy = () =>
    run("lấy hồ sơ từ mục lục chỉnh lý", async () => {
      if (!phong) return;
      const r = await treeSeedFromCl(phong.maDuAn);
      reload();
      toast.success(r.hoSo > 0 ? `Đã lấy ${n(r.hoSo)} hồ sơ từ mục lục chỉnh lý.` : "Cây thư mục đã có đủ hồ sơ của mục lục chỉnh lý.");
    });

  // ---- hành động nhân viên
  const nhanCay = () =>
    run("nhận cây thư mục", async () => {
      const r = await V49x();
      if (r.applied) {
        resetToRoot();
        reload();
        toast.success(`Đã nhận cây thư mục mới (phiên bản ${r.version}), tạo ${n(r.created)} hồ sơ.`);
        goto("drive");
      } else toast.info("Cây thư mục đã là bản mới nhất.");
    });
  const doLan = async () => {
    setLanRecv({ peers: [], peer: "", addr: "", code: "", scanning: true });
    try {
      const peers = await V4a4();
      setLanRecv((s) => (s ? { ...s, peers, scanning: false } : s));
    } catch (e) {
      setLanRecv((s) => (s ? { ...s, scanning: false } : s));
      notifyError(e, "dò máy phát");
    }
  };
  const nhanQuaLan = () =>
    run("nhận qua mạng nội bộ", async () => {
      if (!lanRecv || lanRecv.code.trim().length < 6) return;
      let target: { addr: string; port: number } | null = null;
      if (lanRecv.addr.trim()) {
        const err = V4a7(lanRecv.addr);
        if (err) {
          toast.error(err);
          return;
        }
        target = V4a6(lanRecv.addr);
      } else {
        const p = lanRecv.peers.find((x) => x.instance === lanRecv.peer);
        if (p) target = { addr: p.addr, port: p.port };
      }
      if (!target) {
        toast.error("Chưa chọn máy quản trị hoặc nhập địa chỉ.");
        return;
      }
      const r = await V4a5(target.addr, target.port, lanRecv.code.trim());
      setLanRecv(null);
      resetToRoot();
      reload();
      toast.success(`Đã nhận cây thư mục qua mạng nội bộ, tạo ${n(r.created)} hồ sơ.`);
      goto("drive");
      await nhanMaSite(r);
    });
  const nhanMaSite = async (r: V49v) => {
    const offered = r.offeredSiteCode?.trim();
    if (!offered || !isTauri()) return;
    const current = (await V432())?.trim() ?? "";
    if (current === offered) return;
    const ok = await confirm({
      title: "Nhận mã site từ máy quản trị?",
      body:
        (current ? "Máy này đang dùng một mã site khác. Nhận mã mới sẽ chuyển sang site của máy phát.\n\n" : "") +
        `Vân tay máy phát:\n${r.peerFingerprint ?? "(không rõ)"}\n\nĐối chiếu chuỗi này với chuỗi quản trị đọc cho bạn rồi mới đồng ý.`,
      confirmLabel: "Nhận mã site",
    });
    if (!ok) {
      toast.warning("Đã nhận cây thư mục nhưng bỏ qua mã site vì chưa xác nhận vân tay.");
      return;
    }
    try {
      await V433(offered, true);
      toast.success("Đã nhận mã site. Máy này bắt đầu chia sẻ cho quản trị gom.");
    } catch (e) {
      notifyError(e, "lưu mã site");
    }
  };

  // ---- hồ sơ: thêm / sửa / xoá / nhập
  const luu = () =>
    run("lưu hồ sơ", async () => {
      if (!draft) return;
      const h = V4aw(draft.draft);
      if (!h.tieuDe.trim()) {
        toast.error("Chưa nhập tiêu đề hồ sơ.");
        return;
      }
      if (!h.hoSoSo.trim()) {
        toast.error("Chưa nhập hồ sơ số.");
        return;
      }
      if (draft.mode === "edit" && draft.orig != null) await V40u(draft.orig, hoSoIn(h));
      else {
        if (!phong) return;
        await V40f(phongIn(phong), [hoSoIn(h)]);
      }
      setDraft(null);
      reload();
      toast.success(draft.mode === "edit" ? "Đã sửa hồ sơ." : "Đã thêm hồ sơ.");
    });
  const xoa = async (ids: number[]) => {
    const ok = await confirm({ title: ids.length === 1 ? "Xoá hồ sơ này?" : `Xoá ${n(ids.length)} hồ sơ đã chọn?`, body: "Chỉ xoá khỏi cây thư mục trên máy này. Không thể hoàn tác.", danger: true });
    if (!ok) return;
    await run("xoá hồ sơ", async () => {
      await Promise.all(ids.map((id) => V40t(id)));
      setSel(new Set());
      reload();
      toast.success(`Đã xoá ${n(ids.length)} hồ sơ.`);
    });
  };
  const nhapExcel = (records: V3yr[]) =>
    run("nhập Excel", async () => {
      if (!phong) return;
      const r = await V40f(phongIn(phong), records.map(hoSoIn));
      setImportOpen(false);
      reload();
      toast.success(`Đã nhập ${n(r.hoSo)} hồ sơ.`);
    });
  const quet = (h: V3yr) => {
    if (!phong) return;
    setScanTarget({ phong: phongIn(phong), hoSo: hoSoIn(h), label: `${phong.tenDuAn} · Hộp ${h.hopSo} · HS ${h.hoSoSo}` });
    goto("scan");
  };

  const columns: Column<V3yr>[] = [
    { key: "hopSo", header: "Hộp số", align: "right", width: 80, sortable: true },
    { key: "hoSoSo", header: "Hồ sơ số", align: "right", width: 90, sortable: true },
    { key: "tieuDe", header: "Tiêu đề hồ sơ", sortable: true, className: "whitespace-normal min-w-64" },
    { key: "batDau", header: "Bắt đầu", cell: (h) => V4at(h).batDau, align: "right", width: 100, hideBelow: 1280 },
    { key: "ketThuc", header: "Kết thúc", cell: (h) => V4at(h).ketThuc, align: "right", width: 100, hideBelow: 1280 },
    { key: "soTo", header: "Số tờ", align: "right", width: 70 },
    { key: "thbq", header: "THBQ", width: 110 },
    { key: "nhom", header: "Nhóm / năm", cell: (h) => <span className="text-fg-subtle">{[h.nhom, h.nam].filter(Boolean).join(" · ")}</span>, hideBelow: 1440 },
  ];

  const nextId = () => Math.max(0, ...rows.map((h) => h.id)) + 1;
  const set = (p: Partial<V3yr>) => draft && setDraft({ ...draft, draft: { ...draft.draft, ...p } });

  return (
    <section className="view active">
      <PageHeader
        title="Cây thư mục"
        description="Danh mục hộp, hồ sơ của phông trên máy này để tạo thư mục quét và phát cho máy nhân viên."
        actions={
          <>
            {canAssign && (
              <>
                <Button icon="thuMuc" disabled={!phong || shown.length === 0 || busy} onClick={() => void taoCay()}>
                  Tạo cây trên máy này
                </Button>
                <Button icon="phatChoUser" variant="primary" disabled={!phong || shown.length === 0 || busy} onClick={() => void moPhat()}>
                  Phát cho dự án
                </Button>
                <Button icon={lan?.publishing ? "dungHan" : "phatLan"} disabled={(!phong && !lan?.publishing) || busy} onClick={() => void phatLan()}>
                  {lan?.publishing ? "Dừng phát" : "Phát qua mạng nội bộ"}
                </Button>
              </>
            )}
            {canCreate && (
              <Menu
                items={[
                  { label: "Lấy hồ sơ từ mục lục chỉnh lý", icon: "chinhly", onSelect: () => void layTuChinhLy(), disabled: !phong || !project.projectId, hint: !project.projectId ? "Chọn dự án đang làm ở thanh trên" : undefined },
                  { label: "Nhập từ Excel", icon: "nhapTep", onSelect: () => setImportOpen(true), disabled: !phong },
                  { label: "Thêm hồ sơ", icon: "them", onSelect: () => setDraft({ mode: "add", draft: { id: nextId(), hopSo: "", hoSoSo: "", tieuDe: "", thoiGian: "", soTo: "", thbq: "" } }), disabled: !phong },
                ]}
              />
            )}
          </>
        }
      />

      {canPull && (
        <Card className="mb-4" title="Nhận cây thư mục từ quản trị" description="Quản trị phát danh mục hộp, hồ sơ; máy này nhận về để tạo thư mục quét.">
          <Toolbar>
            <Button icon="taiVe" variant="primary" loading={busy} onClick={() => void nhanCay()}>
              Nhận cây thư mục
            </Button>
            <Button icon="tim" disabled={busy} onClick={() => void doLan()}>
              Nhận qua mạng nội bộ
            </Button>
          </Toolbar>
        </Card>
      )}

      {lan?.publishing && (
        <Banner tone="info" className="mb-4" onClose={() => void phatLan()}>
          Đang phát qua mạng nội bộ. Mã ghép nối: <b className="text-lg tracking-[0.12em]">{lan.code}</b>
          {lan.port ? <span className="text-fg-subtle"> · cổng {lan.port}</span> : null}. Đọc mã này cho nhân viên nhận.
        </Banner>
      )}

      <Card padding="none">
        <div className="border-b border-border-subtle p-3">
          <Toolbar>
            <Field label="Phông" className="w-72">
              <Select size="sm" value={maDa} onChange={setMaDa} options={phongs.map((p) => ({ value: p.maDuAn, label: p.tenDuAn }))} placeholder={phongs.length ? undefined : "Chưa có phông"} />
            </Field>
            {nguons.length > 0 && (
              <Field label="Nguồn" className="w-44">
                <Select size="sm" value={nguon} onChange={setNguon} options={[{ value: "all", label: "Tất cả" }, ...nguons.map((s) => ({ value: s, label: s }))]} />
              </Field>
            )}
            <Field label="Tìm" className="w-64">
              <Input size="sm" prefix="tim" value={q} onChange={(e) => setQ(e.target.value)} placeholder="Tiêu đề, hộp, hồ sơ số" />
            </Field>
            <Toolbar.Spacer />
            {sel.size > 0 && canDelete && (
              <Button variant="dangerOutline" icon="xoa" size="sm" onClick={() => void xoa([...sel].map(Number))}>
                Xoá {n(sel.size)}
              </Button>
            )}
            <span className="text-sm text-fg-subtle">{n(shown.length)} hồ sơ</span>
          </Toolbar>
        </div>
        <Table
          columns={columns}
          rows={shown}
          getRowId={(h) => String(h.id)}
          selection={canDelete ? { selected: sel, onChange: setSel } : undefined}
          state={{
            loading: !loaded,
            empty: chuaChonPhong(maDa)
              ? { title: "Chưa có phông", description: "Phông xuất hiện ở đây sau khi tạo trong Chỉnh lý › Danh mục hoặc khi nhận cây thư mục." }
              : { title: q ? "Không có hồ sơ khớp tìm kiếm" : "Chưa có hồ sơ trong phông này", description: canCreate ? "Lấy từ mục lục chỉnh lý, nhập Excel hoặc thêm tay." : undefined },
          }}
          maxHeight="calc(100dvh - 340px)"
          rowActions={(h) => {
            const pdf = pdfOf[`${h.hopSo}|${h.hoSoSo}`];
            return (
              <>
                {canScan && <IconButton label="Quét vào hồ sơ này" icon="quet" size="sm" disabled={!phong} onClick={() => quet(h)} />}
                {pdf && <IconButton label="Mở PDF" icon="tepPdf" size="sm" onClick={() => void V40y(pdf.pdfPath)} />}
                {canUpdate && <IconButton label="Sửa" icon="sua" size="sm" onClick={() => setDraft({ mode: "edit", draft: { ...h, ...V4at(h) }, orig: h.id })} />}
                {canDelete && <IconButton label="Xoá" icon="xoa" size="sm" onClick={() => void xoa([h.id])} />}
              </>
            );
          }}
        />
      </Card>

      <Help className="mt-4">
        <p>Cây thư mục trên máy dùng để quét đúng vào thư mục Cơ quan › Hộp › Hồ sơ và để gom tệp theo hồ sơ.</p>
        <p>Quản trị lấy hồ sơ từ mục lục chỉnh lý (hoặc Excel), tạo cây, rồi phát cho dự án qua máy chủ; nơi không có máy chủ thì phát qua mạng nội bộ với mã ghép nối 6 số.</p>
      </Help>

      <Dialog open={!!draft} onOpenChange={(o) => !o && setDraft(null)} title={draft?.mode === "add" ? "Thêm hồ sơ" : "Sửa hồ sơ"} size="md" footer={<><Button onClick={() => setDraft(null)}>Huỷ</Button><Button variant="primary" loading={busy} onClick={() => void luu()}>Lưu</Button></>}>
        {draft && (
          <FormGrid cols={2}>
            <Field label="Hộp số">
              <Input value={draft.draft.hopSo} onChange={(e) => set({ hopSo: e.target.value })} autoFocus />
            </Field>
            <Field label="Hồ sơ số" required>
              <Input value={draft.draft.hoSoSo} onChange={(e) => set({ hoSoSo: e.target.value })} />
            </Field>
            <Field label="Tiêu đề hồ sơ" required className="col-span-2">
              <Input value={draft.draft.tieuDe} onChange={(e) => set({ tieuDe: e.target.value })} />
            </Field>
            <Field label="Bắt đầu" hint="Ngày hoặc năm, ví dụ 01/01/2010 hoặc 2010">
              <Input value={draft.draft.ngayBatDau ?? ""} onChange={(e) => set({ ngayBatDau: e.target.value })} />
            </Field>
            <Field label="Kết thúc">
              <Input value={draft.draft.ngayKetThuc ?? ""} onChange={(e) => set({ ngayKetThuc: e.target.value })} />
            </Field>
            <Field label="Số tờ">
              <Input value={draft.draft.soTo} onChange={(e) => set({ soTo: e.target.value })} />
            </Field>
            <Field label="Thời hạn bảo quản">
              <Input value={draft.draft.thbq} onChange={(e) => set({ thbq: e.target.value })} />
            </Field>
            <Field label="Ghi chú" className="col-span-2">
              <Input value={draft.draft.ghiChu ?? ""} onChange={(e) => set({ ghiChu: e.target.value })} />
            </Field>
          </FormGrid>
        )}
      </Dialog>

      <Dialog open={!!publish} onOpenChange={(o) => !o && setPublish(null)} title="Phát cây thư mục cho dự án" size="sm" footer={<><Button onClick={() => setPublish(null)}>Huỷ</Button><Button variant="primary" loading={busy} disabled={!publish?.projectId} onClick={() => void phatChoDuAn()}>Phát</Button></>}>
        {publish && (
          <>
            <p className="mb-3 text-sm text-fg-muted">
              Đưa phông <b>{phong?.tenDuAn}</b> với {n(shown.length)} hồ sơ lên máy chủ. Máy nhân viên thuộc dự án nhận về khi bấm "Nhận cây thư mục".
            </p>
            <Field label="Dự án đích" required>
              <Select value={publish.projectId} onChange={(v) => setPublish({ ...publish, projectId: v })} options={publish.projects.map((p) => ({ value: p.id, label: `${p.code} · ${p.name}` }))} placeholder="— Chọn dự án —" />
            </Field>
            {publish.projects.length === 0 && <p className="mt-2 text-sm text-fg-subtle">Bạn chưa phụ trách dự án nào. Tạo dự án ở Quản trị › Người dùng và dự án.</p>}
          </>
        )}
      </Dialog>

      <Dialog open={!!lanRecv} onOpenChange={(o) => !o && setLanRecv(null)} title="Nhận cây thư mục qua mạng nội bộ" size="md" footer={<><Button onClick={() => setLanRecv(null)}>Huỷ</Button><Button variant="primary" loading={busy} disabled={!lanRecv || lanRecv.code.trim().length < 6} onClick={() => void nhanQuaLan()}>Nhận</Button></>}>
        {lanRecv && (
          <div className="space-y-3">
            {lanRecv.scanning ? (
              <p className="text-sm text-fg-subtle">Đang dò máy quản trị trong mạng…</p>
            ) : lanRecv.peers.length === 0 ? (
              <Banner tone="warning">Không thấy máy nào đang phát. Kiểm tra máy quản trị đã bấm "Phát qua mạng nội bộ" và hai máy cùng mạng, hoặc nhập thẳng địa chỉ bên dưới.</Banner>
            ) : (
              <Field label="Máy quản trị" hint="Chọn máy có vân tay đúng như quản trị đọc cho bạn.">
                <Select value={lanRecv.peer} onChange={(v) => setLanRecv({ ...lanRecv, peer: v })} disabled={lanRecv.addr.trim() !== ""} placeholder="— Chọn máy —" options={lanRecv.peers.map((p) => ({ value: p.instance, label: `${p.project || p.addr} · ${p.addr}:${p.port}${p.fingerprint ? ` · ${p.fingerprint}` : ""}` }))} />
              </Field>
            )}
            <Field label="Hoặc địa chỉ máy quản trị" hint={`Dạng IP hoặc IP:cổng. Không ghi cổng = ${V4a0}.`}>
              <Input value={lanRecv.addr} onChange={(e) => setLanRecv({ ...lanRecv, addr: e.target.value })} placeholder="192.168.1.10" />
            </Field>
            <Field label="Mã ghép nối (6 số)" required>
              <Input inputMode="numeric" maxLength={6} value={lanRecv.code} onChange={(e) => setLanRecv({ ...lanRecv, code: e.target.value.replace(/\D/g, "") })} placeholder="Quản trị đọc cho bạn" />
            </Field>
            <Button size="sm" icon="lamMoi" disabled={lanRecv.scanning} onClick={() => void doLan()}>
              Dò lại
            </Button>
          </div>
        )}
      </Dialog>

      {importOpen && <ImportExcelDialog startId={nextId()} onClose={() => setImportOpen(false)} onImport={(rs) => void nhapExcel(rs)} />}
      {!isTauri() && <EmptyState title="Chỉ dùng trong ứng dụng" compact />}
      <span className="sr-only">
        <Icon name="cayThuMuc" />
      </span>
    </section>
  );
}
