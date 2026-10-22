// Copyright (c) 2026 DigiText. All rights reserved.
// Việc của tôi (cộng tác viên): gói được giao, mục bị trả lại; màn làm việc = ảnh + 16 trường (OcrFieldsView),
// Trước / Tiếp / Xác nhận (Ctrl+Enter); gói kiểm tra chéo: Đạt / Có lỗi. Mỗi lúc một mục, khoá mềm 10 phút.

import { useCallback, useEffect, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41t } from "@/live/useLive";
import { ITEM_STATUS_LABEL, KIND_LABEL, workCheck, workClaimItem, workDone, workGetPackage, workMyPackages, workNavigate, workNextItem, workRejectedItems, workRelease, type WorkItem, type WorkPackage } from "@/api/work";
import V4i7 from "@/features/upload/OcrFieldsView";
import { Icon } from "@/components/Icon";
import { Badge, Button, Card, EmptyState, ErrorBanner, IconButton, Input, Kbd, notifyError, PageHeader, Table, Tip, toast, type Column } from "@/ui";

const n = (v: number) => v.toLocaleString("vi-VN");
const ngay = (s: string | null) => (s ? s.split("-").reverse().join("/") : "—");
const conViec = (p: WorkPackage) => (p.kind === "kiem_tra" ? p.nPending : p.nPending + p.nRejected);
const quaHan = (p: WorkPackage) => !!p.dueAt && new Date(`${p.dueAt}T23:59:59`).getTime() < Date.now() && conViec(p) > 0;

