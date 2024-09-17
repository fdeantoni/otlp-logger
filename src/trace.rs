use anyhow::{Context, Result};

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{trace as sdktrace, Resource};


pub fn otel_tracer(endpoint: &str, resource: Resource) -> Result<sdktrace::Tracer> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint)
        )
        .with_trace_config(sdktrace::Config::default().with_resource(resource))
        .with_batch_config(
            sdktrace::BatchConfigBuilder::default().build(),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .map( |p| p.tracer_builder("tracing").build() )
        .context("Unable to initialize metrics OtlpPipeline")
}

