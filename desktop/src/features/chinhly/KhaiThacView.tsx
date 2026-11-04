// Copyright (c) 2026 DigiText. All rights reserved.
// Tra cứu và kết xuất dữ liệu chỉnh lý (bản sao, offline được): lọc theo phông / phòng ban / mảng / THBQ / kho / giá,
// bảng phân trang 100 dòng, xem văn bản; kết xuất: metadata TT05 / TT02, mục lục hồ sơ 6 kiểu (Excel + in),
// mục lục văn bản, nhãn hộp, sản phẩm có bìa, phụ lục hướng dẫn tra tìm, lịch sử.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { clHoSoList, clRows, clSyncNow, clTaoVanBanTuOcr, str, THBQ_STATUS_LABEL, type ClHoSoFilter, type ClListFilter, type ClRow, type ClTuOcrResp } from "@/api/chinhly";
import { useProject } from "@/state/project";
import { Chip, ChuaChonDuAn, n, Select, tenPhong, THBQ_BADGE, useClCatalog } from "./common";
import { inHtml, inMucLucHoSo, inMucLucVanBan, inNhanHop, inPhuLuc, inSanPham } from "./print";
import { KIEU_MLHS, xuatLichSu, xuatMucLucHoanThien, xuatMucLucVanBan, xuatPhuLuc, xuatTT02, xuatTT05, type KieuMLHS } from "./excelXuat";
import VanBanPanel from "./VanBanPanel";
import { Badge, Banner, Button, Card, Field, FormGrid, Input, Menu, notifyError, PageHeader, Select as UiSelect, Table, toast, type Column, type MenuEntry } from "@/ui";

const PAGE = 100;
const AN_MAC_DINH = ["bo_so", "tap_so", "trung_thua"];

