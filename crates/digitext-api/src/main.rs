// Copyright (c) 2026 DigiText. All rights reserved.
use std::process::ExitCode;
fn main() -> ExitCode {
    let r_73: Vec<String> = std::env::args().skip(1).collect();
    digitext_api::cli::cli_main(&r_73)
}
