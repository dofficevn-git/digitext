// Copyright (c) 2026 DigiText. All rights reserved.
// Tra cứu tài liệu lưu trữ (bản Lưu trữ GĐ4, 5/11/2026 — docs/KE-HOACH-BAN-LUU-TRU.md): một ô tìm không dấu trên mục lục
// hồ sơ – văn bản và lớp chữ của tài liệu, bộ lọc phông / THBQ / loại / năm / chế độ sử dụng, đọc bản có watermark ngay
// trong ứng dụng, chọn tài liệu để gửi yêu cầu khai thác. Cán bộ lưu trữ có thêm tab Nhật ký khai thác.

import { useCallback, useMemo, useState } from "react";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { V41t } from "@/live/useLive";
import { traCuuBoLoc, traCuuDoc, traCuuNhatKy, traCuuTim, type DongNhatKy, type DongTraCuu, type TraCuuQuery } from "@/api/tracuu";
import { YeuCauMoiDialog, type TaiLieuChon } from "@/features/access/AccessView";
import { cdTai, cdTao, type DongChuyenDoi } from "@/api/chuyenDoi";
import { CanDuAn, Badge, Button, Card, Checkbox, Dialog, ErrorBanner, Field, FormGrid, Input, PageHeader, Select, Table, Tabs, Textarea, Tip, Toolbar, notifyError, toast, type Column } from "@/ui";

const fmt = (n: number) => n.toLocaleString("vi-VN");
const PAGE = 50;

const VIEC: Record<string, string> = {
  "khai_thac.xem": "Đọc từ Tra cứu",
  "khai_thac.doc": "Đọc theo yêu cầu",
  "khai_thac.tai": "Tải bản sao",
  "khai_thac.dip": "Cấp gói DIP",
  "khai_thac.chuyen_doi": "Chuyển đổi ra giấy",
};

type Loc = Omit<TraCuuQuery, "projectId" | "limit" | "offset" | "q" | "noiDung">;

/** Lý do không đọc thẳng được (rỗng = đọc được). */
/** Khoá dòng: văn bản, hoặc hồ sơ khi hồ sơ chưa lập văn bản nào. */
const khoa = (d: DongTraCuu) => d.vanBanId ?? `hs:${d.hoSoId}`;

function lyDoKhongDoc(d: DongTraCuu, canBo: boolean, docThang: boolean): string {
  if (!d.vanBanId) return "Hồ sơ chưa lập văn bản — mới có mục lục hồ sơ";
  if (!d.documentId) return "Văn bản chưa có bản số";
  if (d.daHuy) return "Tài liệu đã tiêu huỷ — chỉ còn mục lục";
  if (canBo) return "";
  if (d.hanChe) return "Tài liệu hạn chế sử dụng — gửi yêu cầu khai thác để đọc";
  if (!docThang) return "Đọc tài liệu có thu phí — gửi yêu cầu khai thác";
  return "";
}

function NhatKy({ projectId }: { projectId: string }) {
  const loader = useCallback(() => traCuuNhatKy(projectId), [projectId]);
  const { data, loading, error, reload } = V41t(loader, { topics: ["access"], errorText: () => "Không tải được nhật ký khai thác." });
  const cols: Column<DongNhatKy>[] = [
    { key: "at", header: "Thời gian", width: 150, cell: (r) => new Date(r.at).toLocaleString("vi-VN", { dateStyle: "short", timeStyle: "short" }) },
    { key: "nguoi", header: "Người khai thác", width: 180, cell: (r) => r.nguoi ?? r.tenDangNhap ?? "—" },
    { key: "action", header: "Việc", width: 150, cell: (r) => VIEC[r.action] ?? r.action },
    {
      key: "detail",
      header: "Tài liệu",
      className: "whitespace-normal",
      cell: (r) => {
        const d = r.detail ?? {};
        const s = (k: string) => (typeof d[k] === "string" ? (d[k] as string) : "");
        if (r.action === "khai_thac.dip") return `Gói DIP ${s("maXacThuc")} · ${String(d["soTaiLieu"] ?? "")} tài liệu`;
        return [s("maHoSo"), s("trichYeu") || s("tep")].filter(Boolean).join(" — ") || r.documentId || "—";
      },
    },
    { key: "ip", header: "Địa chỉ máy", width: 130, cell: (r) => r.ip ?? "—" },
  ];
  return (
    <Card title="Nhật ký khai thác" description="500 lần đọc, tải, cấp gói gần nhất." padding="none">
      <Table columns={cols} rows={data ?? []} getRowId={(r) => String(r.id)} dense state={{ loading, error, onRetry: reload, empty: { title: "Chưa có lượt khai thác nào" } }} maxHeight={560} />
    </Card>
  );
}

