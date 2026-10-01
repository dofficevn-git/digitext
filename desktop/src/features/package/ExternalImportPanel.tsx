// Copyright (c) 2026 DigiText. All rights reserved.
// Đóng gói kho tài liệu từ bên ngoài: thư mục đã quét (Phông / Hộp / Hồ sơ) + tệp Excel metadata → kiểm khớp
// (mã phông, mã hồ sơ, đường dẫn, tên tệp, thiếu thừa tệp, số trang, điều kiện Điều 8) → khớp mới đóng gói,
// mỗi hồ sơ một gói. Không tải gì lên máy chủ.

import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { extimportCheck, extimportPack, onExtimportProgress, pdfaVerapdfAvailable, type ExtCheckSummary, type ExtPackSummary, type ExtProgress } from "@/api/extimport";
import { isTauri } from "@/api/ipc";
import { Badge, Banner, Button, Card, Checkbox, ErrorBanner, Field, FormGrid, Help, Input, Select, Stepper, Table, Tip, Toolbar } from "@/ui";

const PHASE_LABEL: Record<ExtProgress["phase"], string> = { excel: "Đang đọc Excel…", check: "Đang kiểm tra", precheck: "Kiểm tra trước khi đóng gói", pack: "Đang đóng gói", pdfa: "Đang kiểm PDF/A" };
const gb = (n: number) => (n >= 1e9 ? `${(n / 1e9).toFixed(1)} GB` : `${(n / 1e6).toFixed(0)} MB`);
const fmt = (n: number) => n.toLocaleString("vi-VN");

