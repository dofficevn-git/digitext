// Copyright (c) 2026 DigiText. All rights reserved.

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { V41p } from "@/live/useLive";
import type { ReactNode } from "react";
import { V41x } from "@/mocks/fixtures";
import type { V3y1 } from "@/domain/types";
import { V414, V40z, V417 } from "@/api/archive";
import { isTauri } from "@/api/ipc";
import { V421, setDriveRoot } from "@/state/driveCore";
import { V407 } from "./session";

type V422 = "grid" | "list";

interface V423 {
  nodes: Record<string, V3y1>;
  path: string[];
  selected: string | null;
  driveMode: V422;
  enterFolder: (V424: string) => void;
  navigateCrumb: (V425: number) => void;
  resetToRoot: () => void;
  select: (V426: string | null) => void;
  setDriveMode: (m: V422) => void;
  createFolder: (name: string) => void;

  reloadArchive: () => void;

  removeArchiveNode: (V427: string) => void;
  renameNode: (V428: string, name: string, V429: string, V42a: string) => void;
  addFolderNode: (V42b: string, name: string, path: string) => void;
}

const V42c = createContext<V423 | null>(null);

export function V42d({ children: V54 }: { children: ReactNode }) {
  const { status } = V407();
  const [V55, V56] = useState<Record<string, V3y1>>(V41x);
  const [path, V57] = useState<string[]>(["root"]);
  const [V58, V59] = useState<string | null>(null);
  const [V5a, V5b] = useState<V422>("grid");
  const [V5c, V5d] = useState("");

  const V5e = useRef(V55);
  V5e.current = V55;
  const V5f = useRef(path);
  V5f.current = path;

  const V5g = useCallback((V5h: string) => {
    V57((p) => [...p, V5h]);
    V59(null);
  }, []);
  const V5i = useCallback((V5j: number) => {
    V57((p) => p.slice(0, V5j + 1));
    V59(null);
  }, []);
  const V5k = useCallback(() => {
    V57(["root"]);
    V59(null);
  }, []);
  const select = useCallback((V5l: string | null) => V59(V5l), []);

  const V5m = useCallback(async (V5n: string) => {
    if (!isTauri()) return;
    const V5o = V5e.current[V5n]?.path;
    if (!V5o) return;
    try {
      const V5p = await V417(V5o);
      V56((V5q) => V421(V5q, V5n, V5p));
    } catch {}
  }, []);

  const V5r = useCallback(
    (name: string) => {
      const V5s = V5f.current[V5f.current.length - 1];
      const V5t = V5e.current[V5s];

      if (isTauri() && V5t?.path) {
        V414(V5t.path, name || "Thư mục mới")
          .then(() => V5m(V5s))
          .catch(() => {});
        return;
      }
      const V5u = "n" + Date.now();
      V56((V5v) => {
        const p = V5v[V5s];
        return {
          ...V5v,
          [V5u]: {
            id: V5u,
            type: "folder",
            name: name || "Dự án mới",
            owner: "VMO0aQ==",
            members: [],
            kids: [],
          },
          [V5s]: { ...p, kids: [V5u, ...(p.kids ?? [])] },
        };
      });
    },
    [V5m],
  );

  const V5w = useCallback(() => {
    void V5m(V5f.current[V5f.current.length - 1]);
  }, [V5m]);

  const V5x = useCallback(() => {
    V40z()
      .then((root) => {
        V5d(root);
        V56((V5y) => setDriveRoot(V5y, root));
      })
      .catch(() => {});
  }, []);

  V41p(V5x, {
    topics: ["settings"],
    enabled: isTauri() && status === "active",
  });

  useEffect(() => {
    void V5m(path[path.length - 1]);
  }, [path, V5c, V5m]);

  const V5z = useCallback((V60: string) => {
    V56((V61) => {
      const V62 = { ...V61 };
      const V63 = new Set<string>();
      const V64 = (V65: string) => {
        V63.add(V65);
        (V62[V65]?.kids ?? []).forEach(V64);
      };
      V64(V60);
      V63.forEach((V66) => delete V62[V66]);
      for (const k of Object.keys(V62)) {
        const node = V62[k];
        if (node.kids?.includes(V60))
          V62[k] = { ...node, kids: node.kids.filter((c) => c !== V60) };
      }
      return V62;
    });
    V57((p) => {
      const i = p.indexOf(V60);
      return i >= 0 ? p.slice(0, i) : p;
    });
    V59(null);
  }, []);

  const V67 = useCallback((V68: string, name: string, V69: string, V6a: string) => {
    V56((V6b) => {
      if (!V6b[V68]) return V6b;
      const V6c = { ...V6b };
      const V6d = (V6e: string) => {
        const V6f = V6c[V6e];
        if (!V6f) return;
        const p =
          V6f.path && V6f.path.startsWith(V69) ? V6a + V6f.path.slice(V69.length) : V6f.path;
        V6c[V6e] = { ...V6f, path: p, ...(V6e === V68 ? { name } : {}) };
        (V6f.kids ?? []).forEach(V6d);
      };
      V6d(V68);
      return V6c;
    });
  }, []);

  const V6g = useCallback((V6h: string, name: string, path: string) => {
    const V6i = `mk:${path}`;
    V56((V6j) => {
      const V6k = V6j[V6h];
      if (!V6k) return V6j;
      return {
        ...V6j,
        [V6i]: { id: V6i, type: "folder", name, kids: [], path },
        [V6h]: { ...V6k, kids: [V6i, ...(V6k.kids ?? [])] },
      };
    });
  }, []);

  const V6l = useMemo<V423>(
    () => ({
      nodes: V55,
      path,
      selected: V58,
      driveMode: V5a,
      enterFolder: V5g,
      navigateCrumb: V5i,
      resetToRoot: V5k,
      select,
      setDriveMode: V5b,
      createFolder: V5r,
      reloadArchive: V5w,
      removeArchiveNode: V5z,
      renameNode: V67,
      addFolderNode: V6g,
    }),
    [V55, path, V58, V5a, V5g, V5i, V5k, select, V5r, V5w, V5z, V67, V6g],
  );
  return <V42c.Provider value={V6l}>{V54}</V42c.Provider>;
}

export function V42e(): V423 {
  const c = useContext(V42c);
  if (!c) throw new Error("useDrive must be used within <DriveProvider>");
  return c;
}
