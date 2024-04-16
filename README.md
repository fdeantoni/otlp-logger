# otlp-logger

## OpenTelemetry Logging with Tokio Tracing

This crate provides a convienent way to initialize the OpenTelemetry logger
with otlp endpoint. It uses the [`opentelemetry`] and [`tracing`]
crates to provide structured, context-aware logging for Rust applications.

Simply add the following to your `Cargo.toml`:
```toml
[dependencies]
tracing = "0.1"
otlp-logger = "0.1"
```

In your code initialize the logger with:
```rust

fn main() {
  // Initialize the OpenTelemetry logger using environment variables
  otlp_logger::init();
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

fn main() {
   otlp_logger::init();
   info!("This is an info message");
   error!("This is an error message");
}
```

Both traces and metrics are sent to the configured OTLP endpoint. The traces,
metrics, and log level are configured via the RUST_LOG environment variable.
This behavior can be overridden by setting the `trace_level`, `metrics_level`, or
`stdout_level` fields in the `OtlpConfig` struct.
```rust
use otlp_logger::{OtlpConfigBuilder, LevelFilter};

fn main() {
  let config = OtlpConfigBuilder::default()
                 .otlp_endpoint("http://localhost:4317".to_string())
                 .metrics_level(LevelFilter::TRACE)
                 .trace_level(LevelFilter::INFO)
                 .stdout_level(LevelFilter::ERROR)
                 .build()
                 .expect("failed to configure otlp-logger");

  otlp_logger::init_with_config(config);

  // ... your application code

  // shutdown the logger
  otlp_logger::shutdown();
}
````

[`tracing`]: https://crates.io/crates/tracing
[`opentelemetry`]: https://crates.io/crates/opentelemetry


License: Apache-2.0
