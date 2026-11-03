// Copyright (c) 2026 DigiText. All rights reserved.
// Giao nhận tài liệu giấy (Bước 1–2 Điều 9 TT 05/2025, 3/11/2026): biên bản giao nhận khi đơn vị bàn giao tài liệu cho
// nhà thầu, và biên bản trả tài liệu gốc sau số hoá (điểm đ khoản 4). Số hộp / hồ sơ / tờ điền sẵn từ mục lục hồ sơ chỉnh
// lý của phông (cl_ho_so: hop_so, so_to) — người lập kiểm đếm thực tế rồi sửa trước khi lập.

import { useMemo, useState } from "react";
import { useProject } from "@/state/project";
import { V460 } from "@/state";
import { clHoSoList, str, type ClRow } from "@/api/chinhly";
import { useClCatalog } from "@/features/chinhly/common";
import { SoBienBan, type NguonDien, type NoiDungDien } from "./RecordsView";
import type { V4ta } from "./recordForms";
import { Card, EmptyState, Help, PageHeader, Tabs } from "@/ui";

type Tab = "nhan" | "tra";
const n = (v: number) => v.toLocaleString("vi-VN");
const homNay = () => {
  const d = new Date();
  return `${String(d.getDate()).padStart(2, "0")}/${String(d.getMonth() + 1).padStart(2, "0")}/${d.getFullYear()}`;
};
const soNguyen = (s: string): number => {
  const m = /(\d+)/.exec(s);
  return m ? Number(m[1]) : Number.MAX_SAFE_INTEGER;
};

/** Gom hồ sơ mục lục theo hộp: số hồ sơ, tổng số tờ, hồ sơ từ – đến. */
export function gomTheoHop(rows: ClRow[]): { hopSo: string; soHoSo: number; soTo: number; tu: string; den: string }[] {
  const m = new Map<string, { hopSo: string; soHoSo: number; soTo: number; hs: string[] }>();
  for (const r of rows) {
    const hop = str(r, "hop_so").trim() || "—";
    const g = m.get(hop) ?? { hopSo: hop, soHoSo: 0, soTo: 0, hs: [] };
    g.soHoSo += 1;
    g.soTo += Number(str(r, "so_to")) || 0;
    const hs = str(r, "ho_so_so").trim() || str(r, "so_tam").trim();
    if (hs) g.hs.push(hs);
    m.set(hop, g);
  }
  const collator = new Intl.Collator("vi", { numeric: true });
  return [...m.values()]
    .sort((a, b) => soNguyen(a.hopSo) - soNguyen(b.hopSo) || collator.compare(a.hopSo, b.hopSo))
    .map((g) => {
      const hs = [...g.hs].sort((a, b) => soNguyen(a) - soNguyen(b) || collator.compare(a, b));
      return { hopSo: g.hopSo, soHoSo: g.soHoSo, soTo: g.soTo, tu: hs[0] ?? "", den: hs[hs.length - 1] ?? "" };
    });
}

