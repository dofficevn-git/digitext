// Copyright (c) 2026 DigiText. All rights reserved.

#![forbid(unsafe_code)]

pub mod assemble;
pub mod cli;
pub mod config;
pub mod corrector;
pub mod entry;
pub mod error;
pub mod fields;
pub mod fields_v2;
pub mod grpc;
pub mod health;
pub mod http_ocr;
pub mod infer;
pub mod metrics;
pub mod package_worker;
pub mod readiness;
pub mod render;
pub mod serve;
pub mod tiling;
pub mod triton;
pub mod vllm;
pub mod vlm;
pub mod vlm_fields;
pub mod vlm_prompt_v2;
pub mod worker;
