// Copyright (c) 2026 DigiText. All rights reserved.
// Đóng gói KHO TÀI LIỆU TỪ BÊN NGOÀI: chọn thư mục đã quét + tệp Excel metadata →
// kiểm khớp (mã phông, mã hồ sơ, đường dẫn, tên tệp, thiếu/thừa tệp, số trang, Điều 8 TT05)
// → CHỈ KHI khớp hết mới đóng gói, mỗi hồ sơ một gói SIP/AIP.

import { useEffect, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";

import {
  extimportCheck,
  extimportPack,
  onExtimportProgress,
  pdfaVerapdfAvailable,
  type ExtCheckSummary,
  type ExtPackSummary,
  type ExtProgress,
} from "@/api/extimport";
import { isTauri } from "@/api/ipc";

const PHASE_LABEL: Record<ExtProgress["phase"], string> = {
  excel: "Đang đọc Excel…",
  check: "Đang kiểm tra",
  precheck: "Kiểm tra trước khi đóng gói",
  pack: "Đang đóng gói",
  pdfa: "Đang kiểm PDF/A",
};

function gb(n: number): string {
  return n >= 1e9 ? `${(n / 1e9).toFixed(1)} GB` : `${(n / 1e6).toFixed(0)} MB`;
}
function fmt(n: number): string {
  return n.toLocaleString("vi-VN");
}

export default function ExternalImportPanel() {
  const [root, setRoot] = useState("");
  const [excel, setExcel] = useState("");
  const [outDir, setOutDir] = useState("");
  const [kind, setKind] = useState<"aip_hoso" | "sip_hoso">("aip_hoso");
  const [agreement, setAgreement] = useState("");
  const [orgCode, setOrgCode] = useState("");
  const [deep, setDeep] = useState(true);
  // "Dự án này có yêu cầu đạt chuẩn ISO 19005 không?" — quyết định theo từng dự án, nhớ theo
  // tệp Excel. Mặc định KHÔNG: TT05 chỉ ghi "PDF/A hai lớp", không dẫn chiếu ISO.
  const [useVera, setUseVeraState] = useState(false);
  const [veraOk, setVeraOk] = useState<boolean | null>(null);
  const isoKey = (x: string) => `extimport.iso.${x.toLowerCase()}`;
  const setUseVera = (v: boolean) => {
    setUseVeraState(v);
    try {
      if (excel) localStorage.setItem(isoKey(excel), v ? "1" : "0");
    } catch {
      /* localStorage có thể bị chặn — chỉ là tiện ích nhớ lựa chọn */
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
  const [err, setErr] = useState("");

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
    const r = await open({
      multiple: false,
      title: "Chọn tệp Excel metadata",
      filters: [{ name: "Excel", extensions: ["xlsx", "xlsm", "xls"] }],
    });
    if (typeof r === "string") {
      setExcel(r);
      // Thư mục kho thường nằm cạnh tệp Excel — điền sẵn cho nhanh.
      if (!root) setRoot(r.replace(/[\\/][^\\/]+$/, ""));
    }
  };

  const runCheck = async () => {
    setErr("");
    setPack(null);
    setBusy("check");
    setProgress(null);
    try {
      setCheck(await extimportCheck({ root, excel, deep, useVerapdf: useVera && veraOk === true }));
    } catch (e) {
      setErr(String((e as { message?: string })?.message ?? e));
    } finally {
      setBusy("");
    }
  };

  const runPack = async () => {
    setErr("");
    setBusy("pack");
    setProgress(null);
    setLastNote("");
    try {
      const r = await extimportPack({
        root,
        excel,
        outDir,
        kind,
        agreement: agreement.trim() || null,
        orgCode: orgCode.trim() || null,
        allowPartial,
        force: false,
        useVerapdf: useVera && veraOk === true,
      });
      setPack(r);
      setCheck(r.check);
    } catch (e) {
      setErr(String((e as { message?: string })?.message ?? e));
    } finally {
      setBusy("");
    }
  };

  const ready = isTauri() && !!root && !!excel && !busy;
  const clean = check != null && check.errors === 0;
  const canPack = ready && !!outDir && check != null && (clean || allowPartial) && (kind !== "sip_hoso" || !!agreement.trim());

  return (
    <div className="card p-4 mb-4">
      <h3 className="mt-0">Đóng gói kho tài liệu từ bên ngoài</h3>
      <p className="muted mt-0">
        Dùng cho tài liệu đã quét sẵn ở nơi khác: một thư mục chia theo Phông / Hộp / Hồ sơ và một tệp Excel
        metadata (mỗi dòng một tài liệu). Hệ thống đối chiếu thư mục với Excel; <b>khớp hết mới đóng gói</b>,
        mỗi hồ sơ một gói tin. Không tải gì lên máy chủ.
      </p>

      <div className="grid gap-2.5 [grid-template-columns:repeat(auto-fill,minmax(320px,1fr))]">
        <div className="field">
          <label>① Tệp Excel metadata</label>
          <div className="flex gap-2">
            <input className="inp flex-1" value={excel} onChange={(e) => setExcel(e.target.value)} placeholder="…\Metadata.xlsx" />
            <button className="btn" onClick={() => void pickExcel()} disabled={!!busy}>Chọn…</button>
          </div>
        </div>
        <div className="field">
          <label>② Thư mục kho (chứa đường dẫn ghi trong Excel)</label>
          <div className="flex gap-2">
            <input className="inp flex-1" value={root} onChange={(e) => setRoot(e.target.value)} placeholder="E:\…" />
            <button className="btn" onClick={() => void pickDir("Chọn thư mục kho tài liệu", setRoot)} disabled={!!busy}>Chọn…</button>
          </div>
        </div>
      </div>

      <div className="flex items-center gap-4 mt-3 flex-wrap">
        <label className="flex items-center gap-1.5">
          <input type="checkbox" checked={deep} onChange={(e) => setDeep(e.target.checked)} disabled={!!busy} />
          Mở từng PDF (số trang, PDF/A hai lớp, dpi, màu, chữ ký số — Điều 8 TT05). Kho lớn mất vài chục phút.
        </label>
        <label
          className="flex items-center gap-1.5"
          title="Bật: từng tệp được veraPDF kiểm đúng chuẩn ISO 19005 theo loại nó tự khai (1b, 2b, 2u, 3b, 3u…), ở bước kiểm và ngay trước khi đóng gói; tệp không đạt là LỖI. Tắt: chỉ đòi tệp khai PDF/A và có hồ sơ màu (TT05 không dẫn chiếu ISO). Lựa chọn được nhớ theo tệp Excel."
        >
          <input
            type="checkbox"
            checked={useVera && veraOk === true}
            onChange={(e) => setUseVera(e.target.checked)}
            disabled={!!busy || veraOk !== true}
          />
          <b>Dự án này yêu cầu đạt chuẩn ISO 19005</b> (kiểm bằng veraPDF, không đạt là lỗi)
          {veraOk === false ? " — máy này chưa có Docker nên không bật được" : ""}
          {veraOk === true ? ". Kho lớn: vài giờ." : ""}
        </label>
        <button className="btn primary" onClick={() => void runCheck()} disabled={!ready}>
          {busy === "check" ? "Đang kiểm tra…" : "③ Kiểm tra khớp với Excel"}
        </button>
      </div>

      {busy && progress && (
        <div className="banner mt-3">
          {PHASE_LABEL[progress.phase]}
          {progress.total > 0 ? `: ${fmt(progress.done)}/${fmt(progress.total)} hồ sơ` : ""}
          {lastNote ? ` · gần nhất hỏng — ${lastNote}` : ""}
        </div>
      )}
      {err && <div className="err-banner mt-3">{err}</div>}

      {check && (
        <div className="mt-4">
          <div className={`banner ${clean ? "" : "warn"}`}>
            <b>{clean ? "ĐÃ KHỚP — được phép đóng gói." : "CHƯA KHỚP — chưa được đóng gói."}</b>{" "}
            {fmt(check.rows)} dòng Excel · {fmt(check.filesOnDisk)} tệp PDF trên đĩa · {fmt(check.dossiers)} hồ sơ ·{" "}
            {gb(check.bytes)} · <b>{fmt(check.dossiersOk)}</b> hồ sơ đạt ·{" "}
            <b>{fmt(check.dossiers - check.dossiersOk)}</b> hồ sơ có lỗi · {fmt(check.errors)} lỗi · {fmt(check.warnings)} cảnh báo
            {check.deep ? (check.verapdf ? " · ISO 19005: YÊU CẦU (đã xác minh veraPDF)" : " · ISO 19005: không yêu cầu") : ""}
            {check.deep && check.pdfa != null
              ? ` · PDF/A ${fmt(check.pdfa)}/${fmt(check.rows)} · đã ký ${fmt(check.signed ?? 0)}/${fmt(check.rows)} · ${fmt(check.pages ?? 0)} trang`
              : " · (chưa mở từng PDF)"}
            {check.deep && Object.keys(check.pdfaFlavours ?? {}).length > 0
              ? ` · Loại tệp: ${Object.entries(check.pdfaFlavours).map(([k, v]) => `${k} ${fmt(v)}`).join(", ")}`
              : ""}
          </div>
          {check.byCode.length > 0 && (
            <div className="tbl-scroll mt-2.5">
              <table className="tbl">
                <thead>
                  <tr>
                    <th>Mức độ</th>
                    <th>Loại vấn đề</th>
                    <th>Số lần</th>
                    <th>Số hồ sơ</th>
                    <th>Ví dụ</th>
                  </tr>
                </thead>
                <tbody>
                  {check.byCode.map((c) => (
                    <tr key={`${c.severity}-${c.code}`}>
                      <td>
                        <span className={`badge ${c.severity === "error" ? "err" : c.severity === "warning" ? "warn" : "info"}`}>
                          {c.severity === "error" ? "LỖI" : c.severity === "warning" ? "Cảnh báo" : "Thông tin"}
                        </span>
                      </td>
                      <td>{c.code}</td>
                      <td>{fmt(c.count)}</td>
                      <td>{fmt(c.dossiers)}</td>
                      <td>{c.example}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          <p className="muted mt-2">
            Báo cáo chi tiết từng hồ sơ / từng dòng Excel (mở bằng Excel): <code>{check.reportDir}</code> —{" "}
            <code>kiem-tra-ho-so.csv</code>, <code>kiem-tra-van-de.csv</code>. Sửa Excel hoặc thư mục theo báo cáo rồi bấm kiểm tra lại.
          </p>
        </div>
      )}

      {check && (
        <div className="mt-4">
          <h4 className="mt-0">④ Đóng gói</h4>
          <div className="grid gap-2.5 [grid-template-columns:repeat(auto-fill,minmax(240px,1fr))]">
            <div className="field">
              <label>Loại gói</label>
              <select className="inp" value={kind} onChange={(e) => setKind(e.target.value as "aip_hoso" | "sip_hoso")} disabled={!!busy}>
                <option value="aip_hoso">AIP hồ sơ — bảo quản</option>
                <option value="sip_hoso">SIP hồ sơ — nộp lưu</option>
              </select>
            </div>
            {kind === "sip_hoso" && (
              <div className="field">
                <label>Mã thoả thuận nộp lưu (bắt buộc với SIP)</label>
                <input className="inp" value={agreement} onChange={(e) => setAgreement(e.target.value)} />
              </div>
            )}
            <div className="field">
              <label>Mã cơ quan lưu trữ (bỏ trống = đoạn đầu mã phông)</label>
              <input className="inp" value={orgCode} onChange={(e) => setOrgCode(e.target.value)} placeholder="H49" />
            </div>
            <div className="field">
              <label>Thư mục ghi gói tin</label>
              <div className="flex gap-2">
                <input className="inp flex-1" value={outDir} onChange={(e) => setOutDir(e.target.value)} />
                <button className="btn" onClick={() => void pickDir("Chọn thư mục ghi gói tin", setOutDir)} disabled={!!busy}>Chọn…</button>
              </div>
            </div>
          </div>
          {!clean && (
            <label className="flex items-center gap-1.5 mt-2.5">
              <input type="checkbox" checked={allowPartial} onChange={(e) => setAllowPartial(e.target.checked)} disabled={!!busy} />
              Kho còn lỗi — vẫn đóng gói trước {fmt(check.dossiersOk)} hồ sơ đã khớp (hồ sơ lỗi bị giữ lại, không đóng).
            </label>
          )}
          <div className="mt-3">
            <button className="btn primary" onClick={() => void runPack()} disabled={!canPack}>
              {busy === "pack" ? "Đang đóng gói…" : "Bắt đầu đóng gói"}
            </button>
            <span className="muted ml-3">
              Ngay trước khi dựng, từng PDF được mở soi lại: số trang phải khớp Excel và tệp phải đạt Điều 8.2.a TT05
              (PDF/A hai lớp, ≥ 200 dpi, màu 24 bit, có chữ ký số). Chạy lại được — hồ sơ đã có gói thì bỏ qua.
            </span>
          </div>
        </div>
      )}

      {pack && (
        <div className="mt-4">
          <div className={`banner ${pack.started && pack.failed === 0 && pack.held === 0 ? "" : "warn"}`}>
            {pack.started ? (
              <>
                <b>Đóng gói xong.</b> Dựng mới {fmt(pack.built)} gói ({gb(pack.bytes)}) · đã có từ trước {fmt(pack.existing)} ·
                giữ lại vì lỗi bước kiểm {fmt(pack.held)} · dựng hỏng {fmt(pack.failed)}. Thư mục: <code>{pack.outDir}</code>
              </>
            ) : (
              <b>Chưa đóng gói: dữ liệu chưa khớp với Excel — xem bảng lỗi phía trên.</b>
            )}
          </div>
          {pack.failures.length > 0 && (
            <div className="tbl-scroll mt-2.5">
              <table className="tbl">
                <thead>
                  <tr>
                    <th>Mã hồ sơ</th>
                    <th>Thư mục</th>
                    <th>Lý do không đóng gói</th>
                  </tr>
                </thead>
                <tbody>
                  {pack.failures.slice(0, 100).map((f) => (
                    <tr key={f.dir}>
                      <td>{f.hsCode}</td>
                      <td>{f.dir}</td>
                      <td>{f.error}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
          {pack.resultFile && (
            <p className="muted mt-2">
              Bảng kết quả từng hồ sơ: <code>{pack.resultFile}</code>
            </p>
          )}
        </div>
      )}
    </div>
  );
}
