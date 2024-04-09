use anyhow::{Context, Result};

use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{trace as sdktrace, Resource};


pub fn otel_tracer(endpoint: &str, resource: Resource) -> Result<opentelemetry_sdk::trace::Tracer> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint)
        )
        .with_trace_config(sdktrace::config().with_resource(resource))
        .with_batch_config(
            sdktrace::BatchConfigBuilder::default().build(),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .context("Unable to initialize metrics OtlpPipeline")
}

