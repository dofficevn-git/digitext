// Copyright (c) 2026 DigiText. All rights reserved.
// Hộp thoại "Nhập mục lục hồ sơ": chọn tệp → kiểm tra (phòng ban / mảng tự nhận, thời gian đã kiểm & điền,
// tiền tố số tạm) → nhập vào máy chủ (tự tạo phòng ban / mảng còn thiếu). Phân tích chạy trên máy; tải tệp đã
// chuyển / CSV kiểm tra được cả khi mất mạng, chỉ bước nhập cần máy chủ.

import { useMemo, useRef, useState } from "react";
import { clRemote, clSyncNow, str, type ClCatalog, type ClRow } from "@/api/chinhly";
import { useProject } from "@/state/project";
import { Badge, Banner, Button, Checkbox, Dialog, Field, Help, Input, KpiTile, notifyError, Stepper, Table, toast, type Column } from "@/ui";
import type { V3y0 } from "@/domain/types";
import { n } from "./common";
import { coLoiChan, docMucLucHoSo, dongMau, soTamCua, xuatTepDaChuyen, xuatTepDanhDau, type DongHoSo, type PhanTichMucLuc } from "./mucLucImport";
import { KET_LUAN_LABEL } from "./thoiGianHoSo";

const BUOC = [
  { key: "tep", label: "Chọn tệp", hint: "Mục lục hồ sơ (.xlsx)" },
  { key: "kiem", label: "Kiểm tra", hint: "Phòng ban, mảng, thời gian" },
  { key: "nhap", label: "Nhập", hint: "Vào phông đang chọn" },
];
const LO = 5000;
const TRANG = 100;

interface KetQuaNhap {
  created: number;
  updated: number;
  skipped: number;
  createdPhongBan: number;
  createdMang: number;
  notes?: string[];
  errors: { row: number; field?: string; message: string }[];
}

const TONE: Record<string, V3y0> = { dung: "ok", da_dien: "info", canh_bao: "warn", loi: "err" };