function LamViec({ pkg, startItem, onBack }: { pkg: WorkPackage; startItem?: WorkItem | null; onBack: () => void }) {
  const [item, setItem] = useState<WorkItem | null | undefined>(undefined);
  const [total, setTotal] = useState(pkg.nTotal);
  const [con, setCon] = useState(conViec(pkg));
  const [hetViec, setHetViec] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [now, setNow] = useState(Date.now());
  const [note, setNote] = useState("");

  const capNhatDem = useCallback(async () => {
    try {
      const d = await workGetPackage(pkg.id);
      setTotal(d.nTotal);
      setCon(conViec(d));
    } catch {
      /* chỉ là số đếm */
    }
  }, [pkg.id]);

  const nhan = useCallback(async () => {
    setBusy(true);
    setHetViec(null);
    try {
      const it = await workNextItem(pkg.id);
      setItem(it);
      setNote("");
      if (!it) setHetViec("Hết việc trong gói này, hoặc mục còn lại đang do người khác làm.");
    } catch (e) {
      notifyError(e, "nhận việc");
      setItem(null);
    } finally {
      setBusy(false);
    }
  }, [pkg.id]);

  useEffect(() => {
    if (startItem) {
      setBusy(true);
      void workClaimItem(startItem.id)
        .then((it) => {
          if (it) {
            setItem(it);
            setNote("");
            setBusy(false);
          } else return nhan();
        })
        .catch(() => nhan());
    } else void nhan();
  }, [nhan, startItem]);

  useEffect(() => {
    const t = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(t);
  }, []);

  const chuyen = async (dir: "prev" | "next") => {
    if (!item) return;
    setBusy(true);
    try {
      const it = await workNavigate(item.id, dir);
      if (it) {
        setItem(it);
        setNote("");
      } else toast.info(dir === "next" ? "Đã tới mục cuối còn làm được." : "Đã tới mục đầu.");
    } catch (e) {
      notifyError(e, "chuyển mục");
    } finally {
      setBusy(false);
    }
  };

  const xacNhan = async () => {
    if (!item || busy) return;
    setBusy(true);
    try {
      await workDone(item.id);
      toast.success(`Đã xác nhận ${item.filename}.`);
      await capNhatDem();
      await nhan();
    } catch (e) {
      notifyError(e, "xác nhận");
      setBusy(false);
    }
  };

  const cham = async (ok: boolean) => {
    if (!item || busy) return;
    if (!ok && !note.trim()) {
      toast.warning("Ghi lỗi gì vào ô ghi chú trước khi chấm Có lỗi.");
      return;
    }
    setBusy(true);
    try {
      await workCheck(item.id, ok, note.trim() || undefined);
      toast.success(`Đã chấm ${ok ? "Đạt" : "Có lỗi"}: ${item.filename}.`);
      await capNhatDem();
      await nhan();
    } catch (e) {
      notifyError(e, "chấm");
      setBusy(false);
    }
  };

  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      if (e.ctrlKey && e.key === "Enter") {
        e.preventDefault();
        if (pkg.kind === "xac_nhan") void xacNhan();
        else void cham(true);
      }
    };
    window.addEventListener("keydown", h);
    return () => window.removeEventListener("keydown", h);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [item, busy, note, pkg.kind]);

  const thoat = () => {
    if (item) void workRelease(item.id).catch(() => null);
    onBack();
  };

  const conKhoa = item?.leaseUntil ? Math.max(0, Math.round((new Date(item.leaseUntil).getTime() - now) / 1000)) : null;

  return (
    <>
      <Card padding="sm" className="mb-3">
        <div className="flex flex-wrap items-center gap-3">
          <IconButton label="Về gói của tôi" icon="truoc" onClick={thoat} />
          <b>{pkg.name}</b>
          <Badge tone="gray">{KIND_LABEL[pkg.kind]}</Badge>
          <span className="text-sm">
            Còn <b>{n(con)}</b>/{n(total)} mục
          </span>
          {item && (
            <span className="text-sm text-fg-subtle">
              Mục {item.ord} · {item.relPath ?? item.hoSoRef ?? ""} · <Badge tone="gray">{ITEM_STATUS_LABEL[item.status]}</Badge>
              {conKhoa != null && ` · giữ mục còn ${Math.floor(conKhoa / 60)}:${String(conKhoa % 60).padStart(2, "0")}`}
            </span>
          )}
          {item?.status === "rejected" && item.reviewNote && (
            <span className="text-sm text-danger-700">
              <Icon name="canhBao" size={14} /> Bị trả: {item.reviewNote}
            </span>
          )}
        </div>
      </Card>
      {item === undefined ? (
        <Card>
          <p className="text-sm text-fg-subtle">Đang nhận việc…</p>
        </Card>
      ) : item === null ? (
        <Card>
          <EmptyState
            compact
            icon="duyet"
            title="Không còn mục nào"
            description={hetViec ?? undefined}
            action={
              <Button icon="lamMoi" onClick={() => void nhan()}>
                Thử nhận lại
              </Button>
            }
          />
        </Card>
      ) : (
        <V4i7
          key={item.id}
          doc={{ id: item.documentId, filename: item.filename, reviewStatus: "pending" }}
          onBack={thoat}
          workMode
          footer={
            <>
              <Button icon="truoc" onClick={() => void chuyen("prev")} disabled={busy}>
                Trước
              </Button>
              <Button iconRight="sau" onClick={() => void chuyen("next")} disabled={busy}>
                Tiếp
              </Button>
              {pkg.kind === "xac_nhan" ? (
                <Button variant="primary" icon="duyet" onClick={() => void xacNhan()} disabled={busy}>
                  Xác nhận hoàn thành <Kbd keys={["Ctrl", "Enter"]} className="ml-1" />
                </Button>
              ) : (
                <>
                  <Input size="sm" className="max-w-60" placeholder="Ghi lỗi nếu có" value={note} onChange={(e) => setNote(e.target.value)} aria-label="Ghi chú lỗi" />
                  <Button variant="primary" onClick={() => void cham(true)} disabled={busy}>
                    Đạt <Kbd keys={["Ctrl", "Enter"]} className="ml-1" />
                  </Button>
                  <Button variant="dangerOutline" onClick={() => void cham(false)} disabled={busy}>
                    Có lỗi
                  </Button>
                </>
              )}
            </>
          }
        />
      )}
    </>
  );
}

