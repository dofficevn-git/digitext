// Copyright (c) 2026 DigiText. All rights reserved.
// Bảo quản và sao lưu — hub 5 tab (0032, 26/11/2026, đủ Điều 29–31): Tổng quan (kiểm toàn vẹn 3 năm, tệp / gói lỗi) ·
// Kế hoạch kiểm tra (Điều 29, có duyệt, danh mục lỗi khắc phục từng dòng) · Sao lưu (lịch, sổ, bản cũ cần xoá, Mẫu 02/03) ·
// Phương tiện (hạn độ bền, Điều 30.4) · Sự cố (Điều 31, Mẫu 04). Xem trước dev: ?view=bk&tab=kehoach.

import { useCallback, useMemo, useState } from "react";
import { V41t } from "@/live/useLive";
import { V4v1, V4v2, type GoiLoi, type V4v0, type V4uz } from "@/api/infoops";
import { bqTongQuan, type TongQuanBaoQuan } from "@/api/baoquan";
import { isTauri } from "@/api/ipc";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { Badge, Banner, Button, Card, KpiRow, KpiTile, notifyError, PageHeader, Table, Tabs, Tip, toast, type Column } from "@/ui";
import KeHoachPanel from "./baoquan/KeHoachPanel";
import SaoLuuPanel from "./baoquan/SaoLuuPanel";
import PhuongTienPanel from "./baoquan/PhuongTienPanel";
import SuCoPanel from "./baoquan/SuCoPanel";
import { fmtN } from "./baoquan/baoQuanFlow";

type Tab = "tongquan" | "kehoach" | "saoluu" | "phuongtien" | "suco";
const TABS: { value: Tab; label: string }[] = [
  { value: "tongquan", label: "Tổng quan" },
  { value: "kehoach", label: "Kế hoạch kiểm tra" },
  { value: "saoluu", label: "Sao lưu" },
  { value: "phuongtien", label: "Phương tiện" },
  { value: "suco", label: "Sự cố" },
];

export default function V4vd() {
  const [tab, setTab] = useState<Tab>(() => {
    const t = import.meta.env.DEV ? new URLSearchParams(window.location.search).get("tab") : null;
    return TABS.some((x) => x.value === t) ? (t as Tab) : "tongquan";
  });
  return (
    <section className="view active">
      <PageHeader
        title="Bảo quản và sao lưu"
        description="Kiểm tra toàn vẹn, sao lưu, phương tiện, sự cố."
        infoTitle="Căn cứ"
        info={
          <>
            <p>Điều 29: kế hoạch kiểm tra hằng năm, mọi tệp được kiểm ít nhất một lần trong 3 năm (băm lại, kiểm PDF/A; tài liệu nhập gói so BLAKE3, zip gói so SHA-256); danh mục tệp lỗi và khắc phục theo Điều 29.4.đ, g; biên bản Mẫu 01.</p>
            <p>Điều 30: sao lưu gia tăng hằng ngày, đầy đủ hằng tháng, đầy đủ 3 năm ra phương tiện riêng; sổ ghi loại, nguồn, phương tiện, kết quả kiểm sau sao lưu, nơi bảo quản (Mẫu 02, 03); phương tiện có hạn độ bền, chuyển đổi trước 1 năm (30.4).</p>
            <p>Điều 31: sự cố được cách ly, phục hồi từ bản sao lưu gần nhất, kiểm tra sau phục hồi, lập biên bản Mẫu 04.</p>
          </>
        }
      />
      <Tabs<Tab> value={tab} onChange={setTab} items={TABS} />
      <div className="mt-3">
        {tab === "tongquan" && <TongQuanPanel onTab={setTab} />}
        {tab === "kehoach" && <KeHoachPanel />}
        {tab === "saoluu" && <SaoLuuPanel />}
        {tab === "phuongtien" && <PhuongTienPanel />}
        {tab === "suco" && <SuCoPanel />}
      </div>
    </section>
  );
}

