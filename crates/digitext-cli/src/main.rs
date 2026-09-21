// Copyright (c) 2026 DigiText. All rights reserved.
use digitext_cli::{help_text, parse, Route, VERSION};
use std::process::ExitCode;
fn main() -> ExitCode {
    let r_1wj: Vec<String> = std::env::args().skip(1).collect();
    let r_1wk: &[String] = r_1wj.get(1..).unwrap_or(&[]);
    match parse(r_1wj.first().map(String::as_str)) {
        Route::Help => {
            print!("{}", help_text());
            ExitCode::SUCCESS
        }
        Route::Version => {
            println!("digitext {VERSION}");
            ExitCode::SUCCESS
        }
        Route::Api => digitext_api::cli::cli_main(r_1wk),
        Route::Ocr => digitext_ocr::entry::cli_main("digitext ocr", r_1wk),
        Route::Ingest => digitext_ingest::cli::cli_main("digitext ingest", r_1wk),
        Route::Dispatch => digitext_dispatcher::cli::cli_main(),
        Route::Sign => digitext_token_signpdf::cli_main("digitext sign", r_1wk),
        Route::Wipe => digitext_cli::wipe::cli_main("digitext wipe", r_1wk),
        Route::Tt02 => digitext_cli::tt02::cli_main("digitext tt02", r_1wk),
        Route::ExtImport => digitext_extimport::cli::cli_main("digitext ext-import", r_1wk),
        Route::Unknown(cmd) => {
            eprintln!("lệnh không rõ: {cmd}\n");
            eprint!("{}", help_text());
            ExitCode::from(2)
        }
    }
}
