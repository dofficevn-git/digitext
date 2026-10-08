// Copyright (c) 2026 DigiText. All rights reserved.
// Kho máy chủ: cây tài liệu đã đẩy lên (theo dự án › hộp › hồ sơ), tải xuống theo loại bản, soát PDF/A lỗi,
// ghép lại PDF/A; tab "Đợt đã đẩy" xem lịch sử các đợt nộp.

import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { V41p } from "@/live/useLive";
import { useDownloads, useNavigation, V460 } from "@/state";
import { isTauri } from "@/api/ipc";
import { docsForGroups, docsReviewSummary, V4fb, V4fc, type DownloadKind, type V4ej, type V4f9, type ReviewSummaryRow } from "@/api/docs";
import { isPdfaOnlyDoc, isSignedDoc, V4vi, V4vj, V4vt, V4vx, V4vy, V4vz, V4w2, filesUnder, findFolderByRelPath, groupKeyOf, leavesUnder, treeFromSummary, type FolderGroup, type V4vl, type V4vu } from "./serverTree";
import { reviewKeyOf } from "@/features/dossier/reviewScope";
import { V4w5, V4w9 } from "./pdfa";
import { V4k1 } from "./progress";
import "./server.css";
import { Icon } from "@/components/Icon";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import V4pp from "@/features/upload/BatchesPanel";
import { Badge, Banner, Button, Card, EmptyState, IconButton, Menu, notifyError, PageHeader, Select, Skeleton, Tabs, Tip, toast, Table } from "@/ui";

const DL_DIR_KEY = "server.download.dir";
function nhoThuMuc(): string | undefined {
  try {
    return localStorage.getItem(DL_DIR_KEY) ?? undefined;
  } catch {
    return undefined;
  }
}
function luuThuMuc(d: string): void {
  try {
    localStorage.setItem(DL_DIR_KEY, d);
  } catch {
    /* bỏ qua */
  }
}
const mb = (b: number) => (b / 1_048_576).toFixed(1);
const n = (v: number) => v.toLocaleString("vi-VN");
const INDENT = 18;
const HIT_MS = 6000;
const TEN_LOAI: Record<DownloadKind, string> = { original: "bản gốc", pdfa: "PDF/A chưa ký số", signed: "đã ký số" };

function statusTone(status: string): "ok" | "err" | "info" | "gray" | "warn" {
  if (status === "signed" || status === "archived") return "ok";
  if (status === "sign_rejected") return "err";
  if (status === "to_sign" || status === "signing" || status === "ocr_done") return "info";
  if (status === "disposed") return "gray";
  return "warn";
}

function Tick({ state, onToggle, title }: { state: V4vx; onToggle: () => void; title: string }) {
  return (
    <input
      type="checkbox"
      className="srv-tick"
      aria-label={title}
      checked={state === "all"}
      ref={(el) => {
        if (el) el.indeterminate = state === "some";
      }}
      onChange={onToggle}
      onClick={(e) => e.stopPropagation()}
    />
  );
}

type Tab = "cay" | "dot";

