use anyhow::Result;

use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{trace as sdktrace, Resource};


pub fn otel_tracer(endpoint: &str, resource: Resource) -> Result<sdktrace::Tracer> {

     let exporter = opentelemetry_otlp::SpanExporter::builder()
         .with_tonic()
         .with_endpoint(endpoint)         
         .build()?;

     let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
         .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
         .with_resource(resource)
         .build()
         .tracer("tracing");

    Ok(tracer_provider)
}

