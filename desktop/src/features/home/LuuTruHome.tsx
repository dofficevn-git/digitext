// Copyright (c) 2026 DigiText. All rights reserved.
// Tổng quan của bản Lưu trữ (4/11/2026, docs/KE-HOACH-BAN-LUU-TRU.md): việc chờ của cán bộ lưu trữ (yêu cầu khai thác,
// cung cấp thông tin), tình trạng bảo quản, lối tắt nhóm Lưu trữ số (cả Nhập gói AIP, GĐ3). Số liệu mục lục thêm ở GĐ4.

import { useCallback } from "react";
import { V3z9 } from "@/domain/permissions";
import { V460 } from "@/state";
import { V3zr } from "@/state/sessionCore";
import { V41t } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { V4to, type V4tl } from "@/api/access";
import { V4ut, V4v1, type V4us, type V4v0 } from "@/api/infoops";
import { type V41v as IconName } from "@/components/Icon";
import { Badge, Banner, Button, Card, KpiRow, KpiTile, Table, type Column } from "@/ui";

type Viec = { id: string; loai: "Khai thác" | "Cung cấp thông tin"; noiDung: string; trangThai: string; ngay: string; mo: () => void };

const TT_KHAI_THAC: Record<V4tl["status"], string> = { submitted: "Chờ duyệt", approved: "Đã duyệt", refused: "Từ chối", expired: "Hết hạn" };
const TT_THONG_TIN: Record<V4us["status"], string> = { submitted: "Chờ duyệt", approved: "Đã duyệt", refused: "Từ chối", fulfilled: "Đã trả kết quả" };

