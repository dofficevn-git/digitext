// Copyright (c) 2026 DigiText. All rights reserved.
// Kho tài liệu trên máy: duyệt thư mục Cơ quan › Hộp › Hồ sơ, xem dạng thẻ / danh sách, tạo thư mục con,
// quét vào hồ sơ, chọn nhiều để xoá hoặc kéo thả di chuyển.

import { useCallback, useEffect, useMemo, useState } from "react";
import { V41p } from "@/live/useLive";
import { V460 } from "@/state";
import { V412, V415, V419, type V418 } from "@/api/archive";
import { V47u } from "@/features/drive/scanTarget";
import V485 from "@/features/drive/CardGrid";
import V48b from "@/features/drive/CardList";
import V48d from "@/features/drive/DetailsPanel";
import { Icon } from "@/components/Icon";
import { V462 as cn } from "@/lib/utils";
import { Button, confirm, EmptyState, IconButton, Input, Menu, notifyError, PageHeader, Select, Tip, toast, Toolbar } from "@/ui";

export default function DriveView() {
  const { nodes, path, driveMode, setDriveMode, navigateCrumb, openDialog, goto, setScanTarget, reloadArchive } = V460();
  const [query, setQuery] = useState("");
  const [sortBy, setSortBy] = useState<"name" | "size">("name");
  const [dir, setDir] = useState<"asc" | "desc">("asc");
  const [marked, setMarked] = useState<Set<string>>(new Set());

  V41p(reloadArchive, { topics: ["hoso", "collected", "scan", "catalog"] });
  useEffect(() => setMarked(new Set()), [path]);

  const cur = nodes[path[path.length - 1]];
  const kids = cur.kids ?? [];
  const q = query.trim().toLowerCase();
  const filtered = q ? kids.filter((id) => nodes[id].name.toLowerCase().includes(q)) : kids;

  const folderPath = cur?.type === "folder" ? cur.path : undefined;
  const [hoSo, setHoSo] = useState<V418 | null>(null);
  const loadHoSo = useCallback(() => {
    setHoSo(null);
    if (folderPath)
      V419(folderPath)
        .then(setHoSo)
        .catch(() => {});
  }, [folderPath]);
  V41p(loadHoSo, { topics: ["hoso"] });

  const quet = () => {
    if (!hoSo) return;
    setScanTarget(V47u(hoSo));
    goto("scan");
  };

  const sorted = useMemo(() => {
    const arr = [...filtered];
    arr.sort((a, b) => {
      const A = nodes[a];
      const B = nodes[b];
      const fa = A.type === "folder" ? 0 : 1;
      const fb = B.type === "folder" ? 0 : 1;
      if (fa !== fb) return fa - fb;
      const c = sortBy === "size" ? (A.size ?? 0) - (B.size ?? 0) : A.name.localeCompare(B.name, "vi");
      return dir === "asc" ? c : -c;
    });
    return arr;
  }, [filtered, nodes, sortBy, dir]);

  const toggleMark = (id: string) =>
    setMarked((m) => {
      const s = new Set(m);
      if (s.has(id)) s.delete(id);
      else s.add(id);
      return s;
    });

  const xoa = async () => {
    const ok = await confirm({ title: `Xoá ${marked.size} mục đã chọn?`, body: "Tệp và thư mục sẽ bị xoá khỏi Kho trên máy này. Không thể hoàn tác.", danger: true });
    if (!ok) return;
    let fail = 0;
    for (const p of marked) {
      try {
        await V412(p);
      } catch {
        fail++;
      }
    }
    setMarked(new Set());
    reloadArchive();
    if (fail) toast.warning(`Đã xoá, ${fail} mục không xoá được.`);
    else toast.success("Đã xoá các mục đã chọn.");
  };
  const diChuyen = async (dest: string, items: string[]) => {
    for (const p of items) {
      if (p === dest) continue;
      try {
        await V415(p, dest);
      } catch (e) {
        notifyError(e, "di chuyển");
      }
    }
    setMarked(new Set());
    reloadArchive();
  };

  const inFolder = path.length > 1;

  return (
    <section className="view active">
      <PageHeader
        title="Kho tài liệu"
        description="Tài liệu quét và gom trên máy này, xếp theo cơ quan, hộp, hồ sơ."
        actions={
          <>
            {inFolder && (
              <Tip text={hoSo ? "" : "Mở đúng thư mục hồ sơ (cấp Cơ quan › Hộp › Hồ sơ) rồi mới quét vào được"}>
                <span>
                  <Button icon="quet" variant={hoSo ? "primary" : "secondary"} disabled={!hoSo} onClick={quet}>
                    {hoSo ? "Quét vào hồ sơ này" : "Quét vào thư mục này"}
                  </Button>
                </span>
              </Tip>
            )}
            <Menu
              label="Tạo"
              trigger="button"
              items={[
                { label: "Thư mục con", icon: "moThuMuc", onSelect: () => openDialog("create", "Tạo thư mục con") },
                { label: "Nhận cây thư mục", icon: "cayThuMuc", onSelect: () => goto("tree") },
              ]}
            />
          </>
        }
      />

      <Toolbar className="mb-3">
        <nav aria-label="Đường dẫn" className="flex min-w-0 flex-wrap items-center gap-1 text-sm">
          {path.map((id, i) => {
            const last = i === path.length - 1;
            return (
              <span key={id} className="flex items-center gap-1">
                {i > 0 && <Icon name="moNhanh" size={12} className="text-fg-subtle" />}
                {last ? (
                  <span className="font-medium text-fg-strong">{nodes[id].name}</span>
                ) : (
                  <button type="button" className="text-fg-muted hover:text-fg-link hover:underline" onClick={() => navigateCrumb(i)}>
                    {nodes[id].name}
                  </button>
                )}
              </span>
            );
          })}
        </nav>
        <Toolbar.Spacer />
        <Input size="sm" prefix="tim" placeholder="Tìm thư mục, tài liệu" value={query} onChange={(e) => setQuery(e.target.value)} className="w-60" aria-label="Tìm trong thư mục" />
        <Select
          size="sm"
          value={`${sortBy}-${dir}`}
          onChange={(v) => {
            const [b, d] = v.split("-");
            setSortBy(b as "name" | "size");
            setDir(d as "asc" | "desc");
          }}
          options={[
            { value: "name-asc", label: "Tên A đến Z" },
            { value: "name-desc", label: "Tên Z đến A" },
            { value: "size-asc", label: "Nhỏ đến lớn" },
            { value: "size-desc", label: "Lớn đến nhỏ" },
          ]}
          className="w-36"
          aria-label="Sắp xếp"
        />
        <div className="flex overflow-hidden rounded border border-border" role="group" aria-label="Kiểu xem">
          <IconButton label="Xem dạng thẻ" icon="luoi" size="sm" noTooltip className={cn("rounded-none", driveMode === "grid" && "bg-brand-50 text-brand-700")} aria-pressed={driveMode === "grid"} onClick={() => setDriveMode("grid")} />
          <IconButton label="Xem dạng danh sách" icon="danhSach" size="sm" noTooltip className={cn("rounded-none border-l border-border", driveMode === "list" && "bg-brand-50 text-brand-700")} aria-pressed={driveMode === "list"} onClick={() => setDriveMode("list")} />
        </div>
      </Toolbar>

      <div className="drive-grid with-details">
        <div>
          {marked.size > 0 ? (
            <Toolbar className="mb-2.5 rounded-md border border-brand-200 bg-brand-50 px-3 py-1.5">
              <b className="text-sm">{marked.size} đã chọn</b>
              <span className="text-sm text-fg-subtle">Kéo thả vào thư mục để di chuyển.</span>
              <Toolbar.Spacer />
              <Button size="sm" variant="dangerOutline" icon="xoa" onClick={() => void xoa()}>
                Xoá
              </Button>
              <Button size="sm" onClick={() => setMarked(new Set())}>
                Bỏ chọn
              </Button>
            </Toolbar>
          ) : (
            <p className="mb-2.5 text-sm text-fg-subtle">{filtered.length.toLocaleString("vi-VN")} mục</p>
          )}

          {kids.length === 0 ? (
            <EmptyState
              icon="thuMuc"
              title="Thư mục trống"
              description={inFolder ? "Quét tài liệu vào đây hoặc tạo thư mục con." : "Nhận cây thư mục từ quản trị hoặc tạo thư mục con để bắt đầu."}
              action={
                inFolder && hoSo ? (
                  <Button variant="primary" icon="quet" onClick={quet}>
                    Quét vào hồ sơ này
                  </Button>
                ) : (
                  <Button icon="cayThuMuc" onClick={() => goto("tree")}>
                    Cây thư mục
                  </Button>
                )
              }
            />
          ) : filtered.length === 0 ? (
            <EmptyState compact icon="tim" title="Không có mục nào khớp tìm kiếm" />
          ) : driveMode === "list" ? (
            <V48b kids={sorted} marked={marked} onToggleMark={toggleMark} onMove={diChuyen} />
          ) : (
            <V485 kids={sorted} marked={marked} onToggleMark={toggleMark} onMove={diChuyen} />
          )}
        </div>
        <V48d />
      </div>
    </section>
  );
}
