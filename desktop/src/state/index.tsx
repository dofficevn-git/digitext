// Copyright (c) 2026 DigiText. All rights reserved.

import type { ReactNode } from "react";
import { V406, V407 } from "./session";
import { V41h, V41i } from "./navigation";
import { V42d, V42e } from "./drive";
import { V42h, V42i } from "./members";
import { V42o, V42p } from "./ui";
import { V42x, V42y } from "./update";
import { V44l, V44m } from "./uploads";
import { V45j, V45k } from "./collects";
import { V45y } from "@/live/LiveProvider";
import { ProjectProvider, useProject } from "./project";
import { DownloadsProvider, useDownloads } from "./downloads";

export {
  V407 as useSession,
  V41i as useNavigation,
  V42e as useDrive,
  V42i as useMembers,
  V42p as useUi,
  V42y as useUpdate,
  V44m as useUploads,
  V45k as useCollects,
  useProject,
  useDownloads,
};
export type { AuthSession } from "./session";
export type { V41b as ScanTarget } from "./navigation";
export type { V42j as DialogKind } from "./ui";

export function V45z({ children: Vck }: { children: ReactNode }) {
  return (
    <V406>
      {}
      <V45y>
        <V41h>
          <ProjectProvider>
          <V42x>
            <V42h>
              <V42o>
                <V44l>
                  <V45j>
                    <DownloadsProvider>
                      <V42d>{Vck}</V42d>
                    </DownloadsProvider>
                  </V45j>
                </V44l>
              </V42o>
            </V42h>
          </V42x>
          </ProjectProvider>
        </V41h>
      </V45y>
    </V406>
  );
}

export function V460() {
  return {
    ...V407(),
    ...V41i(),
    ...V42e(),
    ...V42i(),
    ...V42p(),
    ...V42y(),
    project: useProject(),
  };
}