export default function V4wh() {
  const { serverReveal, clearServerReveal } = useNavigation();
  const { session } = V460();
  const [tab, setTab] = useState<Tab>("cay");
  const [reveal, setReveal] = useState<{ relPath: string; filename: string } | null>(null);
  const [hit, setHit] = useState<{ folderPath: string; filename: string } | null>(null);
  const hitRef = useRef<HTMLTableRowElement | null>(null);
  const [summary, setSummary] = useState<ReviewSummaryRow[] | null>(null);
  // Tệp của từng thư mục lá, nạp khi mở / tích chọn / tải cả thư mục.
  const [docsByPath, setDocsByPath] = useState<ReadonlyMap<string, V4ej[]>>(() => new Map());
  const docsRef = useRef(docsByPath);
  docsRef.current = docsByPath;
  const [openSet, setOpenSet] = useState<Set<string>>(new Set());
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [loading, setLoading] = useState(false);
  const [pdfa, setPdfa] = useState<V4f9[] | null>(null);
  const [pdfaBusy, setPdfaBusy] = useState(false);
  const [ghepLai, setGhepLai] = useState(false);
  const dlw = useDownloads();
  const [loaiTai, setLoaiTai] = useState<DownloadKind>("original");
  const firstOpen = useRef(false);

  const load = useCallback(async () => {
    if (!isTauri() && !import.meta.env.DEV) return;
    setLoading(true);
    try {
      // Số liệu gộp theo hồ sơ (vài nghìn dòng) thay cho cả cây 160.000 tài liệu; tệp nạp khi mở.
      setSummary(await docsReviewSummary({}));
      setDocsByPath(new Map());
    } catch (e) {
      notifyError(e, "tải kho máy chủ");
    } finally {
      setLoading(false);
    }
  }, []);
  V41p(load, { topics: ["docs", "ocr", "sign", "uploadBatches"] });

  const built = useMemo(() => (summary ? treeFromSummary(summary) : null), [summary]);
  const tree = built?.root ?? null;
  const groupOf = useMemo<ReadonlyMap<string, FolderGroup>>(() => built?.groupOf ?? new Map(), [built]);
  const pathOfKey = useMemo<ReadonlyMap<string, string>>(() => built?.pathOfKey ?? new Map(), [built]);
  useEffect(() => {
    if (tree && !firstOpen.current) {
      firstOpen.current = true;
      setOpenSet(V4vt(tree));
    }
  }, [tree]);

  // Nạp tệp cho mọi thư mục lá chưa có dưới `f`; trả về bộ đệm mới.
  const napTep = useCallback(
    async (f: V4vl): Promise<ReadonlyMap<string, V4ej[]>> => {
      const cur = docsRef.current;
      const can = leavesUnder(f, groupOf).filter((l) => !cur.has(l.path));
      if (can.length === 0) return cur;
      const docs = await docsForGroups(can.map((l) => groupOf.get(l.path)!));
      const moi = new Map(docsRef.current);
      for (const l of can) moi.set(l.path, []);
      const col = new Intl.Collator("vi", { numeric: true });
      for (const d of docs) {
        if (d.status === "disposed") continue;
        const path = pathOfKey.get(reviewKeyOf(d)) ?? pathOfKey.get(groupKeyOf({ projectId: d.projectId ?? "", phong: "", hop: "", hoSo: "" }));
        if (!path) continue;
        moi.set(path, [...(moi.get(path) ?? []), d]);
      }
      for (const l of can) moi.set(l.path, [...(moi.get(l.path) ?? [])].sort((a, b) => col.compare(a.filename, b.filename)));
      docsRef.current = moi;
      setDocsByPath(moi);
      return moi;
    },
    [groupOf, pathOfKey],
  );

  const allFiles = useMemo<V4vu[]>(() => (tree ? filesUnder(tree, docsByPath) : []), [tree, docsByPath]);
  const idsByFolder = useMemo<Map<string, string[]>>(() => {
    const m = new Map<string, string[]>();
    if (!tree) return m;
    const walk = (f: V4vl): string[] => {
      const ids: string[] = [];
      for (const c of f.folders) ids.push(...walk(c));
      for (const d of docsByPath.get(f.path) ?? []) ids.push(d.id);
      m.set(f.path, ids);
      return ids;
    };
    walk(tree);
    return m;
  }, [tree, docsByPath]);
  const allDocs = useMemo<V4ej[]>(() => allFiles.map((f) => f.doc), [allFiles]);
  const pdfaBad = useMemo(() => (pdfa ? V4w9(pdfa, allDocs) : []), [pdfa, allDocs]);
  const pdfaBadIds = useMemo(() => new Set(pdfaBad.map((p) => p.id)), [pdfaBad]);

  useEffect(() => {
    if (!serverReveal) return;
    setReveal(serverReveal);
    clearServerReveal();
    setTab("cay");
  }, [serverReveal, clearServerReveal]);
  useEffect(() => {
    if (!reveal || !tree) return;
    const folder = findFolderByRelPath(tree, reveal.relPath);
    setReveal(null);
    if (folder === null) {
      toast.warning(`Chưa thấy "${reveal.filename}" trong kho máy chủ. Có thể chưa đồng bộ xong.`);
      return;
    }
    const node = (() => {
      let cur: V4vl | undefined = tree;
      for (const seg of V4vz(folder)) cur = cur?.folders.find((c) => c.path === seg);
      return cur;
    })();
    void (async () => {
      if (node) await napTep(node).catch((e) => notifyError(e, "nạp tệp của hồ sơ"));
      setOpenSet((s) => new Set([...s, ...V4vz(folder)]));
      firstOpen.current = true;
      setHit({ folderPath: folder, filename: reveal.filename });
    })();
  }, [reveal, tree, napTep]);
  useEffect(() => {
    if (!hit) return;
    hitRef.current?.scrollIntoView({ block: "center", behavior: "smooth" });
    const t = setTimeout(() => setHit(null), HIT_MS);
    return () => clearTimeout(t);
  }, [hit, tree]);

  const toggleOpen = (f: V4vl) => {
    const path = f.path;
    const dangMo = openSet.has(path);
    setOpenSet((s) => {
      const m = new Set(s);
      if (m.has(path)) m.delete(path);
      else m.add(path);
      return m;
    });
    if (!dangMo && groupOf.has(path) && !docsByPath.has(path)) void napTep(f).catch((e) => notifyError(e, "nạp tệp của hồ sơ"));
  };
  // Tích chọn cả thư mục: nạp tệp bên dưới trước (mỗi lượt tối đa 500 hồ sơ).
  const chonThuMuc = async (f: V4vl) => {
    try {
      const m = await napTep(f);
      const ids = filesUnder(f, m).map((x) => x.doc.id);
      toggleSel(ids);
    } catch (e) {
      notifyError(e, "chọn thư mục");
    }
  };
  const taiThuMuc = async (f: V4vl) => {
    try {
      const m = await napTep(f);
      await taiXuong(filesUnder(f, m), `trong "${f.name}"`);
    } catch (e) {
      notifyError(e, "nạp tệp để tải");
    }
  };
  const toggleSel = (ids: readonly string[]) =>
    setSel((s) => {
      const m = new Set(s);
      if (V4vy(ids, s) === "all") for (const id of ids) m.delete(id);
      else for (const id of ids) m.add(id);
      return m;
    });
  const selected = useMemo(() => allFiles.filter((f) => sel.has(f.doc.id)), [allFiles, sel]);
  const selectedPages = selected.reduce((s, f) => s + (f.doc.pageCount ?? 0), 0);

  // Chọn thư mục rồi giao cho bộ tải cấp ứng dụng (state/downloads): chuyển màn vẫn tải tiếp, tiến độ ở góc màn hình.
  const taiXuong = async (files: V4vu[], label: string) => {
    if (files.length === 0) return;
    if (dlw.running) {
      toast.warning("Đang có đợt tải chưa xong — đợi xong hoặc dừng đợt đó trước.");
      return;
    }
    let destDir: string;
    try {
      const r = await openDialog({ directory: true, multiple: false, defaultPath: nhoThuMuc(), title: `Chọn thư mục lưu tệp ${TEN_LOAI[loaiTai]} (giữ nguyên cây thư mục)` });
      if (typeof r !== "string" || !r) return;
      destDir = r;
    } catch (e) {
      notifyError(e, "mở hộp chọn thư mục");
      return;
    }
    luuThuMuc(destDir);
    const loai = loaiTai;
    dlw.start(files, loai, destDir, label, (d) =>
      loai === "pdfa" && !isPdfaOnlyDoc(d)
        ? isSignedDoc(d)
          ? "tệp đã ký số (tải ở loại Đã ký số)"
          : "tệp chưa có bản PDF/A"
        : loai === "signed" && !isSignedDoc(d)
          ? "tệp chưa ký số"
          : null,
    );
  };

  const soatPdfa = async () => {
    setPdfaBusy(true);
    try {
      const r = await V4fc();
      setPdfa(r);
      if (V4w9(r, allDocs).length === 0) toast.success(`Đã soát ${n(r.length)} tài liệu, không tệp nào lỗi PDF/A.`);
    } catch (e) {
      notifyError(e, "soát PDF/A");
    } finally {
      setPdfaBusy(false);
    }
  };
  const ghepLaiPdfa = async () => {
    setGhepLai(true);
    try {
      const r = await V4fb();
      toast.success(`Đã kiểm ${n(r.checked)} tài liệu, xếp hàng ghép lại ${n(r.enqueued)}.`, { description: r.skippedSigned > 0 ? `${n(r.skippedSigned)} tài liệu đã ký chưa đạt PDF/A, phải ký lại theo quy trình.` : undefined });
      if (pdfa) setPdfa(await V4fc());
    } catch (e) {
      notifyError(e, "ghép lại PDF/A");
    } finally {
      setGhepLai(false);
    }
  };
  const diToi = (folderPath: string, filename: string) => {
    setOpenSet((s) => new Set([...s, ...V4vz(folderPath)]));
    firstOpen.current = true;
    setHit({ folderPath, filename });
  };

  const rows: ReactNode[] = [];
  if (tree) {
    const docRow = (d: V4ej, depth: number, folderPath: string) => {
      const isHit = V4w2(folderPath, d.filename, hit);
      rows.push(
        <tr key={d.id} className={isHit ? "srv-hit" : undefined} ref={isHit ? hitRef : undefined}>
          <td className="srv-tickcell">
            <Tick state={sel.has(d.id) ? "all" : "none"} onToggle={() => toggleSel([d.id])} title={`Chọn ${d.filename}`} />
          </td>
          <td>
            <span className="srv-name" style={{ paddingLeft: depth * INDENT + INDENT }}>
              <Icon name="taiLieu" /> {d.filename}
              <Badge tone={statusTone(d.status)}>{V4k1(d.status)}</Badge>
              {isSignedDoc(d) ? <Badge tone="ok">Đã ký số</Badge> : isPdfaOnlyDoc(d) ? <Badge tone="gray">PDF/A</Badge> : null}
              {pdfaBadIds.has(d.id) && <Badge tone="err">PDF/A lỗi</Badge>}
              <span className="srv-size">{mb(d.sizeBytes)} MB</span>
            </span>
          </td>
          <td className="num srv-count" />
          <td className="num srv-count" />
          <td className="num srv-count" />
          <td className="num">{d.pageCount ?? "—"}</td>
          <td className="srv-count">{isSignedDoc(d) ? "Đã ký" : isPdfaOnlyDoc(d) ? "PDF/A" : ""}</td>
          <td>
            <IconButton label={`Tải ${TEN_LOAI[loaiTai]}`} icon="taiVe" size="sm" disabled={dlw.running} onClick={() => void taiXuong([{ doc: d, folderPath }], `"${d.filename}"`)} />
          </td>
        </tr>,
      );
    };
    const walk = (f: V4vl, depth: number) => {
      for (const sub of f.folders) {
        const isOpen = openSet.has(sub.path);
        const ids = idsByFolder.get(sub.path) ?? [];
        rows.push(
          <tr key={`f:${sub.path}`} className="srv-folder">
            <td className="srv-tickcell">
              <Tick state={V4vy(ids, sel)} onToggle={() => void chonThuMuc(sub)} title={`Chọn cả ${sub.name} (${n(sub.docCount)} tệp)`} />
            </td>
            <td>
              <button type="button" className="srv-name w-full text-left" style={{ paddingLeft: depth * INDENT }} onClick={() => toggleOpen(sub)} aria-expanded={isOpen}>
                <span className="srv-caret" aria-hidden>
                  <Icon name={isOpen ? "xuong" : "moNhanh"} size={13} />
                </span>
                <Icon name={sub.depth === 0 ? "hoSo" : "thuMuc"} size={15} /> {sub.name}
              </button>
            </td>
            <td className="num srv-count">{sub.boxCount ?? "—"}</td>
            <td className="num srv-count">{sub.dossierCount ?? "—"}</td>
            <td className="num srv-count">{n(sub.docCount)}</td>
            <td className="num srv-count">{n(sub.pageCount)}</td>
            <td className="srv-count whitespace-nowrap">
              {sub.signedCount > 0 && <Badge tone="ok">{n(sub.signedCount)} đã ký</Badge>} {sub.pdfaOnlyCount > 0 && <Badge tone="gray">{n(sub.pdfaOnlyCount)} PDF/A</Badge>}
            </td>
            <td>
              <IconButton label={`Tải cả ${n(sub.docCount)} tệp trong ${sub.name}`} icon="taiVe" size="sm" disabled={dlw.running || sub.docCount === 0} onClick={() => void taiThuMuc(sub)} />
            </td>
          </tr>,
        );
        if (isOpen) walk(sub, depth + 1);
      }
      const docsO = docsByPath.get(f.path);
      if (groupOf.has(f.path) && docsO === undefined && openSet.has(f.path)) {
        rows.push(
          <tr key={`l:${f.path}`}>
            <td />
            <td colSpan={7} className="text-sm text-fg-subtle" style={{ paddingLeft: depth * INDENT + INDENT }}>
              Đang nạp tệp…
            </td>
          </tr>,
        );
      }
      for (const d of docsO ?? []) docRow(d, depth, f.path);
    };
    walk(tree, 0);
  }
  const rootIds = idsByFolder.get("") ?? [];

  return (
    <section className="view active">
      <PageHeader
        title="Kho máy chủ"
        description="Tài liệu đã đẩy lên máy chủ theo dự án, hộp, hồ sơ. Tải về, soát PDF/A. Chạy OCR và bóc trường ở mục Xử lý."
        actions={
          tab === "cay" &&
          (
            <>
              <Select size="md" value={loaiTai} onChange={(v) => setLoaiTai(v as DownloadKind)} options={[{ value: "original", label: "Bản gốc" }, { value: "pdfa", label: "PDF/A chưa ký số" }, { value: "signed", label: "Đã ký số" }]} className="w-44" aria-label="Loại bản tải" />
              <Button variant="primary" icon="taiVe" disabled={selected.length === 0 || dlw.running} onClick={() => void taiXuong(selected, "đã chọn")}>
                Tải xuống ({n(selected.length)})
              </Button>
              <Menu
                items={[
                  { label: "Soát PDF/A lỗi", icon: "kiemTraPdfa", onSelect: () => void soatPdfa(), disabled: pdfaBusy },
                  { label: "Ghép lại PDF/A (bản cũ chưa đạt 2b)", icon: "ghepLai", onSelect: () => void ghepLaiPdfa(), disabled: ghepLai },
                  "-",
                  { label: "Làm mới", icon: "lamMoi", onSelect: () => void load(), disabled: loading },
                ]}
              />
            </>
          )
        }
      />

      <Tabs<Tab> value={tab} onChange={setTab} items={[{ value: "cay", label: "Cây tài liệu", icon: "hoSo", count: tree?.docCount }, { value: "dot", label: "Đợt đã đẩy", icon: "taiLen" }]} />

      {tab === "cay" && (
        <div className="mt-3 space-y-4">
          {pdfa && pdfaBad.length > 0 && (
            <Card tone="danger" padding="none" title={`${n(pdfaBad.length)} tệp PDF/A lỗi`} description={`Đã soát ${n(pdfa.length)} tài liệu (mỗi lượt tối đa 1.000). Sửa bằng Ghép lại PDF/A trong menu.`} actions={<IconButton label="Đóng" icon="dong" size="sm" onClick={() => setPdfa(null)} />}>
              <Table.Root maxHeight={256} dense>
                  <thead>
                    <tr>
                      <th>Lỗi</th>
                      <th>Đường dẫn</th>
                      <th>Lý do</th>
                      <th />
                    </tr>
                  </thead>
                  <tbody>
                    {pdfaBad.map((p) => (
                      <tr key={p.id}>
                        <td>
                          <Badge tone={p.kind === "not-pdfa" ? "warn" : "err"}>{V4w5(p.kind)}</Badge>
                        </td>
                        <td className="srv-path">
                          <Tip text={p.path}>
                            <span>{p.path}</span>
                          </Tip>
                        </td>
                        <td className="text-sm text-fg-subtle">{p.reason}</td>
                        <td>
                          <Button size="sm" disabled={p.folderPath === ""} onClick={() => diToi(p.folderPath, p.filename)}>
                            Đi tới
                          </Button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
              </Table.Root>
            </Card>
          )}
          {pdfa && pdfaBad.length === 0 && (
            <Banner tone="success" onClose={() => setPdfa(null)}>
              Đã soát {n(pdfa.length)} tài liệu, không tệp nào lỗi PDF/A.
            </Banner>
          )}

          <Card padding="none">
            <div className="flex flex-wrap items-center gap-3 border-b border-border-subtle px-4 py-2.5 text-sm text-fg-subtle">
              <span>
                {n(tree?.docCount ?? 0)} tệp · {n(tree?.pageCount ?? 0)} trang · {mb(tree?.sizeBytes ?? 0)} MB
              </span>
              {selected.length > 0 && (
                <span className="text-fg">
                  Đã chọn {n(selected.length)} tệp ({n(selectedPages)} trang)
                </span>
              )}
              {loading && tree && <span className="ml-auto text-xs">Đang cập nhật…</span>}
            </div>
            {!tree ? (
              loading ? (
                <Skeleton.Rows rows={6} cols={4} />
              ) : (
                <EmptyState compact title="Chưa nạp được kho máy chủ" action={<Button onClick={() => void load()}>Thử lại</Button>} />
              )
            ) : tree.docCount === 0 ? (
              <EmptyState compact icon="mayChu" title="Chưa có tài liệu trên máy chủ" description="Máy quản trị gom rồi đẩy lên ở màn Đưa lên máy chủ." />
            ) : (
              <Table.Root className="srv-tree">
                  <thead>
                    <tr>
                      <th className="srv-tickcell">
                        <Tick state={V4vy(rootIds, sel)} onToggle={() => void (tree && chonThuMuc(tree))} title="Chọn tất cả (nạp tệp của mọi hồ sơ)" />
                      </th>
                      <th>Dự án / hộp / hồ sơ</th>
                      <th className="num">
                        <Tip text={`Cấp ${V4vi + 1} của cây`}>
                          <span>Hộp</span>
                        </Tip>
                      </th>
                      <th className="num">
                        <Tip text={`Cấp ${V4vj + 1} của cây`}>
                          <span>Hồ sơ</span>
                        </Tip>
                      </th>
                      <th className="num">Tệp</th>
                      <th className="num">Trang</th>
                      <th>Ký số / PDF/A</th>
                      <th />
                    </tr>
                  </thead>
                  <tbody>{rows}</tbody>
              </Table.Root>
            )}
          </Card>
        </div>
      )}

      {tab === "dot" && (
        <div className="mt-3">
          <V4pp isSuper={session?.role === "super_admin"} />
        </div>
      )}
    </section>
  );
}