/** `nhanh`: Đóng gói nhanh — chỉ đối chiếu Excel, bỏ qua kiểm dpi / PDF/A / lớp chữ / chữ ký. */
export default function ExternalImportPanel({ nhanh = false }: { nhanh?: boolean } = {}) {
  const [root, setRoot] = useState("");
  const [excel, setExcel] = useState("");
  const [outDir, setOutDir] = useState("");
  const [kind, setKind] = useState<"aip_hoso" | "sip_hoso">("aip_hoso");
  const [agreement, setAgreement] = useState("");
  const [orgCode, setOrgCode] = useState("");
  const [orgName, setOrgName] = useState("");
  const [fondName, setFondName] = useState("");
  const [packagerName, setPackagerName] = useState("");
  const [packagerCode, setPackagerCode] = useState("");
  const [deep, setDeep] = useState(true);
  const [useVera, setUseVeraState] = useState(false);
  const [veraOk, setVeraOk] = useState<boolean | null>(null);
  const isoKey = (x: string) => `extimport.iso.${x.toLowerCase()}`;
  const setUseVera = (v: boolean) => {
    setUseVeraState(v);
    try {
      if (excel) localStorage.setItem(isoKey(excel), v ? "1" : "0");
    } catch {
      /* tiện ích nhớ */
    }
  };
  useEffect(() => {
    if (!excel) return;
    try {
      const v = localStorage.getItem(isoKey(excel));
      if (v != null) setUseVeraState(v === "1");
    } catch {
      /* bỏ qua */
    }
  }, [excel]);
  const [allowPartial, setAllowPartial] = useState(false);
  const [busy, setBusy] = useState<"" | "check" | "pack">("");
  const [progress, setProgress] = useState<ExtProgress | null>(null);
  const [lastNote, setLastNote] = useState("");
  const [check, setCheck] = useState<ExtCheckSummary | null>(null);
  const [pack, setPack] = useState<ExtPackSummary | null>(null);
  const [err, setErr] = useState<unknown>(null);

  useEffect(() => {
    if (isTauri()) pdfaVerapdfAvailable().then(setVeraOk).catch(() => setVeraOk(false));
    let off: (() => void) | undefined;
    void onExtimportProgress((p) => {
      setProgress(p);
      if (p.note) setLastNote(p.note);
    }).then((u) => {
      off = u;
    });
    return () => off?.();
  }, []);

  const pickDir = async (title: string, set: (v: string) => void) => {
    const r = await open({ directory: true, multiple: false, title });
    if (typeof r === "string") set(r);
  };
  const pickExcel = async () => {
    const r = await open({ multiple: false, title: "Chọn tệp Excel metadata", filters: [{ name: "Excel", extensions: ["xlsx", "xlsm", "xls"] }] });
    if (typeof r === "string") {
      setExcel(r);
      if (!root) setRoot(r.replace(/[\\/][^\\/]+$/, ""));
    }
  };
  const runCheck = async () => {
    setErr(null);
    setPack(null);
    setBusy("check");
    setProgress(null);
    try {
      setCheck(await extimportCheck({ root, excel, deep: deep && !nhanh, useVerapdf: !nhanh && useVera && veraOk === true }));
    } catch (e) {
      setErr(e);
    } finally {
      setBusy("");
    }
  };
  const runPack = async () => {
    setErr(null);
    setBusy("pack");
    setProgress(null);
    setLastNote("");
    try {
      const r = await extimportPack({ root, excel, outDir, kind, agreement: agreement.trim() || null, orgCode: orgCode.trim() || null, orgName: orgName.trim() || null, fondName: fondName.trim() || null, packagerName: packagerName.trim() || null, packagerCode: packagerCode.trim() || null, allowPartial, force: false, useVerapdf: !nhanh && useVera && veraOk === true, allowNonTt05: nhanh });
      setPack(r);
      setCheck(r.check);
    } catch (e) {
      setErr(e);
    } finally {
      setBusy("");
    }
  };

  const ready = isTauri() && !!root && !!excel && !busy;
  const clean = check != null && check.errors === 0;
  const canPack = ready && !!outDir && check != null && (clean || allowPartial) && (kind !== "sip_hoso" || !!agreement.trim());
  const step = pack ? 3 : check ? 2 : root && excel ? 1 : 0;

  return (
    <div className="space-y-4">
      <Stepper current={step} steps={[{ key: "chon", label: "Chọn Excel và thư mục" }, { key: "kiem", label: "Kiểm khớp" }, { key: "goi", label: "Đóng gói" }, { key: "xong", label: "Kết quả" }]} />

      {nhanh && (
        <Banner tone="warning" title="Đóng gói nhanh: chỉ đối chiếu với Excel">
          Kiểm mã phông, mã hồ sơ, đường dẫn, tên tệp, thiếu hoặc thừa tệp và số trang. Không kiểm độ phân giải, PDF/A, lớp chữ, chữ ký số, nên gói có thể chưa đạt Điều 8.2.a TT05. Dùng khi chủ dự án chấp nhận bản quét hiện có.
        </Banner>
      )}
      <Card title="Nguồn dữ liệu" description="Một thư mục chia theo Phông / Hộp / Hồ sơ và một tệp Excel metadata, mỗi dòng một tài liệu.">
        <FormGrid cols={2}>
          <Field label="Tệp Excel metadata">
            <div className="flex gap-2">
              <Input value={excel} onChange={(e) => setExcel(e.target.value)} placeholder="…\Metadata.xlsx" />
              <Button onClick={() => void pickExcel()} disabled={!!busy}>
                Chọn
              </Button>
            </div>
          </Field>
          <Field label="Thư mục kho" hint="Chứa các đường dẫn ghi trong Excel">
            <div className="flex gap-2">
              <Input value={root} onChange={(e) => setRoot(e.target.value)} placeholder="E:\…" />
              <Button onClick={() => void pickDir("Chọn thư mục kho tài liệu", setRoot)} disabled={!!busy}>
                Chọn
              </Button>
            </div>
          </Field>
        </FormGrid>
        <Toolbar className="mt-3">
          {!nhanh && (
            <>
              <Field label="Mở từng PDF để kiểm số trang, PDF/A, dpi, màu, chữ ký" inline>
                <Checkbox checked={deep} onChange={setDeep} disabled={!!busy} />
              </Field>
              <Tip text={veraOk === false ? "Máy này chưa có Docker nên không chạy được veraPDF" : "Kiểm đúng chuẩn ISO 19005 bằng veraPDF, tệp không đạt là lỗi"}>
                <span>
                  <Field label="Dự án yêu cầu đạt chuẩn ISO 19005" inline>
                    <Checkbox checked={useVera && veraOk === true} onChange={setUseVera} disabled={!!busy || veraOk !== true} />
                  </Field>
                </span>
              </Tip>
            </>
          )}
          <Toolbar.Spacer />
          <Button variant="primary" icon="kiemTra" loading={busy === "check"} disabled={!ready} onClick={() => void runCheck()}>
            Kiểm tra khớp với Excel
          </Button>
        </Toolbar>
        {!nhanh && <p className="mt-2 text-xs text-fg-subtle">Kho lớn: mở từng PDF mất vài chục phút, kiểm veraPDF mất vài giờ.</p>}
      </Card>

      {busy && progress && (
        <Banner tone="info">
          {PHASE_LABEL[progress.phase]}
          {progress.total > 0 ? `: ${fmt(progress.done)}/${fmt(progress.total)} hồ sơ` : ""}
          {lastNote ? `. Gần nhất hỏng: ${lastNote}` : ""}
        </Banner>
      )}
      {err !== null && <ErrorBanner error={err} />}

      {check && (
        <Card
          tone={clean ? undefined : "warning"}
          title={clean ? "Đã khớp, được phép đóng gói" : "Chưa khớp, chưa được đóng gói"}
          description={`${fmt(check.rows)} dòng Excel · ${fmt(check.filesOnDisk)} tệp PDF · ${fmt(check.dossiers)} hồ sơ · ${gb(check.bytes)} · ${fmt(check.dossiersOk)} hồ sơ đạt · ${fmt(check.dossiers - check.dossiersOk)} hồ sơ lỗi · ${fmt(check.errors)} lỗi · ${fmt(check.warnings)} cảnh báo${check.deep ? (check.verapdf ? " · ISO 19005: đã xác minh" : " · ISO 19005: không yêu cầu") : " · chưa mở từng PDF"}${check.deep && check.pdfa != null ? ` · PDF/A ${fmt(check.pdfa)}/${fmt(check.rows)} · đã ký ${fmt(check.signed ?? 0)} · ${fmt(check.pages ?? 0)} trang` : ""}`}
          padding="none"
        >
          {check.byCode.length > 0 && (
            <Table
              columns={[
                { key: "severity", header: "Mức", width: 100, cell: (c) => <Badge tone={c.severity === "error" ? "err" : c.severity === "warning" ? "warn" : "info"}>{c.severity === "error" ? "Lỗi" : c.severity === "warning" ? "Cảnh báo" : "Thông tin"}</Badge> },
                { key: "code", header: "Vấn đề", width: 220 },
                { key: "count", header: "Số lần", align: "right", width: 80, cell: (c) => fmt(c.count) },
                { key: "dossiers", header: "Hồ sơ", align: "right", width: 80, cell: (c) => fmt(c.dossiers) },
                { key: "example", header: "Ví dụ", className: "whitespace-normal text-sm" },
              ]}
              rows={check.byCode}
              getRowId={(c) => `${c.severity}-${c.code}`}
              maxHeight={320}
              dense
            />
          )}
          <p className="px-4 py-2 text-xs text-fg-subtle">
            Báo cáo từng hồ sơ và từng dòng Excel: <code>{check.reportDir}</code> (kiem-tra-ho-so.csv, kiem-tra-van-de.csv). Sửa Excel hoặc thư mục theo báo cáo rồi kiểm tra lại.
          </p>
        </Card>
      )}

      {check && (
        <Card title="Đóng gói">
          <FormGrid cols={3}>
            <Field label="Loại gói">
              <Select value={kind} onChange={(v) => setKind(v as "aip_hoso" | "sip_hoso")} disabled={!!busy} options={[{ value: "aip_hoso", label: "AIP hồ sơ (bảo quản)" }, { value: "sip_hoso", label: "SIP hồ sơ (nộp lưu)" }]} />
            </Field>
            {kind === "sip_hoso" && (
              <Field label="Mã thoả thuận nộp lưu" required>
                <Input value={agreement} onChange={(e) => setAgreement(e.target.value)} />
              </Field>
            )}
            <Field label="Mã cơ quan lưu trữ" hint="Để trống thì lấy đoạn đầu mã phông">
              <Input value={orgCode} onChange={(e) => setOrgCode(e.target.value)} placeholder="H49" />
            </Field>
            <Field label="Tên cơ quan lưu trữ">
              <Input value={orgName} onChange={(e) => setOrgName(e.target.value)} placeholder="Lưu trữ lịch sử tỉnh …" />
            </Field>
            <Field label="Tên cơ quan hình thành phông">
              <Input value={fondName} onChange={(e) => setFondName(e.target.value)} placeholder="UBND xã …" />
            </Field>
            <Field label="Đơn vị đóng gói" hint="Tên công ty số hoá, nếu có">
              <Input value={packagerName} onChange={(e) => setPackagerName(e.target.value)} />
            </Field>
            <Field label="Mã đơn vị đóng gói">
              <Input value={packagerCode} onChange={(e) => setPackagerCode(e.target.value)} />
            </Field>
            <Field label="Thư mục ghi gói tin" required className="sm:col-span-3">
              <div className="flex gap-2">
                <Input value={outDir} onChange={(e) => setOutDir(e.target.value)} />
                <Button onClick={() => void pickDir("Chọn thư mục ghi gói tin", setOutDir)} disabled={!!busy}>
                  Chọn
                </Button>
              </div>
            </Field>
          </FormGrid>
          {!clean && (
            <div className="mt-3">
              <Field label={`Kho còn lỗi, vẫn đóng gói trước ${fmt(check.dossiersOk)} hồ sơ đã khớp (hồ sơ lỗi giữ lại)`} inline>
                <Checkbox checked={allowPartial} onChange={setAllowPartial} disabled={!!busy} />
              </Field>
            </div>
          )}
          <Toolbar className="mt-3">
            <Button variant="primary" icon="goi" loading={busy === "pack"} disabled={!canPack} onClick={() => void runPack()}>
              Bắt đầu đóng gói
            </Button>
            <span className="text-sm text-fg-subtle">Trước khi dựng, từng PDF được soi lại số trang và điều kiện Điều 8.2.a. Chạy lại được, hồ sơ đã có gói thì bỏ qua.</span>
          </Toolbar>
        </Card>
      )}

      {pack && (
        <Card tone={pack.started && pack.failed === 0 && pack.held === 0 ? "info" : "warning"} title={pack.started ? "Đóng gói xong" : "Chưa đóng gói: dữ liệu chưa khớp với Excel"} padding="none">
          {pack.started && (
            <p className="px-4 py-2 text-sm">
              Dựng mới {fmt(pack.built)} gói ({gb(pack.bytes)}) · đã có từ trước {fmt(pack.existing)} · giữ lại vì lỗi {fmt(pack.held)} · dựng hỏng {fmt(pack.failed)}. Thư mục: <code>{pack.outDir}</code>
            </p>
          )}
          {pack.failures.length > 0 && (
            <Table
              columns={[
                { key: "hsCode", header: "Mã hồ sơ", width: 160 },
                { key: "dir", header: "Thư mục", className: "whitespace-normal text-sm" },
                { key: "error", header: "Lý do", className: "whitespace-normal text-sm" },
              ]}
              rows={pack.failures.slice(0, 100)}
              getRowId={(f) => f.dir}
              maxHeight={320}
              dense
            />
          )}
          {pack.resultFile && (
            <p className="px-4 py-2 text-xs text-fg-subtle">
              Bảng kết quả từng hồ sơ: <code>{pack.resultFile}</code>
            </p>
          )}
        </Card>
      )}

      <Help title="Điều kiện tệp theo TT05">
        <p>Điều 8.2.a: PDF/A hai lớp, độ phân giải từ 200 dpi (bản đồ, bản vẽ từ 300 dpi), màu 24 bit, có chữ ký số. TT05 không dẫn chiếu ISO 19005; chỉ bật kiểm veraPDF khi dự án yêu cầu.</p>
      </Help>
    </div>
  );
}
