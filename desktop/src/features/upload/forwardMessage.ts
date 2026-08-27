// Copyright (c) 2026 DigiText. All rights reserved.

import type { V43b } from "@/api/lansync";

export function V4pu(r: V43b): string {
  let V2c8 = `xJDhuql5IGzDqm4gbcOheSBjaOG7pzog${r.forwarded}IG3hu5tpLCA=${r.skipped}IMSRw6MgY8OzLCBs4buXaSA=${r.failed}/${r.total}.`;
  if (r.retagged > 0) {
    V2c8 += ` ${r.retagged}IGLhuqNuIMSRw6MgYuG7iyBz4butYSBs4bqhaSB0cm9uZyBLaG8gc2F1IGtoaSBnb20g4oCUIMSRw6MgxJHhuql5IGLhuqNuIG3hu5tpIG5o4bqldC4=`;
  }

  if (r.cancelled > 0) {
    V2c8 +=
      ` ${r.cancelled}IHThu4dwIGLhuqFuIMSRw6MgaOG7p3kg4oCUIHBo4bqnbiDEkcOjIGfhu61pIHbhuqtuIG7hurFtIHRyw6puIG3DoXkgY2jhu6csIA==` +
      "YuG6pW0g4oCc4pGhIMSQ4bqpeSBsw6puIG3DoXkgY2jhu6figJ0gbOG6p24gbuG7r2EgbMOgIHThuqNpIHRp4bq/cCB04burIGNo4buXIGThu6tuZy4=";
  }
  if (r.rejected > 0) {
    V2c8 +=
      ` ${r.rejected}IGLhuqNuIGLhu4sgbcOheSBjaOG7pyBU4buqIENI4buQSSAodGjGsOG7nW5nIGRvIG5nxrDhu51pIHF1w6l0IGtow7RuZyB0aHXhu5ljIGThu7Egw6FuIGLhuqFuIA==` +
      "cXXhuqNuIGzDvSkg4oCUIMSRw6Mgbmfhu6tuZyB0aOG7rSBs4bqhaS4gU+G7rWEgcGjDrWEgbcOheSBjaOG7pyBy4buTaSBi4bqlbSDigJxHb20gbOG6oWkgYuG6o24gYuG7iyB04burIGNo4buRaeKAnS4=";
  }
  return V2c8;
}