export default function TraCuuView() {
  const { session } = V460();
  const proj = useProject();
  const projectId = session?.projectId ?? proj.projectId ?? "";
  const [tab, setTab] = useState<"tim" | "nhatky">("tim");
  const [o, setO] = useState("");
  const [noiDung, setNoiDung] = useState(true);
  const [q, setQ] = useState("");
  const [loc, setLoc] = useState<Loc>({});
  const [page, setPage] = useState(1);
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [chon, setChon] = useState<Map<string, TaiLieuChon>>(new Map());
  const [doc, setDoc] = useState<{ src: string; title: string } | null>(null);
  const [dangMo, setDangMo] = useState<string | null>(null);
  const [yeuCau, setYeuCau] = useState(false);

  const taiBoLoc = useCallback(() => traCuuBoLoc(projectId), [projectId]);
  const bl = V41t(taiBoLoc, { topics: ["catalog", "access"], enabled: !!projectId, errorText: () => "Không tải được bộ lọc." });
  const canBo = bl.data?.canBo ?? false;
  const docThang = bl.data?.docThang ?? false;

  const tai = useCallback(
    () => traCuuTim({ projectId, q: q || undefined, noiDung, ...loc, limit: PAGE, offset: (page - 1) * PAGE }),
    [projectId, q, noiDung, loc, page],
  );
  const kq = V41t(tai, { topics: ["catalog"], enabled: !!projectId, errorText: () => "Tra cứu không thành công." });
  const dong = kq.data?.dong ?? [];

  const datLoc = (patch: Loc) => {
    setLoc((l) => ({ ...l, ...patch }));
    setPage(1);
  };
  const tim = () => {
    setQ(o.trim());
    setPage(1);
  };
  const doiChon = (next: Set<string>) => {
    setSel(next);
    setChon((m) => {
      const out = new Map([...m].filter(([id]) => next.has(id)));
      for (const d of dong) {
        if (d.vanBanId && next.has(d.vanBanId) && d.documentId && !out.has(d.vanBanId)) {
          out.set(d.vanBanId, { id: d.documentId, filename: d.trichYeu || `${d.maHoSo}.${d.soThuTu}`, hoSoRef: d.maHoSo, pageCount: d.soTrang });
        }
      }
      return out;
    });
  };

  const [chuyenDoi, setChuyenDoi] = useState<DongTraCuu | null>(null);
  const doc_ = async (d: DongTraCuu) => {
    if (!d.documentId) return;
    setDangMo(khoa(d));
    try {
      const b64 = await traCuuDoc(d.documentId);
      setDoc({ src: `data:application/pdf;base64,${b64}`, title: `${d.maHoSo} · văn bản ${d.soThuTu}` });
    } catch (e) {
      notifyError(e, "mở bản đọc");
    } finally {
      setDangMo(null);
    }
  };

  const namOpts = useMemo(() => {
    const out = [{ value: "", label: "Mọi năm" }];
    const { namMin, namMax } = bl.data ?? {};
    if (namMin && namMax) for (let y = namMax; y >= namMin; y--) out.push({ value: String(y), label: String(y) });
    return out;
  }, [bl.data]);

  const cols: Column<DongTraCuu>[] = [
    {
      key: "maHoSo",
      header: "Hồ sơ",
      width: 220,
      className: "whitespace-normal",
      cell: (d) => (
        <div>
          <div className="font-medium text-fg-strong">{d.maHoSo}</div>
          <div className="line-clamp-2 text-sm text-fg-subtle">{d.tieuDeHoSo}</div>
        </div>
      ),
    },
    { key: "soThuTu", header: "STT", width: 56, align: "right", cell: (d) => d.soThuTu ?? "—" },
    { key: "loaiVb", header: "Loại", width: 110, cell: (d) => d.loaiVb ?? "—" },
    { key: "soKyHieu", header: "Số, ký hiệu", width: 120, cell: (d) => d.soKyHieu || "—" },
    { key: "ngayThang", header: "Ngày", width: 100, cell: (d) => d.ngayThang || "—" },
    {
      key: "trichYeu",
      header: "Trích yếu",
      className: "whitespace-normal",
      cell: (d) => (
        <div>
          <div>{d.vanBanId ? d.trichYeu || "—" : <span className="text-fg-subtle">Chưa lập văn bản</span>}</div>
          {d.coQuan && <div className="text-sm text-fg-subtle">{d.coQuan}</div>}
          {d.khop === "noi_dung" && d.trichDoan && (
            <div className="mt-1 rounded bg-surface-sunken px-2 py-1 text-sm text-fg-muted">
              <span className="font-medium">Trong nội dung{d.trang ? `, trang ${d.trang}` : ""}:</span> …{d.trichDoan}…
            </div>
          )}
        </div>
      ),
    },
    {
      key: "trangThai",
      header: "Ghi chú",
      width: 120,
      cell: (d) => (
        <div className="flex flex-wrap gap-1">
          {d.daHuy && <Badge tone="err">Đã huỷ</Badge>}
          {d.hanChe && <Badge tone="warn">Hạn chế</Badge>}
          {d.kySo === "chua_ky" && <Badge tone="gray">Chưa ký số</Badge>}
          {!d.documentId && <Badge tone="gray">{d.vanBanId ? "Chưa có bản số" : "Chưa số hoá"}</Badge>}
        </div>
      ),
    },
  ];

  const cachTim = (
    <>
            <p>Gõ các từ cần tìm, không cần dấu: “quyet dinh dat dai” khớp “Quyết định … đất đai”. Các từ phải xuất hiện theo đúng thứ tự gõ, có thể cách nhau.</p>
            <p>Bật “Tìm cả trong nội dung” để tìm thêm trong lớp chữ của tài liệu; dòng khớp nội dung có trích đoạn và số trang. Chất lượng phụ thuộc lớp chữ trong gói nhập.</p>
            <p>Tài liệu hạn chế sử dụng chỉ hiện mục lục. Muốn đọc, sao hoặc nhận gói DIP: đánh dấu văn bản rồi bấm Yêu cầu khai thác (Điều 35–42 Luật Lưu trữ 2024).</p>
    </>
  );

  if (!projectId) {
    return (
      <section className="view active">
        <PageHeader title="Tra cứu tài liệu" infoTitle="Cách tìm" info={cachTim} />
        <Card padding="none">
          <CanDuAn />
        </Card>
      </section>
    );
  }

  return (
    <section className="view active space-y-4">
      <PageHeader
        title="Tra cứu tài liệu"
        description={bl.data ? `Kho có ${fmt(bl.data.tongHoSo)} hồ sơ, ${fmt(bl.data.tongVanBan)} văn bản.` : undefined}
        infoTitle="Cách tìm"
        info={cachTim}
      />
      {canBo && <Tabs items={[{ value: "tim", label: "Tra cứu", icon: "tim" }, { value: "nhatky", label: "Nhật ký khai thác", icon: "lichSu" }]} value={tab} onChange={setTab} variant="line" />}

      {tab === "nhatky" && canBo ? (
        <NhatKy projectId={projectId} />
      ) : (
        <>
          <Card padding="md">
            <form
              className="flex flex-wrap items-center gap-2"
              onSubmit={(e) => {
                e.preventDefault();
                tim();
              }}
            >
              <Input prefix="tim" placeholder="Trích yếu, số ký hiệu, cơ quan, tiêu đề hồ sơ" value={o} onChange={(e) => setO(e.target.value)} className="min-w-72 flex-1" aria-label="Từ cần tìm" autoFocus />
              <Button type="submit" variant="primary" icon="tim">
                Tìm
              </Button>
              {q && (
                <Button
                  onClick={() => {
                    setO("");
                    setQ("");
                    setPage(1);
                  }}
                >
                  Xoá từ tìm
                </Button>
              )}
              <Checkbox checked={noiDung} onChange={(v) => { setNoiDung(v); setPage(1); }} text="Tìm cả trong nội dung tài liệu" />
            </form>
            {bl.error && !bl.data ? (
              <ErrorBanner className="mt-3" error={bl.error} onRetry={bl.reload} />
            ) : (
              <div className="mt-3 grid grid-cols-2 gap-2 md:grid-cols-3 xl:grid-cols-6">
                <Select size="sm" value={loc.phongId ?? ""} onChange={(v) => datLoc({ phongId: v || undefined })} aria-label="Phông" options={[{ value: "", label: "Mọi phông" }, ...(bl.data?.phong ?? []).map((p) => ({ value: p.id, label: `${p.ten} (${fmt(p.so)})` }))]} />
                <Select size="sm" value={loc.thbqId ?? ""} onChange={(v) => datLoc({ thbqId: v || undefined })} aria-label="Thời hạn bảo quản" options={[{ value: "", label: "Mọi thời hạn BQ" }, ...(bl.data?.thbq ?? []).map((t) => ({ value: t.id, label: `${t.ten} (${fmt(t.so)})` }))]} />
                <Select size="sm" value={loc.loaiVb ?? ""} onChange={(v) => datLoc({ loaiVb: v || undefined })} aria-label="Loại văn bản" options={[{ value: "", label: "Mọi loại văn bản" }, ...(bl.data?.loaiVb ?? []).map((l) => ({ value: l.ma, label: `${l.ten} (${fmt(l.so)})` }))]} />
                <Select size="sm" value={loc.namTu ? String(loc.namTu) : ""} onChange={(v) => datLoc({ namTu: v ? Number(v) : undefined })} aria-label="Từ năm" options={namOpts.map((x, i) => (i === 0 ? { ...x, label: "Từ năm" } : x))} />
                <Select size="sm" value={loc.namDen ? String(loc.namDen) : ""} onChange={(v) => datLoc({ namDen: v ? Number(v) : undefined })} aria-label="Đến năm" options={namOpts.map((x, i) => (i === 0 ? { ...x, label: "Đến năm" } : x))} />
                <Select size="sm" value={loc.cheDo ?? ""} onChange={(v) => datLoc({ cheDo: (v || undefined) as Loc["cheDo"] })} aria-label="Chế độ sử dụng" options={[{ value: "", label: "Mọi chế độ sử dụng" }, { value: "cong_khai", label: "Công khai" }, { value: "han_che", label: "Hạn chế sử dụng" }]} />
              </div>
            )}
            {canBo && (
              <div className="mt-2">
                <Checkbox checked={!!loc.daHuy} onChange={(v) => datLoc({ daHuy: v || undefined })} text="Hiện cả văn bản đã tiêu huỷ (chỉ cán bộ lưu trữ)" />
              </div>
            )}
          </Card>

          <Card padding="none">
            <div className="p-3">
              <Toolbar>
                <span className="text-sm text-fg-muted">{kq.data ? (q ? `${fmt(kq.data.tong)} dòng khớp “${q}”` : `${fmt(kq.data.tong)} dòng`) : "Đang tìm…"}</span>
                <Toolbar.Spacer />
                {sel.size > 0 && <span className="text-sm text-fg-muted">{fmt(sel.size)} đã chọn</span>}
                <Tip text={sel.size === 0 ? "Đánh dấu các văn bản cần khai thác trong bảng" : "Gửi yêu cầu đọc, sao hoặc cấp gói DIP cho các văn bản đã chọn"}>
                  <span>
                    <Button variant="primary" icon="khaithac" disabled={chon.size === 0} onClick={() => setYeuCau(true)}>
                      Yêu cầu khai thác
                    </Button>
                  </span>
                </Tip>
              </Toolbar>
            </div>
            <Table
              columns={cols}
              rows={dong}
              getRowId={khoa}
              selection={{ selected: sel, onChange: doiChon, canSelect: (d) => !!d.documentId && !d.daHuy }}
              rowActions={(d) => {
                const why = lyDoKhongDoc(d, canBo, docThang);
                return (
                  <span className="flex items-center gap-1">
                    <Tip text={why || "Đọc bản có watermark, không tải về"}>
                      <span>
                        <Button size="sm" icon="xem" disabled={!!why} loading={dangMo === khoa(d)} onClick={() => void doc_(d)}>
                          Đọc
                        </Button>
                      </span>
                    </Tip>
                    {canBo && d.documentId && !d.daHuy && (
                      <Tip text="Chuyển đổi tài liệu số ra giấy: ghép trang thể thức Phụ lục VI, cấp số, ghi sổ">
                        <span>
                          <Button size="sm" icon="inRa" onClick={() => setChuyenDoi(d)}>
                            In ra giấy
                          </Button>
                        </span>
                      </Tip>
                    )}
                  </span>
                );
              }}
              pagination={{ page, pageSize: PAGE, total: kq.data?.tong ?? 0, onChange: setPage }}
              state={{ loading: kq.loading, error: kq.error, onRetry: kq.reload, empty: { title: q ? "Không có hồ sơ, văn bản nào khớp" : "Kho chưa có hồ sơ", description: q ? "Thử bớt từ, bỏ bộ lọc, hoặc bật tìm cả trong nội dung." : "Lập hồ sơ ở Dữ liệu chủ hoặc nhập gói AIP." } }}
            />
          </Card>

        </>
      )}

      <Dialog open={!!doc} onOpenChange={(v) => !v && setDoc(null)} title={doc?.title ?? "Bản đọc"} description="Bản đọc có watermark, không tải về. Lượt đọc được ghi vào nhật ký khai thác." size="full" bodyClassName="p-0">
        {doc && <embed src={doc.src} type="application/pdf" className="h-full w-full" />}
      </Dialog>

      {chuyenDoi && projectId && <ChuyenDoiDialog projectId={projectId} d={chuyenDoi} onClose={() => setChuyenDoi(null)} />}
      {yeuCau && (
        <YeuCauMoiDialog
          projectId={projectId}
          chonSan={[...chon.values()]}
          onClose={() => setYeuCau(false)}
          onDone={() => {
            setYeuCau(false);
            setSel(new Set());
            setChon(new Map());
          }}
        />
      )}
    </section>
  );
}

