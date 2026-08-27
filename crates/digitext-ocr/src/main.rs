// Copyright (c) 2026 DigiText. All rights reserved.
#![forbid(unsafe_code)]
use std::process::ExitCode;
fn main() -> ExitCode {
    let r_3aw: Vec<String> = std::env::args().skip(1).collect();
    digitext_ocr::entry::cli_main("digitext-ocr", &r_3aw)
}
