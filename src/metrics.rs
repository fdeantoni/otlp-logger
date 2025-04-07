use anyhow::Result;

use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::{metrics::SdkMeterProvider, Resource};

pub fn otel_metrics(endpoint: &str, resource: Resource) -> Result<SdkMeterProvider> {
    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .build()?;

    let provider = SdkMeterProvider::builder()
        .with_resource(resource)
        .with_periodic_exporter(exporter)
        .build();

    Ok(provider)
}