export default function HandoverView() {
  const { session } = V460();
  const proj = useProject();
  const projectId = session?.projectId ?? proj.projectId ?? null;
  const { cat } = useClCatalog();
  const [tab, setTab] = useState<Tab>("nhan");
  const phongOptions = useMemo(() => cat.phong.map((p) => ({ value: p.id, label: `${str(p, "ma_du_an") || "—"} · ${str(p, "ten")}` })), [cat.phong]);

  // Điền sẵn từ mục lục chỉnh lý của phông đã chọn: tổng hộp / hồ sơ / tờ + danh mục hộp.
  const nguonDien: NguonDien = useMemo(
    () => ({
      label: "Phông (mục lục chỉnh lý)",
      options: phongOptions,
      dien: async (kind: V4ta, phongId: string): Promise<NoiDungDien> => {
        const phong = cat.phong.find((p) => p.id === phongId);
        const { rows } = await clHoSoList({ phongId, limit: 200000 } as Parameters<typeof clHoSoList>[0]);
        const hops = gomTheoHop(rows);
        const tongTo = hops.reduce((a, h) => a + h.soTo, 0);
        const fields: Record<string, string> = {
          phong: str(phong, "ten"),
          maPhong: str(phong, "ma_du_an"),
          ngay: homNay(),
          tongHop: String(hops.length),
          tongHoSo: String(rows.length),
          tongTo: tongTo ? String(tongTo) : "",
        };
        const danhMucHop = hops.map((h): Record<string, string> => {
          const dong: Record<string, string> = { hopSo: h.hopSo, soHoSo: String(h.soHoSo), soTo: h.soTo ? String(h.soTo) : "" };
          if (kind === "giao_nhan_giay") {
            dong.tuHoSo = h.tu;
            dong.denHoSo = h.den;
          }
          return dong;
        });
        return { fields, lists: { danhMucHop } };
      },
    }),
    [cat.phong, phongOptions],
  );

  if (!projectId) {
    return (
      <section className="view active">
        <PageHeader title="Giao nhận tài liệu" description="Bước 1–2 Điều 9 TT 05/2025: khảo sát, kiểm đếm, lập biên bản giao nhận tài liệu giấy." />
        <Card padding="md">
          <EmptyState icon="duAn" title="Chọn dự án đang làm ở thanh trên" description="Biên bản giao nhận ghi theo dự án." />
        </Card>
      </section>
    );
  }
  return (
    <section className="view active">
      <PageHeader title="Giao nhận tài liệu" description={`Bước 1–2 Điều 9 TT 05/2025: khảo sát, kiểm đếm hộp, hồ sơ, tờ và lập biên bản giao nhận tài liệu giấy; sau số hoá lập biên bản trả tài liệu gốc. Dự án ${proj.projectLabel || ""}.`} />
      <Help title="Cách làm theo Điều 9" className="mb-3">
        <ul className="list-disc space-y-1 pl-5 text-sm">
          <li>Khoản 1: khảo sát phông, kiểm đếm số hộp, số hồ sơ, số tờ khi nhận; chụp ảnh hiện trạng nếu cần; biên bản giao nhận là căn cứ để nhận lại tài liệu.</li>
          <li>Khoản 2: vệ sinh tài liệu, đếm lại hồ sơ, giữ nguyên trật tự — ghi kết quả vào cột Tình trạng của danh mục hộp.</li>
          <li>Điểm đ khoản 4: trả tài liệu gốc về kho có biên bản; đối chiếu với biên bản giao nhận.</li>
          <li>Nút <b>Điền từ dữ liệu</b> lấy số hộp, hồ sơ, tờ từ mục lục chỉnh lý của phông ({n(cat.phong.length)} phông có mục lục); sửa lại theo kiểm đếm thực tế trước khi lập.</li>
        </ul>
      </Help>
      <Tabs<Tab>
        value={tab}
        onChange={setTab}
        items={[
          { value: "nhan", label: "Nhận tài liệu", icon: "nhapKho" },
          { value: "tra", label: "Trả tài liệu gốc", icon: "hoanTac" },
        ]}
      />
      <div className="mt-3">
        {tab === "nhan" && <SoBienBan kinds={["giao_nhan_giay"]} projectId={projectId} title="Biên bản giao nhận tài liệu giấy" description="Mỗi phông (hoặc mỗi đợt bàn giao) một biên bản. Lọc theo mã phông ở cột Phông / ngày." nguonDien={nguonDien} nutLap="Lập biên bản giao nhận" embedded />}
        {tab === "tra" && <SoBienBan kinds={["tra_tai_lieu_giay"]} projectId={projectId} title="Biên bản trả tài liệu gốc" description="Lập khi trả tài liệu giấy về kho sau khi số hoá và kiểm tra xong." nguonDien={nguonDien} nutLap="Lập biên bản trả" embedded />}
      </div>
    </section>
  );
}
