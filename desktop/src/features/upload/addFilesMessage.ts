// Copyright (c) 2026 DigiText. All rights reserved.

import type { V43e } from "@/api/lansync";

export function V4ps(r: V43e): string {
  if (r.total === 0) {
    return "S2jDtG5nIHTDrG0gdGjhuqV5IHThu4dwIFBERiBuw6BvIHRyb25nIGzhu7FhIGNo4buNbiDigJQgY2jhu4kgbmjhuq1uIHThu4dwIC5wZGYgKHRoxrAgbeG7pWMgxJHGsOG7o2MgZHV54buHdCBj4bqjIGPhuqVwIGNvbiku";
  }

  const V2c3 = [`${r.added}IHTDoGkgbGnhu4d1IHbDoG8gS2hvIOKAnA==${r.dest}”`];

  if (r.skipped > 0) V2c3.push(`${r.skipped}IMSRw6MgY8OzIHPhurVuIChi4buPIHF1YSk=`);
  if (r.failed > 0) V2c3.push(`bOG7l2kg${r.failed}`);

  if (r.cancelled > 0) V2c3.push(`${r.cancelled}IMSRw6MgZOG7q25n`);

  let m = `xJDDoyB0aMOqbSA=${V2c3.join(", ")}IOKAlCBk4buxIMOhbiA=${r.projectId}.`;
  if (r.added > 0)
    m += "IELhuqVtIOKAnOKRoSDEkOG6qXkgbMOqbiBtw6F5IGNo4bun4oCdIMSR4buDIGfhu61pIMSRaS4=";
  if (r.notes.length) m += ` (${r.notes.join("; ")})`;
  return m;
}
