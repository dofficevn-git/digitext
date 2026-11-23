// Copyright (c) 2026 DigiText. All rights reserved.
// Kiểm tra PDF/A cho tệp hoặc thư mục bất kỳ: loại tự khai (PDF/A-1B, 2B, 3B…), dấu hiệu bắt buộc, và khi máy có
// Docker thì veraPDF kết luận đạt / không đạt kèm luật vi phạm.

import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { onExtimportProgress, pdfaCheck, pdfaVerapdfAvailable, type ExtProgress, type KetQuaTrang, type PdfaFileReport, type PdfaSummary } from "@/api/extimport";
import { isTauri } from "@/api/ipc";
import { Badge, Banner, Button, Card, Checkbox, ErrorBanner, Field, Help, Table, Tip, Toolbar, type Column } from "@/ui";

const fmt = (n: number) => n.toLocaleString("vi-VN");
function badgeOf(f: PdfaFileReport): { tone: "err" | "ok" | "info"; text: string } {
  if (f.error) return { tone: "err", text: "Hỏng" };
  if (f.verapdf) return f.verapdf.compliant ? { tone: "ok", text: "Đạt" } : { tone: "err", text: "Không đạt" };
  if (f.looksPdfa) return { tone: "info", text: "Có khai PDF/A" };
  return { tone: "err", text: "Không phải PDF/A" };
}

/** Danh sách số trang, cắt bớt khi quá dài. */
const soTrang = (ds: number[]): string => (ds.length > 20 ? `${ds.slice(0, 20).join(", ")} … và ${fmt(ds.length - 20)} trang nữa` : ds.join(", "));

/** Vì sao không đo được — nói thẳng lý do máy gặp, không đoán hộ. */
function viKhongDo(t: KetQuaTrang): string {
  const vi = t.khongDoVi ?? [];
  if (vi.length === 0) return "Không rút được ảnh của trang nào nên không đo được";
  return `Không đo được: ${vi.join("; ")}`;
}

/** Ô tóm tắt trong cột "Trang trắng / lệch". */
function OTrang({ t }: { t: KetQuaTrang | null }) {
  if (!t) return <span className="text-fg-subtle">—</span>;
  if (t.daDo === 0) return <Tip text={viKhongDo(t)}><span className="text-fg-subtle">không đo được</span></Tip>;
  const xau = t.trangTrang.length + t.trangGanTrang.length + t.trangLech.length + t.trangNgang.length;
  if (xau === 0) return <Badge tone="ok">Sạch</Badge>;
  return (
    <span className="flex flex-wrap gap-1">
      {t.trangTrang.length > 0 && <Badge tone="warn">{fmt(t.trangTrang.length)} trắng</Badge>}
      {t.trangGanTrang.length > 0 && <Badge tone="info">{fmt(t.trangGanTrang.length)} gần trắng</Badge>}
      {t.trangLech.length > 0 && <Badge tone="warn">{fmt(t.trangLech.length)} lệch</Badge>}
      {t.trangNgang.length > 0 && <Badge tone="err">{fmt(t.trangNgang.length)} ngang</Badge>}
    </span>
  );
}

