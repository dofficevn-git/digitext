// Copyright (c) 2026 DigiText. All rights reserved.
// Màn bóc tách trường một tài liệu: trang ảnh bên trái (khung ô kéo được), 16 trường bên phải, thanh Lưu / Duyệt
// cố định đáy. `workMode`: cộng tác viên làm mục trong gói việc, chỉ Lưu; `footer`: nút thêm (Trước / Tiếp / Xác nhận).

import { useCallback, useEffect, useMemo, useRef, useState, type ReactNode } from "react";
import { V41p } from "@/live/useLive";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import { V4en, V4f5, V4ep, type V4eh } from "@/api/docs";
import V4hr, { type V4hp } from "../dossier/ReviewControls";
import { V40p } from "@/api/archive";
import { V4fh, V4fg, type V4fe } from "@/api/ocr";
import { V4hz, V4hv, V4hw, V4hu, V4i0, type V4hy } from "./pageChunks";
import { V4gv, V4gt, type V4gk } from "../dossier/ocrBoxes";
import V4h3 from "../dossier/OcrBoxLayer";
import { V4gb, V4fq, V4fo, V4fw, V4g8, V4fz, V4g7, V4fn, V4fv, V4fy, type V4fs } from "./ocrLogic";
import { Badge, Banner, Button, ErrorBanner, IconButton, Input, Skeleton, Tip, errorText, notifyError, toast } from "@/ui";

const khoa = (pageNo: number, key: string) => `${pageNo}|${key}`;
const CHUA_CO = "Chưa có, nhập tay nếu văn bản có";
const coGiaTri = (m: Record<string, unknown>) => Object.keys(m).length > 0;

function TextareaTuGian({ value, onChange, placeholder, id }: { value: string; onChange: (v: string) => void; placeholder?: string; id?: string }) {
  const ref = useRef<HTMLTextAreaElement>(null);
  const [scroll, setScroll] = useState(false);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    el.style.height = "auto";
    const g = V4gb(el.scrollHeight);
    el.style.height = `${g.height}px`;
    setScroll(g.scroll);
  }, [value]);
  return <textarea ref={ref} id={id} className="inp w-full resize-none box-border" rows={1} placeholder={placeholder} value={value} onChange={(e) => onChange(e.target.value)} style={{ overflowY: scroll ? "auto" : "hidden" }} />;
}

export interface V4i6 {
  id: string;
  filename: string;
  sizeBytes?: number;
  reviewStatus: V4eh | string;
  reviewedAt?: string | null;
  reviewNote?: string | null;
}

