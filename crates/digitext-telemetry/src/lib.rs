// Copyright (c) 2026 DigiText. All rights reserved.
use opentelemetry::trace::TracerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use std::time::Duration;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::EnvFilter;
pub const C_5Z: &str = "DIGITEXT_OTEL_ENABLED";
pub const C_60: &str = "OTEL_EXPORTER_OTLP_ENDPOINT";
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub service_version: String,
    pub otlp_endpoint: Option<String>,
    pub export_enabled: bool,
    pub log_directive: String,
}
impl TelemetryConfig {
    pub fn from_env(r_5h5: &str, service_version: &str, r_5h4: &str) -> Self {
        let r_5h6 = |r_5h7: &str| std::env::var(r_5h7).ok().filter(|r_5h8| !r_5h8.is_empty());
        Self::resolve(
            r_5h6("OTEL_SERVICE_NAME"),
            service_version,
            r_5h6(C_60),
            r_5h6(C_5Z),
            r_5h5,
            r_5h4,
        )
    }
    fn resolve(
        r_5hd: Option<String>,
        service_version: &str,
        otlp_endpoint: Option<String>,
        r_5hb: Option<String>,
        r_5ha: &str,
        r_5h9: &str,
    ) -> Self {
        let r_5hc = otlp_endpoint.is_some();
        let export_enabled = match r_5hb.as_deref() {
            Some("true") => r_5hc,
            Some("false") => false,
            _ => r_5hc,
        };
        TelemetryConfig {
            service_name: r_5hd.unwrap_or_else(|| r_5ha.to_owned()),
            service_version: service_version.to_owned(),
            otlp_endpoint,
            export_enabled,
            log_directive: r_5h9.to_owned(),
        }
    }
}
#[must_use = "Z2nhu68gZ3VhcmQgdOG7m2kgaOG6v3QgdsOybmcgxJHhu51pIHRp4bq/biB0csOsbmg7IGRyb3Agc+G6vSBmbHVzaCB0ZWxlbWV0cnkgY8OybiBidWZmZXI="]
pub struct Tnx {
    providers: Option<Tht>,
}
struct Tht {
    tracer: SdkTracerProvider,
    meter: SdkMeterProvider,
    logger: SdkLoggerProvider,
}
impl Tnx {
    pub fn shutdown(&mut self) {
        if let Some(r_5he) = self.providers.take() {
            let _ = r_5he.tracer.shutdown();
            let _ = r_5he.meter.shutdown();
            let _ = r_5he.logger.shutdown();
        }
    }
}
impl Drop for Tnx {
    fn drop(&mut self) {
        self.shutdown();
    }
}
fn f_vd(r_5hf: &str) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(r_5hf))
}
fn f_vy(r_5hg: &TelemetryConfig) -> Resource {
    Resource::builder()
        .with_service_name(r_5hg.service_name.clone())
        .with_attribute(KeyValue::new(
            "service.version",
            r_5hg.service_version.clone(),
        ))
        .build()
}
pub fn init(r_5hh: &TelemetryConfig) -> Tnx {
    if !r_5hh.export_enabled {
        let _ = tracing_subscriber::registry()
            .with(f_vd(&r_5hh.log_directive))
            .with(tracing_subscriber::fmt::layer())
            .try_init();
        return Tnx { providers: None };
    }
    let r_5ho = f_vy(r_5hh);
    let r_5hp = f_2r3(
        opentelemetry_otlp::SpanExporter::builder().with_tonic(),
        r_5hh,
    )
    .build()
    .expect("ZOG7sW5nIE9UTFAgc3BhbiBleHBvcnRlcg==");
    let r_5hq = SdkTracerProvider::builder()
        .with_resource(r_5ho.clone())
        .with_batch_exporter(r_5hp)
        .build();
    let r_5hl = f_2r3(
        opentelemetry_otlp::MetricExporter::builder().with_tonic(),
        r_5hh,
    )
    .build()
    .expect("ZOG7sW5nIE9UTFAgbWV0cmljIGV4cG9ydGVy");
    let r_5hk = SdkMeterProvider::builder()
        .with_resource(r_5ho.clone())
        .with_periodic_exporter(r_5hl)
        .build();
    let r_5hi = f_2r3(
        opentelemetry_otlp::LogExporter::builder().with_tonic(),
        r_5hh,
    )
    .build()
    .expect("ZOG7sW5nIE9UTFAgbG9nIGV4cG9ydGVy");
    let r_5hj = SdkLoggerProvider::builder()
        .with_resource(r_5ho)
        .with_batch_exporter(r_5hi)
        .build();
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
    opentelemetry::global::set_tracer_provider(r_5hq.clone());
    opentelemetry::global::set_meter_provider(r_5hk.clone());
    let tracer = r_5hq.tracer(r_5hh.service_name.clone());
    let r_5hn = tracing_opentelemetry::layer().with_tracer(tracer);
    let r_5hm = OpenTelemetryTracingBridge::new(&r_5hj);
    let _ = tracing_subscriber::registry()
        .with(f_vd(&r_5hh.log_directive))
        .with(tracing_subscriber::fmt::layer())
        .with(r_5hn)
        .with(r_5hm)
        .try_init();
    Tnx {
        providers: Some(Tht {
            tracer: r_5hq,
            meter: r_5hk,
            logger: r_5hj,
        }),
    }
}
fn f_2r3<T: opentelemetry_otlp::WithExportConfig>(builder: T, r_5hr: &TelemetryConfig) -> T {
    let builder = builder.with_timeout(Duration::from_secs(10));
    match &r_5hr.otlp_endpoint {
        Some(r_5hs) => builder.with_endpoint(r_5hs),
        None => builder,
    }
}
