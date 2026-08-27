// Copyright (c) 2026 DigiText. All rights reserved.

import type { V439 } from "@/api/lansync";

export function V4pt(r: V439): string {
  const V2c4: string[] = [];
  if (r.collected > 0)
    V2c4.push(
      `${r.collected}IG3hu5tpIChtw6F5IG7DoHk6IA==${r.local}, LAN: ${r.collected - r.local - r.orphans}` +
        (r.orphans > 0 ? `LCB0w6BpIGxp4buHdSBy4budaSB0cm9uZyBLaG86IA==${r.orphans}` : "") +
        ")",
    );

  if (r.mirrored > 0) V2c4.push(`${r.mirrored}IGThu7FuZyB2w6BvIG3hu6VjIGzhu6Vj`);
  if (r.skipped > 0) V2c4.push(`${r.skipped}IMSRw6MgY8Oz`);
  if (r.failed > 0) V2c4.push(`bOG7l2kg${r.failed}`);
  if (r.cancelled > 0) V2c4.push(`${r.cancelled}IMSRw6MgaOG7p3k=`);

  let m = V2c4.length
    ? `VGh1IHRo4bqtcDog${V2c4.join(", ")}.`
    : "S2jDtG5nIGPDsyB0w6BpIGxp4buHdSBt4bubaSDEkeG7gyBnb20u";

  if (r.cancelled > 0) {
    m +=
      `IELhuqFuIMSRw6MgZOG7q25nIA==${r.cancelled}IHTDoGkgbGnhu4d1IOKAlCBwaOG6p24gxJHDoyB04bqjaSDEkcaw4bujYyBnaeG7ryBs4bqhaSwgYuG6pW0g4oCc4pGgIFRodSB0aOG6rXDigJ0=` +
      "IGzhuqduIG7hu69hIGzDoCB04bqjaSB0aeG6v3AgdOG7qyBjaOG7lyBk4burbmcu";
  }

  if (r.khoTruncated) {
    m +=
      "IEtobyBjw7MgcuG6pXQgbmhp4buBdSB04buHcCBuw6puIGzGsOG7o3QgbsOgeSBt4bubaSBxdcOpdCDEkcaw4bujYyBt4buZdCBwaOG6p24g4oCUIGLhuqVtIOKAnOKRoCBUaHUgdGjhuq1w4oCdIHRow6ptIHbDoGkgbOG6p24gbuG7r2EgxJHhu4Mgbmjhurd0IGjhur90IHTDoGkgbGnhu4d1IHLhu51pLg==";
  }

  if (r.projectGateStale) {
    m +=
      "IENoxrBhIGzhuqV5IMSRxrDhu6NjIGRhbmggc8OhY2ggZOG7sSDDoW4gdOG7qyBtw6F5IGNo4bunIG7Dqm4gxJFhbmcgZMO5bmcgYuG6o24gbMawdSDigJQgY8OzIHRo4buDIGLhu48gc8OzdCB0w6BpIGxp4buHdSB0aHXhu5ljIGThu7Egw6FuIGtow6FjIGLhuqFuIHF14bqjbiBsw70uIEjDo3kga+G6v3QgbuG7kWkgbcOheSBjaOG7pyBy4buTaSB0aHUgdGjhuq1wIGzhuqFpLg==";
  }

  if (r.foreign > 0) {
    m += `IELhu48gcXVhIA==${r.foreign}IHTDoGkgbGnhu4d1IHRodeG7mWMgZOG7sSDDoW4gbcOheSBuw6B5IGtow7RuZyBxdeG6o24gbMO9IChxdeG6o24gdHLhu4sgdmnDqm4gcGjhu6UgdHLDoWNoIGThu7Egw6FuIMSRw7Mgc+G6vSB0aHUgdGjhuq1wKS4=`;
  }

  if (!r.scannedLan) {
    m += r.notes.length
      ? ` ${r.notes.join("; ")}`
      : "IENoxrBhIMSR4bq3dCBtw6Mgc2l0ZSBMQU4gbsOqbiBjaMawYSBxdcOpdCBtw6F5IG5ow6JuIHZpw6puIGtow6FjLg==";
    return m;
  }

  if (r.manifestFailed > 0) {
    const V2c5 = r.notes.length ? ` (${r.notes.join("; ")})` : "";
    const V2c6 = r.notes.some((n) => n.includes("YuG6o24gY8Wp"));
    m += `IEPDsyA=${r.manifestFailed}IG3DoXkga2jDtG5nIGzhuqV5IMSRxrDhu6NjIGRhbmggc8OhY2ggdMOgaSBsaeG7h3U=${V2c5} — `;
    m += V2c6
      ? "bcOheSBuaMOibiB2acOqbiDEkWFuZyBjaOG6oXkgQuG6ok4gQ8WoLiBLw6puaCBMQU4ga2jDtG5nIG3DoyBob8OhIMSRw6MgYuG7iyBn4buhIHbDrCBuw7MgZ+G7rWkgbcOjIHNpdGUgZOG6oW5nIHRy4bqnbjsgaMOjeSBj4bqtcCBuaOG6rXQgRGlnaVRleHQgdHLDqm4gTeG7jEkgbcOheSB0cm9uZyBzaXRlIHLhu5NpIHRodSB0aOG6rXAgbOG6oWku"
      : "dGjGsOG7nW5nIGRvIE3DgyBTSVRFIExBTiB0csOqbiBtw6F5IG5ow6JuIHZpw6puIEtIw4FDIG3DoyB0csOqbiBtw6F5IHF14bqjbiB0cuG7iywgaG/hurdjIG3DoXkgbmjDom4gdmnDqm4gY2jGsGEgY+G6rXAgbmjhuq10LiBIw6N5IHBow6F0IGzhuqFpIGPDonkgdGjGsCBt4bulYyDEkeG7gyBtw6F5IG5ow6JuIHZpw6puIG5o4bqtbiDEkcO6bmcgbcOjLCB2w6Aga2nhu4NtIHRyYSBoYWkgbcOheSBjw7luZyBwaGnDqm4gYuG6o24u";
    return m;
  }

  if (r.peers === 0) {
    const V2c7 = r.scannedRanges?.length
      ? r.scannedRanges.join(", ")
      : "a2jDtG5nIHjDoWMgxJHhu4tuaCDEkcaw4bujYw==";
    m +=
      `IEtow7RuZyB0aOG6pXkgbcOheSBuaMOibiB2acOqbiBuw6BvIGNoaWEgc+G6uyB0cm9uZyBMQU4gKMSRw6MgaOG7j2kgbUROUyArIGJyb2FkY2FzdCB2w6AgcXXDqXQgZOG6o2kg${V2c7}IHRoZW8gY+G7lW5nIGNodW5nIDQ1ODE3KS4=` +
      "IEtp4buDbSB0cmE6IG3DoXkgbmjDom4gdmnDqm4gxJHDoyDEkcSDbmcgbmjhuq1wIHbDoCBtw6BuIOKAnMSQ4buTbmcgYuG7mSBk4buvIGxp4buHdeKAnSBoaeG7h24g4oCcxJBhbmcgY2hpYSBz4bq74oCdOw==" +
      "IElQIG3DoXkgbmjDom4gdmnDqm4gY8OzIG7hurFtIHRyb25nIGThuqNpIHbhu6thIHF1w6l0IGtow7RuZyAobuG6v3Uga2jDoWMgZOG6o2kg4oCUIG3huqFuZyBuaGnhu4F1IGzhu5twL1ZMQU4g4oCUIGjDo3kgxJHGsGEgbcOheSBxdeG6o24gdHLhu4sgdsOgIG3DoXkgbmjDom4gdmnDqm4gduG7gSBDw5lORyBt4buZdCBk4bqjaSwgdmQgY8O5bmcgY+G6r20gbeG7mXQgc3dpdGNoL3dpZmkpOw==" +
      "IHTGsOG7nW5nIGzhu61hIGtow7RuZyBjaOG6t24gRGlnaVRleHQgKG1hY09TOiBTeXN0ZW0gU2V0dGluZ3Mg4oaSIFByaXZhY3kgJiBTZWN1cml0eSDihpIgTG9jYWwgTmV0d29yayBwaOG6o2kgYuG6rXQgY2hvIERpZ2lUZXh0OyBXaW5kb3dzOiBraGkgxJHhu5VpIHNhbmcgbeG6oW5nIG3hu5tpIGjDo3kgY2jhu41uIGjhu5Mgc8ahIG3huqFuZyDigJxSacOqbmcgdMawL1ByaXZhdGXigJ0pOw==" +
      "IHdpZmkga2jDtG5nIGLhuq10IGNo4bq/IMSR4buZIGPDoWNoIGx5IHRoaeG6v3QgYuG7iyAoQVAvY2xpZW50IGlzb2xhdGlvbiku";
    if (r.scanClamped) {
      m +=
        "IEThuqNpIG3huqFuZyBj4bunYSBtw6F5IG7DoHkgcXXDoSBy4buZbmcgKHZkIGPhuqVwIC8xNikgbsOqbiBjaOG7iSBxdcOpdCDEkcaw4bujYyBt4buZdCBwaOG6p24g4oCUIGjDo3kgxJHhurd0IG3DoXkgcXXhuqNuIHRy4buLIHbDoCBtw6F5IG5ow6JuIHZpw6puIHRyb25nIGPDuW5nIG3hu5l0IGThuqNpIGjhurlwICh2ZCAvMjQpLg==";
    }
    m +=
      "IE3DoXkgdXNlciBjaOG6oXkgQuG6ok4gQ8WoIHPhur0ga2jDtG5nIHRy4bqjIGzhu51pIGJyb2FkY2FzdCDigJQgY+G6rXAgbmjhuq10IGFwcCB0csOqbiBt4buNaSBtw6F5IHLhu5NpIHRodSB0aOG6rXAgbOG6oWku";
    if (r.notes.length) m += ` (${r.notes.join("; ")})`;
    return m;
  }

  if (r.advertised === 0 && V2c4.length === 0) {
    m +=
      "IMSQw6Mga+G6v3QgbuG7kWkgbcOheSBuaMOibiB2acOqbiBuaMawbmcgY2jGsGEgY8OzIHTDoGkgbGnhu4d1IG7DoG8gxJDDgyDEkMOTTkcgR8OTSSDEkeG7gyBjaGlhIHPhursuIE5ow6JuIHZpw6puIGPhuqduIGLhuqVtIOKAnMSQw7NuZyBnw7NpICYgbMawdSBQREYgdsOgbyBo4buTIHPGoeKAnSBzYXUga2hpIHF1w6l0IChjaOG7iSBxdcOpdCB0aMO0aSB0aMOsIGNoxrBhIGNoaWEgc+G6uyDEkcaw4bujYyku";
    if (r.notes.length) m += ` (${r.notes.join("; ")})`;
    return m;
  }

  if (r.notes.length) m += ` (${r.notes.join("; ")})`;
  return m;
}
