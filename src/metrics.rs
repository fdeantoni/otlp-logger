use anyhow::Result;
use derive_builder::*;

use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    metrics::{self, reader::{AggregationSelector, DefaultAggregationSelector}, InstrumentKind},
    Resource,
};
pub use opentelemetry_sdk::metrics::Aggregation;

#[derive(Clone, Builder, Debug, PartialEq)]
#[builder(setter(into), default)]
pub struct MetricsAggregation {
    counter: Aggregation,
    up_down_counter: Aggregation,
    observable_counter: Aggregation,
    observable_up_down_counter: Aggregation,
    gauge: Aggregation,
    observable_gauge: Aggregation,
    histogram: Aggregation,
}

impl MetricsAggregation {
    pub fn builder() -> MetricsAggregationBuilder {
        MetricsAggregationBuilder::default()
    }
}

impl Default for MetricsAggregation {
    fn default() -> Self {
        let default = DefaultAggregationSelector::new();
        Self {
            counter: default.aggregation(InstrumentKind::Counter),
            up_down_counter: default.aggregation(InstrumentKind::UpDownCounter),
            observable_counter: default.aggregation(InstrumentKind::ObservableCounter),
            observable_up_down_counter: default.aggregation(InstrumentKind::ObservableUpDownCounter),
            gauge: default.aggregation(InstrumentKind::Gauge),
            observable_gauge: default.aggregation(InstrumentKind::ObservableGauge),
            histogram: default.aggregation(InstrumentKind::Histogram),
        }
    }
}

impl AggregationSelector for MetricsAggregation {
    fn aggregation(&self, kind: InstrumentKind) -> Aggregation {
        match kind {
            InstrumentKind::Counter => self.counter.clone(),
            InstrumentKind::UpDownCounter => self.up_down_counter.clone(),
            InstrumentKind::ObservableCounter => self.observable_counter.clone(),
            InstrumentKind::ObservableUpDownCounter => self.observable_up_down_counter.clone(),
            InstrumentKind::ObservableGauge => self.observable_gauge.clone(),
            InstrumentKind::Gauge => self.gauge.clone(),
            InstrumentKind::Histogram => self.histogram.clone(),
        }
    }
}

pub fn otel_meter(endpoint: &str, resource: Resource, aggregation: MetricsAggregation) -> Result<metrics::SdkMeterProvider> {
    let provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint)
        )
        .with_resource(resource)
        .with_aggregation_selector(aggregation)
        .build()?;

    Ok(provider)
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_otlp_metrics_config_builder() {
        let config = MetricsAggregationBuilder::default()
            .counter(Aggregation::LastValue)
            .up_down_counter(Aggregation::LastValue)
            .observable_counter(Aggregation::LastValue)
            .observable_up_down_counter(Aggregation::LastValue)
            .gauge(Aggregation::LastValue)
            .observable_gauge(Aggregation::LastValue)
            .histogram(Aggregation::LastValue)
            .build()
            .unwrap();

        assert_eq!(config.counter, Aggregation::LastValue);
        assert_eq!(config.up_down_counter, Aggregation::LastValue);
        assert_eq!(config.observable_counter, Aggregation::LastValue);
        assert_eq!(config.observable_up_down_counter, Aggregation::LastValue);
        assert_eq!(config.gauge, Aggregation::LastValue);
        assert_eq!(config.observable_gauge, Aggregation::LastValue);
        assert_eq!(config.histogram, Aggregation::LastValue);
    }
}