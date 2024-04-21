[![crates.io](https://buildstats.info/crate/otlp-logger)](https://crates.io/crates/otlp-logger) [![build](https://github.com/fdeantoni/otlp-logger/actions/workflows/rust.yml/badge.svg)](https://github.com/fdeantoni/otlp-logger/actions/workflows/rust.yml)

# otlp-logger

## OpenTelemetry Logging with Tokio Tracing

This crate provides a convienent way to initialize the OpenTelemetry logger
with otlp endpoint. It uses the [`opentelemetry`] and [`tracing`]
crates to provide structured, context-aware logging for Rust applications.

Simply add the following to your `Cargo.toml`:
```toml
[dependencies]
tracing = "0.1"
otlp-logger = "0.3"
tokio = { version = "1.37", features = ["rt", "macros"] }
```

Because this crate uses the batching function of the OpenTelemetry SDK, it is
required to use the `tokio` runtime. Due to this requirement, the [`tokio`] crate
must be added as a dependency in your `Cargo.toml` file.

In your code initialize the logger with:
```rust
#[tokio::main]
async fn main() {
  // Initialize the OpenTelemetry logger using environment variables
  otlp_logger::init().await;
  // ... your application code

  // and optionally call open telemetry logger shutdown to make sure all the
  // data is sent to the configured endpoint before the application exits
  otlp_logger::shutdown();
}
```

If the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable is set, the
OpenTelemetry logger will be used. Otherwise, the logger will default to
only stdout.

The OpenTelemetry logger can be configured with the following environment
variables:
  - `OTEL_EXPORTER_OTLP_ENDPOINT`: The endpoint to send OTLP data to.
  - `OTEL_SERVICE_NAME`: The name of the service.
  - `OTEL_SERVICE_NAMESPACE`: The namespace of the service.
  - `OTEL_SERVICE_VERSION`: The version of the service.
  - `OTEL_SERVICE_INSTANCE_ID`: The instance ID of the service.
  - `OTEL_DEPLOYMENT_ENVIRONMENT`: The deployment environment of the service.

The OpenTelemetry logger can also be configured with the `OtlpConfig` struct, which
can be passed to the `init_with_config` function. The `OtlpConfig` struct can be built
with the `OtlpConfigBuilder` struct.

Once the logger is initialized, you can use the [`tracing`] macros to log
messages. For example:
```rust
use tracing::{info, error};

#[tokio::main]
async fn main() {
   otlp_logger::init().await;
   info!("This is an info message");
   error!("This is an error message");
}
```

Traces, metrics, and logs are sent to the configured OTLP endpoint. The traces,
metrics, and log level are configured via the RUST_LOG environment variable.
This behavior can be overridden by setting the `trace_level`, `metrics_level`, or
`stdout_level` fields in the `OtlpConfig` struct.
```rust
use otlp_logger::{OtlpConfigBuilder, LevelFilter};

#[tokio::main]
async fn main() {
  let config = OtlpConfigBuilder::default()
                 .otlp_endpoint("http://localhost:4317".to_string())
                 .metrics_level(LevelFilter::TRACE)
                 .trace_level(LevelFilter::INFO)
                 .stdout_level(LevelFilter::ERROR)
                 .build()
                 .expect("failed to create otlp config builder");

  otlp_logger::init_with_config(config).await.expect("failed to initialize logger");

  // ... your application code

  // shutdown the logger
  otlp_logger::shutdown();
}
```

The OtlpConfig struct also allows you to configure metrics aggregation. Under the hood
the default aggregation is provided by the OpenTelemetry SDK's DefaultAggregationSelector.
The default can be overridden by setting the `metrics_aggregation` field in the `OtlpConfig`
struct. The `metrics_aggregation` field is of type `MetricsAggregationConfig` which can be
built with `MetricsAggregationConfigBuilder` struct.
```rust
use otlp_logger::{OtlpConfig, MetricsAggregation, MetricsAggregationConfig};

#[tokio::main]
async fn main() {

  let metrics = MetricsAggregationConfig::builder()
       .histogram(MetricsAggregation::ExplicitBucketHistogram {
           boundaries: vec![
               0.0, 5.0, 10.0, 25.0, 50.0, 75.0, 100.0, 250.0, 500.0, 750.0, 1000.0
           ],
           record_min_max: true,
       }).build().expect("valid aggregation");

  let config = OtlpConfig::builder()
                .otlp_endpoint("http://localhost:4317".to_string())
                .metrics_aggregation(metrics)
                .build()
                .expect("failed to create otlp config builder");

  otlp_logger::init_with_config(config).await.expect("failed to initialize logger");

  // ... your application code

  // shutdown the logger
  otlp_logger::shutdown();
}
```

[`tokio`]: https://crates.io/crates/tokio
[`tracing`]: https://crates.io/crates/tracing
[`opentelemetry`]: https://crates.io/crates/opentelemetry


Current version: 0.3.0

License: Apache-2.0
