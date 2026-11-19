// Copyright (c) 2026 DigiText. All rights reserved.
// Tiện ích › Kiểm chữ ký số: chọn thư mục trên ổ cứng, xác minh MỌI chữ ký trong từng tệp PDF,
// xuất báo cáo Excel. Khác «Kiểm PDF/A» ở chỗ nó không nói gì về chất lượng tệp, chỉ nói về
// chữ ký: còn toàn vẹn không, ai ký, ký lúc nào, có chứng minh được về sau không.

import { useEffect, useMemo, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { chukyHuy, chukyQuet, onChuKyProgress, type ChuKyProgress, type ChuKyRa, type KetQuaQuet, type TepDaKy } from "@/api/chuky";
import { isTauri } from "@/api/ipc";
import { notifyError, toast, Badge, Banner, Button, Card, ErrorBanner, Help, KpiRow, KpiTile, Table, Tip, Toolbar, type Column } from "@/ui";
import { mucChuKy, xepTheoViecCanLam, xuatExcel } from "./chuKyXuat";

const fmt = (n: number) => n.toLocaleString("vi-VN");
const coTep = (b: number) => (b >= 1048576 ? `${(b / 1048576).toFixed(1)} MB` : `${Math.max(1, Math.round(b / 1024))} KB`);

const TONE: Record<string, "ok" | "warn" | "err" | "gray"> = {
  "Đạt": "ok",
  "Đạt, có cảnh báo": "warn",
  "Không đạt": "err",
  "Chưa ký": "gray",
  "Bỏ qua": "err",
};

/** Bảng chi tiết từng chữ ký của một tệp, mở ra khi bấm vào dòng. */
function BangChuKy({ ds }: { ds: ChuKyRa[] }) {
  return (
    <table className="w-full text-sm">
      <thead className="text-xs text-fg-subtle">
        <tr className="text-left">
          <th className="py-1 pr-3 font-medium">Chữ ký</th>
          <th className="py-1 pr-3 font-medium">Người ký</th>
          <th className="py-1 pr-3 font-medium">Nhà cấp</th>
          <th className="py-1 pr-3 font-medium">Thời điểm ký</th>
          <th className="py-1 pr-3 font-medium">Toàn vẹn</th>
          <th className="py-1 pr-3 font-medium">Dấu thời gian</th>
        </tr>
      </thead>
      <tbody>
        {ds.map((c) => (
          <tr key={c.thuTu} className="border-t border-border-subtle align-top">
            <td className="py-1 pr-3">
              {c.nhan}
              {!c.phuTronTep && (
                <Tip text="Chữ ký này được ký trước, nên nó chỉ phủ tới bản sửa của chính nó. Đó là cách PDF hoạt động, không phải dấu hiệu bị sửa.">
                  <span className="ml-1 text-xs text-fg-subtle">(ký trước)</span>
                </Tip>
              )}
            </td>
            <td className="py-1 pr-3">
              {c.nguoiKy ?? <span className="text-fg-subtle">—</span>}
              {c.toChuc && <span className="block text-xs text-fg-subtle">{c.toChuc}</span>}
            </td>
            <td className="py-1 pr-3">{c.nhaCap ?? <span className="text-fg-subtle">—</span>}</td>
            <td className="py-1 pr-3">
              {c.thoiDiemTsa ? (
                c.thoiDiemTsa
              ) : c.thoiDiemKhai ? (
                <Tip text="Thời điểm do máy người ký tự ghi vào tệp. Không có dấu thời gian nên không chứng minh được.">
                  <span className="text-fg-subtle">{c.thoiDiemKhai} (tệp tự khai)</span>
                </Tip>
              ) : (
                <span className="text-fg-subtle">—</span>
              )}
            </td>
            <td className="py-1 pr-3">{c.loi ? <Badge tone="err">Không xác minh được</Badge> : c.toanVen ? <Badge tone="ok">Có</Badge> : <Badge tone="err">Không</Badge>}</td>
            <td className="py-1 pr-3">
              {c.coTsa ? "Có" : <span className="text-warning-700">Không</span>}
              {!c.ganChungThu && (
                <Tip text="Chữ ký không gắn chứng thư người ký (thiếu thuộc tính ESS signing-certificate), chưa đạt PAdES cơ bản.">
                  <span className="block text-xs text-warning-700">chưa gắn chứng thư</span>
                </Tip>
              )}
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

export default function KiemChuKyPanel() {
  const [paths, setPaths] = useState<string[]>([]);
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState<ChuKyProgress | null>(null);
  const [res, setRes] = useState<KetQuaQuet | null>(null);
  const [err, setErr] = useState<unknown>(null);
  const [xuat, setXuat] = useState(false);

  useEffect(() => {
    if (!isTauri()) return;
    let off: (() => void) | undefined;
    void onChuKyProgress(setProgress).then((u) => {
      off = u;
    });
    return () => off?.();
  }, []);

  const pickFiles = async () => {
    const r = await open({ multiple: true, title: "Chọn tệp PDF cần kiểm chữ ký", filters: [{ name: "PDF", extensions: ["pdf"] }] });
    if (Array.isArray(r) && r.length) setPaths(r);
    else if (typeof r === "string") setPaths([r]);
  };
  const pickDir = async () => {
    const r = await open({ directory: true, multiple: false, title: "Chọn thư mục chứa PDF (quét cả thư mục con)" });
    if (typeof r === "string") setPaths([r]);
  };
  const run = async () => {
    setErr(null);
    setRes(null);
    setProgress(null);
    setBusy(true);
    try {
      setRes(await chukyQuet(paths));
    } catch (e) {
      setErr(e);
    } finally {
      setBusy(false);
    }
  };
  const luuExcel = async () => {
    if (!res) return;
    setXuat(true);
    try {
      const p = await xuatExcel(res, paths);
      if (p) toast.success(`Đã lưu báo cáo: ${p}`);
    } catch (e) {
      notifyError(e);
    } finally {
      setXuat(false);
    }
  };

  const rows = useMemo(() => (res ? xepTheoViecCanLam(res.tep) : []), [res]);

  const cols: Column<TepDaKy>[] = [
    { key: "ketLuan", header: "Kết luận", width: 150, cell: (t) => <Badge tone={TONE[t.ketLuan] ?? "gray"}>{t.ketLuan}</Badge> },
    { key: "ten", header: "Tệp", className: "whitespace-normal", cell: (t) => <Tip text={t.duongDan}><span>{t.ten}</span></Tip> },
    {
      key: "soChuKy",
      header: "Số chữ ký",
      width: 110,
      cell: (t) => (t.soChuKy === 0 ? <span className="text-fg-subtle">—</span> : fmt(t.soChuKy)),
    },
    {
      key: "nguoiKy",
      header: "Người ký",
      className: "whitespace-normal",
      cell: (t) => {
        const ten = [...new Set(t.chuKy.map((c) => c.nguoiKy).filter((x): x is string => !!x))];
        return ten.length === 0 ? <span className="text-fg-subtle">—</span> : <span>{ten.join(" · ")}</span>;
      },
    },
    {
      key: "muc",
      header: "Mức chữ ký",
      width: 130,
      cell: (t) =>
        t.soChuKy === 0 ? (
          <span className="text-fg-subtle">—</span>
        ) : (
          <Tip text={mucChuKy(t)}>
            <span className={t.coDss ? undefined : "text-warning-700"}>{mucChuKy(t).split(" — ")[0]}</span>
          </Tip>
        ),
    },
    { key: "co", header: "Cỡ", width: 90, cell: (t) => coTep(t.co) },
    {
      key: "viec",
      header: "Việc cần làm",
      className: "whitespace-normal",
      cell: (t) => {
        const d = [...t.loi, ...t.canhBao];
        if (d.length === 0) return <span className="text-fg-subtle">—</span>;
        return (
          <Tip text={d.join("\n")}>
            <span className={t.loi.length > 0 ? "line-clamp-2 text-danger-700" : "line-clamp-2 text-warning-700"}>{d[0]}</span>
          </Tip>
        );
      },
    },
    { key: "thuMuc", header: "Thư mục", className: "text-sm text-fg-subtle", cell: (t) => t.thuMuc, hideBelow: 1440 },
  ];

  return (
    <div className="space-y-4">
      <Card title="Kiểm chữ ký số" description="Chọn thư mục để biết từng tệp PDF có chữ ký nào, ai ký, còn toàn vẹn không.">
        <Toolbar>
          <Button icon="tepPdf" disabled={busy} onClick={() => void pickFiles()}>
            Chọn tệp PDF
          </Button>
          <Button icon="thuMuc" disabled={busy} onClick={() => void pickDir()}>
            Chọn thư mục
          </Button>
          <span className="text-sm text-fg-subtle">{paths.length === 0 ? "Chưa chọn gì" : paths.length === 1 ? paths[0] : `${fmt(paths.length)} tệp đã chọn`}</span>
          <Toolbar.Spacer />
          {busy ? (
            <Button icon="dungHan" onClick={() => void chukyHuy()}>
              Dừng
            </Button>
          ) : null}
          <Button variant="primary" icon="kySo" loading={busy} disabled={!isTauri() || paths.length === 0} onClick={() => void run()}>
            Kiểm chữ ký
          </Button>
        </Toolbar>
      </Card>

      {busy && progress && progress.total > 0 && (
        <Banner tone="info">
          Đang kiểm: {fmt(progress.done)}/{fmt(progress.total)} tệp — {progress.name}
        </Banner>
      )}
      {err !== null && <ErrorBanner error={err} />}

      {res && (
        <>
          <KpiRow>
            <KpiTile label="Đạt" value={res.soDat} tone="ok" />
            <KpiTile label="Cần xem lại" value={res.soCanhBao} tone="warn" />
            <KpiTile label="Không đạt" value={res.soKhongDat} tone="err" />
            <KpiTile label="Chưa ký" value={res.soChuaKy} />
          </KpiRow>

          <Card
            padding="none"
            tone={res.soKhongDat === 0 ? undefined : "warning"}
            title={`${fmt(res.soPdf)} tệp PDF, ${fmt(res.tep.reduce((s, t) => s + t.soChuKy, 0))} chữ ký`}
            description={[
              res.boQuaKhongPdf > 0 ? `${fmt(res.boQuaKhongPdf)} mục không phải PDF, đã bỏ` : "",
              res.boQuaQuaLon > 0 ? `${fmt(res.boQuaQuaLon)} tệp quá lớn, chưa kiểm` : "",
              res.biCat ? "quét dừng ở trần 50.000 tệp — chia nhỏ thư mục rồi quét lại" : "",
              "tệp không đạt xếp trên",
            ]
              .filter(Boolean)
              .join(" · ")}
            actions={
              <Button icon="tepExcel" loading={xuat} onClick={() => void luuExcel()}>
                Xuất Excel
              </Button>
            }
          >
            <Table
              columns={cols}
              rows={rows}
              getRowId={(t) => t.duongDan}
              maxHeight={520}
              dense
              renderExpanded={(t) => {
                if (t.chuKy.length === 0 && t.loi.length === 0 && t.canhBao.length === 0) return null;
                return (
                  <div className="space-y-3 px-8 py-3 text-sm">
                    {t.loi.length > 0 && (
                      <ul className="list-disc text-danger-700">
                        {t.loi.map((x, i) => (
                          <li key={i}>{x}</li>
                        ))}
                      </ul>
                    )}
                    {t.canhBao.length > 0 && (
                      <ul className="list-disc text-warning-700">
                        {t.canhBao.map((x, i) => (
                          <li key={i}>{x}</li>
                        ))}
                      </ul>
                    )}
                    {t.chuKy.length > 0 && <BangChuKy ds={t.chuKy} />}
                  </div>
                );
              }}
            />
          </Card>
        </>
      )}

      <Help title="Tệp ký nhiều lần: chỉ chữ ký cuối phủ hết tệp">
        <p>Một công văn phát hành theo quy trình của Ban Cơ yếu thường mang bốn chữ ký chồng lên nhau: lãnh đạo ký phê duyệt, rồi văn thư thêm số công văn, thêm ngày tháng, và cuối cùng ký dấu tổ chức. Mỗi lần ký là một bản sửa mới nối vào cuối tệp.</p>
        <p>Vì thế chữ ký ký trước chỉ phủ tới bản sửa của chính nó, không phủ tới cuối tệp. Đây là cách PDF hoạt động, <b>không</b> phải dấu hiệu tệp bị sửa — cột Toàn vẹn của nó vẫn ghi Có. Dấu hiệu bị sửa thật là Toàn vẹn ghi Không, hoặc câu báo có byte nối thêm sau dấu kết thúc tệp.</p>
        <p>Màn này xác minh <b>từng</b> chữ ký trên đúng vùng byte của nó, nên mở một dòng ra là thấy cả bốn người ký và bốn kết luận riêng.</p>
      </Help>

      <Help title="Dấu thời gian: vì sao quan trọng với tài liệu lưu trữ">
        <p>Chứng thư số nào cũng có ngày hết hạn. Khi chứng thư hết hạn, chữ ký <b>không</b> mất hiệu lực — nhưng muốn chứng minh nó được ký lúc chứng thư còn hiệu lực thì phải có <i>dấu thời gian</i> do một bên thứ ba cấp (TSA). Không có dấu thời gian thì thời điểm ký chỉ là con số máy người ký tự ghi vào tệp, ai sửa cũng được.</p>
        <p>Vì vậy cột Thời điểm ký phân biệt hai thứ: có dấu thời gian thì ghi thẳng, còn không thì ghi kèm chữ «tệp tự khai». Với tài liệu bảo quản vĩnh viễn, thiếu dấu thời gian là điều nên sửa từ bước ký, không phải lúc nhập kho.</p>
      </Help>

      <Help title="Màn này không kiểm chất lượng tệp">
        <p>Ở đây chỉ nói về chữ ký. Tệp đạt hết ở màn này vẫn có thể không phải PDF/A, thiếu lớp chữ, hay quét dưới độ phân giải TT05 — dùng Tiện ích › Kiểm PDF/A cho việc đó.</p>
      </Help>
    </div>
  );
}