function TongQuanPanel({ onTab }: { onTab: (t: Tab) => void }) {
  const { session } = V460();
  const canManage = V3z9(session?.permissions, "bk.manage");
  const loader = useCallback(async () => {
    if (!isTauri()) return { report: null as V4v0 | null, bq: null as TongQuanBaoQuan | null };
    const [report, bq] = await Promise.all([V4v1(), bqTongQuan().catch(() => null)]);
    return { report, bq };
  }, []);
  const { data, error, loading, reload } = V41t(loader, { topics: ["ops"], errorText: () => "Không tải được số liệu bảo quản." });
  const report = data?.report ?? null;
  const bq = data?.bq ?? null;
  const [dangKiem, setDangKiem] = useState(false);
  const lichTre = useMemo(() => (bq?.lich ?? []).filter((l) => l.tinhTrang === "tre" || l.tinhTrang === "chua_co"), [bq]);

  const chayKiem = async () => {
    setDangKiem(true);
    try {
      const r = await V4v2();
      const goi = r.goiDaKiem ? `, ${fmtN(r.goiDaKiem)} gói` : "";
      const loi = r.loi + (r.goiLoi ?? 0);
      if (loi > 0) toast.warning(`Đã kiểm ${fmtN(r.daKiem)} tài liệu${goi}, ${fmtN(loi)} lỗi — xem danh mục lỗi ở tab Kế hoạch kiểm tra.`);
      else toast.success(`Đã kiểm ${fmtN(r.daKiem)} tài liệu${goi}, không có lỗi.`);
      reload();
    } catch (e) {
      notifyError(e, "chạy kiểm tra");
    } finally {
      setDangKiem(false);
    }
  };

  const colsLoi: Column<V4uz>[] = [
    { key: "filename", header: "Tên tệp", className: "whitespace-normal" },
    { key: "hoSoRef", header: "Mã hồ sơ", width: 160, cell: (t) => t.hoSoRef ?? "—" },
    { key: "chiTiet", header: "Chi tiết lỗi", className: "whitespace-normal text-danger-700", cell: (t) => t.chiTiet ?? "—" },
    { key: "checkedAt", header: "Thời điểm kiểm", width: 150, cell: (t) => new Date(t.checkedAt).toLocaleString("vi-VN") },
  ];
  const colsGoiLoi: Column<GoiLoi>[] = [
    { key: "label", header: "Gói", className: "whitespace-normal" },
    { key: "objid", header: "OBJID", width: 260, className: "break-all", cell: (g) => g.objid ?? "—" },
    { key: "chiTiet", header: "Chi tiết lỗi", className: "whitespace-normal text-danger-700", cell: (g) => g.chiTiet ?? "—" },
    { key: "checkedAt", header: "Thời điểm kiểm", width: 150, cell: (g) => new Date(g.checkedAt).toLocaleString("vi-VN") },
  ];
  const tyLe = report && report.tongTaiLieu > 0 ? Math.round((report.daKiem3Nam / report.tongTaiLieu) * 100) : 0;
  const dangTai = loading && !data;

  return (
    <div className="space-y-4">
      {error != null && <Banner tone="danger" actions={<Button size="sm" onClick={reload}>Thử lại</Button>}>Không tải được số liệu bảo quản.</Banner>}
      {lichTre.length > 0 && (
        <Banner tone="danger" title={lichTre.length === 1 ? `Lịch sao lưu «${lichTre[0].ten}»: ${lichTre[0].tinhTrang === "chua_co" ? "chưa có bản nào" : `trễ ${fmtN(lichTre[0].treGio)} giờ`}` : `${fmtN(lichTre.length)} lịch sao lưu trễ hoặc chưa có bản`} actions={<Button size="sm" onClick={() => onTab("saoluu")}>Xem sao lưu</Button>}>
          Sao lưu theo lịch (Điều 30.1). Sau khi sao lưu xong, ghi nhận ở tab Sao lưu để tắt cảnh báo.
        </Banner>
      )}
      <Card title="Kiểm tra toàn vẹn" description="Mục tiêu kiểm toàn bộ tài liệu trong 3 năm; máy chủ tự kiểm một lô mỗi ngày và theo kế hoạch đã duyệt." actions={canManage && <Button size="sm" variant="primary" icon="kiemTra" loading={dangKiem} onClick={() => void chayKiem()}>Chạy kiểm tra một lô</Button>}>
        <KpiRow>
          <KpiTile label="Tài liệu đã kiểm trong 3 năm" value={report?.daKiem3Nam ?? 0} sub={report ? `${tyLe}% của ${fmtN(report.tongTaiLieu)} tài liệu` : undefined} loading={dangTai} />
          <KpiTile label="Tệp lỗi (lần kiểm gần nhất)" value={report?.soLoi ?? 0} tone={report && report.soLoi > 0 ? "err" : "ok"} loading={dangTai} />
          {report?.tongGoi ? <KpiTile label="Gói lưu trữ đã kiểm" value={report.goiDaKiem ?? 0} sub={`${fmtN(report.tongGoi)} gói zip trong kho${report.goiSoLoi ? ` · ${fmtN(report.goiSoLoi)} lỗi` : ""}`} tone={report.goiSoLoi ? "err" : "ok"} /> : null}
          {bq && (
            <>
              <KpiTile label="Lỗi đang mở" value={bq.loiMo} sub="chờ khắc phục / kiểm lại" tone={bq.loiMo ? "err" : "ok"} onClick={() => onTab("kehoach")} />
              <KpiTile label="Kế hoạch kiểm tra" value={bq.khDangChay} sub={bq.khChoDuyet ? `${fmtN(bq.khChoDuyet)} chờ duyệt` : "đang kiểm"} tone={bq.khChoDuyet ? "warn" : undefined} onClick={() => onTab("kehoach")} />
              <KpiTile label="Lịch sao lưu trễ" value={bq.lichTre} sub={bq.banCuCanXoa ? `${fmtN(bq.banCuCanXoa)} bản cũ cần xoá` : `${fmtN(bq.lich.length)} lịch`} tone={bq.lichTre ? "err" : "ok"} onClick={() => onTab("saoluu")} />
              <KpiTile label="Phương tiện sắp hết hạn" value={bq.phuongTienSapHet} sub="trong 1 năm" tone={bq.phuongTienSapHet ? "warn" : undefined} onClick={() => onTab("phuongtien")} />
              <KpiTile label="Sự cố đang mở" value={bq.suCoMo} tone={bq.suCoMo ? "err" : "ok"} onClick={() => onTab("suco")} />
            </>
          )}
        </KpiRow>
        <Tip text={`${fmtN(report?.daKiem3Nam ?? 0)} / ${fmtN(report?.tongTaiLieu ?? 0)} tài liệu`}>
          <div className="mt-3 h-2 rounded bg-border-subtle">
            <div className="h-2 rounded bg-accent" style={{ width: `${Math.min(100, tyLe)}%` }} />
          </div>
        </Tip>
        {!bq && data && <p className="mt-2 text-sm text-fg-subtle">Máy chủ chưa áp 0032: chưa có kế hoạch kiểm tra, lịch sao lưu, phương tiện, sự cố.</p>}
      </Card>
      {report && report.tepLoi.length > 0 && (
        <Card tone="danger" padding="none" title={<span className="flex items-center gap-2">Tệp lỗi <Badge tone="err">{fmtN(report.tepLoi.length)}</Badge></span>} description="Lần kiểm gần nhất lỗi. Khắc phục và kiểm lại ở tab Kế hoạch kiểm tra › Danh mục lỗi.">
          <Table columns={colsLoi} rows={report.tepLoi} getRowId={(t) => t.documentId} dense maxHeight={320} />
        </Card>
      )}
      {report?.goiLoi && report.goiLoi.length > 0 && (
        <Card tone="danger" padding="none" title={`Gói lỗi (${fmtN(report.goiLoi.length)})`} description="Zip gói trong kho không còn khớp mã băm SHA-256 lúc nhập hoặc dựng — khôi phục từ bản sao lưu.">
          <Table columns={colsGoiLoi} rows={report.goiLoi} getRowId={(g) => g.packageId} dense maxHeight={320} />
        </Card>
      )}
    </div>
  );
}