export default function MyWorkView() {
  const loader = useCallback(async () => {
    if (!isTauri()) return { pkgs: [] as WorkPackage[], rejected: [] as WorkItem[] };
    const [pkgs, rejected] = await Promise.all([workMyPackages(), workRejectedItems()]);
    return { pkgs, rejected };
  }, []);
  const { data, error, loading, reload } = V41t(loader, { topics: ["work"], errorText: () => "Không tải được danh sách việc được giao." });
  const pkgs = [...(data?.pkgs ?? [])].sort((a, b) => Number(quaHan(b)) - Number(quaHan(a)) || (a.dueAt ?? "9").localeCompare(b.dueAt ?? "9"));
  const rejected = data?.rejected ?? [];
  const [lam, setLam] = useState<{ pkg: WorkPackage; item?: WorkItem } | null>(null);

  const rejectedCols: Column<WorkItem>[] = [
    { key: "filename", header: "Tài liệu", className: "whitespace-normal" },
    { key: "relPath", header: "Đường dẫn", className: "text-fg-subtle whitespace-normal", cell: (i) => i.relPath ?? i.hoSoRef ?? "—" },
    { key: "reviewNote", header: "Lý do", className: "text-danger-700 whitespace-normal" },
  ];

  return (
    <section className="view active">
      <PageHeader title="Việc của tôi" description="Gói xác nhận trường và kiểm tra chéo được giao cho bạn." actions={!lam && <IconButton label="Làm mới" icon="lamMoi" onClick={reload} />} />
      {lam ? (
        <LamViec
          pkg={lam.pkg}
          startItem={lam.item}
          onBack={() => {
            setLam(null);
            reload();
          }}
        />
      ) : (
        <div className="space-y-4">
          {error != null && <ErrorBanner error={error} onRetry={reload} />}
          {rejected.length > 0 && (
            <Card title={`Mục bị trả lại (${n(rejected.length)})`} description="Làm lại rồi xác nhận, mục sẽ về hàng chờ duyệt." padding="none" tone="warning">
              <Table
                columns={rejectedCols}
                rows={rejected}
                getRowId={(i) => i.id}
                dense
                rowActions={(i) => {
                  const pkg = pkgs.find((p) => p.id === i.packageId);
                  return pkg ? (
                    <Button size="sm" variant="primary" onClick={() => setLam({ pkg, item: i })}>
                      Làm lại
                    </Button>
                  ) : null;
                }}
              />
            </Card>
          )}
          <Card title="Gói được giao">
            {loading && pkgs.length === 0 ? (
              <p className="text-sm text-fg-subtle">Đang tải…</p>
            ) : pkgs.length === 0 ? (
              <EmptyState compact icon="goi" title="Chưa được giao gói nào" description="Khi quản trị giao việc, gói sẽ hiện ở đây." />
            ) : (
              <div className="grid gap-3" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))" }}>
                {pkgs.map((p) => {
                  const con = conViec(p);
                  const qh = quaHan(p);
                  return (
                    <div key={p.id} className={`rounded-md border p-3 ${qh ? "border-danger-200 bg-danger-50" : "border-border-subtle"}`}>
                      <div className="flex flex-wrap items-center gap-2">
                        <b>{p.name}</b>
                        <Badge tone="gray">{KIND_LABEL[p.kind]}</Badge>
                        {qh && <Badge tone="err">Quá hạn</Badge>}
                      </div>
                      <div className={`mt-1 text-sm ${qh ? "text-danger-700" : "text-fg-subtle"}`}>Hạn {ngay(p.dueAt)}</div>
                      <div className="mt-1 text-sm">
                        Còn <b>{n(con)}</b> việc / {n(p.nTotal)} · đã duyệt {n(p.nApproved)}
                        {p.nRejected > 0 && p.kind === "xac_nhan" && <span className="text-danger-700"> · bị trả {n(p.nRejected)}</span>}
                      </div>
                      <Tip text={`${n(p.nTotal - con)}/${n(p.nTotal)} mục đã làm`}>
                        <div className="mt-2 h-2 rounded bg-border-subtle">
                          <div className="h-2 rounded bg-brand-500" style={{ width: `${((p.nTotal - con) / Math.max(1, p.nTotal)) * 100}%` }} />
                        </div>
                      </Tip>
                      <Button variant="primary" icon="chay" className="mt-3" disabled={con === 0} onClick={() => setLam({ pkg: p })}>
                        {con === 0 ? "Đã xong" : "Làm việc"}
                      </Button>
                    </div>
                  );
                })}
              </div>
            )}
          </Card>
        </div>
      )}
    </section>
  );
}
