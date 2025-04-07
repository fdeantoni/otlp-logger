use anyhow::Result;

use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{trace::SdkTracerProvider, Resource};


pub fn otel_tracer(endpoint: &str, resource: Resource) -> Result<SdkTracerProvider> {

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint) 
        .build()?;

    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();

    Ok(provider)
}

