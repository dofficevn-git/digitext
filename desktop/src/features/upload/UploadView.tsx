// Copyright (c) 2026 DigiText. All rights reserved.
// Đưa lên máy chủ. Máy nhân viên: bật chia sẻ để quản trị gom qua mạng nội bộ. Máy quản trị: ba tab
// "Gom từ máy nhân viên" · "Thêm từ ổ đĩa" · "Đối soát" (ma trận hồ sơ × máy, tệp lỗi), nút "Đẩy lên máy chủ" ở đầu.

import { Fragment, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { V41p } from "@/live/useLive";
import { open } from "@tauri-apps/plugin-dialog";
import { V460, useCollects, useUploads } from "@/state";
import { isTauri } from "@/api/ipc";
import { V3yw } from "@/domain/permissions";
import V4pp from "./BatchesPanel";
import KhoScanPanel from "./KhoScanPanel";
import { V4ps } from "./addFilesMessage";
import { V4pt } from "./collectMessage";
import { V4pu } from "./forwardMessage";
import { V4ia, V4it, V4iv, V4iw, V4ix, V4iy, type V4ii, type V4ij } from "./syncMatrix";
import { V4j0, V4j2, V4j3, V4j4, V4j9, V4jf, V4jg, type V4iz } from "./khoFilter";
import { V4pw, V4px, V4q0, V4q2, V4q4, V4q6, type V4pv } from "./collectedFilter";
import { listHoSo, V40w, type V40d, type V40v } from "@/api/archive";
import { V48j } from "@/api/catalog";
import { V4br, type V4bm } from "@/api/dossier";
import { V4cl, V4cm } from "@/features/dossier/logic";
import type { V3yl, V3ym, V3yo } from "@/domain/types";
import "./khogom.css";
import { V4ak, type V4a9 } from "@/api/users";
import { V431, V432, V433, V434, V435, V436, V43i, V43j, collectedClearErrors, collectedWipeAll, type V42z, type V43g, type V43h } from "@/api/lansync";
import { V4qa } from "./siteCodeWidth";
import { V4qb, V4qc } from "./sharePort";
import { Icon } from "@/components/Icon";
import { useProject } from "@/state/project";
import { V462 as cn } from "@/lib/utils";
import { Badge, Banner, Button, Card, Checkbox, confirm, ErrorBanner, Field, Help, IconButton, Input, KpiRow, KpiTile, Menu, notifyError, PageHeader, Select, Tabs, Tip, toast, Toolbar, Table } from "@/ui";

const SITE_PLACEHOLDER = "Dán mã site hoặc bấm Sinh mã mạnh";
const MONO = { flex: "0 0 auto", fontFamily: "ui-monospace, SFMono-Regular, Menlo, Consolas, monospace" } as const;
const mb = (b: number) => (b / 1_048_576).toFixed(1);
const fmtN = (n: number) => n.toLocaleString("vi-VN");
const UNSORTED = "__unsorted__";
const LOAD_MAX = 200_000;
const AUTO_HIDE_ROWS = 400;

function MaSiteYeu({ isAdmin }: { isAdmin: boolean }) {
  return (
    <Banner tone="warning" title="Mã site mạng nội bộ đang yếu" className="mb-3">
      Mã ngắn hoặc 6 số có thể bị dò trong vài giây trên mạng nội bộ. Hệ thống vẫn chạy, nhưng nên đổi sớm:{" "}
      {isAdmin ? "bấm Sinh mã mạnh rồi phát lại cây thư mục, các máy nhân viên sẽ tự nhận mã mới." : "báo quản trị sinh mã mạnh rồi phát lại cây thư mục cho bạn."}
    </Banner>
  );
}

export default function V4qm() {
  const { session } = V460();
  const isAdmin = !!session && V3yw(session.role, "admin");
  const isOperate = !!session && V3yw(session.role, "operate");
  return (
    <section className="view active">
      {isAdmin ? <MayQuanTri /> : isOperate ? <MayNhanVien /> : <PageHeader title="Đưa lên máy chủ" description="Tài khoản này không có gì để đồng bộ." />}
    </section>
  );
}

// ---------------- Máy nhân viên
function MayNhanVien() {
  const [status, setStatus] = useState<V42z | null>(null);
  const [code, setCode] = useState("");
  const [port, setPort] = useState("");
  const [busy, setBusy] = useState(false);
  const [nangCao, setNangCao] = useState(false);

  const load = useCallback(async () => {
    if (!isTauri()) return;
    try {
      setStatus(await V431());
      setCode((await V432()) ?? "");
      setPort(String((await V435()) ?? ""));
    } catch (e) {
      notifyError(e, "đọc trạng thái chia sẻ");
    }
  }, []);
  V41p(load, { topics: ["lan", "settings", "tree"] });

  const luu = async () => {
    const err = V4qc(port);
    if (err) {
      toast.error(err);
      return;
    }
    setBusy(true);
    try {
      await V436(port.trim());
      await V433(code.trim());
      await load();
      toast.success(code.trim() ? "Đã lưu mã site và bật chia sẻ." : "Đã tắt chia sẻ.");
    } catch (e) {
      notifyError(e, "lưu mã site");
    } finally {
      setBusy(false);
    }
  };

  return (
    <>
      <PageHeader title="Đưa lên máy chủ" description="Máy này chia sẻ tài liệu đã đóng gói cho máy quản trị gom qua mạng nội bộ. Quản trị tự thu thập, bạn không phải gửi tay." />
      <div className="max-w-2xl space-y-4">
        {status?.siteCodeWeak && <MaSiteYeu isAdmin={false} />}
        <Card
          title={
            <span className="flex items-center gap-2">
              <span className={cn("dot", status?.sharing ? "ok" : "")} />
              {status?.sharing ? "Đang chia sẻ" : "Chưa chia sẻ"}
            </span>
          }
        >
          {status?.sharing ? (
            <p className="text-sm">
              Đang chia sẻ <b>{fmtN(status.fileCount)}</b> tài liệu cho máy quản trị
              {status.port != null && (
                <>
                  {" "}
                  qua cổng <b>{status.port}</b>
                  {status.port === V4qb ? " (cổng chung)" : ""}
                </>
              )}
              .
            </p>
          ) : (
            <p className="text-sm text-fg-muted">{status && !status.hasSiteCode ? "Mã site thường tự đến khi bạn nhận cây thư mục. Nếu chưa có, dán mã quản trị đưa vào ô bên dưới." : "Chưa bật chia sẻ."}</p>
          )}
          {status?.deviceFingerprint && (
            <p className="mt-2 text-sm text-fg-subtle">
              Vân tay máy này: <code className="rounded bg-gray-100 px-1">{status.deviceFingerprint}</code>. Quản trị đối chiếu chuỗi này khi gom lần đầu.
            </p>
          )}
          {status && status.blocked.length > 0 && (
            <Banner tone="warning" className="mt-3" title={`${fmtN(status.blocked.length)} hồ sơ chưa chia sẻ được`}>
              <ul className="list-disc pl-4">
                {status.blocked.slice(0, 8).map((b) => (
                  <li key={b.hoSoRef}>
                    Hộp/hồ sơ <b>{b.hoSoRef}</b>: {b.reason}
                  </li>
                ))}
              </ul>
              {status.blocked.length > 8 && <p>…và {fmtN(status.blocked.length - 8)} hồ sơ khác.</p>}
            </Banner>
          )}
        </Card>
        <Card title="Mã site mạng nội bộ">
          <Field label="Mã site" hint="Để trống rồi Lưu = xoá mã và tắt chia sẻ (dùng khi máy nhận nhầm mã của site khác).">
            <div className="flex flex-wrap items-center gap-2">
              <Input style={{ ...MONO, width: V4qa(code) }} value={code} placeholder={SITE_PLACEHOLDER} onChange={(e) => setCode(e.target.value)} className="w-auto" />
              <Button variant="primary" loading={busy} onClick={() => void luu()}>
                Lưu và bật
              </Button>
            </div>
          </Field>
          <button type="button" className="mt-3 text-sm text-fg-link hover:underline" onClick={() => setNangCao((v) => !v)}>
            {nangCao ? "Ẩn nâng cao" : "Nâng cao"}
          </button>
          {nangCao && (
            <div className="mt-2">
              <Field label="Cổng chia sẻ cố định" hint={`Để trống = cổng chung ${V4qb}. Chỉ ghim cổng khác khi cổng chung bị chiếm hoặc tường lửa riêng.`}>
                <Input value={port} placeholder={String(V4qb)} onChange={(e) => setPort(e.target.value)} className="w-40" />
              </Field>
            </div>
          )}
        </Card>
      </div>
    </>
  );
}

// ---------------- Máy quản trị
type Tab = "gom" | "them" | "doisoat";

function MayQuanTri() {
  const { session } = V460();
  const project = useProject();
  const { startForward, retry, running: forwarding } = useUploads();
  const { startCollect, startAddFiles, running: collecting } = useCollects();
  const runningRef = useRef(false);
  runningRef.current = forwarding || collecting;
  // Phiên vừa kết thúc → nạp đủ bảng (bỏ qua nhịp 5 s).
  const wasRunning = useRef(false);
  useEffect(() => {
    if (wasRunning.current && !(forwarding || collecting)) {
      lastLoad.current = 0;
      void loadCollectedRef.current().catch(() => undefined);
    }
    wasRunning.current = forwarding || collecting;
  }, [forwarding, collecting]);
  const isSuper = session?.role === "super_admin" || session?.role === "manager";
  const [tab, setTab] = useState<Tab>("gom");
  const [siteCode, setSiteCode] = useState("");
  const [share, setShare] = useState<V42z | null>(null);
  const [collected, setCollected] = useState<V43h | null>(null);
  const [phongs, setPhongs] = useState<V40v[]>([]);
  const [hoSos, setHoSos] = useState<V40d[]>([]);
  const [hideEmpty, setHideEmpty] = useState(false);
  const hideTouched = useRef(false);
  const [busy, setBusy] = useState<"" | "collect" | "forward" | "code" | "retry" | "add">("");
  const [dateF, setDateF] = useState<V4pv>(V4pw);
  const [flt, setFlt] = useState<V4iz>(V4j0);
  const [phongBans, setPhongBans] = useState<V3yl[]>([]);
  const [mangs, setMangs] = useState<V3ym[]>([]);
  const [thbqs, setThbqs] = useState<V3yo[]>([]);
  const [dossiers, setDossiers] = useState<V4bm[]>([]);
  const [tuDongDay, setTuDongDayState] = useState<boolean>(() => {
    try {
      return localStorage.getItem("upload.autoForward") !== "0";
    } catch {
      return true;
    }
  });
  const setTuDongDay = (v: boolean) => {
    setTuDongDayState(v);
    try {
      localStorage.setItem("upload.autoForward", v ? "1" : "0");
    } catch {
      /* tiện ích nhớ */
    }
  };
  // Đẩy thẳng từ ổ đĩa: không chép vào Kho, tệp vừa ghi sổ là gửi lên máy chủ ngay.
  const [dayThang, setDayThangState] = useState<boolean>(() => {
    try {
      return localStorage.getItem("upload.directPush") === "1";
    } catch {
      return false;
    }
  });
  const setDayThang = (v: boolean) => {
    setDayThangState(v);
    try {
      localStorage.setItem("upload.directPush", v ? "1" : "0");
    } catch {
      /* tiện ích nhớ */
    }
  };
  const [projects, setProjects] = useState<V4a9[]>([]);
  const [projectId, setProjectId] = useState("");
  const [themPhong, setThemPhong] = useState("");
  const [themHoSo, setThemHoSo] = useState("");
  const [themViTri, setThemViTri] = useState("");
  const [ketQua, setKetQua] = useState<{ tone: "info" | "success" | "warning" | "danger"; text: string } | null>(null);

  // Kho gom có thể vài chục nghìn dòng: khi đang đẩy, sự kiện bắn liên tục → nạp lại tối đa 5 s/lần.
  const lastLoad = useRef(0);
  const scheduled = useRef<number | null>(null);
  const loadCollectedRef = useRef<() => Promise<void>>(async () => {});
  loadCollectedRef.current = async () => {
    const wait = 5000 - (Date.now() - lastLoad.current);
    if (wait > 0) {
      if (scheduled.current == null)
        scheduled.current = window.setTimeout(() => {
          scheduled.current = null;
          void loadCollectedRef.current().catch((e) => notifyError(e, "nạp kho gom"));
        }, wait);
      return;
    }
    lastLoad.current = Date.now();
    // Đang có phiên gửi/gom: chỉ lấy con số (1 dòng), giữ bảng cũ — nạp 40.000 dòng mỗi 5 s
    // vừa nặng WebView vừa giữ khoá SQLite mà vòng gửi đang cần. Phiên xong nạp đủ.
    if (runningRef.current) {
      const s = await V43i(1);
      setCollected((prev) => ({ ...s, rows: prev?.rows ?? s.rows }));
      return;
    }
    setCollected(await V43i(LOAD_MAX));
  };
  useEffect(
    () => () => {
      if (scheduled.current != null) window.clearTimeout(scheduled.current);
    },
    [],
  );

  const load = useCallback(async () => {
    if (!isTauri()) return;
    try {
      setSiteCode((await V432()) ?? "");
      setShare(await V431());
      await loadCollectedRef.current();
    } catch (e) {
      notifyError(e, "nạp kho gom");
    }
    try {
      const [ps, hs] = await Promise.all([V40w(), listHoSo()]);
      setPhongs(ps);
      setHoSos(hs);
      if (!hideTouched.current) setHideEmpty(hs.length > AUTO_HIDE_ROWS);
    } catch {
      setPhongs([]);
      setHoSos([]);
    }
    try {
      const [cat, ds] = await Promise.all([V48j(), V4br()]);
      setPhongBans(cat.phongBans);
      setMangs(cat.mangs);
      setThbqs(cat.thbqs);
      setDossiers(ds);
    } catch {
      setPhongBans([]);
      setMangs([]);
      setThbqs([]);
      setDossiers([]);
    }
  }, []);
  V41p(load, { topics: ["collected", "lan", "hoso", "catalog", "dossier", "settings"] });

  useEffect(() => {
    if (!isTauri()) return;
    V4ak()
      .then((ps) => {
        setProjects(ps);
        setProjectId((cur) => cur || project.projectId || session?.projectId || (ps.length === 1 ? ps[0].id : ""));
      })
      .catch(() => setProjects([]));
  }, [project.projectId, session?.projectId]);

  const run = async (kind: typeof busy, ctx: string, f: () => Promise<void>) => {
    setBusy(kind);
    try {
      await f();
      await load();
    } catch (e) {
      notifyError(e, ctx);
    } finally {
      setBusy("");
    }
  };

  const luuMa = () =>
    run("code", "lưu mã site", async () => {
      await V433(siteCode.trim());
      toast.success("Đã lưu mã site. Phổ biến mã cho máy nhân viên, hoặc phát lại cây thư mục để họ tự nhận.");
    });
  const sinhMa = () =>
    run("code", "sinh mã site", async () => {
      const c = await V434();
      setSiteCode(c);
      toast.success("Đã sinh mã site mới.", { description: "Phát lại cây thư mục để máy nhân viên tự nhận mã mới; máy chưa nhận sẽ tạm không gom được." });
    });
  const thuThap = () =>
    run("collect", "thu thập", async () => {
      const r = await startCollect();
      setKetQua({ tone: r.collected > 0 ? "success" : "info", text: V4pt(r) });
    });
  const dayLen = () =>
    run("forward", "đẩy lên máy chủ", async () => {
      const r = await startForward();
      setKetQua({ tone: r.failed > 0 || r.rejected > 0 ? "warning" : "success", text: V4pu(r) });
    });
  const xoaSach = async () => {
    const total = collected?.total ?? 0;
    if (total === 0) return;
    const ok = await confirm({ title: "Xoá sạch kho gom trên máy này?", body: `${fmtN(total)} bản (kể cả bản đã đẩy) và tệp của chúng trong thư mục Kho sẽ bị xoá. Dữ liệu đã lên máy chủ không bị ảnh hưởng. Không hoàn tác được.`, confirmLabel: "Xoá sạch", danger: true, typeToConfirm: "XOA" });
    if (!ok) return;
    await run("retry", "xoá kho gom", async () => {
      const r = await collectedWipeAll();
      toast.success(`Đã xoá ${fmtN(r.rows)} bản và ${fmtN(r.filesRemoved)} tệp.`, { description: [r.filesMissing > 0 ? `${fmtN(r.filesMissing)} tệp đã không còn` : "", r.filesFailed > 0 ? `${fmtN(r.filesFailed)} tệp không xoá được` : ""].filter(Boolean).join(" · ") || undefined });
    });
  };
  const xoaLoi = () =>
    run("retry", "xoá dấu lỗi", async () => {
      const k = await collectedClearErrors();
      toast.success(k > 0 ? `Đã xoá dấu lỗi của ${fmtN(k)} bản. Bấm Đẩy lên máy chủ để gửi lại.` : "Không có bản nào đang lỗi.");
    });
  const gomLaiTuChoi = () =>
    run("retry", "gom lại bản bị từ chối", async () => {
      const k = await V43j();
      toast.success(k > 0 ? `Đã bỏ ${fmtN(k)} bản bị từ chối. Bấm Thu thập để lấy lại từ máy nhân viên.` : "Không có bản nào bị từ chối.");
    });
  const guiLai = async (ids: number[], label?: string) => {
    try {
      await retry(ids);
      await load();
    } catch (e) {
      notifyError(e, `gửi lại ${label ?? `${ids.length} tài liệu`}`);
    }
  };

  const themTep = async (thuMuc: boolean) => {
    let paths: string[] = [];
    try {
      const r = await open({ multiple: true, directory: thuMuc, title: thuMuc ? "Chọn thư mục tài liệu" : "Chọn tệp PDF để thêm vào Kho", filters: thuMuc ? undefined : [{ name: "PDF", extensions: ["pdf"] }] });
      paths = typeof r === "string" ? [r] : Array.isArray(r) ? r : [];
    } catch (e) {
      notifyError(e, "mở hộp chọn tệp");
      return;
    }
    if (!paths.length) return;
    await run("add", "thêm tệp", async () => {
      const r = await startAddFiles({ paths, projectId: projectId.trim() || undefined, destRel: destRel || undefined, phongMa: themHoSo && themHoSo !== UNSORTED ? themPhong || undefined : undefined, projectLabel: projectLabel || undefined, direct: dayThang });
      setKetQua({ tone: r.direct ? (r.pushFailed + r.pushRejected > 0 ? "warning" : r.pushed > 0 ? "success" : "info") : r.added > 0 ? "success" : "info", text: V4ps(r) });
      if (r.direct) await load();
      if (!r.direct && tuDongDay && r.added > 0 && !forwarding) {
        setBusy("forward");
        try {
          const f = await startForward();
          setKetQua({ tone: f.failed > 0 ? "warning" : "success", text: `${V4ps(r)} ${V4pu(f)}` });
        } catch (e) {
          notifyError(e, "đẩy lên máy chủ");
          setKetQua({ tone: "warning", text: `${V4ps(r)} Bản vẫn ở kho gom, bấm Đẩy lên máy chủ để gửi lại.` });
        }
      }
    });
  };

  const pending = collected?.pending ?? 0;
  const rejected = collected?.rejected ?? 0;
  const tree = useMemo(() => V4it(hoSos), [hoSos]);
  const dossierIndex = useMemo(() => V4j9(dossiers, hoSos, thbqs), [dossiers, hoSos, thbqs]);
  const byDate = useMemo(() => V4q2(collected?.rows ?? [], dateF), [collected, dateF]);
  const filtered = useMemo(() => V4jf(byDate, flt, tree, dossierIndex), [byDate, flt, tree, dossierIndex]);
  const rows = filtered.rows;
  const filtering = V4px(dateF) || V4j2(flt);
  const stats = useMemo(() => {
    if (filtering || !collected) return V4q4(rows);
    return { collected: collected.total, forwarded: collected.forwarded, notForwarded: collected.total - collected.forwarded, rejected: collected.rejected };
  }, [filtering, collected, rows]);
  const errRows = useMemo(() => V4q6(rows), [rows]);
  const hoSoFiltered = useMemo(() => V4jg(hoSos, flt, dossierIndex), [hoSos, flt, dossierIndex]);
  const matrix = useMemo(() => V4iv({ phong: phongs, hoSo: hoSoFiltered, collected: rows, adminUsername: session?.username ?? null, hideEmptyRows: hideEmpty }), [phongs, hoSoFiltered, rows, session, hideEmpty]);
  const truncated = (collected?.rows.length ?? 0) >= LOAD_MAX;
  const hoSoCuaPhong = useMemo(() => hoSos.filter((h) => (h.phongId ?? "") === themPhong), [hoSos, themPhong]);
  const destRel = useMemo(() => {
    if (!phongs.length) return themViTri.trim();
    if (!themPhong || !themHoSo || themHoSo === UNSORTED) return "";
    const ten = phongs.find((p) => p.maDuAn === themPhong)?.tenDuAn || themPhong;
    return `${ten}/${themHoSo}`;
  }, [phongs, themPhong, themHoSo, themViTri]);
  // Không chọn phông = giữ nguyên cây thư mục đã chọn (14/10/2026); đã chọn phông thì phải chọn hồ sơ.
  const thieuHoSo = phongs.length > 0 && !!themPhong && !themHoSo;
  const projectLabel = useMemo(() => {
    const p = projects.find((x) => x.id === projectId.trim()) ?? (projects.length === 1 ? projects[0] : undefined);
    return p ? `${p.code} · ${p.name}`.split("/").join("-").split("\\").join("-") : "";
  }, [projects, projectId]);
  const dest = useMemo(() => {
    const hoSo = !!(themHoSo && themHoSo !== UNSORTED);
    if (!destRel && !hoSo && projectLabel) return `${projectLabel}/‹tên thư mục đã chọn›/… (tệp lẻ vào ${projectLabel}/Chưa phân loại)`;
    const tail = destRel || "Chưa phân loại";
    return hoSo || !projectLabel ? tail : `${projectLabel}/${tail}`;
  }, [themHoSo, destRel, projectLabel]);
  const pbCuaPhong = useMemo(() => V4cl(phongBans, flt.maDa), [phongBans, flt.maDa]);
  const mangCuaPb = useMemo(() => V4cm(mangs, flt.maDv), [mangs, flt.maDv]);
  const busyAny = busy !== "" || forwarding || collecting;

  return (
    <>
      <PageHeader
        title="Đưa lên máy chủ"
        description={session?.projectName ? `Gom tài liệu từ máy nhân viên hoặc ổ đĩa vào Kho rồi đẩy lên máy chủ. ${session.projectName}.` : "Gom tài liệu từ máy nhân viên hoặc ổ đĩa vào Kho rồi đẩy lên máy chủ."}
        actions={
          <>
            {pending > 0 && (
              <Button variant="primary" icon="taiLen" loading={busy === "forward" || forwarding} disabled={busyAny} onClick={() => void dayLen()}>
                Đẩy lên máy chủ ({fmtN(pending)})
              </Button>
            )}
            <Menu
              items={[
                { label: `Gom lại bản bị từ chối${rejected ? ` (${fmtN(rejected)})` : ""}`, icon: "lamMoi", onSelect: () => void gomLaiTuChoi(), disabled: rejected === 0 || busyAny },
                { label: "Xoá dấu lỗi, thử lại tất cả", icon: "lamMoi", onSelect: () => void xoaLoi(), disabled: errRows.length === 0 || busyAny },
                "-",
                { label: `Xoá sạch kho gom${collected?.total ? ` (${fmtN(collected.total)})` : ""}`, icon: "xoa", danger: true, onSelect: () => void xoaSach(), disabled: !collected?.total || busyAny },
              ]}
            />
          </>
        }
      />
      {share?.siteCodeWeak && <MaSiteYeu isAdmin />}
      {ketQua && (
        <Banner tone={ketQua.tone} className="mb-3" onClose={() => setKetQua(null)}>
          {ketQua.text}
        </Banner>
      )}

      <Tabs<Tab>
        value={tab}
        onChange={setTab}
        items={[
          { value: "gom", label: "Gom từ máy nhân viên", icon: "phatLan" },
          { value: "them", label: "Thêm từ ổ đĩa", icon: "nhapTep" },
          { value: "doisoat", label: "Đối soát", icon: "kiemTra", count: collected?.total },
        ]}
      />

      {tab === "gom" && (
        <div className="mt-3 max-w-3xl space-y-4">
          <Card title="Thu thập qua mạng nội bộ" description="Máy nhân viên đã bật chia sẻ và cùng mã site sẽ được gom tự động.">
            <Toolbar>
              <Button variant="primary" icon="phatLan" loading={busy === "collect" || collecting} disabled={busyAny} onClick={() => void thuThap()}>
                Thu thập
              </Button>
              <span className="text-sm text-fg-subtle">
                Chờ đẩy: <b>{fmtN(pending)}</b>
                {rejected > 0 && (
                  <>
                    {" "}
                    · bị từ chối: <b className="text-danger-700">{fmtN(rejected)}</b>
                  </>
                )}
              </span>
            </Toolbar>
          </Card>
          <Card title="Mã site mạng nội bộ" description="Mọi máy trong site phải cùng một mã. Máy nhân viên tự nhận mã khi nhận cây thư mục.">
            <div className="flex flex-wrap items-center gap-2">
              <Input style={{ ...MONO, width: V4qa(siteCode) }} value={siteCode} placeholder={SITE_PLACEHOLDER} onChange={(e) => setSiteCode(e.target.value)} className="w-auto" aria-label="Mã site" />
              <Button loading={busy === "code"} disabled={busyAny} onClick={() => void luuMa()}>
                Lưu mã
              </Button>
              <Button icon="tuDong" disabled={busyAny} onClick={() => void sinhMa()}>
                Sinh mã mạnh
              </Button>
            </div>
            {share?.deviceFingerprint && (
              <p className="mt-2 text-sm text-fg-subtle">
                Vân tay máy này: <code className="rounded bg-gray-100 px-1">{share.deviceFingerprint}</code>
              </p>
            )}
          </Card>
          <Help title="Không thấy máy nhân viên nào?">
            <ul>
              <li>Máy nhân viên đã đăng nhập và màn Đưa lên máy chủ hiện "Đang chia sẻ".</li>
              <li>Hai máy cùng một dải mạng (cùng switch / wifi); mạng nhiều lớp (VLAN) hoặc wifi bật cách ly thiết bị sẽ không thấy nhau.</li>
              <li>Tường lửa cho phép DigiText: Windows chọn hồ sơ mạng "Riêng tư"; macOS bật Local Network cho DigiText.</li>
              <li>Cả hai máy cùng phiên bản DigiText. Máy chạy bản cũ không trả lời quảng bá.</li>
              <li>Cổng chung là 45817; máy nhân viên ghim cổng khác thì nhập thẳng địa chỉ ở màn Cây thư mục.</li>
            </ul>
          </Help>
        </div>
      )}

      {tab === "them" && (
        <div className="mt-3 max-w-3xl space-y-4">
          <Card title="Thêm PDF có sẵn vào Kho" description={dayThang ? "Từ ổ cứng của dự án. Tệp giữ nguyên trên ổ, ghi sổ Kho rồi gửi thẳng lên máy chủ, không chép." : "Từ ổ cứng, USB hoặc thư mục quét cũ. Tệp được chép vào Kho máy này rồi đẩy lên máy chủ."}>
            <div className="grid gap-3 sm:grid-cols-2">
              <Field label="Dự án đích">
                <Select value={projectId} onChange={setProjectId} placeholder={projects.length ? "— Chọn dự án —" : "Dự án của tài khoản này"} options={projects.map((p) => ({ value: p.id, label: `${p.code} · ${p.name}` }))} />
              </Field>
              {phongs.length > 0 ? (
                <>
                  <Field label="Phông">
                    <Select
                      value={themPhong}
                      onChange={(v) => {
                        setThemPhong(v);
                        setThemHoSo("");
                      }}
                      placeholder="— Không chọn: giữ cây thư mục —"
                      options={phongs.map((p) => ({ value: p.maDuAn, label: p.tenDuAn || p.maDuAn }))}
                    />
                  </Field>
                  <Field label="Hộp · hồ sơ" className="sm:col-span-2">
                    <Select value={themHoSo} disabled={!themPhong} onChange={setThemHoSo} placeholder={themPhong ? "— Chọn hồ sơ theo danh mục —" : "— Chọn phông trước —"}>
                      {hoSoCuaPhong.map((h) => (
                        <option key={h.id} value={`${h.hopSo}/${h.hoSoSo}`}>
                          Hộp {h.hopSo || "—"} · Hồ sơ {h.hoSoSo || "—"}
                          {h.tieuDe ? ` — ${h.tieuDe}` : ""}
                        </option>
                      ))}
                      <option value={UNSORTED}>Chưa phân loại (không kê khai)</option>
                    </Select>
                  </Field>
                </>
              ) : (
                <Field label="Vị trí trong Kho" className="sm:col-span-2" hint="Ví dụ: Sở Tài chính/01/007">
                  <Input value={themViTri} onChange={(e) => setThemViTri(e.target.value)} />
                </Field>
              )}
            </div>
            <Toolbar className="mt-4">
              <Button variant="primary" icon="tepPdf" loading={busy === "add"} disabled={busyAny || thieuHoSo} onClick={() => void themTep(false)}>
                Chọn tệp PDF
              </Button>
              <Button icon="thuMuc" disabled={busyAny || thieuHoSo} onClick={() => void themTep(true)}>
                Chọn thư mục
              </Button>
              <Field label="Đẩy thẳng từ ổ đĩa, không chép vào Kho" inline>
                <Checkbox checked={dayThang} onChange={setDayThang} disabled={busyAny} />
              </Field>
              {!dayThang && (
                <Field label="Đẩy lên máy chủ ngay sau khi thêm" inline>
                  <Checkbox checked={tuDongDay} onChange={setTuDongDay} disabled={busyAny} />
                </Field>
              )}
            </Toolbar>
            <p className="mt-2 text-sm text-fg-subtle">
              {thieuHoSo ? "Đã chọn phông thì chọn hồ sơ theo danh mục (hoặc bỏ chọn phông để giữ nguyên cây thư mục)." : dayThang ? `Tệp giữ nguyên trên ổ đĩa; máy chủ xếp theo cây: ${dest}. Tệp gửi lỗi cần cắm lại ổ này để gửi lại.` : `Sẽ lưu vào Kho: ${dest}`}
            </p>
          </Card>
        </div>
      )}

      {tab === "doisoat" && (
        <div className="mt-3">
          <KhoScanPanel disabled={busyAny} startAddFiles={startAddFiles} startForward={startForward} onChanged={load} />
        </div>
      )}
      {tab === "doisoat" && collected && (
        <div className="mt-3 space-y-4">
          <KpiRow>
            <KpiTile label="Đã thu thập" value={stats.collected} />
            <KpiTile label="Đã lên máy chủ" value={stats.forwarded} tone="ok" />
            <KpiTile label="Chưa lên máy chủ" value={stats.notForwarded} tone={stats.notForwarded > 0 ? "warn" : undefined} sub={rejected > 0 ? `máy chủ từ chối ${fmtN(rejected)}` : undefined} />
            <KpiTile label="Tệp lỗi" value={errRows.length} tone={errRows.length > 0 ? "err" : undefined} />
          </KpiRow>

          <Card padding="none">
            <div className="border-b border-border-subtle p-3">
              <Toolbar>
                <Select size="sm" value={flt.maDa} onChange={(v) => setFlt((f) => ({ ...f, maDa: v, maDv: "", mangCvid: "" }))} placeholder="Tất cả phông" options={phongs.map((p) => ({ value: p.maDuAn, label: p.tenDuAn || p.maDuAn }))} className="w-52" aria-label="Phông" />
                <Select size="sm" value={flt.maDv} disabled={!flt.maDa || pbCuaPhong.length === 0} onChange={(v) => setFlt((f) => ({ ...f, maDv: v, mangCvid: "" }))} placeholder={V4j4("phongBan", !!flt.maDa, pbCuaPhong.length)} options={pbCuaPhong.map((x) => ({ value: x.maDv, label: x.tenDonVi || x.maDonVi || x.maDv }))} className="w-44" aria-label="Phòng ban" />
                <Select size="sm" value={flt.mangCvid} disabled={!flt.maDv || mangCuaPb.length === 0} onChange={(v) => setFlt((f) => ({ ...f, mangCvid: v }))} placeholder={V4j4("mang", !!flt.maDv, mangCuaPb.length)} options={mangCuaPb.map((m) => ({ value: String(m.mangCvid), label: m.tenMang || m.maMang }))} className="w-40" aria-label="Mảng" />
                <Select size="sm" value={flt.thbq} disabled={thbqs.length === 0} onChange={(v) => setFlt((f) => ({ ...f, thbq: v }))} placeholder={thbqs.length ? "Tất cả THBQ" : "Chưa có THBQ"} options={thbqs.map((t) => ({ value: String(t.thbqid), label: t.tenThbq }))} className="w-36" aria-label="Thời hạn bảo quản" />
                <Input size="sm" prefix="tim" value={flt.search} placeholder="Tên tệp, số hồ sơ, người quét" onChange={(e) => setFlt((f) => ({ ...f, search: e.target.value }))} className="w-60" aria-label="Tìm" />
                <Toolbar.Spacer />
                <Input size="sm" type="date" aria-label="Từ ngày thu thập" value={dateF.from} max={dateF.to || undefined} onChange={(e) => setDateF((r) => ({ ...r, from: e.target.value }))} className="w-36" />
                <Input size="sm" type="date" aria-label="Đến ngày thu thập" value={dateF.to} min={dateF.from || undefined} onChange={(e) => setDateF((r) => ({ ...r, to: e.target.value }))} className="w-36" />
                {V4px(dateF) && <IconButton label="Bỏ lọc ngày" icon="dong" size="sm" onClick={() => setDateF(V4pw)} />}
                <Field label="Chỉ hồ sơ có dữ liệu" inline>
                  <Checkbox
                    checked={hideEmpty}
                    onChange={(v) => {
                      hideTouched.current = true;
                      setHideEmpty(v);
                    }}
                  />
                </Field>
              </Toolbar>
            </div>

            {matrix.groups.length === 0 ? (
              <p className="p-4 text-sm text-fg-subtle">
                {V4j2(flt) ? "Không có tài liệu nào khớp bộ lọc. Chọn lại Tất cả phông hoặc bỏ bớt tiêu chí." : filtering ? "Không có bản nào thu thập trong khoảng ngày đã chọn." : "Chưa có hồ sơ trong mục lục và kho gom còn trống. Bấm Thu thập ở tab Gom từ máy nhân viên."}
              </p>
            ) : (
              <Table.Root className="kg-table" maxHeight="min(62vh, 640px)">
                  <thead>
                    <tr>
                      <th className="kg-corner kg-c1">Hộp</th>
                      <th className="kg-corner kg-c2">Hồ sơ</th>
                      <th className="kg-corner kg-c3">Tổng tệp</th>
                      {matrix.columns.map((c) => (
                        <th key={c.key} className="kg-colhead">
                          <div className="kg-colname">
                            <Icon name={c.isAdmin ? "mayTram" : "nguoi"} size={15} /> <b>{c.label}</b>
                          </div>
                          <div className="text-fg-subtle">
                            {fmtN(c.tally.total)} tài liệu · {mb(c.tally.sizeBytes)} MB
                          </div>
                        </th>
                      ))}
                    </tr>
                  </thead>
                  <tbody>
                    {matrix.groups.map((g) => (
                      <Fragment key={g.key}>
                        <tr className="kg-grouprow">
                          <th className="kg-rowhead kg-groupname" colSpan={3}>
                            <Icon name={g.key === V4ia ? "canhBao" : "thuMuc"} size={15} /> {g.label}
                          </th>
                          <td colSpan={matrix.columns.length}>
                            <span className="text-fg-subtle">{g.key === V4ia ? "Không ghép được vào hồ sơ nào trong mục lục. Kiểm tra phông và số hồ sơ." : `${fmtN(g.rows.length)} hồ sơ · ${fmtN(g.tally.total)} tài liệu đã gom`}</span>
                          </td>
                        </tr>
                        {g.rows.map((r) => (
                          <tr key={r.key}>
                            <th className="kg-rowhead kg-c1 kg-hoso">{r.hopSo || "—"}</th>
                            <th className="kg-rowhead kg-c2 kg-hoso">
                              <div>{r.hoSoSo || "—"}</div>
                              {r.tieuDe && <div className="text-fg-subtle">{r.tieuDe}</div>}
                            </th>
                            <th className="kg-rowhead kg-c3">
                              <div className="kg-count">
                                <span>{fmtN(r.tally.total)}</span>
                                {r.inTree && g.key !== V4ia && g.key !== "" && (
                                  <IconButton
                                    label={`Bổ sung tài liệu vào hộp ${r.hopSo || "—"}, hồ sơ ${r.hoSoSo || "—"}`}
                                    icon="them"
                                    size="sm"
                                    onClick={() => {
                                      setThemPhong(g.key);
                                      setThemHoSo(`${r.hopSo}/${r.hoSoSo}`);
                                      setTab("them");
                                    }}
                                  />
                                )}
                              </div>
                            </th>
                            {matrix.columns.map((c) => (
                              <td key={c.key} className="kg-cell">
                                <MatrixCell cell={r.cells.get(c.key)} col={c} queued={forwarding} onResend={(ids) => void guiLai(ids)} />
                              </td>
                            ))}
                          </tr>
                        ))}
                      </Fragment>
                    ))}
                  </tbody>
              </Table.Root>
            )}

            {(truncated || matrix.hiddenRows > 0 || matrix.unmatched > 0 || filtering || (filtered.unknown > 0 && V4j3(flt))) && (
              <p className="border-t border-border-subtle px-4 py-2 text-xs text-fg-subtle">
                {filtering && <>Đang lọc: hiện {fmtN(rows.length)}/{fmtN(collected.rows.length)} bản đã nạp. </>}
                {filtered.unknown > 0 && V4j3(flt) && <>Đã loại {fmtN(filtered.unknown)} tài liệu chưa có trong mục lục chỉnh lý (không xét được phòng ban, mảng, THBQ). </>}
                {truncated && <>Kho có {fmtN(collected.total)} bản, bảng chỉ nạp {fmtN(LOAD_MAX)} bản mới nhất. </>}
                {matrix.hiddenRows > 0 && <>Đang ẩn {fmtN(matrix.hiddenRows)} hồ sơ chưa có dữ liệu{!hideTouched.current ? " (mục lục lớn nên tự ẩn)" : ""}. </>}
                {matrix.unmatched > 0 && <>{fmtN(matrix.unmatched)} tài liệu chưa khớp cây thư mục (nhóm cuối bảng).</>}
              </p>
            )}
          </Card>

          {errRows.length > 0 && <BangLoi rows={errRows} queued={forwarding} onResend={(id, name) => void guiLai([id], name)} onClearAll={() => void xoaLoi()} />}

          <Card title="Đợt đã đẩy" padding="sm">
            <V4pp isSuper={!!isSuper} />
          </Card>
        </div>
      )}
      {tab === "doisoat" && !collected && (
        <Card className="mt-3">
          <ErrorBanner error="Chưa nạp được kho gom." onRetry={() => void load()} />
        </Card>
      )}
    </>
  );
}

function BangLoi({ rows, queued, onResend, onClearAll }: { rows: V43g[]; queued: boolean; onResend: (id: number, name: string) => void; onClearAll: () => void }) {
  const cmp = new Intl.Collator("vi", { numeric: true, sensitivity: "base" }).compare;
  const sorted = [...rows].sort((a, b) => cmp(a.phongMa ?? "", b.phongMa ?? "") || cmp(a.hoSoRef ?? "", b.hoSoRef ?? "") || cmp(a.filename, b.filename) || a.id - b.id);
  return (
    <Card
      tone="danger"
      padding="none"
      title={`${fmtN(rows.length)} tệp lỗi chưa lên được máy chủ`}
      actions={
        <Button size="sm" icon="lamMoi" disabled={queued} onClick={onClearAll}>
          Xoá lỗi, thử lại tất cả
        </Button>
      }
    >
      <Table.Root className="kg-errtbl" maxHeight={320} dense>
          <thead>
            <tr>
              <th>Tệp</th>
              <th>Phông / hộp / hồ sơ</th>
              <th>Lý do</th>
              <th>Đường dẫn trong Kho</th>
              <th>Thu thập lúc</th>
              <th />
            </tr>
          </thead>
          <tbody>
            {sorted.map((r) => (
              <tr key={r.id}>
                <td>{r.filename}</td>
                <td>{[r.phongMa, r.hoSoRef].filter(Boolean).join(" / ") || "—"}</td>
                <td className="kg-why">
                  {r.forwardFatal ? <Badge tone="err">Máy chủ từ chối</Badge> : <Badge tone="warn">Lỗi {r.forwardTries} lần</Badge>}
                  {r.forwardError ? <span className="ml-1 text-fg-subtle">{r.forwardError}</span> : ""}
                </td>
                <td>
                  <Tip text={r.storagePath}>
                    <code className="kg-path">{r.storagePath}</code>
                  </Tip>
                </td>
                <td>{V4q0(r.collectedAt)}</td>
                <td>
                  <IconButton label={queued ? "Xếp hàng gửi lại, chạy khi phiên hiện tại xong" : "Gửi lại lên máy chủ"} icon="lamMoi" size="sm" onClick={() => onResend(r.id, r.filename)} />
                </td>
              </tr>
            ))}
          </tbody>
      </Table.Root>
    </Card>
  );
}

function MatrixCell({ cell, col, queued, onResend }: { cell: V4ij | undefined; col: V4ii; queued: boolean; onResend: (ids: number[]) => void }) {
  if (!cell || cell.files.length === 0) {
    return (
      <span className="kg-empty" aria-label={`${col.label}: chưa có tài liệu`}>
        ·
      </span>
    );
  }
  const t = cell.tally;
  const retryIds = V4iy(cell);
  return (
    <Tip text={`${col.label}: ${V4ix(t)} · ${mb(t.sizeBytes)} MB`}>
      <div className="kg-files">
        <span className={`kg-dot ${V4iw(t)}`} />
        <span className="kg-fcount">
          <Icon name="taiLieu" /> ({fmtN(t.total)})
        </span>
        {retryIds.length > 0 && <IconButton label={queued ? `Xếp hàng gửi lại ${retryIds.length} tài liệu` : `Gửi lại ${retryIds.length} tài liệu`} icon="lamMoi" size="sm" onClick={() => onResend(retryIds)} />}
      </div>
    </Tip>
  );
}
