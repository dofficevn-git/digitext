// Copyright (c) 2026 DigiText. All rights reserved.
// Thống kê chỉnh lý từ máy chủ: tổng kết chỉnh lý (theo phòng ban), nhân viên (từ lịch sử sửa trường), phần mềm
// (theo phông), mượn trả. Xuất Excel. Cần máy chủ.

import { useCallback, useMemo, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { V460 } from "@/state";
import { clRemote, qs, str, num, type ClRow } from "@/api/chinhly";
import { CanMayChu, ChuaChonDuAn, fmtTime, n, Select, tenPhong, useClCatalog, useClSync } from "./common";
import { writeXlsx } from "./excel";
import { Button, Card, ErrorBanner, Field, FormGrid, Input, KpiRow, KpiTile, notifyError, PageHeader, Table, toast } from "@/ui";

type Kind = "chinh-ly" | "nhan-vien" | "phan-mem" | "muon-tra";
interface Col {
  k: string;
  l: string;
  num?: boolean;
  fmt?: (r: ClRow) => string;
}
const COLS: Record<Kind, Col[]> = {
  "chinh-ly": [
    { k: "ten_phong", l: "Phông" },
    { k: "ten_phong_ban", l: "Phòng ban" },
    { k: "hop_vv", l: "Hộp vĩnh viễn", num: true },
    { k: "hs_vv", l: "Hồ sơ vĩnh viễn", num: true },
    { k: "hop_co_thoi_han", l: "Hộp có thời hạn", num: true },
    { k: "hs_co_thoi_han", l: "Hồ sơ có thời hạn", num: true },
    { k: "bo", l: "Bó loại", num: true },
    { k: "tap", l: "Tập loại", num: true },
    { k: "hs_het", l: "Hết THBQ", num: true },
    { k: "hs_trung_thua", l: "Trùng thừa", num: true },
    { k: "hs_chua_danh_so", l: "Chưa đánh số", num: true },
    { k: "hs_tong", l: "Tổng hồ sơ", num: true },
    { k: "vb_tong", l: "Tổng văn bản", num: true },
  ],
  "nhan-vien": [
    { k: "actor_name", l: "Nhân viên" },
    { k: "username", l: "Tài khoản" },
    { k: "so_ho_so", l: "Hồ sơ", num: true },
    { k: "so_van_ban", l: "Văn bản", num: true },
    { k: "truong_tong", l: "Trường thực hiện", num: true },
    { k: "truong_moi", l: "Trường thêm mới", num: true },
    { k: "truong_sua", l: "Trường đã sửa", num: true },
    { k: "ngay_lam_viec", l: "Ngày làm việc", num: true },
    { k: "dau", l: "Từ", fmt: (r) => fmtTime(str(r, "dau")) },
    { k: "cuoi", l: "Đến", fmt: (r) => fmtTime(str(r, "cuoi")) },
  ],
  "phan-mem": [
    { k: "ten_phong", l: "Phông" },
    { k: "khoi_tao", l: "Khởi tạo", fmt: (r) => fmtTime(str(r, "khoi_tao")) },
    { k: "so_phong_ban", l: "Phòng ban", num: true },
    { k: "hs_tao", l: "Hồ sơ tạo", num: true },
    { k: "vb_tong", l: "Văn bản", num: true },
    { k: "hop_vv", l: "Hộp vĩnh viễn", num: true },
    { k: "hs_vv", l: "Hồ sơ vĩnh viễn", num: true },
    { k: "hop_co_thoi_han", l: "Hộp có thời hạn", num: true },
    { k: "hs_co_thoi_han", l: "Hồ sơ có thời hạn", num: true },
    { k: "bo", l: "Bó loại", num: true },
    { k: "tap", l: "Tập loại", num: true },
  ],
  "muon-tra": [],
};
const TEN: Record<Kind, string> = { "chinh-ly": "Tổng kết chỉnh lý", "nhan-vien": "Tổng kết nhân viên chỉnh lý", "phan-mem": "Tổng kết phần mềm", "muon-tra": "Mượn trả" };
const MO_TA: Record<Kind, string> = { "chinh-ly": "Số hộp, hồ sơ, văn bản theo từng phòng ban của phông.", "nhan-vien": "Tính từ lịch sử sửa từng trường trên máy chủ.", "phan-mem": "Khối lượng đã lập trên phần mềm theo từng phông.", "muon-tra": "Phiếu mượn, quá hạn, đã trả trong kỳ." };

export default function ThongKeView({ kind, embedded }: { kind: Kind; embedded?: boolean }) {
  const { session } = V460();
  const projectId = session?.projectId ?? "";
  const { cat } = useClCatalog();
  const sync = useClSync();
  const online = sync?.phase !== "offline";
  const [phongId, setPhongId] = useState("");
  const [from, setFrom] = useState("");
  const [to, setTo] = useState("");

  const loader = useCallback(async () => {
    if (!isTauri() || !online || !projectId) return { rows: [] as ClRow[], mt: null as Record<string, number> | null };
    const path = `/cl/thong-ke/${kind}${qs({ projectId, phongId, from: from ? from.split("-").reverse().join("/") : "", to: to ? to.split("-").reverse().join("/") : "" })}`;
    if (kind === "muon-tra") return { rows: [] as ClRow[], mt: await clRemote<Record<string, number>>("GET", path) };
    return { rows: await clRemote<ClRow[]>("GET", path), mt: null };
  }, [online, projectId, kind, phongId, from, to]);
  const { data, error, loading, reload } = V41t(loader, { topics: ["chinhLy"], errorText: () => "Không lấy được số liệu thống kê." });
  const rows = useMemo(() => data?.rows ?? [], [data]);
  const mt = data?.mt ?? null;
  const cols = COLS[kind];
  const tong = (k: string) => rows.reduce((s, r) => s + (num(r, k) ?? 0), 0);

  const xuat = async () => {
    const aoa = [[`BÁO CÁO ${TEN[kind].toUpperCase()}`], [], cols.map((c) => c.l), ...rows.map((r) => cols.map((c) => (c.fmt ? c.fmt(r) : c.num ? (num(r, c.k) ?? 0) : str(r, c.k)))), ["TỔNG CỘNG", ...cols.slice(1).map((c) => (c.num ? tong(c.k) : ""))]];
    try {
      await writeXlsx([{ name: TEN[kind].slice(0, 31), aoa, merges: [`A1:${String.fromCharCode(64 + cols.length)}1`] }], `${kind}_${new Date().toISOString().slice(0, 10)}.xlsx`);
      toast.success("Đã xuất Excel.");
    } catch (e) {
      notifyError(e, "xuất Excel");
    }
  };

  const body = (
    <>
      <ChuaChonDuAn />
      <CanMayChu sync={sync} viec="Thống kê cần máy chủ." />
      {(kind !== "phan-mem" || kind === "phan-mem") && (
        <Card className="mb-4" padding="sm">
          <FormGrid cols={4}>
            {kind !== "phan-mem" && kind !== "muon-tra" && (
              <Field label="Phông">
                <Select size="sm" value={phongId} onChange={setPhongId} rows={cat.phong} render={tenPhong} empty="Tất cả phông" />
              </Field>
            )}
            {kind !== "chinh-ly" && (
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
      {error != null && <ErrorBanner error={error} className="mb-4" onRetry={reload} />}
      {kind === "muon-tra" ? (
        <Card>
          <KpiRow>
            <KpiTile label="Đang mượn" value={mt?.dangMuon ?? 0} tone="info" />
            <KpiTile label="Quá hạn" value={mt?.quaHan ?? 0} tone={(mt?.quaHan ?? 0) > 0 ? "err" : undefined} />
            <KpiTile label="Đã trả trong kỳ" value={mt?.daTra ?? 0} tone="ok" />
            <KpiTile label="Phiếu lập trong kỳ" value={mt?.lapPhieu ?? 0} />
          </KpiRow>
        </Card>
      ) : (
        <Card padding="none" title={TEN[kind]} actions={<Button size="sm" icon="tepExcel" disabled={rows.length === 0} onClick={() => void xuat()}>Xuất Excel</Button>}>
          <Table.Root dense maxHeight={560}>
            <Table.Head>
              <tr>
                <Table.HeadCell align="right" width={50}>
                  STT
                </Table.HeadCell>
                {cols.map((c) => (
                  <Table.HeadCell key={c.k} align={c.num ? "right" : "left"}>
                    {c.l}
                  </Table.HeadCell>
                ))}
              </tr>
            </Table.Head>
            <Table.Body>
              {rows.map((r, i) => (
                <Table.Row key={i}>
                  <Table.Cell align="right">{i + 1}</Table.Cell>
                  {cols.map((c) => (
                    <Table.Cell key={c.k} align={c.num ? "right" : "left"}>
                      {c.fmt ? c.fmt(r) : c.num ? n(num(r, c.k) ?? 0) : str(r, c.k)}
                    </Table.Cell>
                  ))}
                </Table.Row>
              ))}
              {rows.length > 0 && (
                <Table.Row className="bg-gray-50 font-semibold">
                  <Table.Cell />
                  <Table.Cell>Tổng cộng</Table.Cell>
                  {cols.slice(1).map((c) => (
                    <Table.Cell key={c.k} align={c.num ? "right" : "left"}>
                      {c.num ? n(tong(c.k)) : ""}
                    </Table.Cell>
                  ))}
                </Table.Row>
              )}
              {rows.length === 0 && (
                <Table.FullRow colSpan={cols.length + 1}>
                  <p className="px-3 py-4 text-sm text-fg-subtle">{loading && !data ? "Đang lấy số liệu…" : "Chưa có số liệu."}</p>
                </Table.FullRow>
              )}
            </Table.Body>
          </Table.Root>
        </Card>
      )}
    </>
  );
  if (embedded) return body;
  return (
    <section className="view active">
      <PageHeader title={TEN[kind]} description={MO_TA[kind]} />
      {body}
    </section>
  );
}
