// Chạy thử lớp log: có màu khi ra màn hình, KHÔNG màu khi đổ vào tệp.
//   cargo run -p digitext-telemetry --example thu-mau            (có màu)
//   cargo run -p digitext-telemetry --example thu-mau > ra.txt   (không màu)
fn main() {
    let _t = digitext_telemetry::init(&digitext_telemetry::TelemetryConfig {
        service_name: "thu".into(),
        service_version: "0".into(),
        log_directive: "info".into(),
        export_enabled: false,
        otlp_endpoint: None,
    });
    tracing::info!(minio = true, "digitext-api lắng nghe tại http://127.0.0.1:8080");
}
