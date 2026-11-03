// Copyright (c) 2026 DigiText. All rights reserved.

import { createContext, useCallback, useContext, useMemo, useRef, useState } from "react";
import type { ReactNode } from "react";
import type { V3xu, V3xw, V3xx, V3xt, V3xTienIch } from "@/domain/types";
import { confirm } from "@/ui/dialogs/confirmStore";
import type { V40m, V409, V408 } from "@/api/archive";

export interface V41a {
  relPath: string;
  filename: string;
}

export interface V41b {
  phong: V408;
  hoSo: V409;

  label: string;
}

interface V41c {
  view: V3xt;
  goto: (v: V3xt) => void;

  hoSoPane: V3xu;

  gotoHoSo: (p: V3xu) => void;

  khoGiaTab: V3xw;

  gotoKhoGia: (t: V3xw) => void;

  tienIchTab: V3xTienIch;
  gotoTienIch: (t: V3xTienIch) => void;

  reportTab: V3xx;

  gotoReport: (t: V3xx) => void;
  scanTarget: V41b | null;
  setScanTarget: (t: V41b | null) => void;

  serverReveal: V41a | null;

  gotoServerFile: (r: V41a) => void;

  clearServerReveal: () => void;

  editSession: V40m | null;

  editPdfOut: string | null;

  openEditor: (V41d: V40m, V41e: string | null) => void;

  setEditorDirty: (V41f: boolean) => void;
}

const V41g = createContext<V41c | null>(null);

// Bản dev chạy trong trình duyệt (xem trước giao diện, chụp màn): mở thẳng một màn bằng ?view=…&cl=….
const V3c0 = import.meta.env.DEV ? new URLSearchParams(window.location.search) : null;

export function V41h({ children: V2s }: { children: ReactNode }) {
  const [view, V2t] = useState<V3xt>(() => (V3c0?.get("view") as V3xt | null) ?? "home");
  const [V2u, V2v] = useState<V3xu>(() => (V3c0?.get("cl") as V3xu | null) ?? "laphoso");
  const [V2y, V2z] = useState<V3xw>(() => (V3c0?.get("view") === "khogia" ? (V3c0.get("kg") as V3xw | null) : null) ?? "khogia");
  const [tienIchTab, setTienIchTab] = useState<V3xTienIch>(() => (V3c0?.get("view") === "tienich" ? (V3c0.get("tab") as V3xTienIch | null) : null) ?? "pdfa");
  const [V30, V31] = useState<V3xx>("sohoa");
  const [V32, V33] = useState<V41b | null>(null);
  const [V34, V35] = useState<V41a | null>(null);
  const [V36, V37] = useState<V40m | null>(null);
  const [V38, V39] = useState<string | null>(null);

  const V3a = useRef(false);

  const V3b = () => {
    const content = document.querySelector(".content");
    if (content) content.scrollTop = 0;
  };

  const V3c = useCallback((V3d: boolean) => {
    V3a.current = V3d;
  }, []);

  const V3e = useCallback(async () => {
    if (!V3a.current) return true;
    const V3f = await confirm({ title: "Rời Trình xử lý ảnh?", body: "Có thay đổi chưa lưu. Rời đi thì các thay đổi bị bỏ.", confirmLabel: "Rời đi", danger: true });
    if (V3f) V3a.current = false;
    return V3f;
  }, []);

  const goto = useCallback(
    async (v: V3xt) => {
      if (v !== "editor" && !(await V3e())) return;
      V2t(v);

      if (v !== "editor") {
        V37(null);
        V39(null);
      }
      V3b();
    },
    [V3e],
  );

  const V3g = useCallback(
    (p: V3xu) => {
      if (!V3e()) return;
      V2v(p);
      V2t("dossier");
      V37(null);
      V39(null);
      V3b();
    },
    [V3e],
  );

  const V3i = useCallback(
    (t: V3xw) => {
      if (!V3e()) return;
      V2z(t);
      V2t("khogia");
      V37(null);
      V39(null);
      V3b();
    },
    [V3e],
  );

  const gotoTienIch = useCallback(
    (t: V3xTienIch) => {
      if (!V3e()) return;
      setTienIchTab(t);
      V2t("tienich");
      V37(null);
      V39(null);
      V3b();
    },
    [V3e],
  );

  const V3j = useCallback(
    (t: V3xx) => {
      if (!V3e()) return;
      V31(t);
      V2t("report");
      V37(null);
      V39(null);
      V3b();
    },
    [V3e],
  );

  const V3k = useCallback(
    (r: V41a) => {
      if (!V3e()) return;
      V35(r);
      V2t("server");
      V37(null);
      V39(null);
      V3b();
    },
    [V3e],
  );

  const V3l = useCallback(() => V35(null), []);

  const V3m = useCallback((V3n: V40m, V3o: string | null) => {
    V37(V3n);
    V39(V3o);
    V2t("editor");
    V3b();
  }, []);

  const V3p = useMemo<V41c>(
    () => ({
      view,
      goto,
      hoSoPane: V2u,
      gotoHoSo: V3g,
      khoGiaTab: V2y,
      gotoKhoGia: V3i,
      tienIchTab,
      gotoTienIch,
      reportTab: V30,
      gotoReport: V3j,
      scanTarget: V32,
      setScanTarget: V33,
      serverReveal: V34,
      gotoServerFile: V3k,
      clearServerReveal: V3l,
      editSession: V36,
      editPdfOut: V38,
      openEditor: V3m,
      setEditorDirty: V3c,
    }),
    [view, goto, V2u, V3g, V2y, V3i, tienIchTab, gotoTienIch, V30, V3j, V32, V34, V3k, V3l, V36, V38, V3m, V3c],
  );
  return <V41g.Provider value={V3p}>{V2s}</V41g.Provider>;
}

export function V41i(): V41c {
  const c = useContext(V41g);
  if (!c) throw new Error("useNavigation must be used within <NavigationProvider>");
  return c;
}
