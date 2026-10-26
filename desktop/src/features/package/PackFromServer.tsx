// Copyright (c) 2026 DigiText. All rights reserved.
// Đóng gói › Từ hồ sơ (26/10/2026): phông / hộp / hồ sơ lấy từ TÀI LIỆU ĐÃ LÊN MÁY CHỦ của DỰ ÁN ĐANG CHỌN
// (tầng 1 của đường dẫn = phông, đúng quy ước "mỗi thư mục đẩy lên là một phông"). Trước đây tab này dùng danh mục
// hồ sơ lập trên máy quản trị (bảng cục bộ, không phân dự án) nên hiện lẫn phông của dự án khác và rỗng với dự án
// đẩy theo thư mục. Mỗi hồ sơ chọn → một gói trên máy chủ (gói mức tài liệu: mỗi tài liệu một gói).

import { useEffect, useMemo, useState } from "react";
import { V41t } from "@/live/useLive";
import { isTauri } from "@/api/ipc";
import { docsForGroups, docsReviewSummary, type ReviewSummaryRow, type V4ej } from "@/api/docs";
import { reviewKey, reviewKeyOf } from "@/features/dossier/reviewScope";
import { V4rv } from "@/api/packaging";
import { useProject } from "@/state/project";
import { V4sk, V4sn, V4su, type V4sj, type V4sm } from "./packageWire";
import { Badge, Button, Card, Dialog, EmptyState, ErrorBanner, Field, FormGrid, IconButton, Input, notifyError, Select, Table, Tip, toast, type Column, errorText } from "@/ui";

const n = (v: number) => v.toLocaleString("vi-VN");
const so = (s: string | null | undefined): string => {
  const m = /(\d+)/.exec(s ?? "");
  return m ? m[1] : (s ?? "").trim();
};
const CHUA_PHAN_LOAI = "Chưa phân loại";
const KINDS: V4sj[] = ["aip_hoso", "aip_tailieu", "sip_hoso", "sip_tailieu", "dip"];
const THBQ: { value: string; label: string }[] = [
  { value: "01", label: "Vĩnh viễn" },
  { value: "02", label: "70 năm" },
  { value: "03", label: "50 năm" },
  { value: "04", label: "30 năm" },
  { value: "05", label: "20 năm" },
  { value: "06", label: "10 năm" },
  { value: "07", label: "Khác" },
];
function nho(key: string): string {
  try {
    return localStorage.getItem(key) ?? "";
  } catch {
    return "";
  }
}
function luu(key: string, v: string) {
  try {
    localStorage.setItem(key, v);
  } catch {
    /* tiện ích nhớ */
  }
}
/** Mã phông gợi ý từ tên thư mục: bỏ dấu, giữ chữ số và chữ cái, nối bằng dấu chấm — người dùng sửa được. */
function goiYMaPhong(ten: string): string {
  return ten
    .normalize("NFD")
    .replace(/[̀-ͯ]/g, "")
    .replace(/đ/g, "d")
    .replace(/Đ/g, "D")
    .toUpperCase()
    .replace(/[^A-Z0-9]+/g, ".")
    .replace(/^\.+|\.+$/g, "")
    .slice(0, 40);
}

