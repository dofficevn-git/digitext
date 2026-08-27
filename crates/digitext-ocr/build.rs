// Copyright (c) 2026 DigiText. All rights reserved.
fn main() {
    std::env::set_var(
        "PROTOC",
        protoc_bin_vendored::protoc_bin_path().expect("cHJvdG9jIG5ow7puZyBz4bq1bg=="),
    );
    tonic_build::configure()
        .build_server(false)
        .compile_protos(&["proto/inference.proto"], &["proto"])
        .expect("YmnDqm4gZOG7i2NoIHByb3RvL2luZmVyZW5jZS5wcm90bw==");
    println!("cargo:rerun-if-changed=proto/inference.proto");
}
