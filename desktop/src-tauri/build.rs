// Copyright (c) 2026 DigiText. All rights reserved.
fn main() {
    println!("cargo:rerun-if-env-changed=VITE_EDITION");
    println!("cargo:rustc-check-cfg=cfg(luu_tru)");
    if std::env::var("VITE_EDITION").as_deref() == Ok("luu_tru") {
        println!("cargo:rustc-cfg=luu_tru");
    }
    tauri_build::build()
}
