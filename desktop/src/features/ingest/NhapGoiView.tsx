// Copyright (c) 2026 DigiText. All rights reserved.
// Nhập gói vào Hệ thống (bản Lưu trữ GĐ3, 4/11/2026 — docs/KE-HOACH-BAN-LUU-TRU.md): chọn tệp / thư mục gói AIP-SIP,
// tải lần lượt lên máy chủ cơ quan; máy chủ kiểm gói (cùng bộ kiểm của Kiểm tra gói tin), dựng mục lục hồ sơ – văn bản,
// lưu tài liệu và zip gốc vào kho. Bộ hồ sơ hàng trăm GB giao bằng ổ cứng thì dùng lệnh máy chủ `digitext-api nhap-goi`.

import { useCallback, useMemo, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { V460 } from "@/state";
import { useProject } from "@/state/project";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { ingestBatchCreate, ingestBatches, ingestBatchFinish, ingestBatchItems, ingestDiscover, ingestUpload, type GoiTimThay, type GoiTrongLo, type KetQuaGoi, type KetQuaNhap, type LoNhap, type TienDoTai } from "@/api/ingest";
import { Badge, Banner, Button, Card, Checkbox, EmptyState, ErrorBanner, Field, Help, Input, KpiRow, KpiTile, PageHeader, Table, Toolbar, toast, type Column } from "@/ui";

const fmt = (n: number) => n.toLocaleString("vi-VN");
const mb = (b: number) => (b >= 1024 ** 3 ? `${(b / 1024 ** 3).toLocaleString("vi-VN", { maximumFractionDigits: 1 })} GB` : `${(b / 1024 ** 2).toLocaleString("vi-VN", { maximumFractionDigits: 1 })} MB`);
const KQ: Record<KetQuaNhap, { text: string; tone: "ok" | "gray" | "err" | "warn" }> = {
  nhap: { text: "Đã nhập", tone: "ok" },
  da_co: { text: "Đã có", tone: "gray" },
  tu_choi: { text: "Từ chối", tone: "err" },
  loi: { text: "Lỗi", tone: "warn" },
};

type Dong = GoiTimThay & { tienDo?: TienDoTai; kq?: KetQuaGoi; loi?: string };

function ChiTietLo({ id }: { id: string }) {
  const loader = useCallback(() => ingestBatchItems(id), [id]);
  const { data, loading, error, reload } = V41t(loader, { topics: ["packages"], errorText: () => "Không tải được các gói của lần nhập." });
  const cols: Column<GoiTrongLo>[] = [
    { key: "ketQua", header: "Kết quả", width: 100, cell: (g) => <Badge tone={KQ[g.ketQua].tone}>{KQ[g.ketQua].text}</Badge> },
    { key: "tenTep", header: "Gói", className: "whitespace-normal break-all" },
    { key: "maHoSo", header: "Hồ sơ", className: "whitespace-normal", cell: (g) => [g.maHoSo, g.tieuDe].filter(Boolean).join(" — ") || "—" },
    { key: "soVanBan", header: "Văn bản", width: 80, align: "right" },
    { key: "ketLuanKiem", header: "Kiểm gói", width: 150, cell: (g) => g.ketLuanKiem ?? "—" },
    { key: "thongBao", header: "Ghi chú", className: "whitespace-normal", cell: (g) => <span className="text-sm text-fg-subtle">{g.thongBao ?? ""}</span> },
  ];
  return (
    <div className="bg-surface-sunken p-3">
      <Table
        columns={cols}
        rows={data ?? []}
        getRowId={(g) => g.id}
        dense
        maxHeight={360}
        renderExpanded={(g) =>
          g.chiTiet && g.chiTiet.length > 0 ? (
            <ul className="space-y-1 p-3 text-sm">
              {g.chiTiet.map((f, i) => (
                <li key={i}>
                  <Badge tone={f.level === "err" ? "err" : "warn"}>{f.level === "err" ? "Lỗi" : "Cảnh báo"}</Badge> <span className="text-fg-subtle">{f.group}:</span> {f.message}
                </li>
              ))}
            </ul>
          ) : (
            <p className="p-3 text-sm text-fg-subtle">Không có lỗi hay cảnh báo.</p>
          )
        }
        state={{ loading: loading && !data, error: data ? undefined : error, onRetry: reload, empty: { title: "Lần nhập chưa có gói nào" } }}
      />
    </div>
  );
}

export default function NhapGoiView() {
  const { session } = V460();
  const proj = useProject();
  const projectId = session?.projectId ?? proj.projectId ?? "";
  const dungDuoc = isTauri() || import.meta.env.DEV;

  const [dongs, setDongs] = useState<Dong[]>([]);
  const [nhanLoi, setNhanLoi] = useState(false);
  const [ghiChu, setGhiChu] = useState("");
  const [dangChay, setDangChay] = useState(false);
  const [err, setErr] = useState<unknown>(null);
  const [moLo, setMoLo] = useState<string | null>(null);
  const dung = useRef(false);

  const taiLo = useCallback(async () => (projectId ? ingestBatches(projectId) : ([] as LoNhap[])), [projectId]);
  const lo = V41t(taiLo, { topics: ["packages"], enabled: dungDuoc && !!projectId, errorText: () => "Không tải được các lần nhập trước." });

  const chon = async (thuMuc: boolean) => {
    const r = thuMuc
      ? await open({ directory: true, multiple: false, title: "Chọn thư mục chứa gói (quét cả thư mục con)" })
      : await open({ multiple: true, title: "Chọn tệp gói (.zip)", filters: [{ name: "Gói tin", extensions: ["zip"] }] });
    const paths = Array.isArray(r) ? r : typeof r === "string" ? [r] : [];
    if (!paths.length) return;
    setErr(null);
    try {
      const found = await ingestDiscover(paths);
      if (!found.length) toast.warning("Không thấy gói nào (tệp .zip hoặc thư mục có METS.xml).");
      setDongs(found);
    } catch (e) {
      setErr(e);
    }
  };

  const capNhat = (path: string, patch: Partial<Dong>) => setDongs((ds) => ds.map((d) => (d.path === path ? { ...d, ...patch } : d)));

  const nhap = async () => {
    if (!projectId) return;
    const can = dongs.filter((d) => !d.laThuMuc && !d.kq);
    if (!can.length) return;
    setErr(null);
    setDangChay(true);
    dung.current = false;
    let batch = "";
    try {
      batch = await ingestBatchCreate(projectId, ghiChu.trim() || undefined);
      for (const d of can) {
        if (dung.current) break;
        try {
          const kq = await ingestUpload(batch, d.path, nhanLoi, (t) => capNhat(d.path, { tienDo: t }));
          capNhat(d.path, { kq, tienDo: undefined, loi: undefined });
        } catch (e) {
          capNhat(d.path, { tienDo: undefined, loi: e instanceof Error ? e.message : String(e) });
        }
      }
    } catch (e) {
      setErr(e);
    } finally {
      if (batch) await ingestBatchFinish(batch).catch(() => undefined);
      setDangChay(false);
      void lo.reload();
    }
  };

  const tk = useMemo(() => {
    const xong = dongs.filter((d) => d.kq);
    return {
      goi: dongs.length,
      nen: dongs.filter((d) => d.laThuMuc).length,
      dung: dongs.filter((d) => !d.laThuMuc).reduce((s, d) => s + d.size, 0),
      nhap: xong.filter((d) => d.kq?.ketQua === "nhap").length,
      tuChoi: xong.filter((d) => d.kq?.ketQua === "tu_choi" || d.kq?.ketQua === "loi").length + dongs.filter((d) => d.loi).length,
      vanBan: xong.reduce((s, d) => s + (d.kq?.soVanBan ?? 0), 0),
      chuaKy: xong.reduce((s, d) => s + (d.kq?.soChuaKy ?? 0), 0),
      daXong: xong.length,
    };
  }, [dongs]);

  const cols: Column<Dong>[] = [
    { key: "name", header: "Gói", className: "whitespace-normal break-all" },
    { key: "size", header: "Dung lượng", width: 110, align: "right", cell: (d) => (d.laThuMuc ? "—" : mb(d.size)) },
    {
      key: "trangThai",
      header: "Trạng thái",
      width: 220,
      cell: (d) => {
        if (d.laThuMuc) return <span className="text-sm text-warning-700">Thư mục chưa nén — nén thành .zip</span>;
        if (d.kq) return <Badge tone={KQ[d.kq.ketQua].tone}>{KQ[d.kq.ketQua].text}</Badge>;
        if (d.loi) return <Badge tone="err">Lỗi tải lên</Badge>;
        if (d.tienDo?.buoc === "nhap") return <span className="text-sm text-run-700">Máy chủ đang kiểm và nhập…</span>;
        if (d.tienDo) return <span className="text-sm text-run-700">Đang tải {Math.floor((d.tienDo.daGui / Math.max(1, d.tienDo.tong)) * 100)}%</span>;
        return <span className="text-sm text-fg-subtle">Chờ</span>;
      },
    },
    { key: "hoSo", header: "Hồ sơ", className: "whitespace-normal", cell: (d) => (d.kq ? [d.kq.maHoSo, d.kq.tieuDe].filter(Boolean).join(" — ") : "") },
    { key: "vb", header: "Văn bản", width: 80, align: "right", cell: (d) => (d.kq ? fmt(d.kq.soVanBan) : "") },
    { key: "tb", header: "Ghi chú", className: "whitespace-normal", cell: (d) => <span className="text-sm text-fg-subtle">{d.loi ?? d.kq?.thongBao ?? ""}</span> },
  ];

  const loCols: Column<LoNhap>[] = [
    { key: "createdAt", header: "Thời gian", width: 150, cell: (l) => new Date(l.createdAt).toLocaleString("vi-VN", { dateStyle: "short", timeStyle: "short" }) },
    { key: "nguon", header: "Cách nhập", width: 120, cell: (l) => (l.nguon === "lenh" ? "Lệnh máy chủ" : "Tải lên") },
    { key: "nguoiTao", header: "Người nhập", width: 160, cell: (l) => l.nguoiTao ?? "—" },
    { key: "ketQua", header: "Kết quả", className: "whitespace-normal", cell: (l) => `${fmt(l.nhap)} nhập · ${fmt(l.daCo)} đã có · ${fmt(l.tuChoi)} từ chối · ${fmt(l.loi)} lỗi` },
    { key: "soVanBan", header: "Văn bản", width: 90, align: "right", cell: (l) => fmt(l.soVanBan) },
    { key: "soChuaKy", header: "Chưa ký", width: 80, align: "right", cell: (l) => (l.soChuaKy ? <span className="text-warning-700">{fmt(l.soChuaKy)}</span> : "0") },
    { key: "ghiChu", header: "Ghi chú", className: "whitespace-normal", cell: (l) => l.ghiChu ?? "" },
  ];

  if (!projectId) {
    return (
      <section className="view active">
        <PageHeader title="Nhập gói vào Hệ thống" />
        <Card padding="md">
          <EmptyState icon="duAn" title="Chọn dự án ở thanh trên" description="Gói được nhập vào kho của dự án đang chọn." />
        </Card>
      </section>
    );
  }

  const conCho = dongs.filter((d) => !d.laThuMuc && !d.kq).length;
  return (
    <section className="view active space-y-4">
      <PageHeader title="Nhập gói vào Hệ thống" description="Tiếp nhận gói AIP/SIP đã đóng theo TT 05/2025/TT-BNV: máy chủ kiểm gói, dựng mục lục hồ sơ – văn bản và lưu tài liệu vào kho." />
      {!isTauri() && <Banner tone="info">Bản xem trước trên trình duyệt — chạy trong ứng dụng desktop để nhập gói thật.</Banner>}
      <Card title="Gói cần nhập" description="Chọn tệp .zip, hoặc một thư mục (quét cả thư mục con). Gói đã nhập trước đó sẽ được báo là đã có, không nhập trùng.">
        <Toolbar>
          <Button icon="goi" onClick={() => void chon(false)} disabled={dangChay}>
            Chọn tệp gói
          </Button>
          <Button icon="thuMuc" onClick={() => void chon(true)} disabled={dangChay}>
            Chọn thư mục
          </Button>
          <Button variant="primary" icon="nhapKho" loading={dangChay} disabled={conCho === 0 || dangChay} onClick={() => void nhap()}>
            {conCho > 0 ? `Nhập ${fmt(conCho)} gói` : "Nhập"}
          </Button>
          {dangChay && (
            <Button variant="dangerOutline" onClick={() => (dung.current = true)}>
              Dừng sau gói đang nhập
            </Button>
          )}
        </Toolbar>
        <div className="mt-3 grid gap-3 md:grid-cols-[1fr_auto] md:items-end">
          <Field label="Ghi chú cho lần nhập" hint="Ví dụ: số biên bản bàn giao, đợt giao.">
            <Input value={ghiChu} onChange={(e) => setGhiChu(e.target.value)} disabled={dangChay} placeholder="Bàn giao lần 1 theo biên bản số …" />
          </Field>
          <Checkbox checked={nhanLoi} onChange={setNhanLoi} disabled={dangChay} text="Nhận cả gói kiểm không đạt (trừ chữ ký số hỏng)" />
        </div>
        {nhanLoi && (
          <Banner tone="warning" className="mt-3">
            Gói không đạt TT05 (thiếu trường, sai mã băm, không phải PDF/A…) vẫn được nhập. Chỉ bật khi cơ quan đã chấp nhận gói; gói có chữ ký số bị sửa sau khi ký luôn bị từ chối.
          </Banner>
        )}
      </Card>

      <ErrorBanner error={err} />

      {dongs.length > 0 && (
        <>
          <KpiRow className="grid grid-cols-2 gap-3 md:grid-cols-5">
            <KpiTile label="Gói đã chọn" value={fmt(tk.goi)} sub={mb(tk.dung)} />
            <KpiTile label="Đã nhập" value={fmt(tk.nhap)} tone={tk.nhap ? "ok" : undefined} sub={`${fmt(tk.daXong)}/${fmt(tk.goi - tk.nen)} gói đã xử lý`} />
            <KpiTile label="Từ chối / lỗi" value={fmt(tk.tuChoi)} tone={tk.tuChoi ? "err" : undefined} />
            <KpiTile label="Văn bản" value={fmt(tk.vanBan)} />
            <KpiTile label="Tệp chưa ký số" value={fmt(tk.chuaKy)} tone={tk.chuaKy ? "warn" : undefined} />
          </KpiRow>
          {tk.nen > 0 && <Banner tone="warning">{fmt(tk.nen)} thư mục gói đã giải nén: nén lại thành .zip để tải lên, hoặc nhập trên máy chủ bằng lệnh nhap-goi.</Banner>}
          <Card padding="none" title="Danh sách gói">
            <Table columns={cols} rows={dongs} getRowId={(d) => d.path} dense maxHeight={480} />
          </Card>
        </>
      )}

      <Card padding="none" title="Các lần nhập" description="Bấm một lần nhập để xem từng gói và lý do từ chối.">
        <Table
          columns={loCols}
          rows={lo.data ?? []}
          getRowId={(l) => l.id}
          dense
          maxHeight={420}
          onRowClick={(l) => setMoLo((cu) => (cu === l.id ? null : l.id))}
          rowClassName={(l) => (l.id === moLo ? "bg-brand-50" : undefined)}
          state={{ loading: lo.loading && !lo.data, error: lo.data ? undefined : lo.error, onRetry: lo.reload, empty: { title: "Chưa nhập gói nào" } }}
        />
        {moLo && <ChiTietLo id={moLo} />}
      </Card>

      <Help title="Điều 9 khoản 7 TT 05/2025/TT-BNV — nhập Hệ thống">
        <p>Mỗi gói được kiểm như Tiện ích › Kiểm tra gói tin (cấu trúc, METS, EAD, PREMIS, PDF/A, chữ ký số). Gói đạt được nhập: hồ sơ và văn bản vào mục lục (phông theo mã phông trong OBJID của gói), tài liệu vào kho, zip gốc lưu không xoá được.</p>
        <p>Tệp chưa ký số vẫn nhận và được đếm ở cột Chưa ký; gói có chữ ký bị sửa sau khi ký luôn bị từ chối. Nhập lại một gói đã nhập không tạo bản trùng.</p>
        <p>Bộ hồ sơ lớn giao bằng ổ cứng: cắm vào máy chủ và chạy <code>digitext-api nhap-goi --du-an &lt;mã&gt; --nguoi &lt;tên đăng nhập&gt; &lt;thư mục&gt;</code> — kết quả hiện ở bảng Các lần nhập.</p>
      </Help>
    </section>
  );
}