export default function NhapMucLucDialog({ cat, phong, offline, onClose, onImported }: { cat: ClCatalog; phong: ClRow; offline: boolean; onClose: () => void; onImported: () => void }) {
  const { projectId } = useProject();
  const [buoc, setBuoc] = useState(0);
  const [pt, setPt] = useState<PhanTichMucLuc | null>(null);
  const [tienTo, setTienTo] = useState("");
  const [chiVanDe, setChiVanDe] = useState(false);
  const [trang, setTrang] = useState(1);
  const [busy, setBusy] = useState(false);
  const [tienDo, setTienDo] = useState("");
  const [kq, setKq] = useState<KetQuaNhap | null>(null);
  const [loiDoc, setLoiDoc] = useState<unknown>(null);
  const fileRef = useRef<HTMLInputElement>(null);

  const maPhong = str(phong, "ma_du_an");
  const soTam = useMemo(() => (pt ? soTamCua(pt, tienTo) : new Map<number, string>()), [pt, tienTo]);
  const chan = pt ? coLoiChan(pt) : true;
  const tenPhongLech = !!pt?.tenPhongTrongTep && pt.tenPhongTrongTep.toLowerCase().replace(/\s+/g, " ") !== str(phong, "ten").toLowerCase().replace(/\s+/g, " ");

  const docTep = async (f: File) => {
    setBusy(true);
    setLoiDoc(null);
    setKq(null);
    try {
      const r = await docMucLucHoSo(f, cat.thbq);
      setPt(r);
      setTienTo(r.tienToGoiY);
      setTrang(1);
      setChiVanDe(false);
      setBuoc(1);
    } catch (e) {
      setLoiDoc(e);
    } finally {
      setBusy(false);
    }
  };

  const nhap = async () => {
    if (!pt || chan || !projectId) return;
    setBusy(true);
    setBuoc(2);
    setKq(null);
    try {
      const rows = pt.rows.map((r) => dongMau(r, soTam.get(r.dong) ?? ""));
      const tong: KetQuaNhap = { created: 0, updated: 0, skipped: 0, createdPhongBan: 0, createdMang: 0, notes: [], errors: [] };
      for (let i = 0; i < rows.length; i += LO) {
        setTienDo(`Đang gửi dòng ${n(i + 1)}–${n(Math.min(i + LO, rows.length))} / ${n(rows.length)}…`);
        const r = await clRemote<KetQuaNhap>("POST", "/cl/ho-so/import", { projectId, phongId: phong.id, mode: "upsert", createMissing: true, rows: rows.slice(i, i + LO) });
        tong.created += r.created;
        tong.updated += r.updated;
        tong.skipped += r.skipped;
        tong.createdPhongBan += r.createdPhongBan ?? 0;
        tong.createdMang += r.createdMang ?? 0;
        tong.notes?.push(...(r.notes ?? []));
        for (const e of r.errors) tong.errors.push({ ...e, row: pt.rows[i + e.row - 1]?.dong ?? e.row });
      }
      setTienDo("Đang đồng bộ về máy…");
      try {
        await clSyncNow();
      } catch {
        /* đồng bộ nền sẽ kéo sau */
      }
      setKq(tong);
      if (tong.errors.length === 0) toast.success(`Đã nhập ${n(tong.created + tong.updated)} hồ sơ.`);
      onImported();
    } catch (e) {
      notifyError(e, "nhập mục lục");
      setBuoc(1);
    } finally {
      setBusy(false);
      setTienDo("");
    }
  };

  const rowsHien = useMemo(() => {
    if (!pt) return [] as DongHoSo[];
    return chiVanDe ? pt.rows.filter((r) => r.loi.length > 0 || r.tg.ketLuan !== "dung") : pt.rows;
  }, [pt, chiVanDe]);
  const rowsTrang = rowsHien.slice((trang - 1) * TRANG, trang * TRANG);

  const cols: Column<DongHoSo>[] = [
    { key: "dong", header: "Dòng", width: 60, align: "right", cell: (r) => r.dong },
    { key: "so_tam", header: "Số tạm", width: 90, cell: (r) => soTam.get(r.dong) },
    { key: "tieu_de", header: "Tiêu đề", className: "max-w-[360px] whitespace-normal", cell: (r) => r.tieuDe },
    { key: "mang", header: "Mảng", className: "text-sm", cell: (r) => (r.mang ? `${r.mang.ma} · ${r.mang.ten}` : "") },
    { key: "goc", header: "Thời gian gốc", className: "text-sm whitespace-pre-line", cell: (r) => r.thoiGianGoc || <span className="text-fg-subtle">trống</span> },
    { key: "bd", header: "Bắt đầu", width: 100, cell: (r) => r.tg.batDau || <span className="text-fg-subtle">—</span> },
    { key: "kt", header: "Kết thúc", width: 100, cell: (r) => r.tg.ketThuc || <span className="text-fg-subtle">—</span> },
    { key: "thbq", header: "THBQ", width: 90, className: "text-sm", cell: (r) => r.thbq ?? <span className="text-danger-700">{r.thbqGoc || "trống"}</span> },
    {
      key: "kl",
      header: "Kết luận",
      width: 96,
      cell: (r) => (r.loi.length ? <Badge tone="err">Lỗi</Badge> : <Badge tone={TONE[r.tg.ketLuan]}>{KET_LUAN_LABEL[r.tg.ketLuan]}</Badge>),
    },
    { key: "ghi", header: "Ghi chú kiểm tra", className: "max-w-[320px] whitespace-normal text-sm", cell: (r) => [...r.loi, ...r.tg.ghiChu].join("; ") },
  ];

  const taiDanhDau = async () => {
    if (!pt) return;
    try {
      const t = await xuatTepDanhDau(pt);
      toast.success(`Đã xuất tệp đánh dấu: điền ${n(t.daDien)} ô, cần xem ${n(t.canhBao)}, không điền được ${n(t.khongDien)}, lỗi ${n(t.loi)}.`);
    } catch (e) {
      notifyError(e, "xuất tệp đánh dấu");
    }
  };

  const vanDeChan = pt ? pt.vanDe.filter((v) => v.muc === "loi") : [];
  // Mọi lỗi chặn nhập trong một bảng: lỗi dòng nhóm (không phải dòng hồ sơ nên không có trong bảng dưới) + lỗi dòng hồ sơ.
  const dsLoi = useMemo(() => {
    if (!pt) return [] as { dong: number; den: number; so: number; laHoSo: boolean; loai: string; tieuDe: string; noiDung: string }[];
    const tieuDeDong = new Map(pt.rows.map((r) => [r.dong, r.tieuDe]));
    const tat = [
      ...pt.vanDe.filter((v) => v.muc === "loi").map((v) => ({ dong: v.dong, den: v.dong, so: 1, laHoSo: false, loai: v.dong ? "Dòng nhóm" : "Cả tệp", tieuDe: tieuDeDong.get(v.dong) ?? "", noiDung: v.noiDung })),
      ...pt.rows.filter((r) => r.loi.length > 0).map((r) => ({ dong: r.dong, den: r.dong, so: 1, laHoSo: true, loai: `Hồ sơ số ${r.hoSoSo}`, tieuDe: r.tieuDe, noiDung: r.loi.join("; ") })),
    ].sort((a, b) => a.dong - b.dong);
    // Gộp các dòng hồ sơ liền nhau cùng một lỗi (vd. thiếu "I." ở dòng phòng ban → cả trăm hồ sơ phía dưới
    // báo "chưa có dòng phòng ban") để lỗi gốc ở dòng nhóm không bị chìm.
    const out: typeof tat = [];
    for (const x of tat) {
      const t = out[out.length - 1];
      if (t && t.laHoSo && x.laHoSo && t.noiDung === x.noiDung) {
        t.den = x.dong;
        t.so++;
        t.loai = `${n(t.so)} hồ sơ`;
        t.tieuDe = "";
        continue;
      }
      out.push({ ...x });
    }
    return out;
  }, [pt]);
  const colsLoi: Column<(typeof dsLoi)[number]>[] = [
    { key: "dong", header: "Dòng Excel", width: 90, align: "right", cell: (r) => (!r.dong ? "—" : r.den > r.dong ? `${r.dong}–${r.den}` : r.dong) },
    { key: "loai", header: "Loại", width: 110, className: "text-sm", cell: (r) => r.loai },
    { key: "tieu_de", header: "Tiêu đề", className: "max-w-[300px] whitespace-normal text-sm", cell: (r) => r.tieuDe },
    { key: "loi", header: "Lỗi phải sửa", className: "whitespace-normal text-sm text-danger-700", cell: (r) => r.noiDung },
  ];
  const soDongLoi = pt ? pt.rows.filter((r) => r.loi.length > 0).length : 0;

  const footer = (
    <>
      {buoc === 1 && pt && (
        <>
          <Button icon="tepExcel" disabled={busy} onClick={() => void taiDanhDau()}>
            Tải tệp gốc đã đánh dấu
          </Button>
          <Button icon="tepExcel" disabled={busy} onClick={() => void xuatTepDaChuyen(pt, tienTo, maPhong, cat.thbq).catch((e) => notifyError(e, "tải tệp đã chuyển"))}>
            Tải tệp đã chuyển
          </Button>
        </>
      )}
      <span className="flex-1" />
      <Button onClick={onClose} disabled={busy}>
        {kq ? "Đóng" : "Huỷ"}
      </Button>
      {buoc === 1 && (
        <>
          <Button disabled={busy} onClick={() => setBuoc(0)}>
            Chọn tệp khác
          </Button>
          <Button variant="primary" icon="nhapTep" disabled={busy || chan || offline || !projectId} onClick={() => void nhap()}>
            Nhập vào hệ thống
          </Button>
        </>
      )}
    </>
  );

  return (
    <Dialog open onOpenChange={(o) => !o && !busy && onClose()} closable={!busy} size="xl" title="Nhập mục lục hồ sơ" description={`Phông ${maPhong} — ${str(phong, "ten")}`} footer={footer}>
      <Stepper steps={BUOC} current={buoc} className="mb-4" />

      {buoc === 0 && (
        <div className="space-y-4">
          <p className="text-base">
            Chọn tệp mục lục hồ sơ của phông (cột Hộp số · Hồ sơ số · Tiêu đề hồ sơ · Thời gian BĐ - KT · Số tờ · THBQ · Ghi chú). Dòng <b>I. …</b> trong cột Tiêu đề là phòng ban, dòng <b>1. …</b> là mảng công việc.
          </p>
          <div className="flex items-center gap-3">
            <Button variant="primary" icon="moThuMuc" loading={busy} onClick={() => fileRef.current?.click()}>
              Chọn tệp
            </Button>
            <input
              ref={fileRef}
              type="file"
              accept=".xlsx,.xls,.xlsm"
              hidden
              aria-hidden
              onChange={(e) => {
                const f = e.target.files?.[0];
                e.target.value = "";
                if (f) void docTep(f);
              }}
            />
          </div>
          {loiDoc !== null && <Banner tone="danger">Không đọc được tệp. Kiểm tra tệp có phải Excel (.xlsx) và không bị khoá.</Banner>}
          <Help title="Hệ thống làm gì với tệp">
            <ul className="list-disc space-y-1 pl-5 text-sm">
              <li>Tự nhận phòng ban (I., II.…) và mảng công việc (1., 2.…); chưa có trong Danh mục thì tạo mới với mã 01, 02… theo thứ tự.</li>
              <li>Kiểm cột thời gian: ô trống, sai định dạng, ngày đầu sau ngày cuối, năm không khớp năm ghi ở tiêu đề. Tự điền: chỉ năm → cả năm, chỉ tháng → cả tháng, một ngày → lặp lại, mốc sai → suy từ năm trong ô hoặc tiêu đề.</li>
              <li>Số tạm = tiền tố theo THBQ + hồ sơ số (VV.001, 20N.001…) để các tệp của cùng phông không trùng nhau.</li>
              <li>Hộp số không nhập (gán ở bước Hệ thống hoá), chỉ giữ ở sheet Kiểm tra của tệp đã chuyển.</li>
              <li>Dòng nhóm không bắt đầu bằng I. hoặc 1. sẽ báo lỗi — sửa tệp rồi chọn lại.</li>
              <li>
                "Tải tệp gốc đã đánh dấu" trả lại chính tệp của anh (tên_DADIEN.xlsx) với cột thời gian đã điền và ô được tô màu + ghi chú: xanh = đã điền (Cũ → Mới), cam = cần xem lại, vàng =
                không điền được, đỏ nhạt = lỗi phải sửa trước khi nhập.
              </li>
            </ul>
          </Help>
        </div>
      )}

      {buoc === 1 && pt && (
        <div className="space-y-4">
          <div className="grid grid-cols-3 gap-3 sm:grid-cols-6">
            <KpiTile label="Phòng ban" value={pt.phongBans.length} />
            <KpiTile label="Mảng" value={pt.phongBans.reduce((a, p) => a + p.mangs.length, 0)} />
            <KpiTile label="Hồ sơ" value={pt.rows.length} tone="info" />
            <KpiTile label="Thời gian đã điền" value={pt.soDaDien} tone="info" />
            <KpiTile label="Cảnh báo" value={pt.soCanhBao} tone={pt.soCanhBao ? "warn" : undefined} />
            <KpiTile label="Lỗi chặn nhập" value={pt.soLoi} tone={pt.soLoi ? "err" : "ok"} />
          </div>

          {dsLoi.length > 0 && (
            <div className="space-y-2">
              <Banner tone="danger" title={`Lỗi phải sửa trước khi nhập: ${n(vanDeChan.length)} ở dòng nhóm, ${n(soDongLoi)} dòng hồ sơ`}>
                Sửa trong Excel theo bảng dưới, hoặc bấm "Tải tệp gốc đã đánh dấu" — các ô lỗi được tô đỏ kèm ghi chú. Sửa xong lưu, đóng Excel rồi bấm "Chọn tệp khác".
              </Banner>
              <Table columns={colsLoi} rows={dsLoi} getRowId={(r) => `${r.dong}-${r.loai}-${r.noiDung}`} dense maxHeight={240} rowClassName={() => "bg-danger-50"} />
            </div>
          )}
          {tenPhongLech && (
            <Banner tone="warning">
              Tệp ghi phông "{pt.tenPhongTrongTep}", phông đang chọn là "{str(phong, "ten")}". Kiểm tra lại trước khi nhập.
            </Banner>
          )}
          {offline && !chan && <Banner tone="warning">Đang mất kết nối máy chủ. Tải tệp đã chuyển để nhập sau, hoặc chờ có mạng.</Banner>}

          <div className="grid gap-4 sm:grid-cols-[1fr_260px]">
            <div className="rounded-md border border-border-subtle bg-gray-25 p-3">
              <div className="mb-1 text-sm font-medium text-fg-muted">Phòng ban và mảng công việc nhận được</div>
              {pt.phongBans.length === 0 ? (
                <div className="text-sm text-fg-subtle">Chưa nhận được phòng ban nào (cần dòng "I. …" trong cột Tiêu đề).</div>
              ) : (
                <ul className="space-y-1 text-sm">
                  {pt.phongBans.map((p) => (
                    <li key={p.ma + p.dong}>
                      <b>
                        {p.ma} · {p.ten}
                      </b>{" "}
                      <span className="text-fg-subtle">({n(p.soHoSo)} hồ sơ, dòng {p.dong})</span>
                      {p.mangs.length > 0 && (
                        <ul className="ml-4 list-disc space-y-0.5 pl-2">
                          {p.mangs.map((m) => (
                            <li key={m.ma + m.dong}>
                              {m.ma} · {m.ten} <span className="text-fg-subtle">({n(m.soHoSo)})</span>
                            </li>
                          ))}
                        </ul>
                      )}
                    </li>
                  ))}
                </ul>
              )}
            </div>
            <div className="space-y-3">
              <Field label="Tiền tố số tạm" hint={`Gợi ý theo THBQ nhiều nhất trong tệp (${pt.thbqTep || "—"}). Số tạm: ${soTam.get(pt.rows[0]?.dong ?? 0) ?? "—"}`}>
                <Input value={tienTo} onChange={(e) => setTienTo(e.target.value.toUpperCase())} placeholder="VV" />
              </Field>
              <Checkbox
                checked={chiVanDe}
                onChange={(v) => {
                  setChiVanDe(v);
                  setTrang(1);
                }}
                label="Chỉ dòng có vấn đề"
              />
            </div>
          </div>

          <Table
            columns={cols}
            rows={rowsTrang}
            getRowId={(r) => String(r.dong)}
            dense
            maxHeight={380}
            rowClassName={(r) => (r.loi.length ? "bg-danger-50" : r.tg.ketLuan === "canh_bao" ? "bg-warning-50" : r.tg.ketLuan === "loi" ? "bg-danger-50" : undefined)}
            pagination={{ page: trang, pageSize: TRANG, total: rowsHien.length, onChange: setTrang }}
            state={{ empty: { title: chiVanDe ? "Không có dòng nào có vấn đề" : "Tệp không có dòng hồ sơ" } }}
          />
        </div>
      )}

      {buoc === 2 && (
        <div className="space-y-4">
          {busy && <Banner tone="info">{tienDo || "Đang nhập…"}</Banner>}
          {kq && (
            <>
              <Banner tone={kq.errors.length ? "warning" : "success"} detail={kq.errors.length ? kq.errors.map((e) => `Dòng ${e.row}: ${e.field ? e.field + ": " : ""}${e.message}`).join("\n") : undefined}>
                Đã thêm {n(kq.created)}, cập nhật {n(kq.updated)}, bỏ qua {n(kq.skipped)} hồ sơ
                {kq.createdPhongBan || kq.createdMang ? ` · tạo mới ${n(kq.createdPhongBan)} phòng ban, ${n(kq.createdMang)} mảng` : ""}
                {kq.errors.length ? ` · ${n(kq.errors.length)} dòng lỗi (xem chi tiết)` : "."}
              </Banner>
              {!!kq.notes?.length && (
                <Banner tone="info" detail={kq.notes.join("\n")}>
                  {n(kq.notes.length)} phòng ban / mảng trùng số thứ tự với mục đã có trong phông nên được đặt mã khác (xem chi tiết).
                </Banner>
              )}
              {pt && (
                <div className="flex flex-wrap gap-2">
                  <Button icon="tepExcel" onClick={() => void taiDanhDau()}>
                    Tải tệp gốc đã đánh dấu
                  </Button>
                  <Button icon="tepExcel" onClick={() => void xuatTepDaChuyen(pt, tienTo, maPhong, cat.thbq).catch((e) => notifyError(e, "tải tệp đã chuyển"))}>
                    Tải tệp đã chuyển
                  </Button>
                </div>
              )}
            </>
          )}
        </div>
      )}
    </Dialog>
  );
}
