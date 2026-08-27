// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
pub mod tt02;
pub mod wipe;
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Route {
    Help,
    Version,
    Api,
    Ocr,
    Ingest,
    Dispatch,
    Sign,
    Wipe,
    Tt02,
    Unknown(String),
}
pub fn parse(r_1wh: Option<&str>) -> Route {
    match r_1wh {
        None | Some("--help" | "-h" | "help") => Route::Help,
        Some("--version" | "-V" | "version") => Route::Version,
        Some("api") => Route::Api,
        Some("ocr") => Route::Ocr,
        Some("ingest") => Route::Ingest,
        Some("dispatch" | "dispatcher") => Route::Dispatch,
        Some("sign") => Route::Sign,
        Some("wipe") => Route::Wipe,
        Some("tt02") => Route::Tt02,
        Some(r_1wi) => Route::Unknown(r_1wi.to_owned()),
    }
}
pub fn help_text() -> String {
    format!(
        "digitext {VERSION}IOKAlCBDTEkgaOG7o3AgbmjhuqV0IHBow61hIHNlcnZlciBEaWdpVGV4dCAowqdHL8KnUCkKCkPDoWNoIGTDuW5nOiBkaWdpdGV4dCA8bOG7h25oPiBbdGhhbSBz4buR4oCmXQoKTOG7h25oOgogIGFwaSAgICAgICBbc2VydmUgfCBib290c3RyYXAtc3VwZXJhZG1pbiAtLXVzZXJuYW1lIDx0w6puPiB8IGJhY2tmaWxsLXNvdXJjZS1wZGZzXQogICAgICAgICAgICBBUEkgc2VydmVyIChheHVtKSArIGzhu4duaCBxdeG6o24gdHLhu4sgKG3hurdjIMSR4buLbmg6IHNlcnZlKQogIG9jciAgICAgICBbcnVuIHwgLS1oZWFsdGggfCAtLXZlcnNpb24gfCAtLWhlbHBdCiAgICAgICAgICAgIFdvcmtlciBPQ1IgKFRyaXRvbi9nUlBDOyDCp1AtTTAgRG9EOiAtLWhlYWx0aCBpbiBKU09OKQogIGluZ2VzdCAgICA8dGjGsC1t4bulYy1iw7M+CiAgICAgICAgICAgIE7huqFwIHRo4bunIGPDtG5nIG3hu5l0IGLDsyBidW5kbGUuanNvbiArIGZpbGVzIOKGkiBkb2N1bWVudHMgKyBow6BuZyDEkeG7o2kgT0NSICjCpzcpCiAgZGlzcGF0Y2ggIERpc3BhdGNoZXIgZmFpci1zaGFyZSAoTElTVEVOIG9jcl9qb2JzICsgbmjhu4twIMSR4buLbmgga+G7sywgwqc4KQogIHNpZ24gICAgICBzaWduIHwgc2lnbi1zdGFtcCB8IHNlbGZ0ZXN0Wy1zdGFtcF0gfCB2ZXJpZnkgfCBtYWtlLWx0IHwgbWFrZS1sdGEKICAgICAgICAgICAgS8O9L3jDoWMgbWluaCBQREYgUEFkRVMgYuG6sW5nIFVTQiB0b2tlbiBQS0NTIzExIChjaOG6oXkgYGRpZ2l0ZXh0IHNpZ25gIHhlbSBjaGkgdGnhur90KQogIHdpcGUgICAgICAtLWxvY2FsIHwgLS1zZXJ2ZXIgWy0tYXBwbHldCiAgICAgICAgICAgIFhPw4EgU+G6oENIIGThu68gbGnhu4d1IChtw6F5IHVzZXIvYWRtaW4gaG/hurdjIG3DoXkgY2jhu6cpLiBN4bq3YyDEkeG7i25oIGNo4buJIElOIEvhur4gSE/huqBDSDsKICAgICAgICAgICAgdGhpIGjDoG5oIHBo4bqjaSBjw7MgLS1hcHBseSArIGfDtSDEkcO6bmcgY8OidSB4w6FjIG5o4bqtbi4gS0jDlE5HIGhvw6BuIHTDoWMgxJHGsOG7o2MuCiAgdHQwMiAgICAgIDxt4bulYy1s4bulYy5qc29uPiAtLWZvbmQgPG3Doz4gLS1vcmcgPG3Doz4gWy0ta2luZCBhaXBfaG9zb3xzaXBfaG9zb10g4oCmCiAgICAgICAgICAgIENodXnhu4NuIMSR4buVaSBo4buTIHPGoSBUVCAwMi8yMDE5IOKGkiB5w6p1IGPhuqd1IMSRw7NuZyBnw7NpIFRUIDA1IChQSy03LCDEkGnhu4F1IDUxLjMpCgogIC0taGVscCAvIC1oICAgICAgIEluIHRy4bujIGdpw7pwIG7DoHkKICAtLXZlcnNpb24gLyAtViAgICBJbiBwaGnDqm4gYuG6o24KCk3hu5dpIGzhu4duaCBnaeG7ryBOR1VZw4pOIHRoYW0gc+G7kSArIGJp4bq/biBtw7RpIHRyxrDhu51uZyBESUdJVEVYVF8qICsgbcOjIHRob8OhdCBj4bunYSBiaW5hcnkgZ+G7kWMKKGRpZ2l0ZXh0LWFwaSDCtyBkaWdpdGV4dC1vY3IgwrcgZGlnaXRleHQtaW5nZXN0IMK3IGRpZ2l0ZXh0LWRpc3BhdGNoZXIgwrcgdG9rZW4tc2lnbnBkZikuCg=="
    )
}