export default function KhaiThacView() {
  const { session, khoGiaTab } = V460();
  // 3/11/2026: cùng một màn phục vụ hai mục menu — "Mục lục và kết xuất" (bước 3, dữ liệu chủ) và "Kết nối dữ liệu chủ"
  // (bước 4, sau OCR): chỉ khác tiêu đề và danh sách Kết xuất.
  const ketNoi = khoGiaTab === "ketnoi";
  const canEdit = (session?.permissions ?? []).includes("dossier.update");
  const { cat } = useClCatalog();
  const [phongId, setPhongId] = useState("");
  const [phongBanId, setPhongBanId] = useState("");
  const [mangId, setMangId] = useState("");
  const [thbqId, setThbqId] = useState("");
  const [khoId, setKhoId] = useState("");
  const [giaId, setGiaId] = useState("");
  const [status, setStatus] = useState("");
  const [trungThua, setTrungThua] = useState(false);
  const [themLoc, setThemLoc] = useState(false);
  const [q, setQ] = useState("");
  const [qA, setQA] = useState("");
  const [page, setPage] = useState(1);
  const [open, setOpen] = useState<ClRow | null>(null);
  const [kieu, setKieu] = useState<KieuMLHS | 0>(0);
  const [busy, setBusy] = useState(false);
  const [anCot, setAnCot] = useState<Set<string>>(() => new Set(AN_MAC_DINH));
  const [ocrKq, setOcrKq] = useState<ClTuOcrResp | null>(null);
  const { projectId } = useProject();

  useEffect(() => {
    const t = window.setTimeout(() => {
      setQA(q.trim());
      setPage(1);
    }, 400);
    return () => window.clearTimeout(t);
  }, [q]);

  const phong = cat.phong.find((p) => p.id === phongId);
  const phongBans = useMemo(() => cat.phong_ban.filter((p) => str(p, "phong_id") === phongId), [cat.phong_ban, phongId]);
  const mangs = useMemo(() => cat.mang.filter((m) => str(m, "phong_ban_id") === phongBanId), [cat.mang, phongBanId]);
  const gias = useMemo(() => cat.gia.filter((g) => !khoId || str(g, "kho_id") === khoId), [cat.gia, khoId]);
  const locGoc: Omit<ClHoSoFilter, "limit" | "offset"> = useMemo(
    () => ({ phongId: phongId || undefined, phongBanId: phongBanId || undefined, mangId: mangId || undefined, thbqIds: thbqId ? [thbqId] : [], khoId: khoId || undefined, giaId: giaId || undefined, status: (status || undefined) as ClHoSoFilter["status"], trungThua: trungThua ? true : undefined, q: qA || undefined, sort: "ho_so_so" }),
    [phongId, phongBanId, mangId, thbqId, khoId, giaId, status, trungThua, qA],
  );
  const loader = useCallback(() => (isTauri() ? clHoSoList({ ...locGoc, limit: PAGE, offset: (page - 1) * PAGE }) : Promise.resolve({ rows: [] as ClRow[], total: 0 })), [locGoc, page]);
  const { data, error, loading, reload } = V41t(loader, { topics: ["chinhLy"], errorText: () => "Không tải được danh sách hồ sơ." });
  const rows = useMemo(() => data?.rows ?? [], [data]);
  const total = data?.total ?? 0;
  const datLoc = <T,>(setter: (v: T) => void) => (v: T) => {
    setter(v);
    setPage(1);
  };

  /** Toàn bộ hồ sơ khớp bộ lọc (không phân trang), dùng cho kết xuất. */
  const layTatCa = async (): Promise<ClRow[]> => (await clHoSoList({ ...locGoc, limit: 200000 })).rows;
  /** Đọc hết một bảng của bản sao theo trang (mirror kẹp 200.000 dòng / lần; phông lớn × nhiều tệp vượt mức đó). */
  const layHetBang = async (table: "van_ban" | "van_ban_file", f: Omit<ClListFilter, "limit" | "offset">): Promise<ClRow[]> => {
    const out: ClRow[] = [];
    const LO = 100000;
    for (let offset = 0; ; offset += LO) {
      const r = await clRows(table, { ...f, limit: LO, offset });
      out.push(...r.rows);
      if (r.rows.length < LO) return out;
    }
  };
  const vanBanCua = async (hs: ClRow[]): Promise<Map<string, ClRow[]>> => {
    const ids = new Set(hs.map((h) => h.id));
    const all = await layHetBang("van_ban", { order: "so_thu_tu" });
    const m = new Map<string, ClRow[]>();
    const loai = new Map(cat.loai_van_ban.map((l) => [str(l, "ma"), str(l, "ten")]));
    for (const v of all) {
      const k = str(v, "ho_so_id");
      if (!ids.has(k)) continue;
      v.ten_loai_vb = loai.get(str(v, "loai_vb")) ?? "";
      if (!m.has(k)) m.set(k, []);
      m.get(k)!.push(v);
    }
    return m;
  };
  /** Văn bản id → tên tệp quét thật (cl_van_ban_file, dòng đầu theo so_tt) của các hồ sơ đã chọn. */
  const tepCuaVanBan = async (vbs: Map<string, ClRow[]>): Promise<Map<string, string>> => {
    const ids = new Set<string>();
    for (const xs of vbs.values()) for (const v of xs) ids.add(v.id);
    const m = new Map<string, string>();
    if (ids.size === 0) return m;
    for (const f of await layHetBang("van_ban_file", { order: "so_tt" })) {
      const vb = str(f, "van_ban_id");
      const ten = str(f, "name");
      if (ids.has(vb) && ten && !m.has(vb)) m.set(vb, ten);
    }
    return m;
  };
  const taoTuOcr = async () => {
    if (!phongId || !projectId) return;
    setBusy(true);
    setOcrKq(null);
    let tid = toast.info("Tạo văn bản từ OCR: đang ghép tài liệu với hồ sơ…");
    try {
      const kq = await clTaoVanBanTuOcr(projectId, phongId, undefined, (t, lan) => {
        toast.dismiss(tid);
        tid = toast.info(`Tạo văn bản từ OCR: lô ${lan} — ${n(t.taiLieuXet)} tài liệu, thêm ${n(t.created)}, cập nhật ${n(t.updated)}…`);
      });
      try {
        await clSyncNow();
      } catch {
        /* đồng bộ nền sẽ kéo sau */
      }
      setOcrKq(kq);
      reload();
    } catch (e) {
      notifyError(e, "tạo văn bản từ OCR");
    } finally {
      toast.dismiss(tid);
      setBusy(false);
    }
  };
  const chay = async (ten: string, f: (all: ClRow[]) => Promise<unknown>) => {
    if (total === 0) {
      toast.warning("Không có hồ sơ khớp bộ lọc để kết xuất.");
      return;
    }
    setBusy(true);
    const id = toast.info(`${ten}: đang chuẩn bị ${n(total)} hồ sơ…`);
    try {
      const all = await layTatCa();
      await f(all);
      toast.success(`${ten}: ${n(all.length)} hồ sơ.`);
    } catch (e) {
      notifyError(e, ten.toLowerCase());
    } finally {
      toast.dismiss(id);
      setBusy(false);
    }
  };
  const tenPhongHien = str(phong, "ten") || (rows[0] ? str(rows[0], "ten_phong") : "");
  const tenPb = phongBanId ? str(phongBans.find((p) => p.id === phongBanId), "ten") : "";
  const coThoiHan = status === "GL" || status === "HET" || (thbqId !== "" && str(cat.thbq.find((t) => t.id === thbqId), "trang_thai") !== "1");

  const ketXuat: MenuEntry[] = [
    {
      label: "Tạo văn bản từ OCR (phông đang chọn)",
      icon: "taiLieu",
      disabled: busy || !phongId || !projectId || !canEdit,
      hint: !phongId ? "Chọn phông" : "Cần máy chủ; văn bản máy tạo đánh dấu chưa duyệt",
      onSelect: () => void taoTuOcr(),
    },
    "-",
    { label: "Metadata TT05 (Excel)", icon: "tepExcel", onSelect: () => void chay("Xuất metadata TT05", async (all) => { const vbs = await vanBanCua(all); return xuatTT05(all, vbs, phong, await tepCuaVanBan(vbs)); }) },
    { label: "Metadata TT02 (Excel)", icon: "tepExcel", onSelect: () => void chay("Xuất metadata TT02", async (all) => xuatTT02(all, await vanBanCua(all), tenPhongHien)) },
    "-",
    { label: "Mục lục hồ sơ (Excel)", icon: "tepExcel", hint: kieu === 0 ? "Theo số hồ sơ" : `Kiểu ${kieu}`, onSelect: () => void chay("Xuất mục lục hồ sơ", (all) => xuatMucLucHoanThien(all, kieu, status === "HET" || trungThua, tenPhongHien)) },
    { label: "In mục lục hồ sơ", icon: "inRa", onSelect: () => void chay("In mục lục hồ sơ", async (all) => inHtml("Mục lục hồ sơ", inMucLucHoSo(all, tenPhongHien, tenPb))) },
    { label: "In sản phẩm mục lục có bìa", icon: "inRa", onSelect: () => void chay("In sản phẩm", async (all) => inHtml("Mục lục hồ sơ (sản phẩm)", inSanPham(all, tenPhongHien, tenPb, str(phong, "phong_so"), str(phong, "muc_luc_so"), coThoiHan))) },
    "-",
    { label: "Mục lục văn bản (Excel)", icon: "tepExcel", onSelect: () => void chay("Xuất mục lục văn bản", async (all) => xuatMucLucVanBan(all, await vanBanCua(all))) },
    { label: "In mục lục văn bản", icon: "inRa", onSelect: () => void chay("In mục lục văn bản", async (all) => inHtml("Mục lục văn bản", inMucLucVanBan(all, await vanBanCua(all), tenPhongHien, tenPb))) },
    "-",
    { label: "In nhãn hộp", icon: "inRa", onSelect: () => void chay("In nhãn hộp", async (all) => inHtml("Nhãn hộp", inNhanHop(all, tenPhongHien))) },
    { label: "In phụ lục hướng dẫn tra tìm", icon: "inRa", onSelect: () => void chay("In phụ lục", async (all) => inHtml("Hướng dẫn tra tìm mục lục", inPhuLuc(all, tenPhongHien))) },
    { label: "Phụ lục hướng dẫn tra tìm (Excel)", icon: "tepExcel", onSelect: () => void chay("Xuất phụ lục", (all) => xuatPhuLuc(all, str(phong, "ma_du_an") || "phong")) },
    "-",
    {
      label: "Lịch sử thay đổi (Excel)",
      icon: "lichSu",
      onSelect: () =>
        void chay("Xuất lịch sử", async (all) => {
          const ls = await clRows("lich_su", { limit: 200000, order: "at DESC" });
          const ids = new Set(all.map((r) => r.id));
          const st = new Map(all.map((r) => [r.id, str(r, "so_tam")]));
          const sub = ls.rows.filter((l) => ids.has(str(l, "ho_so_id"))).map((l) => ({ ...l, so_tam: st.get(str(l, "ho_so_id")) ?? "" }));
          await xuatLichSu(sub, str(phong, "ma_du_an") || "LichSu");
        }),
    },
  ];

  const cols: Column<ClRow>[] = [
    { key: "ten_kho", header: "Kho", cell: (r) => str(r, "ten_kho"), hideBelow: 1400 },
    { key: "ten_gia", header: "Giá", cell: (r) => str(r, "ten_gia"), hideBelow: 1400 },
    { key: "ma_ho_so", header: "Mã hồ sơ", className: "text-xs text-fg-subtle", cell: (r) => str(r, "ma_ho_so") },
    { key: "hop_so", header: "Hộp", align: "right", width: 70, cell: (r) => str(r, "hop_so") },
    { key: "bo_so", header: "Bó", align: "right", width: 60, cell: (r) => str(r, "bo_so") },
    { key: "tap_so", header: "Tập", align: "right", width: 60, cell: (r) => str(r, "tap_so") },
    { key: "ho_so_so", header: "Hồ sơ số", align: "right", width: 90, cell: (r) => str(r, "ho_so_so") },
    { key: "tieu_de", header: "Tiêu đề", className: "max-w-[420px] whitespace-normal", cell: (r) => str(r, "tieu_de") },
    { key: "ten_thbq", header: "THBQ", className: "text-sm", cell: (r) => str(r, "ten_thbq") },
    { key: "so_to", header: "Số tờ", align: "right", width: 70, cell: (r) => str(r, "so_to") },
    { key: "ngay_bat_dau", header: "Bắt đầu", width: 100, cell: (r) => str(r, "ngay_bat_dau") },
    { key: "ngay_ket_thuc", header: "Kết thúc", width: 100, cell: (r) => str(r, "ngay_ket_thuc") },
    { key: "trung_thua", header: "Trùng thừa", width: 90, cell: (r) => (r.trung_thua === true ? <Badge tone="gray">trùng thừa</Badge> : null) },
    { key: "so_van_ban", header: "Văn bản", align: "right", width: 80, cell: (r) => str(r, "so_van_ban") },
    {
      key: "thbq_status",
      header: "Trạng thái",
      width: 110,
      cell: (r) => {
        const st = str(r, "thbq_status");
        return st ? <Badge tone={THBQ_BADGE[st] ?? "gray"}>{THBQ_STATUS_LABEL[st] ?? st}</Badge> : null;
      },
    },
  ];
  const TEN_COT: Record<string, string> = { ten_kho: "Kho", ten_gia: "Giá", ma_ho_so: "Mã hồ sơ", bo_so: "Bó", tap_so: "Tập", trung_thua: "Trùng thừa", so_van_ban: "Văn bản" };
  const colsHien = cols.filter((c) => !anCot.has(c.key));
  const soLocThem = [mangId, thbqId, khoId, giaId, trungThua ? "1" : ""].filter(Boolean).length;
  // Kết xuất: 4 mục đầu (Tạo văn bản từ OCR · — · TT05 · TT02) thuộc "Kết nối dữ liệu chủ"; phần còn lại thuộc mục lục.
  const ketXuatHien = ketNoi ? ketXuat.slice(0, 4) : ketXuat.slice(5);

  return (
    <section className="view active">
      <PageHeader
        title={ketNoi ? "Kết nối dữ liệu chủ" : "Mục lục và kết xuất"}
        description={ketNoi ? "Bước 4 Điều 9 TT 05/2025: ghép tài liệu đã quét và OCR với hồ sơ trong mục lục, tạo văn bản, duyệt rồi xuất metadata TT05." : "Bước 3 Điều 9 TT 05/2025: tra cứu hồ sơ theo phông, phòng ban, mảng, THBQ, kho, giá; kết xuất mục lục, nhãn hộp, sản phẩm, phụ lục."}
        actions={
          <>
            <UiSelect size="sm" className="w-64" value={String(kieu)} onChange={(v) => setKieu(Number(v) as KieuMLHS | 0)} aria-label="Kiểu mục lục hồ sơ" options={[{ value: "0", label: "Mục lục theo số hồ sơ (mẫu kho)" }, ...KIEU_MLHS.map(([k, l]) => ({ value: String(k), label: `${k}. ${l}` }))]} />
            <Menu trigger="button" label={busy ? "Đang kết xuất…" : "Kết xuất"} items={ketXuatHien} />
            <Menu label="Cột hiển thị" items={Object.entries(TEN_COT).map(([k, l]) => ({ label: `${anCot.has(k) ? "Hiện" : "Ẩn"} cột ${l}`, onSelect: () => setAnCot((s) => { const x = new Set(s); if (!x.delete(k)) x.add(k); return x; }) }))} />
          </>
        }
      />
      <ChuaChonDuAn />
      {ocrKq && (
        <Banner
          tone={ocrKq.errors.length ? "warning" : "success"}
          className="mb-4"
          onClose={() => setOcrKq(null)}
          detail={[...ocrKq.errors.map((e) => `Lỗi: ${e.message}`), ...ocrKq.taiLieuKhongKhop.map((t) => `Không ghép: ${t.filename} — ${t.lyDo}`), ...(ocrKq.notes ?? [])].join("\n") || undefined}
        >
          Tạo văn bản từ OCR: {n(ocrKq.hoSoXet)} hồ sơ xét, {n(ocrKq.hoSoKhop)} có tài liệu · {n(ocrKq.taiLieuXet)} tài liệu → thêm {n(ocrKq.created)}, cập nhật {n(ocrKq.updated)}, chỉ nối tệp {n(ocrKq.daLienKetTep)} (nhập tay / đã duyệt), không đổi {n(ocrKq.giuNguyen)}
          {ocrKq.taiLieuKhongKhop.length ? ` · ${n(ocrKq.taiLieuKhongKhop.length)} tài liệu không ghép được hồ sơ` : ""}
          {ocrKq.errors.length ? ` · ${n(ocrKq.errors.length)} lỗi` : ""}. Văn bản máy tạo ở trạng thái "chưa duyệt" — mở hồ sơ để soát và bấm Đã duyệt.
        </Banner>
      )}
      <Card className="mb-4" padding="sm">
        <FormGrid cols={4}>
          <Field label="Phông">
            <Select
              size="sm"
              value={phongId}
              onChange={(v) => {
                setPhongId(v);
                setPhongBanId("");
                setMangId("");
                setPage(1);
              }}
              rows={cat.phong}
              render={tenPhong}
              empty="Tất cả"
            />
          </Field>
          <Field label="Phòng ban">
            <Select
              size="sm"
              value={phongBanId}
              onChange={(v) => {
                setPhongBanId(v);
                setMangId("");
                setPage(1);
              }}
              rows={phongBans}
              empty="Tất cả"
            />
          </Field>
          <Field label="Trạng thái THBQ">
            <UiSelect size="sm" value={status} onChange={datLoc(setStatus)} placeholder="Tất cả" options={[{ value: "VV", label: "Vĩnh viễn" }, { value: "GL", label: "Giữ lại" }, { value: "HET", label: "Hết THBQ" }]} />
          </Field>
          <Field label="Tìm">
            <Input size="sm" prefix="tim" value={q} onChange={(e) => setQ(e.target.value)} placeholder="Tiêu đề, số tạm, số hoặc mã hồ sơ" />
          </Field>
        </FormGrid>
        <div className="mt-2 flex flex-wrap items-center gap-2">
          <Button size="sm" variant="ghost" iconRight={themLoc ? "len" : "xuong"} onClick={() => setThemLoc((v) => !v)} aria-expanded={themLoc}>
            {soLocThem > 0 ? `Thêm bộ lọc (${soLocThem})` : "Thêm bộ lọc"}
          </Button>
          <span className="text-sm text-fg-subtle">{n(total)} hồ sơ khớp</span>
        </div>
        {themLoc && (
          <FormGrid cols={4} className="mt-3">
            <Field label="Mảng công việc">
              <Select size="sm" value={mangId} onChange={datLoc(setMangId)} rows={mangs} empty="Tất cả" disabled={!phongBanId} />
            </Field>
            <Field label="Thời hạn bảo quản">
              <Select size="sm" value={thbqId} onChange={datLoc(setThbqId)} rows={cat.thbq} empty="Tất cả" />
            </Field>
            <Field label="Kho">
              <Select
                size="sm"
                value={khoId}
                onChange={(v) => {
                  setKhoId(v);
                  setGiaId("");
                  setPage(1);
                }}
                rows={cat.kho}
                empty="Tất cả"
              />
            </Field>
            <Field label="Giá">
              <Select size="sm" value={giaId} onChange={datLoc(setGiaId)} rows={gias} empty="Tất cả" render={(g) => `${str(g, "ten")} (hộp ${str(g, "tu_hop")} đến ${str(g, "den_hop")})`} />
            </Field>
            <Field label="Trùng thừa" inline className="self-end">
              <Chip on={trungThua} onClick={() => datLoc(setTrungThua)(!trungThua)}>
                Chỉ hồ sơ trùng thừa
              </Chip>
            </Field>
          </FormGrid>
        )}
      </Card>
      <Card padding="none">
        <Table
          columns={colsHien}
          rows={rows}
          getRowId={(r) => r.id}
          dense
          onRowClick={(r) => setOpen(r)}
          rowClassName={(r) => (open?.id === r.id ? "bg-brand-50" : undefined)}
          pagination={{ page, pageSize: PAGE, total, onChange: setPage }}
          state={{ loading: loading && !data, error, onRetry: reload, empty: { title: qA || soLocThem || status || phongId ? "Không có hồ sơ nào khớp bộ lọc" : "Chưa có hồ sơ chỉnh lý nào" } }}
        />
      </Card>
      {open && <VanBanPanel cat={cat} hoSo={open} canEdit={canEdit} onClose={() => setOpen(null)} />}
    </section>
  );
}