export default function PdfaCheckPanel() {
  const [paths, setPaths] = useState<string[]>([]);
  const [useVera, setUseVera] = useState(true);
  const [doTrang, setDoTrang] = useState(false);
  const [veraOk, setVeraOk] = useState<boolean | null>(null);
  const [busy, setBusy] = useState(false);
  const [progress, setProgress] = useState<ExtProgress | null>(null);
  const [res, setRes] = useState<PdfaSummary | null>(null);
  const [err, setErr] = useState<unknown>(null);

  useEffect(() => {
    if (!isTauri()) return;
    pdfaVerapdfAvailable()
      .then(setVeraOk)
      .catch(() => setVeraOk(false));
    let off: (() => void) | undefined;
    void onExtimportProgress((p) => {
      if (p.phase === "pdfa") setProgress(p);
    }).then((u) => {
      off = u;
    });
    return () => off?.();
  }, []);

  const pickFiles = async () => {
    const r = await open({ multiple: true, title: "Chọn tệp PDF cần kiểm", filters: [{ name: "PDF", extensions: ["pdf"] }] });
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
      setRes(await pdfaCheck({ paths, useVerapdf: useVera && veraOk === true, scanPages: doTrang }));
    } catch (e) {
      setErr(e);
    } finally {
      setBusy(false);
    }
  };

  const cols: Column<PdfaFileReport>[] = [
    { key: "kq", header: "Kết luận", width: 140, cell: (f) => { const b = badgeOf(f); return <Badge tone={b.tone}>{b.text}</Badge>; } },
    { key: "declared", header: "Loại tự khai", width: 120, cell: (f) => f.declared ?? "Không khai" },
    { key: "name", header: "Tệp", className: "whitespace-normal", cell: (f) => <Tip text={f.path}><span>{f.name}</span></Tip> },
    { key: "hasOutputIntent", header: "Hồ sơ màu", width: 100, cell: (f) => (f.hasOutputIntent ? "Có" : <span className="text-danger-700">Không</span>) },
    { key: "textPages", header: "Lớp chữ", width: 100, cell: (f) => `${f.textPages}/${f.pages} trang` },
    { key: "minDpi", header: "dpi", width: 110, cell: (f) => `${f.minDpi ?? "—"}${f.isMap ? " (bản đồ, cần 300)" : ""}` },
    {
      key: "trangLoi",
      header: "Trang lỗi",
      width: 200,
      cell: (f) =>
        f.trangLoi ? (
          <Tip text={f.trangLoi}>
            <span className="line-clamp-2 text-danger-700">{f.trangLoi}</span>
          </Tip>
        ) : (
          "—"
        ),
    },
    { key: "trang", header: "Trang trắng / lệch", width: 170, cell: (f) => <OTrang t={f.trang} /> },
    { key: "signed", header: "Ký số", width: 80, cell: (f) => (f.signed ? "Có" : "Không") },
    { key: "producer", header: "Phần mềm tạo", className: "text-sm text-fg-subtle", cell: (f) => f.producer ?? "—", hideBelow: 1440 },
  ];

  return (
    <div className="space-y-4">
      <Card title="Kiểm tra PDF/A" description="Chọn tệp hoặc thư mục để biết loại PDF/A và lý do.">
        <Toolbar>
          <Button icon="tepPdf" disabled={busy} onClick={() => void pickFiles()}>
            Chọn tệp PDF
          </Button>
          <Button icon="thuMuc" disabled={busy} onClick={() => void pickDir()}>
            Chọn thư mục
          </Button>
          <span className="text-sm text-fg-subtle">{paths.length === 0 ? "Chưa chọn gì" : paths.length === 1 ? paths[0] : `${fmt(paths.length)} tệp đã chọn`}</span>
        </Toolbar>
        <Toolbar className="mt-3">
          <Tip text={veraOk === false ? "Máy này chưa có Docker và veraPDF, chỉ xem được loại tự khai và dấu hiệu bắt buộc" : "Kết quả veraPDF dùng để nghiệm thu"}>
            <span>
              <Field label="Kiểm đúng chuẩn ISO bằng veraPDF" inline>
                <Checkbox checked={useVera && veraOk === true} onChange={setUseVera} disabled={busy || veraOk !== true} />
              </Field>
            </span>
          </Tip>
          <Tip text="Giải nén ảnh từng trang để đo lượng mực và góc dòng chữ. Chậm hơn hẳn, nên chỉ bật khi cần soát chất lượng quét.">
            <span>
              <Field label="Kiểm trang trắng và trang lệch" inline>
                <Checkbox checked={doTrang} onChange={setDoTrang} disabled={busy} />
              </Field>
            </span>
          </Tip>
          <Toolbar.Spacer />
          <Button variant="primary" icon="kiemTraPdfa" loading={busy} disabled={!isTauri() || paths.length === 0} onClick={() => void run()}>
            Kiểm tra PDF/A
          </Button>
        </Toolbar>
      </Card>

      {busy && progress && progress.total > 0 && (
        <Banner tone="info">
          Đang kiểm PDF/A: {fmt(progress.done)}/{fmt(progress.total)} bước
        </Banner>
      )}
      {err !== null && <ErrorBanner error={err} />}

      {res && (
        <Card
          tone={res.notOk === 0 ? undefined : "warning"}
          padding="none"
          title={`${fmt(res.total)} tệp: ${fmt(res.ok)} ${res.verapdfUsed ? "đạt veraPDF" : "có khai PDF/A đủ dấu hiệu"}, ${fmt(res.notOk)} không đạt`}
          description={`Theo loại: ${Object.entries(res.byKind).map(([k, v]) => `${k} ${fmt(v)}`).join(" · ")}${!res.verapdfUsed ? " · chưa chạy veraPDF, đây là loại tự khai" : ""}${res.truncated ? " · quá 5.000 tệp, chỉ kiểm 5.000 tệp đầu" : ""}`}
        >
          <Table columns={cols} rows={res.files} getRowId={(f) => f.path} maxHeight={480} dense renderExpanded={(f) => {
            const d = [...(f.error ? [f.error] : f.problems), ...(f.verapdf?.rules ?? [])];
            const t = f.trang;
            const coTrang = t && (t.trangTrang.length > 0 || t.trangGanTrang.length > 0 || t.trangLech.length > 0 || t.trangNgang.length > 0);
            if (!d.length && !coTrang && !f.trangLoi) return null;
            return (
              <div className="space-y-2 px-8 py-2 text-sm">
                {d.length > 0 && <ul className="list-disc">{d.map((x, i) => <li key={i}>{x}</li>)}</ul>}
                {f.trangLoi && (
                  <div>
                    <b>Trang lỗi:</b> {f.trangLoi}
                  </div>
                )}
                {coTrang && t && (
                  <div className="space-y-1">
                    {t.trangTrang.length > 0 && <div><b>Trang trắng ({fmt(t.trangTrang.length)}):</b> {soTrang(t.trangTrang.map((x) => x.trang))}</div>}
                    {t.trangGanTrang.length > 0 && <div><b>Gần như trắng, nên xem lại ({fmt(t.trangGanTrang.length)}):</b> {soTrang(t.trangGanTrang.map((x) => x.trang))}</div>}
                    {t.trangLech.length > 0 && (
                      <div>
                        <b>Trang lệch ({fmt(t.trangLech.length)}):</b>{" "}
                        {t.trangLech.slice(0, 20).map((x) => `${x.trang} (${(x.goc ?? 0).toFixed(1)}°)`).join(", ")}
                        {t.trangLech.length > 20 ? ` … và ${fmt(t.trangLech.length - 20)} trang nữa` : ""}
                      </div>
                    )}
                    {t.trangNgang.length > 0 && <div><b>Trang quét ngang 90° ({fmt(t.trangNgang.length)}):</b> {soTrang(t.trangNgang)}</div>}
                    <div className="text-xs text-fg-subtle">
                      Đã soát {fmt(t.daDo)} trang
                      {(t.trangChu ?? 0) > 0 ? `, trong đó ${fmt(t.trangChu ?? 0)} trang chữ thật đọc theo lệnh vẽ` : ""}
                      {t.khongDo > 0 ? `, bỏ qua ${fmt(t.khongDo)} trang${(t.khongDoVi ?? []).length > 0 ? ` (${(t.khongDoVi ?? []).join("; ")})` : ""}` : ""}.
                    </div>
                  </div>
                )}
              </div>
            );
          }} />
          <p className="px-4 py-2 text-xs text-fg-subtle">
            {res.total > res.files.length ? `Bảng hiện ${fmt(res.files.length)} tệp đầu, tệp không đạt xếp trước. ` : ""}
            {res.reportFile ? (
              <>
                Bảng đầy đủ: <code>{res.reportFile}</code>
              </>
            ) : null}
          </p>
        </Card>
      )}

      <Help title="Trang trắng và trang lệch đo thế nào">
        <p>Mỗi trang được rút nhỏ về bản xám rồi lấy mức sáng ở phân vị 90 làm nền giấy, nên giấy ố vàng hay bản quét xám vẫn đo đúng. Điểm nào tối hơn nền 60 mức thì tính là mực; dưới 0,15 % diện tích coi là trang trắng, từ đó đến 0,6 % thì báo gần như trắng để người soát mở ra xem. Vùng 3 % sát mép bị bỏ qua vì gáy sách, bóng nắp máy quét và lỗ ghim đều nằm đó.</p>
        <p>Ngưỡng lấy từ 332 trang quét thật của Bắc Giang: trang ít mực nhất mà vẫn có nội dung đạt 1,14 %, tức gấp bảy lần ngưỡng trang trắng.</p>
        <p>Góc lệch đo bằng cách chiếu các điểm mực lên trục dọc ở từng góc thử từ âm 6 đến dương 6 độ: dòng chữ dồn vào ít hàng nhất ở đúng góc nghiêng. Từ 1 độ trở lên thì báo lệch. Trang nào chữ dựng đứng thì báo là quét ngang 90 độ chứ không báo lệch.</p>
        <p>Đo được bản quét lưu dưới dạng JPEG, CCITT G3 / G4 (bản quét đen trắng), Flate, LZW, RunLength hoặc không nén; ảnh nằm trong Form XObject hay trang thừa kế tài nguyên từ nút cha cũng tìm ra. Ảnh JPEG 2000 và ảnh JBIG2 thì phải vẽ lại trang mới đọc được nên bị bỏ qua — số trang bỏ qua và lý do ghi ngay trong dòng của tệp.</p>
        <p>Trang không có ảnh nào, tức văn bản điện tử gốc, được soát theo lệnh vẽ của chính trang đó: không lệnh nào để lại vết thì là trang trắng, ma trận đặt chữ mang góc xoay thì là trang lệch, và <code>/Rotate 90</code> thì là trang ngang. Ba câu này là số đúng chứ không phải ước lượng, nhưng trang kiểu đó không có lượng mực nên không bao giờ rơi vào mục gần như trắng. Trang có ảnh mà máy không giải nổi thì vẫn tính là chưa soát, không đọc lệnh vẽ để kết luận thay.</p>
      </Help>

      <Help title="Lớp chữ không phải PDF/A">
        <p>Bôi được chữ bằng Ctrl+A chỉ chứng minh tệp có lớp chữ. PDF/A còn đòi hồ sơ màu, siêu dữ liệu XMP, font nhúng và không có thành phần bị cấm; veraPDF kiểm đủ các luật đó theo ISO 19005.</p>
      </Help>
    </div>
  );
}
