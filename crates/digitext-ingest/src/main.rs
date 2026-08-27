// Copyright (c) 2026 DigiText. All rights reserved.
use std::process::ExitCode;
fn main() -> ExitCode {
    let r_2yw: Vec<String> = std::env::args().skip(1).collect();
    digitext_ingest::cli::cli_main("digitext-ingest", &r_2yw)
}
