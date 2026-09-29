// Copyright (c) 2026 DigiText. All rights reserved.
// Đưa dữ liệu chỉnh lý lập trước đây trên máy này (SQLite cũ) vào bản sao đồng bộ. Đặt ở Cài đặt › Dữ liệu.

import { useCallback, useState } from "react";
import { isTauri } from "@/api/ipc";
import { V41p } from "@/live/useLive";
import { clMigratePreview, clMigrateRun, type ClMigrateReport } from "@/api/chinhly";
import { n } from "./common";
import { Banner, Button, Card, ErrorBanner, notifyError } from "@/ui";

export default function ChuyenDuLieuCu({ onDone }: { onDone?: () => void }) {
  const [rep, setRep] = useState<ClMigrateReport | null>(null);
  const [res, setRes] = useState<ClMigrateReport | null>(null);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState<unknown>(null);
  const load = useCallback(async () => {
    try {
      setRep(await clMigratePreview());
      setErr(null);
    } catch (e) {
      setErr(e);
    }
  }, []);
  V41p(load, { topics: ["chinhLy", "dossier"], enabled: isTauri() });
  const tong = rep ? rep.phong + rep.thbq + rep.phongBan + rep.mang + rep.hoSo + rep.vanBan : 0;
  const chay = async () => {
    setBusy(true);
    try {
      setRes(await clMigrateRun());
      onDone?.();
      void load();
    } catch (e) {
      notifyError(e, "chuyển dữ liệu cũ");
    } finally {
      setBusy(false);
    }
  };
  return (
    <Card title="Dữ liệu chỉnh lý cũ trên máy này" description="Hồ sơ, văn bản, danh mục lập bằng bản cũ được đưa vào bản sao đồng bộ rồi đẩy lên máy chủ.">
      {err !== null && <ErrorBanner error={err} onRetry={() => void load()} />}
      {rep && (
        <p className="text-sm">
          {tong === 0 ? "Mọi hồ sơ, văn bản, danh mục lập trước đây trên máy đã được nối với máy chủ." : `Chưa lên máy chủ: ${n(rep.hoSo)} hồ sơ, ${n(rep.vanBan)} văn bản, ${n(rep.phong)} phông, ${n(rep.phongBan)} phòng ban, ${n(rep.mang)} mảng, ${n(rep.thbq)} THBQ. Số tạm trùng trong phông sẽ được báo khi đẩy.`}
        </p>
      )}
      <div className="mt-3 flex flex-wrap items-center gap-2">
        <Button variant="primary" icon="taiLen" loading={busy} disabled={tong === 0} onClick={() => void chay()}>
          Đưa dữ liệu cũ lên máy chủ
        </Button>
      </div>
      {res && (
        <Banner tone={res.errors.length ? "warning" : "success"} className="mt-3" detail={res.errors.length ? res.errors.join("\n") : undefined}>
          Đã chuyển {n(res.hoSo)} hồ sơ, {n(res.vanBan)} văn bản, {n(res.phong)} phông, {n(res.phongBan)} phòng ban, {n(res.mang)} mảng.{res.errors.length ? ` ${res.errors.length} lỗi.` : ""}
        </Banner>
      )}
    </Card>
  );
}
