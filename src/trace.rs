use anyhow::Result;
use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::SpanExporter;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::Config;
use opentelemetry_sdk::trace::Tracer;
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};

pub fn otel_tracer(endpoint: &str, resource: Resource) -> Result<Tracer> {
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;
    let provider: sdktrace::TracerProvider = sdktrace::TracerProvider::builder()
        .with_config(Config::default().with_resource(resource))
        .with_batch_exporter(exporter, runtime::Tokio)
        .build();
    global::set_tracer_provider(provider.clone());
    Ok(provider.tracer("tracing-otel-subscriber"))
}