/** Hộp thoại chuyển đổi một tài liệu số ra giấy (Điều 13–16, PL VI): mục đích, người nhận, căn cứ, chức vụ người ký bản in. */
function ChuyenDoiDialog({ projectId, d, onClose }: { projectId: string; d: DongTraCuu; onClose: () => void }) {
  const [mucDich, setMucDich] = useState("");
  const [nguoiNhan, setNguoiNhan] = useState("");
  const [canCu, setCanCu] = useState("");
  const [chucVu, setChucVu] = useState("");
  const [busy, setBusy] = useState(false);
  const [xong, setXong] = useState<DongChuyenDoi | null>(null);
  const [err, setErr] = useState<unknown>(null);
  const chay = async () => {
    if (!d.documentId) return;
    setBusy(true);
    setErr(null);
    try {
      const r = await cdTao({ projectId, documentId: d.documentId, mucDich: mucDich || undefined, nguoiNhan: nguoiNhan || undefined, canCu: canCu || undefined, chucVu: chucVu || undefined });
      setXong(r);
      toast.success(`Đã chuyển đổi, số ${r.so_chuyen_doi} — chữ ký số: ${r.ky_so_ket_luan ?? "—"}.`);
    } catch (e) {
      setErr(e);
    } finally {
      setBusy(false);
    }
  };
  const tai = async () => {
    if (!xong) return;
    setBusy(true);
    try {
      const p = await cdTai(xong);
      if (p) toast.success(`Đã lưu: ${p}`);
    } catch (e) {
      notifyError(e, "tải bản chuyển đổi");
    } finally {
      setBusy(false);
    }
  };
  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      closable={!busy}
      title="Chuyển đổi tài liệu số ra giấy"
      description={`${d.maHoSo}${d.soThuTu ? ` · văn bản ${d.soThuTu}` : ""} — ${d.trichYeu || d.tieuDeHoSo}`}
      size="md"
      footer={
        xong ? (
          <>
            <Button onClick={onClose}>Đóng</Button>
            <Button variant="primary" icon="tepPdf" loading={busy} onClick={() => void tai()}>
              Tải PDF để in
            </Button>
          </>
        ) : (
          <>
            <Button onClick={onClose} disabled={busy}>
              Huỷ
            </Button>
            <Button variant="primary" icon="inRa" loading={busy} onClick={() => void chay()}>
              Chuyển đổi
            </Button>
          </>
        )
      }
    >
      <div className="space-y-3">
        {err !== null && <ErrorBanner error={err} />}
        {xong ? (
          <div className="rounded-md border border-border-subtle bg-surface-sunken p-3 text-sm">
            <p>
              Số chuyển đổi <b>{xong.so_chuyen_doi}</b> · {xong.so_trang ?? "?"} trang tài liệu + 1 trang thể thức.
            </p>
            <p className="mt-1">Chữ ký số tại thời điểm chuyển đổi: <b>{xong.ky_so_ket_luan ?? "—"}</b>{xong.ky_so_chi_tiet ? ` — ${xong.ky_so_chi_tiet}` : ""}</p>
            <p className="mt-1 text-fg-subtle">In bản PDF, trình người có thẩm quyền ký và đóng dấu lên trang thể thức cuối. Lượt này đã ghi vào sổ chuyển đổi và nhật ký khai thác.</p>
          </div>
        ) : (
          <>
            <p className="text-sm text-fg-muted">Hệ thống kiểm chữ ký số của bản gốc, ghép trang thể thức "TÀI LIỆU LƯU TRỮ CHUYỂN ĐỔI" (Mục 1 Phụ lục VI) vào cuối, đánh chân trang và cấp số chuyển đổi theo năm.</p>
            <Field label="Mục đích chuyển đổi">
              <Textarea rows={2} value={mucDich} onChange={(e) => setMucDich(e.target.value)} placeholder="VD: cung cấp bản giấy theo yêu cầu khai thác số …" autoFocus />
            </Field>
            <FormGrid cols={2}>
              <Field label="Người / đơn vị nhận">
                <Input value={nguoiNhan} onChange={(e) => setNguoiNhan(e.target.value)} />
              </Field>
              <Field label="Chức vụ người ký bản in" hint="Mặc định: Lãnh đạo cơ quan">
                <Input value={chucVu} onChange={(e) => setChucVu(e.target.value)} placeholder="LÃNH ĐẠO CƠ QUAN" />
              </Field>
            </FormGrid>
            <Field label="Căn cứ (văn bản yêu cầu, phiếu khai thác…)">
              <Input value={canCu} onChange={(e) => setCanCu(e.target.value)} />
            </Field>
          </>
        )}
      </div>
    </Dialog>
  );
}
