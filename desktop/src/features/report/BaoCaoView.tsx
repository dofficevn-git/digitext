// Copyright (c) 2026 DigiText. All rights reserved.
// Báo cáo thống kê TT05 Điều 44.3b (27/11/2026, đợt 3 nhóm 3A): danh mục báo cáo và cột do máy chủ định nghĩa (routes/bao_cao.rs),
// màn này chỉ dựng bộ lọc theo `boLoc`, bảng theo `cot`, dòng tổng; Excel / PDF máy chủ dựng rồi tải về qua hộp thoại Lưu.

import { useCallback, useEffect, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { useProject } from "@/state/project";
import { bcChay, bcDanhMuc, bcTai, oChu, type DinhNghiaBaoCao, type DongBaoCao, type ThamSoBaoCao } from "@/api/baoCao";
import { CanMayChu, ChuaChonDuAn, Select, tenPhong, useClCatalog, useClSync } from "@/features/chinhly/common";
import { Badge, Button, Card, ErrorBanner, Field, FormGrid, Input, notifyError, PageHeader, Skeleton, Table, toast } from "@/ui";

const namNay = new Date().getFullYear();

export default function BaoCaoView() {
  const projectId = useProject().projectId ?? "";
  const { cat } = useClCatalog();
  const sync = useClSync();
  const online = sync?.phase !== "offline";
  const dungDuoc = isTauri() && online && !!projectId;
  const [ma, setMa] = useState<string>("");
  const [phongId, setPhongId] = useState("");
  const [namTu, setNamTu] = useState("");
  const [namDen, setNamDen] = useState("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");
  const [tai, setTai] = useState<"xlsx" | "pdf" | null>(null);

  const dmLoader = useCallback(() => (isTauri() ? bcDanhMuc() : Promise.resolve({ nhom: [], baoCao: [] })), []);
  const dm = V41t(dmLoader, { topics: [], errorText: () => "Không tải được danh mục báo cáo." });
  const defs = useMemo(() => dm.data?.baoCao ?? [], [dm.data]);
  const nhoms = useMemo(() => dm.data?.nhom ?? [], [dm.data]);
  useEffect(() => {
    if (!ma && defs.length > 0) setMa(defs[0].ma);
  }, [defs, ma]);
  const def: DinhNghiaBaoCao | undefined = defs.find((d) => d.ma === ma);

  const thamSo: ThamSoBaoCao = useMemo(() => {
    const p: ThamSoBaoCao = { projectId };
    if (def?.boLoc.includes("phong") && phongId) p.phongId = phongId;
    if (def?.boLoc.includes("nam")) {
      if (namTu) p.namTu = namTu;
      if (namDen) p.namDen = namDen;
    }
    if (def?.boLoc.includes("thoi_gian")) {
      if (from) p.from = from;
      if (to) p.to = to;
    }
    return p;
  }, [projectId, def, phongId, namTu, namDen, from, to]);

  const loader = useCallback(async () => {
    if (!dungDuoc || !def) return null;
    return bcChay(def.ma, thamSo);
  }, [dungDuoc, def, thamSo]);
  const { data, error, loading, reload } = V41t(loader, { topics: [], errorText: () => "Không lấy được số liệu báo cáo." });
  const rows: DongBaoCao[] = useMemo(() => data?.rows ?? [], [data]);

  const xuat = async (format: "xlsx" | "pdf") => {
    if (!def) return;
    setTai(format);
    try {
      const path = await bcTai(def.ma, thamSo, format);
      if (path) toast.success(`Đã lưu ${format === "xlsx" ? "Excel" : "PDF"}: ${path}`);
    } catch (e) {
      notifyError(e, `xuất ${format === "xlsx" ? "Excel" : "PDF"}`);
    } finally {
      setTai(null);
    }
  };

  return (
    <section className="view active">
      <PageHeader
        title="Thống kê theo TT 05/2025"
        description="18 báo cáo Điều 44.3b, xuất Excel hoặc PDF."
        infoTitle="Cách dùng"
        info={
          <>
            <p>Chọn báo cáo ở cột trái, đặt bộ lọc, số liệu lấy thẳng từ máy chủ. Excel để tính tiếp; PDF có đầu văn bản hành chính và khối ký để trình.</p>
            <p>Báo cáo "Sao lưu dữ liệu" là toàn máy chủ (sổ sao lưu không theo dự án). Tài liệu số chưa gắn vào văn bản của mục lục xếp dòng "(chưa gắn phông)".</p>
          </>
        }
      />
      <ChuaChonDuAn />
      <CanMayChu sync={sync} viec="Báo cáo thống kê cần máy chủ." />
      <div className="grid gap-4 lg:grid-cols-[260px_1fr]">
        <Card padding="none" title="Báo cáo">
          {dm.loading && !dm.data ? (
            <div className="space-y-2 p-3">
              <Skeleton className="h-5 w-3/4" />
              <Skeleton className="h-5 w-2/3" />
              <Skeleton className="h-5 w-4/5" />
            </div>
          ) : (
            <nav aria-label="Danh mục báo cáo" className="max-h-[70vh] overflow-auto py-1">
              {nhoms.map((n) => {
                const ds = defs.filter((d) => d.nhom === n.ma);
                if (ds.length === 0) return null;
                return (
                  <div key={n.ma} className="px-2 py-1.5">
                    <p className="px-2 pb-1 text-xs font-semibold text-fg-subtle">{n.ten}</p>
                    {ds.map((d) => (
                      <button
                        key={d.ma}
                        type="button"
                        aria-current={d.ma === ma ? "page" : undefined}
                        className={`block w-full rounded-md px-2 py-1.5 text-left text-sm hover:bg-surface-hover ${d.ma === ma ? "bg-brand-50 font-medium text-brand-700" : ""}`}
                        onClick={() => setMa(d.ma)}
                      >
                        {d.ten}
                      </button>
                    ))}
                  </div>
                );
              })}
              {dm.error != null && <ErrorBanner error={dm.error} className="m-2" onRetry={dm.reload} />}
            </nav>
          )}
        </Card>
        <div className="min-w-0 space-y-3">
          {def && (
            <Card padding="sm">
              <FormGrid cols={4}>
                {def.boLoc.includes("phong") && (
                  <Field label="Phông">
                    <Select size="sm" value={phongId} onChange={setPhongId} rows={cat.phong} render={tenPhong} empty="Tất cả phông" />
                  </Field>
                )}
                {def.boLoc.includes("nam") && (
                  <>
                    <Field label="Từ năm">
                      <Input size="sm" type="number" min={1800} max={2100} placeholder={`VD ${namNay - 5}`} value={namTu} onChange={(e) => setNamTu(e.target.value)} />
                    </Field>
                    <Field label="Đến năm">
                      <Input size="sm" type="number" min={1800} max={2100} placeholder={`VD ${namNay}`} value={namDen} onChange={(e) => setNamDen(e.target.value)} />
                    </Field>
                  </>
                )}
                {def.boLoc.includes("thoi_gian") && (
                  <>
                    <Field label="Từ ngày">
                      <Input size="sm" type="date" value={from} max={to || undefined} onChange={(e) => setFrom(e.target.value)} />
                    </Field>
                    <Field label="Đến ngày">
                      <Input size="sm" type="date" value={to} min={from || undefined} onChange={(e) => setTo(e.target.value)} />
                    </Field>
                  </>
                )}
              </FormGrid>
            </Card>
          )}
          {error != null && <ErrorBanner error={error} onRetry={reload} />}
          {def && (
            <Card
              padding="none"
              title={def.ten}
              description={def.canCu}
              actions={
                <>
                  <Badge tone="gray">{rows.length.toLocaleString("vi-VN")} dòng</Badge>
                  <Button size="sm" icon="tepExcel" disabled={!dungDuoc || rows.length === 0} loading={tai === "xlsx"} onClick={() => void xuat("xlsx")}>
                    Excel
                  </Button>
                  <Button size="sm" icon="tepPdf" disabled={!dungDuoc || rows.length === 0} loading={tai === "pdf"} onClick={() => void xuat("pdf")}>
                    PDF
                  </Button>
                </>
              }
            >
              <Table.Root dense maxHeight={600}>
                <Table.Head>
                  <tr>
                    <Table.HeadCell align="right" width={50}>
                      STT
                    </Table.HeadCell>
                    {def.cot.map((c) => (
                      <Table.HeadCell key={c.k} align={c.kieu === "so" ? "right" : c.kieu === "ma" || c.kieu === "ngay" || c.kieu === "bool" ? "center" : "left"}>
                        {c.ten}
                      </Table.HeadCell>
                    ))}
                  </tr>
                </Table.Head>
                <Table.Body>
                  {rows.map((r, i) => (
                    <Table.Row key={i}>
                      <Table.Cell align="right">{i + 1}</Table.Cell>
                      {def.cot.map((c) => (
                        <Table.Cell key={c.k} align={c.kieu === "so" ? "right" : c.kieu === "ma" || c.kieu === "ngay" || c.kieu === "bool" ? "center" : "left"} className={c.kieu === "chu" ? "max-w-[360px] whitespace-normal" : undefined}>
                          {c.kieu === "bool" && r[c.k] === true ? <Badge tone="warn">Có</Badge> : oChu(c, r[c.k])}
                        </Table.Cell>
                      ))}
                    </Table.Row>
                  ))}
                  {data?.tong && rows.length > 0 && (
                    <Table.Row className="bg-gray-50 font-semibold">
                      <Table.Cell />
                      {def.cot.map((c, i) => (
                        <Table.Cell key={c.k} align={c.kieu === "so" ? "right" : "left"}>
                          {c.kieu === "so" ? oChu(c, data.tong?.[i]) : (data.tong?.[i] ?? "")}
                        </Table.Cell>
                      ))}
                    </Table.Row>
                  )}
                  {rows.length === 0 && (
                    <Table.FullRow colSpan={def.cot.length + 1}>
                      <p className="px-3 py-4 text-sm text-fg-subtle">{loading && !data ? "Đang lấy số liệu…" : !dungDuoc ? "Cần kết nối máy chủ và chọn dự án." : "Không có số liệu trong phạm vi đã chọn."}</p>
                    </Table.FullRow>
                  )}
                </Table.Body>
              </Table.Root>
            </Card>
          )}
        </div>
      </div>
    </section>
  );
}
