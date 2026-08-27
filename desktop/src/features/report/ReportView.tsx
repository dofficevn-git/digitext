// Copyright (c) 2026 DigiText. All rights reserved.

import { useCallback, useRef, useState } from "react";
import { V41p } from "@/live/useLive";
import type { ReactNode } from "react";
import { isTauri, V3xq } from "@/api/ipc";
import { V4e3 } from "@/features/dossier/excel";
import { V4o7, type V4o1 } from "./reportLogic";
import { Icon } from "@/components/Icon";

interface V4o8<T> {
  title: string;
  subtitle?: string;
  columns: V4o1<T>[];
  fetchRows: () => Promise<T[]>;
  sheetName: string;
  fileBase: string;

  filters?: ReactNode;
}

export default function V4o9<T>({
  title: V21t,
  subtitle: V21u,
  columns,
  fetchRows: V21v,
  sheetName: V21w,
  fileBase: V21x,
  filters: V21y,
}: V4o8<T>) {
  const [V21z, V220] = useState<T[]>([]);
  const [V221, V222] = useState(false);
  const [V223, V224] = useState<string | null>(null);
  const [V225, V226] = useState(false);

  const V227 = useRef(V21v);
  V227.current = V21v;

  const V228 = useCallback(async () => {
    if (!isTauri()) {
      V226(true);
      return;
    }
    V222(true);
    V224(null);
    try {
      V220(await V227.current());
    } catch (V229) {
      V224(V229 instanceof V3xq || V229 instanceof Error ? V229.message : String(V229));
      V220([]);
    } finally {
      V222(false);
      V226(true);
    }
  }, []);

  V41p(V228, { topics: ["dossier", "hoso", "stats", "docs"] });

  const V22a = async () => {
    if (V21z.length === 0) {
      window.alert("Không có dữ liệu để xuất.");
      return;
    }
    try {
      await V4e3(V4o7(columns, V21z), V21w, `${V21x}.xlsx`);
    } catch (V22b) {
      window.alert(
        `Xuất Excel thất bại: ${V22b instanceof Error ? V22b.message : String(V22b)}`,
      );
    }
  };

  return (
    <section className="view active">
      <div className="page-head">
        <h2>{V21t}</h2>
        {V21u && <p className="muted">{V21u}</p>}
      </div>

      <div className="dt-toolbar">
        {V21y}
        <div className="spacer flex-1" />
        <button className="btn" onClick={() => void V228()} disabled={V221}>
          <Icon name="lamMoi" />
          Lấy dữ liệu
        </button>
        <button className="btn primary" onClick={() => void V22a()} disabled={V21z.length === 0}>
          <Icon name="xuatTep" />
          Xuất báo cáo
        </button>
      </div>

      {!isTauri() && (
        <div className="err-banner" role="alert">
          Báo cáo lấy số liệu từ cơ sở dữ liệu — chỉ hoạt động trong ứng dụng desktop.
        </div>
      )}
      {V223 && (
        <div className="err-banner" role="alert">
          Không tải được báo cáo: {V223}
        </div>
      )}

      <div className="tbl-scroll">
        <table className="tbl dt">
          <thead>
            <tr>
              <th className="tnum w-11">STT</th>
              {columns.map((c, i) => (
                <th key={i} className={c.num ? "tnum" : undefined}>
                  {c.head}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {V21z.map((r, ri) => (
              <tr key={ri}>
                <td className="tnum">{ri + 1}</td>
                {columns.map((c, V22c) => (
                  <td key={V22c} className={c.num ? "tnum" : undefined}>
                    {c.value(r)}
                  </td>
                ))}
              </tr>
            ))}
            {V21z.length === 0 && (
              <tr>
                <td colSpan={columns.length + 1} className="muted text-center p-4.5">
                  {V221 ? "Đang tải…" : V225 ? "Không có dữ liệu." : ""}
                </td>
              </tr>
            )}
          </tbody>
        </table>
      </div>
    </section>
  );
}