export default function PackFromServer() {
  const proj = useProject();
  const projectId = proj.projectId ?? "";
  const { data, error, loading, refreshing, reload } = V41t(() => docsReviewSummary({ projectId }), { topics: ["docs", "dossier", "packages"], pollMs: 60_000, enabled: (isTauri() || import.meta.env.DEV) && !!projectId });
  const rows = useMemo(() => (data ?? []).filter((r) => r.projectId === projectId), [data, projectId]);
  const [phong, setPhong] = useState("");
  const [hop, setHop] = useState("");
  const [q, setQ] = useState("");
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [dialog, setDialog] = useState(false);
  const collator = useMemo(() => new Intl.Collator("vi", { numeric: true }), []);

  // Đổi dự án → bỏ bộ lọc và lựa chọn cũ (hồ sơ của dự án khác không được lẫn vào gói).
  useEffect(() => {
    setPhong("");
    setHop("");
    setSel(new Set());
  }, [projectId]);

  const dsPhong = useMemo(() => {
    const m = new Map<string, { hoSo: number; tep: number }>();
    for (const r of rows) {
      const k = r.phong || "";
      const v = m.get(k) ?? { hoSo: 0, tep: 0 };
      v.hoSo += 1;
      v.tep += r.files;
      m.set(k, v);
    }
    return [...m.entries()].sort((a, b) => collator.compare(a[0], b[0]));
  }, [rows, collator]);
  useEffect(() => {
    if (dsPhong.length === 1 && !phong) setPhong(dsPhong[0][0] || CHUA_PHAN_LOAI);
  }, [dsPhong, phong]);
  const phongKey = phong === CHUA_PHAN_LOAI ? "" : phong;
  const trongPhong = useMemo(() => (phong ? rows.filter((r) => (r.phong || "") === phongKey) : []), [rows, phong, phongKey]);
  const dsHop = useMemo(() => [...new Set(trongPhong.map((r) => r.hop || ""))].sort(collator.compare), [trongPhong, collator]);
  const loc = useMemo(() => {
    const k = q.trim().toLowerCase();
    return trongPhong
      .filter((r) => (!hop || (r.hop || "") === hop) && (!k || `${r.hop} ${r.hoSo}`.toLowerCase().includes(k)))
      .sort((a, b) => collator.compare(a.hop, b.hop) || collator.compare(a.hoSo, b.hoSo));
  }, [trongPhong, hop, q, collator]);
  const idOf = (r: ReviewSummaryRow) => reviewKey(r.projectId, r.phong, r.hop, r.hoSo);
  const daChon = useMemo(() => rows.filter((r) => sel.has(idOf(r))), [rows, sel]);

  const tiLe = (a: number, b: number, tone: "ok" | "warn") => (b === 0 ? "—" : <Badge tone={a >= b ? "ok" : a === 0 ? "gray" : tone}>{`${n(a)}/${n(b)}`}</Badge>);
  const cols: Column<ReviewSummaryRow>[] = [
    { key: "hop", header: "Hộp", width: 120, sortable: true, sortValue: (r) => r.hop, cell: (r) => r.hop || "—" },
    { key: "hoSo", header: "Hồ sơ", width: 140, sortable: true, sortValue: (r) => r.hoSo, cell: (r) => r.hoSo || "—" },
    { key: "files", header: "Tệp", align: "right", width: 80, sortable: true, cell: (r) => n(r.files) },
    { key: "pages", header: "Trang", align: "right", width: 90, sortable: true, cell: (r) => n(r.pages) },
    { key: "pdfa", header: "PDF/A", width: 110, cell: (r) => tiLe(r.pdfaDocs, r.files, "warn") },
    { key: "signed", header: "Ký số", width: 110, cell: (r) => tiLe(r.signedDocs, r.files, "warn") },
    { key: "approved", header: "Đã duyệt", width: 110, cell: (r) => tiLe(r.approved, r.files, "warn") },
  ];

  if (!projectId) {
    return (
      <Card padding="md">
        <EmptyState icon="duAn" title="Chọn dự án đang làm ở thanh trên" description="Phông, hộp, hồ sơ để đóng gói lấy từ tài liệu đã lên máy chủ của dự án đó." />
      </Card>
    );
  }
  return (
    <Card padding="none">
      <div className="flex flex-wrap items-end gap-3 border-b border-border-subtle p-3">
        <Field label={`Phông · dự án ${proj.projectLabel || projectId}`}>
          <Select
            size="sm"
            value={phong}
            onChange={(v) => {
              setPhong(v);
              setHop("");
            }}
            placeholder={dsPhong.length ? "— Chọn phông —" : loading ? "Đang tải…" : "Dự án chưa có tài liệu"}
            options={dsPhong.map(([p, v]) => ({ value: p || CHUA_PHAN_LOAI, label: `${p || CHUA_PHAN_LOAI} · ${n(v.hoSo)} hồ sơ · ${n(v.tep)} tệp` }))}
            className="w-96"
          />
        </Field>
        <Field label="Hộp">
          <Select size="sm" value={hop} disabled={!phong} onChange={setHop} placeholder="Tất cả" options={dsHop.map((h) => ({ value: h, label: h || "—" }))} className="w-40" />
        </Field>
        <Field label="Tìm">
          <Input size="sm" prefix="tim" value={q} onChange={(e) => setQ(e.target.value)} placeholder="Số hộp, số hồ sơ" className="w-56" />
        </Field>
        <span className="flex-1" />
        <span className="text-sm text-fg-subtle">
          {phong ? `${n(loc.length)} hồ sơ` : `${n(dsPhong.length)} phông · ${n(rows.length)} hồ sơ`}
          {sel.size ? ` · đã chọn ${n(sel.size)}` : ""}
        </span>
        <IconButton label="Tải lại" icon="lamMoi" size="sm" variant="secondary" loading={refreshing} onClick={reload} />
        <Button variant="primary" icon="goi" disabled={sel.size === 0} onClick={() => setDialog(true)}>
          Đóng gói ({n(sel.size)})
        </Button>
      </div>
      <Table
        columns={cols}
        rows={loc}
        getRowId={idOf}
        dense
        selection={{ selected: sel, onChange: setSel }}
        maxHeight="calc(100dvh - 360px)"
        state={{
          loading: loading && !data,
          error: data ? undefined : error,
          onRetry: reload,
          empty: !phong ? { title: "Chọn phông để xem hồ sơ", description: "Mỗi thư mục đã đẩy lên máy chủ là một phông; bên trong là hộp, hồ sơ." } : { title: "Không có hồ sơ khớp bộ lọc" },
        }}
      />
      {dialog && <PackServerDialog rows={daChon} projectId={projectId} projectLabel={proj.projectLabel || projectId} onClose={() => setDialog(false)} />}
    </Card>
  );
}

