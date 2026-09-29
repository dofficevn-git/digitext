// Copyright (c) 2026 DigiText. All rights reserved.
// Văn bản của một hồ sơ trên máy này: bảng, thêm / sửa / xoá (đơn và nhiều), tìm nhanh.

import { useCallback, useMemo, useState } from "react";
import { V41p } from "@/live/useLive";
import { V4by, V4bz, V4c0, V4c1, type V4bm, type V4bp, type V4bq } from "@/api/dossier";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import V4ck from "./VanBanForm";
import { V4cu, V4cw, V4cx, V4cy, V4cz } from "./logic";
import { Button, Card, confirm, IconButton, Input, notifyError, PageHeader, Table, toast, Toolbar, type Column } from "@/ui";

type Draft = { mode: "add" | "edit"; draft: V4bp; id?: number } | null;
const n = (v: number) => v.toLocaleString("vi-VN");

export default function V4dy({ dossier, onBack }: { dossier: V4bm; onBack: () => void }) {
  const { session } = V460();
  const canEdit = V3z9(session?.permissions, "dossier.update");
  const [rows, setRows] = useState<V4bq[]>([]);
  const [loaded, setLoaded] = useState(false);
  const [draft, setDraft] = useState<Draft>(null);
  const [q, setQ] = useState("");
  const [sel, setSel] = useState<Set<string>>(new Set());
  const [busy, setBusy] = useState(false);

  const reload = useCallback(() => {
    V4by(dossier.id)
      .then((rs) => {
        setRows(rs);
        setLoaded(true);
      })
      .catch((e) => {
        setLoaded(true);
        notifyError(e, "nạp văn bản");
      });
  }, [dossier.id]);
  V41p(reload, { topics: ["dossier"] });

  const shown = useMemo(() => {
    const k = q.trim().toLowerCase();
    return k ? rows.filter((v) => [v.trichYeu, v.soKyHieu, v.coQuanBanHanh, v.loaiVanBan].filter(Boolean).join(" ").toLowerCase().includes(k)) : rows;
  }, [rows, q]);

  const luu = async () => {
    if (!draft) return;
    const err = V4cz(draft.draft);
    if (err) {
      toast.error(err);
      return;
    }
    setBusy(true);
    try {
      const body = V4cy(draft.draft);
      if (draft.mode === "add") await V4bz(dossier.id, body);
      else if (draft.id != null) await V4c0(draft.id, body);
      setDraft(null);
      reload();
      toast.success(draft.mode === "add" ? "Đã thêm văn bản." : "Đã lưu văn bản.");
    } catch (e) {
      notifyError(e, "lưu văn bản");
    } finally {
      setBusy(false);
    }
  };
  const xoa = async (ids: number[]) => {
    const ok = await confirm({ title: ids.length === 1 ? "Xoá văn bản này?" : `Xoá ${n(ids.length)} văn bản đã chọn?`, body: "Không thể hoàn tác.", danger: true });
    if (!ok) return;
    try {
      await Promise.all(ids.map((id) => V4c1(id)));
      setSel(new Set());
      reload();
      toast.success(`Đã xoá ${n(ids.length)} văn bản.`);
    } catch (e) {
      notifyError(e, "xoá văn bản");
    }
  };

  const cols: Column<V4bq>[] = [
    { key: "thuTu", header: "STT", align: "right", width: 60, sortable: true },
    { key: "loaiVanBan", header: "Loại VB", width: 120, sortable: true },
    { key: "soKyHieu", header: "Số, ký hiệu", width: 140, sortable: true },
    { key: "ngayThang", header: "Ngày", width: 100 },
    { key: "coQuanBanHanh", header: "Cơ quan ban hành", className: "whitespace-normal", hideBelow: 1366 },
    { key: "trichYeu", header: "Trích yếu", className: "whitespace-normal min-w-64" },
    { key: "toSo", header: "Tờ số", align: "right", width: 70 },
    { key: "soTrang", header: "Trang", align: "right", width: 70 },
  ];

  return (
    <>
      <PageHeader
        onBack={onBack}
        title={`Văn bản của hồ sơ ${dossier.soTam}`}
        description={V4cu(dossier)}
        actions={
          canEdit && (
            <Button variant="primary" icon="them" onClick={() => setDraft({ mode: "add", draft: V4cw() })}>
              Thêm văn bản
            </Button>
          )
        }
      />
      <Card padding="none">
        <div className="border-b border-border-subtle p-3">
          <Toolbar>
            <Input size="sm" prefix="tim" value={q} onChange={(e) => setQ(e.target.value)} placeholder="Trích yếu, số ký hiệu, cơ quan" className="w-72" aria-label="Tìm văn bản" />
            <Toolbar.Spacer />
            {canEdit && sel.size > 0 && (
              <Button size="sm" variant="dangerOutline" icon="xoa" onClick={() => void xoa([...sel].map(Number))}>
                Xoá {n(sel.size)}
              </Button>
            )}
            <span className="text-sm text-fg-subtle">{n(shown.length)} văn bản</span>
          </Toolbar>
        </div>
        <Table
          columns={cols}
          rows={shown}
          getRowId={(v) => String(v.id)}
          selection={canEdit ? { selected: sel, onChange: setSel } : undefined}
          rowActions={canEdit ? (v) => (
            <>
              <IconButton label="Sửa" icon="sua" size="sm" onClick={() => setDraft({ mode: "edit", draft: V4cx(v), id: v.id })} />
              <IconButton label="Xoá" icon="xoa" size="sm" onClick={() => void xoa([v.id])} />
            </>
          ) : undefined}
          state={{ loading: !loaded, empty: { title: q ? "Không có văn bản khớp tìm kiếm" : "Hồ sơ chưa có văn bản", description: canEdit && !q ? "Thêm tay, nhập Excel ở màn Tài liệu đã quét, hoặc soát OCR từ PDF." : undefined } }}
          maxHeight="calc(100dvh - 300px)"
        />
      </Card>
      {draft && <V4ck mode={draft.mode} draft={draft.draft} busy={busy} onChange={(p) => setDraft({ ...draft, draft: { ...draft.draft, ...p } })} onClose={() => setDraft(null)} onSave={() => void luu()} />}
    </>
  );
}
