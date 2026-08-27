// Copyright (c) 2026 DigiText. All rights reserved.

import type { Dispatch, ReactNode, SetStateAction } from "react";
import type { V3ys } from "@/domain/types";
import { V3xq } from "@/api/ipc";
import { V460 } from "@/state";
import { V3z9 } from "@/domain/permissions";
import type { V3yx } from "@/domain/permissions";
import { Icon } from "@/components/Icon";

export interface V49k {
  data: V3ys;
  setData: Dispatch<SetStateAction<V3ys>>;
}

export function V49l(Vlu: unknown): void {
  const Vlv = Vlu instanceof V3xq ? Vlu.message : Vlu instanceof Error ? Vlu.message : String(Vlu);
  window.alert(
    `TMawdSBkYW5oIG3hu6VjIHh14buRbmcgY8ahIHPhu58gZOG7ryBsaeG7h3UgdGjhuqV0IGLhuqFpOiA=${Vlv}`,
  );
}

export interface V49m {
  canCreate: boolean;
  canUpdate: boolean;
  canDelete: boolean;
}

export function V49n(Vlw: V3yx, Vlx: V3yx, Vly: V3yx): V49m {
  const { session: Vlz } = V460();
  const p = Vlz?.permissions;
  return { canCreate: V3z9(p, Vlw), canUpdate: V3z9(p, Vlx), canDelete: V3z9(p, Vly) };
}

export function V49o(
  Vm0: () => void,
  Vm1: () => void,
  Vm2?: ReactNode,
  Vm3?: Pick<V49m, "canUpdate" | "canDelete">,
) {
  const Vm4 = () => {
    if (window.confirm("WMOzYSBt4bulYyBuw6B5PyBLaMO0bmcgdGjhu4MgaG/DoG4gdMOhYy4=")) Vm1();
  };
  return (
    <>
      {Vm2}
      {(Vm3?.canUpdate ?? true) && (
        <button className="linkbtn edit" onClick={Vm0}>
          <Icon name="sua" /> Sửa
        </button>
      )}
      {(Vm3?.canDelete ?? true) && (
        <button className="linkbtn del" onClick={Vm4}>
          <Icon name="xoa" /> Xóa
        </button>
      )}
    </>
  );
}