interface KetQua {
  key: string;
  ok: number;
  loi: string[];
}

function PackServerDialog({ rows, projectId, projectLabel, onClose }: { rows: ReviewSummaryRow[]; projectId: string; projectLabel: string; onClose: () => void }) {
  const dsPhong = useMemo(() => [...new Set(rows.map((r) => r.phong || CHUA_PHAN_LOAI))], [rows]);
  const motPhong = dsPhong.length === 1 ? dsPhong[0] : "";
  const [kind, setKind] = useState<V4sj>("aip_hoso");
  const [fondCode, setFondCode] = useState(() => (motPhong ? nho(`pack.fondCode.${motPhong}`) || goiYMaPhong(motPhong) : ""));
  const [fondName, setFondName] = useState(() => (motPhong ? nho(`pack.fondName.${motPhong}`) || motPhong : ""));
  const [orgCode, setOrgCode] = useState(() => nho("pack.archiveOrgCode"));
  const [orgName, setOrgName] = useState(() => nho("pack.archiveOrgName"));
  const [maintenance, setMaintenance] = useState("01");
  const [agreement, setAgreement] = useState("");
  const [dip, setDip] = useState<Partial<V4sm>>({ requestDate: new Date().toISOString().slice(0, 10) });
  const [busy, setBusy] = useState(false);
  const [tienDo, setTienDo] = useState("");
  const [ketQua, setKetQua] = useState<Map<string, KetQua> | null>(null);
  const idOf = (r: ReviewSummaryRow) => reviewKey(r.projectId, r.phong, r.hop, r.hoSo);

  const chay = async () => {
    setBusy(true);
    luu("pack.archiveOrgCode", orgCode.trim());
    luu("pack.archiveOrgName", orgName.trim());
    if (motPhong) {
      luu(`pack.fondCode.${motPhong}`, fondCode.trim());
      luu(`pack.fondName.${motPhong}`, fondName.trim());
    }
    const out = new Map<string, KetQua>();
    const tepTheoNhom = new Map<string, V4ej[]>();
    try {
      const all = await docsForGroups(
        rows.map((r) => ({ projectId: r.projectId, phong: r.phong, hop: r.hop, hoSo: r.hoSo })),
        (d, t) => setTienDo(`Đang nạp tài liệu ${d}/${t} lô…`),
      );
      for (const d of all) {
        if (d.status === "disposed") continue;
        const k = reviewKeyOf(d);
        tepTheoNhom.set(k, [...(tepTheoNhom.get(k) ?? []), d]);
      }
    } catch (e) {
      notifyError(e, "nạp tài liệu trên máy chủ");
      setBusy(false);
      return;
    }
    let i = 0;
    for (const r of rows) {
      i += 1;
      setTienDo(`Đang tạo gói ${i}/${rows.length}…`);
      const key = idOf(r);
      const kq: KetQua = { key, ok: 0, loi: [] };
      out.set(key, kq);
      const ds = (tepTheoNhom.get(key) ?? []).sort((a, b) => a.filename.localeCompare(b.filename, "vi", { numeric: true }));
      if (ds.length === 0) {
        kq.loi.push("không có tài liệu nào trên máy chủ");
        continue;
      }
      const hopSo = so(r.hop);
      const hsSo = so(r.hoSo);
      const base: V4sm = {
        ...V4sn,
        ...(kind === "dip" ? dip : {}),
        kind,
        fondCode: fondCode.trim(),
        archiveOrgCode: orgCode.trim(),
        fondName: fondName.trim() || r.phong,
        archiveOrgName: orgName.trim(),
        fileCode: `${fondCode.trim()}.${hopSo}.${hsSo}`,
        title: `${r.phong || CHUA_PHAN_LOAI} – ${r.hop || "hộp —"} – ${r.hoSo || "hồ sơ —"}`,
        maintenance: kind.startsWith("sip") ? "01" : maintenance,
        numberOfPage: String(ds.reduce((a, d) => a + (d.pageCount ?? 0), 0) || ""),
        paperFileCode: `${hopSo}.${hsSo}`,
        submissionAgreement: agreement,
        certTrichYeu: dip.certTrichYeu || `${r.hop} – ${r.hoSo}`,
      };
      const nhom = kind === "aip_tailieu" || kind === "sip_tailieu" ? ds.map((d) => [d]) : [ds];
      for (const [j, g] of nhom.entries()) {
        const form: V4sm = nhom.length > 1 ? { ...base, fileCode: `${base.fileCode}.${String(j + 1).padStart(3, "0")}`, title: g[0].filename } : base;
        const { request, errors } = V4su(form, g.map((d) => ({ documentId: d.id, filename: d.filename })));
        if (!request) {
          kq.loi.push(...errors);
          break;
        }
        try {
          await V4rv({ projectId, hoSoRef: g[0].hoSoRef, request });
          kq.ok += 1;
        } catch (e) {
          kq.loi.push(errorText(e));
        }
      }
      setKetQua(new Map(out));
    }
    setKetQua(out);
    setBusy(false);
    setTienDo("");
    const tong = [...out.values()].reduce((a, k) => a + k.ok, 0);
    const loi = [...out.values()].filter((k) => k.loi.length > 0).length;
    if (tong > 0) toast.success(`Đã tạo ${n(tong)} gói. Máy chủ đang dựng, theo dõi ở tab Gói tin.`, { description: loi ? `${n(loi)} hồ sơ có lỗi, xem cột Kết quả.` : undefined });
    else toast.error("Không tạo được gói nào. Xem cột Kết quả.");
  };

  const cols: Column<ReviewSummaryRow>[] = [
    ...(dsPhong.length > 1 ? [{ key: "phong", header: "Phông", className: "whitespace-normal", cell: (r: ReviewSummaryRow) => r.phong || CHUA_PHAN_LOAI } as Column<ReviewSummaryRow>] : []),
    { key: "hop", header: "Hộp", width: 110, cell: (r) => r.hop || "—" },
    { key: "hoSo", header: "Hồ sơ", width: 130, cell: (r) => r.hoSo || "—" },
    { key: "files", header: "Tệp", align: "right", width: 70, cell: (r) => n(r.files) },
    { key: "signed", header: "Đã ký", align: "right", width: 80, cell: (r) => n(r.signedDocs) },
    ...(ketQua
      ? [
          {
            key: "kq",
            header: "Kết quả",
            className: "whitespace-normal text-sm",
            cell: (r: ReviewSummaryRow) => {
              const kq = ketQua.get(idOf(r));
              if (!kq) return "…";
              return (
                <>
                  {kq.ok > 0 && <span className="text-success-700">Đã tạo {kq.ok} gói. </span>}
                  {kq.loi.length > 0 && <span className="text-danger-700">{kq.loi.join(" · ")}</span>}
                </>
              );
            },
          } as Column<ReviewSummaryRow>,
        ]
      : []),
  ];
  const chuaKy = rows.filter((r) => r.signedDocs < r.files).length;

  return (
    <Dialog
      open
      onOpenChange={(o) => !o && !busy && onClose()}
      closable={!busy}
      title={`Đóng gói ${n(rows.length)} hồ sơ`}
      description={`Dự án ${projectLabel}${motPhong ? ` · phông ${motPhong}` : ` · ${dsPhong.length} phông`}. Mỗi hồ sơ một gói; gói lấy bản ký số, không có thì lấy PDF/A.`}
      size="xl"
      footer={
        <>
          <span className="self-center text-sm text-fg-subtle">{tienDo || (chuaKy ? `${n(chuaKy)} hồ sơ còn tài liệu chưa ký số.` : "")}</span>
          <span className="flex-1" />
          <Button onClick={onClose} disabled={busy}>
            {ketQua ? "Đóng" : "Huỷ"}
          </Button>
          <Button variant="primary" icon="goi" loading={busy} disabled={!!ketQua && !busy} onClick={() => void chay()}>
            Đóng gói {V4sk[kind]}
          </Button>
        </>
      }
    >
      {!motPhong && <ErrorBanner error="Đã chọn hồ sơ của nhiều phông: mã phông và tên cơ quan bên dưới áp cho tất cả. Nên đóng gói từng phông một." className="mb-3" />}
      <FormGrid cols={3}>
        <Field label="Loại gói">
          <Select value={kind} onChange={(v) => setKind(v as V4sj)} options={KINDS.map((k) => ({ value: k, label: V4sk[k] }))} />
        </Field>
        <Field label="Mã phông" hint="Fondcode trong gói; gợi ý từ tên thư mục">
          <Input value={fondCode} onChange={(e) => setFondCode(e.target.value)} />
        </Field>
        <Field label="Tên cơ quan hình thành phông">
          <Input value={fondName} onChange={(e) => setFondName(e.target.value)} placeholder={motPhong || "UBND xã …"} />
        </Field>
        <Field label="Mã cơ quan lưu trữ">
          <Input value={orgCode} onChange={(e) => setOrgCode(e.target.value)} />
        </Field>
        <Field label="Tên cơ quan lưu trữ">
          <Input value={orgName} onChange={(e) => setOrgName(e.target.value)} placeholder="Lưu trữ lịch sử tỉnh …" />
        </Field>
        {kind.startsWith("sip") ? (
          <Field label="Mã yêu cầu nộp" hint="Thoả thuận nộp lưu, bắt buộc với SIP">
            <Input value={agreement} onChange={(e) => setAgreement(e.target.value)} />
          </Field>
        ) : (
          <Field label="Thời hạn bảo quản" hint="Áp cho mọi hồ sơ trong lần này">
            <Select value={maintenance} onChange={setMaintenance} options={THBQ} />
          </Field>
        )}
      </FormGrid>
      {kind === "dip" && (
        <Card className="mt-3" title="Thông tin cấp bản sao (DIP)" padding="sm">
          <FormGrid cols={3}>
            {(
              [
                ["requestId", "Mã yêu cầu khai thác"],
                ["requestDate", "Ngày yêu cầu"],
                ["certMaXacThuc", "Mã xác thực lưu trữ"],
                ["certTenLuuTru", "Tên lưu trữ lịch sử"],
                ["certNguoiNhanHoTen", "Họ tên người nhận"],
                ["certNguoiNhanDinhDanh", "Mã định danh người nhận"],
                ["certTenLoai", "Tên loại tài liệu"],
                ["certMucDich", "Mục đích sử dụng"],
              ] as const
            ).map(([k, label]) => (
              <Field key={k} label={label}>
                <Input type={k === "requestDate" ? "date" : "text"} value={(dip[k] as string | undefined) ?? ""} onChange={(e) => setDip((d) => ({ ...d, [k]: e.target.value }))} />
              </Field>
            ))}
          </FormGrid>
        </Card>
      )}
      <div className="mt-3">
        <Table columns={cols} rows={rows} getRowId={idOf} maxHeight={300} dense />
      </div>
      <p className="mt-2 text-xs text-fg-subtle">
        <Tip text="Tên gói và mã hồ sơ: <mã phông>.<số hộp>.<số hồ sơ>">
          <span>Mã hồ sơ trong gói ghi theo <b>mã phông.số hộp.số hồ sơ</b>; tiêu đề hồ sơ lấy theo tên thư mục.</span>
        </Tip>
      </p>
    </Dialog>
  );
}
