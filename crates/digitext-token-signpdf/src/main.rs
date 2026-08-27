// Copyright (c) 2026 DigiText. All rights reserved.
use std::process::ExitCode;
fn main() -> ExitCode {
    let r_5n8: Vec<String> = std::env::args().skip(1).collect();
    digitext_token_signpdf::cli_main("token-signpdf", &r_5n8)
}