export default function LuuTruHome() {
  const { session, goto, gotoKhoGia } = V460();
  const has = (p: Parameters<typeof V3z9>[1]) => V3z9(session?.permissions, p);
  const coKhaiThac = has("access.view");
  const coBaoQuan = has("bk.view");
  const dungDuoc = isTauri() || import.meta.env.DEV;

  const loader = useCallback(async () => {
    const [kt, tt, bq] = await Promise.all([
      coKhaiThac ? V4to() : Promise.resolve([] as V4tl[]),
      coKhaiThac ? V4ut() : Promise.resolve([] as V4us[]),
      coBaoQuan ? V4v1().catch(() => null) : Promise.resolve(null as V4v0 | null),
    ]);
    return { kt, tt, bq };
  }, [coKhaiThac, coBaoQuan]);
  const { data, loading, error, reload } = V41t(loader, {
    topics: ["access", "info", "ops"],
    enabled: dungDuoc,
    errorText: () => "Không lấy được việc chờ xử lý.",
  });

  const kt = data?.kt ?? [];
  const tt = data?.tt ?? [];
  const bq = data?.bq ?? null;
  const ktCho = kt.filter((r) => r.status === "submitted").length;
  const ktHieuLuc = kt.filter((r) => r.status === "approved").length;
  const ttCho = tt.filter((r) => r.status === "submitted" || r.status === "approved").length;
  const dangTai = loading && !data;

  const viec: Viec[] = [
    ...kt.filter((r) => r.status === "submitted").map((r) => ({ id: `kt-${r.id}`, loai: "Khai thác" as const, noiDung: r.mucDich, trangThai: TT_KHAI_THAC[r.status], ngay: r.createdAt, mo: () => goto("access") })),
    ...tt.filter((r) => r.status === "submitted" || r.status === "approved").map((r) => ({ id: `tt-${r.id}`, loai: "Cung cấp thông tin" as const, noiDung: r.chuDe, trangThai: TT_THONG_TIN[r.status], ngay: r.createdAt, mo: () => goto("info") })),
  ].sort((a, b) => b.ngay.localeCompare(a.ngay));

  const cols: Column<Viec>[] = [
    { key: "loai", header: "Loại", width: 160, cell: (v) => <Badge tone={v.loai === "Khai thác" ? "info" : "gray"}>{v.loai}</Badge> },
    { key: "noiDung", header: "Nội dung yêu cầu", cell: (v) => <span className="line-clamp-1">{v.noiDung}</span> },
    { key: "trangThai", header: "Trạng thái", width: 140 },
    { key: "ngay", header: "Ngày gửi", width: 120, cell: (v) => new Date(v.ngay).toLocaleDateString("vi-VN") },
    { key: "mo", header: "", width: 90, align: "right", cell: (v) => <Button size="sm" variant="ghost" onClick={v.mo}>Mở</Button> },
  ];

  type LoiTat = { label: string; icon: IconName; run: () => void; show: boolean };
  const loiTat = ([
    { label: "Nhập gói AIP", icon: "nhapKho", run: () => goto("nhapgoi"), show: has("submission.approve") },
    { label: "Tra cứu tài liệu", icon: "tim", run: () => goto("tracuu"), show: has("access.request") || has("access.review") },
    { label: "Mục lục hồ sơ", icon: "danhmuc", run: () => gotoKhoGia("khaithac"), show: has("dossier.read") || has("catalog.read") },
    { label: "Khai thác", icon: "luuTruSo", run: () => goto("access"), show: coKhaiThac },
    { label: "Cung cấp thông tin", icon: "thongTin", run: () => goto("info"), show: coKhaiThac },
    { label: "Bảo quản và sao lưu", icon: "baoquan", run: () => goto("bk"), show: coBaoQuan },
    { label: "Tiêu huỷ", icon: "huy", run: () => goto("disposal"), show: has("disposal.view") },
  ] satisfies LoiTat[]).filter((l) => l.show);

  const gio = new Date().getHours();
  const chao = gio < 11 ? "Chào buổi sáng" : gio < 14 ? "Chào buổi trưa" : gio < 18 ? "Chào buổi chiều" : "Chào buổi tối";
  const offlineDays = session ? V3zr(session.offlineExpiry) : 0;

  return (
    <section className="view active">
      <header className="mb-4">
        <h1 className="m-0 text-xl font-semibold text-fg-strong">
          {chao}, {session?.displayName ?? ""}
        </h1>
        <p className="mt-0.5 text-sm text-fg-subtle">{session?.mode === "offline" ? `Ngoại tuyến, còn ${offlineDays} ngày` : "Đã kết nối máy chủ lưu trữ"}</p>
        {loiTat.length > 0 && (
          <nav aria-label="Lối tắt" className="mt-3 flex flex-wrap gap-2">
            {loiTat.map((l) => (
              <Button key={l.label} size="sm" icon={l.icon} onClick={l.run}>
                {l.label}
              </Button>
            ))}
          </nav>
        )}
      </header>
      {session?.mode === "offline" && (
        <Banner tone="warning" className="mb-4">
          Đang ngoại tuyến: không tra cứu và khai thác được trên máy chủ lưu trữ. Kết nối mạng rồi đăng nhập lại.
        </Banner>
      )}
      {coKhaiThac && (
        <KpiRow className="mb-3 grid grid-cols-2 gap-3 md:grid-cols-4">
          <KpiTile icon="luuTruSo" label="Khai thác chờ duyệt" value={ktCho} tone={ktCho ? "warn" : undefined} loading={dangTai} error={data ? undefined : error} onClick={() => goto("access")} />
          <KpiTile icon="duyet" label="Khai thác đang hiệu lực" value={ktHieuLuc} loading={dangTai} onClick={() => goto("access")} />
          <KpiTile icon="thongTin" label="Cung cấp thông tin" value={ttCho} sub="yêu cầu đang xử lý" tone={ttCho ? "warn" : undefined} loading={dangTai} onClick={() => goto("info")} />
          {coBaoQuan && (
            <KpiTile
              icon="baoquan"
              label="Tài liệu lỗi toàn vẹn"
              value={bq?.soLoi ?? 0}
              sub={bq ? `${bq.daKiem3Nam.toLocaleString("vi-VN")}/${bq.tongTaiLieu.toLocaleString("vi-VN")} đã kiểm trong 3 năm` : undefined}
              tone={bq?.soLoi ? "err" : bq ? "ok" : undefined}
              loading={dangTai}
              onClick={() => goto("bk")}
            />
          )}
        </KpiRow>
      )}
      {coKhaiThac && (
        <Card title="Việc chờ xử lý" padding="none">
          <Table
            columns={cols}
            rows={viec}
            getRowId={(v) => v.id}
            dense
            maxHeight={420}
            state={{ loading: dangTai, error: data ? undefined : error, onRetry: reload, empty: { title: "Không có yêu cầu nào đang chờ" } }}
          />
        </Card>
      )}
    </section>
  );
}
