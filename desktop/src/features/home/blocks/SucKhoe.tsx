// Copyright (c) 2026 DigiText. All rights reserved.
// "Sức khoẻ hệ thống": toàn vẹn tài liệu, kiểm tra định kỳ, sao lưu — tách từ Tổng quan cũ (quyền bk.view).

import { useCallback } from "react";
import { V460 } from "@/state";
import { V4v1, V4v4 } from "@/api/infoops";
import { DataCard, DongTrangThai, n, pct, useSo } from "./common";

export default function SucKhoeCard({ className }: { className?: string }) {
  const { goto } = V460();
  const loadOps = useCallback(async () => {
    const [ir, bk] = await Promise.all([V4v1(), V4v4().catch(() => [])]);
    const last = bk.filter((b) => b.ketQua === "dat").map((b) => new Date(b.createdAt).getTime()).reduce((a, b) => Math.max(a, b), 0);
    return { loi: ir.soLoi, daKiem: ir.daKiem3Nam, tong: ir.tongTaiLieu, backupHours: last ? (Date.now() - last) / 3_600_000 : null };
  }, []);
  const ops = useSo(loadOps, ["ops"], true);
  return (
    <DataCard title="Sức khoẻ hệ thống" icon="baoquan" {...ops} open={() => goto("bk")} className={className}>
      {(d) => (
        <ul className="divide-y divide-border-subtle">
          <DongTrangThai tone={d.loi > 0 ? "err" : "ok"} label="Lỗi toàn vẹn tài liệu" value={n(d.loi)} sub={d.loi > 0 ? "cần lập biên bản kiểm tra" : undefined} />
          <DongTrangThai tone={pct(d.daKiem, d.tong) >= 33 ? "ok" : "info"} label="Đã kiểm tra trong 3 năm" value={`${pct(d.daKiem, d.tong)}%`} sub={`${n(d.daKiem)} / ${n(d.tong)} tài liệu`} />
          <DongTrangThai tone={d.backupHours === null || d.backupHours >= 26 ? "err" : "ok"} label="Sao lưu đạt gần nhất" value={d.backupHours === null ? "Chưa có" : d.backupHours < 26 ? "Trong ngày" : `${Math.round(d.backupHours / 24)} ngày trước`} sub={d.backupHours === null || d.backupHours >= 26 ? "cần sao lưu ít nhất mỗi ngày" : undefined} />
        </ul>
      )}
    </DataCard>
  );
}
