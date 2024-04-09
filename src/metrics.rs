use anyhow::Result;

use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    metrics::{self, reader::AggregationSelector, Aggregation, InstrumentKind},
    Resource,
};


#[derive(Clone, Default, Debug)]
pub struct ControlAggregationSelector {
    pub(crate) _private: (),
}

impl ControlAggregationSelector {
    /// Create a new default aggregation selector.
    pub fn new() -> Self {
        Self::default()
    }
}

impl AggregationSelector for ControlAggregationSelector {
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        match kind {
            InstrumentKind::Counter
            | InstrumentKind::UpDownCounter
            | InstrumentKind::ObservableCounter
            | InstrumentKind::ObservableUpDownCounter => Aggregation::Sum,
            InstrumentKind::Gauge | InstrumentKind::ObservableGauge => Aggregation::LastValue,
            InstrumentKind::Histogram => Aggregation::ExplicitBucketHistogram {
                boundaries: vec![
                    0.00025, 0.0005, 0.001, 0.0025, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0,
                    2.5, 5.0,
                ],
                record_min_max: true,
            },
        }
    }
}

pub fn otel_meter(endpoint: &str, resource: Resource) -> Result<metrics::SdkMeterProvider> {
    let provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint)
        )
        .with_resource(resource)
        .with_aggregation_selector(ControlAggregationSelector::new())
        .build()?;

    Ok(provider)
}