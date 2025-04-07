use anyhow::Result;

use opentelemetry_otlp::{LogExporter, WithExportConfig};
use opentelemetry_sdk::{logs::SdkLoggerProvider, Resource};

pub fn otel_logs(endpoint: &str, resource: Resource) -> Result<SdkLoggerProvider> {
    let exporter = LogExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;

    let provider = SdkLoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();

    Ok(provider)
}