export default function V4i7({ doc, onBack, workMode = false, footer }: { doc: V4i6; onBack: () => void; workMode?: boolean; footer?: ReactNode }) {
  const { session } = V460();
  const coQuyenDuyet = !workMode && V3z9(session?.permissions, "document.review");

  const [review, setReview] = useState<V4hp>({ id: doc.id, filename: doc.filename, reviewStatus: doc.reviewStatus, reviewedAt: doc.reviewedAt ?? null, reviewNote: doc.reviewNote ?? null });
  const [pages, setPages] = useState<string[]>([]);
  const [pdfPath, setPdfPath] = useState<string | null>(null);
  const [conTrang, setConTrang] = useState(false);
  const [dangNapThem, setDangNapThem] = useState(false);
  const [tienDoTai, setTienDoTai] = useState<V4hy | null>(null);
  const [loiTrang, setLoiTrang] = useState<string | null>(null);
  const cuon = useRef<HTMLDivElement>(null);
  const dangNap = useRef(false);
  const [kichThuoc, setKichThuoc] = useState<Record<number, { w: number; h: number }>>({});
  const [fieldsRaw, setFieldsRaw] = useState<V4fe[]>([]);
  const [drafts, setDrafts] = useState<Record<string, string>>({});
  const [bboxDrafts, setBboxDrafts] = useState<Record<string, number[]>>({});
  const [chonKhoa, setChonKhoa] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [err, setErr] = useState<unknown>(null);

  const napFields = useCallback(async () => {
    setFieldsRaw(await V4fg(doc.id));
  }, [doc.id]);
  V41p(napFields, { topics: ["ocrFields"], skipInitial: true });

  useEffect(() => {
    let off: (() => void) | null = null;
    let cancelled = false;
    void V4ep((p) => {
      if (p.id === doc.id) setTienDoTai({ received: p.received, total: p.total });
    }).then((u) => {
      if (cancelled) u();
      else off = u;
    });
    return () => {
      cancelled = true;
      off?.();
    };
  }, [doc.id]);

  useEffect(() => {
    let alive = true;
    setLoading(true);
    setErr(null);
    setDrafts({});
    setBboxDrafts({});
    setPages([]);
    setKichThuoc({});
    setPdfPath(null);
    setConTrang(false);
    setLoiTrang(null);
    setTienDoTai(null);
    dangNap.current = false;
    Promise.all([
      napFields().catch((e) => alive && setErr(e)),
      V4en(doc.id, doc.sizeBytes)
        .then(async (r) => {
          const { from, count } = V4hu(0);
          const p = await V40p(r.path, from, count);
          if (!alive) return;
          setPdfPath(r.path);
          setPages(p);
          setConTrang(V4hv(p.length, count));
        })
        .catch((e) => {
          if (!alive) return;
          setPages([]);
          setLoiTrang(errorText(e));
        }),
    ]).finally(() => {
      if (alive) setLoading(false);
    });
    return () => {
      alive = false;
    };
  }, [doc.id, doc.sizeBytes, napFields]);

  const napToiTrang = useCallback(
    async (den: number) => {
      if (!pdfPath || !conTrang || dangNap.current) return;
      dangNap.current = true;
      setDangNapThem(true);
      try {
        let co = pages.length;
        let con = true;
        const them: string[] = [];
        while (con && co < den) {
          const { from, count } = V4hu(co);
          const p = await V40p(pdfPath, from, count);
          them.push(...p);
          co += p.length;
          con = V4hv(p.length, count);
        }
        if (them.length > 0) setPages((cu) => [...cu, ...them]);
        setConTrang(con);
      } catch (e) {
        setLoiTrang(errorText(e));
        setConTrang(false);
      } finally {
        dangNap.current = false;
        setDangNapThem(false);
      }
    },
    [pdfPath, conTrang, pages.length],
  );
  const napThem = useCallback(() => napToiTrang(pages.length + 1), [napToiTrang, pages.length]);

  const fields = useMemo(() => V4fy(V4fv(fieldsRaw)), [fieldsRaw]);
  const soCoGiaTri = useMemo(() => V4fz(fields), [fields]);
  const thayDoi = useMemo(() => V4g8(fields, { drafts, bboxDrafts }), [fields, drafts, bboxDrafts]);
  const giaTri = (f: V4fs) => drafts[khoa(f.pageNo, f.fieldKey)] ?? f.value;
  const bbox = (f: V4fs) => bboxDrafts[khoa(f.pageNo, f.fieldKey)] ?? f.bbox;
  const chuaLuu = (f: V4fs) => V4g7(f, { drafts, bboxDrafts });
  const doiKhung = (b: V4gk) => setBboxDrafts((m) => ({ ...m, [b.key]: V4gv(b) }));

  const luu = async () => {
    setSaving(true);
    setErr(null);
    try {
      for (const s of thayDoi) {
        await V4fh({ documentId: doc.id, pageNo: s.pageNo, fieldKey: s.fieldKey, value: s.value, engineVersionId: s.engineVersionId || undefined, bbox: s.bbox ?? undefined, verified: true });
      }
      await napFields();
      setDrafts({});
      setBboxDrafts({});
      const chiKhoangTrang = thayDoi.length === 0 && (coGiaTri(drafts) || coGiaTri(bboxDrafts));
      const daLuu = thayDoi.length > 0 ? `Đã lưu ${thayDoi.length} trường.` : chiKhoangTrang ? "Không có thay đổi thực sự, chỉ khác khoảng trắng." : "Không có gì để lưu.";
      if (workMode) toast.success(daLuu, { description: "Kiểm tra đủ rồi thì bấm Xác nhận hoàn thành." });
      else if (!coQuyenDuyet) toast.success(daLuu, { description: "Bạn không có quyền duyệt nên trạng thái duyệt giữ nguyên." });
      else if (review.reviewStatus === "approved") toast.success(daLuu, { description: "Tài liệu đã ở trạng thái đã duyệt." });
      else {
        const r = await V4f5(doc.id, "approved");
        setReview((v) => ({ ...v, ...r }));
        toast.success(`${daLuu} Đã duyệt tài liệu này.`);
      }
    } catch (e) {
      setErr(e);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="card p-0 overflow-hidden">
      <div className="flex flex-wrap items-center gap-3 px-4 py-3">
        <IconButton label="Về danh sách" icon="truoc" onClick={onBack} />
        <b className="min-w-0 truncate">{doc.filename}</b>
        {!workMode && (
          <span className="whitespace-nowrap">
            <V4hr
              doc={review}
              canReview={coQuyenDuyet}
              showApprove={false}
              onReviewed={(r) => {
                setReview((v) => ({ ...v, ...r }));
                setErr(null);
                toast.success(r.reviewStatus === "approved" ? "Đã duyệt tài liệu này." : r.reviewStatus === "rejected" ? "Đã từ chối tài liệu này." : "Đã mở lại để soát tiếp.");
              }}
              onError={(m) => notifyError(m, "duyệt")}
            />
          </span>
        )}
        <div className="flex-1" />
        <IconButton label="Tải lại trường" icon="lamMoi" size="sm" onClick={() => void napFields()} />
      </div>

      {err !== null && <ErrorBanner error={err} className="mx-4 mb-3" />}

      <div className="flex items-start gap-3 p-3">
        <div
          ref={cuon}
          onScroll={(e) => {
            if (conTrang && !dangNapThem && V4hw(e.currentTarget)) void napThem();
          }}
          className="grow shrink basis-[55%] max-h-[72vh] overflow-auto"
        >
          {loading ? (
            <div className="p-4 text-sm text-fg-subtle">{V4hz(tienDoTai)}</div>
          ) : pages.length === 0 ? (
            <Banner tone={loiTrang ? "danger" : "info"} className="m-2">
              {loiTrang ?? "Không hiển thị được nội dung vì tệp không phải PDF ảnh."}
            </Banner>
          ) : (
            pages.map((src, i) => {
              const soTrang = i + 1;
              const size = kichThuoc[soTrang];
              const boxes = size
                ? V4fw(fields, soTrang)
                    .map((f) => V4gt(khoa(soTrang, f.fieldKey), bbox(f), giaTri(f), f.confidence, f.fieldKey))
                    .filter((b): b is V4gk => b != null)
                : [];
              return (
                <div key={i} className="mb-2.5 border border-border">
                  <V4h3
                    src={src}
                    boxes={boxes}
                    pageW={size?.w ?? 0}
                    pageH={size?.h ?? 0}
                    selectedKey={chonKhoa}
                    onSelect={setChonKhoa}
                    onChange={doiKhung}
                    boxLabel={(b) => V4fo(b.field ?? "")}
                    boxColorFn={(b) => V4fq(b.field ?? b.key)}
                    onNatural={(w, h) => setKichThuoc((m) => (m[soTrang]?.w === w && m[soTrang]?.h === h ? m : { ...m, [soTrang]: { w, h } }))}
                  />
                </div>
              );
            })
          )}
          {!loading && pages.length > 0 && (
            <div className="flex items-center gap-2 px-1 pb-3 pt-1.5 text-sm text-fg-subtle">
              <span>{V4i0(pages.length, conTrang, dangNapThem)}</span>
              {conTrang && !dangNapThem && (
                <Button size="sm" icon="xuong" onClick={() => void napThem()}>
                  Xem thêm trang
                </Button>
              )}
            </div>
          )}
          {loiTrang && pages.length > 0 && <p className="px-1 pb-3 text-sm text-danger-700">{loiTrang}</p>}
        </div>

        <div className="grow shrink basis-[45%] max-h-[72vh] overflow-auto">
          {!loading && (
            <p className="mb-2 px-0.5 text-sm text-fg-subtle">
              Đã có giá trị <b>{soCoGiaTri}</b>/{V4fn.length} trường. Trường trống là máy chưa bóc được, nhập tay rồi bấm {coQuyenDuyet ? "Duyệt" : "Lưu"} ở cuối màn.
            </p>
          )}
          {loading ? (
            <Skeleton.Rows rows={8} />
          ) : (
            fields.map((f) => {
              const k = khoa(f.pageNo, f.fieldKey);
              const inputId = `ocr-field-${k.replace(/[^a-zA-Z0-9]/g, "-")}`;
              const khungSua = k in bboxDrafts;
              const dangChon = chonKhoa === k;
              return (
                <div
                  key={k}
                  className="card mb-2 p-2.5"
                  style={{ borderLeft: `4px ${f.placeholder ? "dashed" : "solid"} ${V4fq(f.fieldKey)}`, outline: dangChon ? "2px solid var(--color-brand-500)" : "none" }}
                  onFocusCapture={() => {
                    setChonKhoa(k);
                    if (f.pageNo > pages.length) void napToiTrang(f.pageNo);
                  }}
                >
                  <div className="mb-1 flex flex-wrap items-center gap-2">
                    <span aria-hidden className="size-2.5 flex-none rounded-sm" style={{ background: V4fq(f.fieldKey) }} />
                    <label htmlFor={inputId} className="font-semibold">
                      {f.label}
                    </label>
                    <span className="text-xs text-fg-subtle">trang {f.pageNo}</span>
                    {f.placeholder ? <Badge tone="gray">chưa có</Badge> : f.corrected ? <Badge tone="ok">đã soát</Badge> : <Badge tone="gray">máy bóc</Badge>}
                    {khungSua && <Badge tone="gray">khung đã sửa</Badge>}
                    {chuaLuu(f) && <Badge tone="warn">chưa lưu</Badge>}
                    {f.confidence != null && !f.corrected && (
                      <Tip text="Độ tin cậy máy bóc">
                        <span className="text-xs text-fg-subtle">{Math.round(f.confidence * 100)}%</span>
                      </Tip>
                    )}
                    <div className="flex-1" />
                    {f.verified && <Badge tone="ok">đã xác nhận</Badge>}
                  </div>
                  {f.fieldKey === "trichYeu" ? (
                    <TextareaTuGian id={inputId} placeholder={f.placeholder ? CHUA_CO : undefined} value={giaTri(f)} onChange={(v) => setDrafts((d) => ({ ...d, [k]: v }))} />
                  ) : (
                    <Input id={inputId} placeholder={f.placeholder ? CHUA_CO : undefined} value={giaTri(f)} onChange={(e) => setDrafts((d) => ({ ...d, [k]: e.target.value }))} />
                  )}
                </div>
              );
            })
          )}
        </div>
      </div>

      <div className="sticky bottom-0 flex flex-wrap items-center gap-2.5 border-t border-border bg-surface px-4 py-2.5">
        <span className="text-sm text-fg-subtle">{thayDoi.length > 0 ? `${thayDoi.length} thay đổi chưa lưu` : "Không có thay đổi chưa lưu"}</span>
        <div className="flex-1" />
        {footer}
        <Button onClick={onBack} disabled={saving}>
          Đóng
        </Button>
        <Button variant="primary" icon={coQuyenDuyet ? "duyet" : "luu"} loading={saving} disabled={loading} onClick={() => void luu()}>
          {coQuyenDuyet ? "Duyệt" : "Lưu"}
        </Button>
      </div>
    </div>
  );
}
