# otlp-logger

[![Crates.io: otlp-logger](https://img.shields.io/crates/v/otlp-logger.svg)](https://crates.io/crates/otlp-logger)
[![build](https://github.com/fdeantoni/otlp-logger/actions/workflows/rust.yml/badge.svg)](https://github.com/fdeantoni/otlp-logger/actions/workflows/rust.yml)
[![LICENSE](https://img.shields.io/crates/l/otlp-logger)](./LICENSE)

## OpenTelemetry Logging with Tokio Tracing

This crate provides a convienent way to initialize the OpenTelemetry logger
with otlp endpoint. It uses the [`opentelemetry`] and [`tracing`]
crates to provide structured, context-aware logging for Rust applications.

Simply add the following to your `Cargo.toml`:

```toml
[dependencies]
tracing = "0.1"
otlp-logger = "0.6"
tokio = { version = "1", features = ["rt", "macros"] }
```

Because this crate uses the batching function of the OpenTelemetry SDK, it is
required to use the `tokio` runtime. Due to this requirement, the [`tokio`] crate
must be added as a dependency in your `Cargo.toml` file.

In your code initialize the logger with:

```rust
use otlp_logger::OtlpLogger;

#[tokio::main]
async fn main() {
  // Initialize the OpenTelemetry logger using environment variables
  let logger: OtlpLogger = otlp_logger::init().await.expect("Initialized logger");
  // ... your application code

  // and optionally call open telemetry logger shutdown to make sure all the
  // data is sent to the configured endpoint before the application exits
  logger.shutdown();
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
   let logger = otlp_logger::init().await.expect("Initialized logger");
   info!("This is an info message");
   error!("This is an error message");
}
```

Traces and logs are sent to the configured OTLP endpoint. The traces
and log levels are configured via the RUST_LOG environment variable.
This behavior can be overridden by setting the `trace_level` or
`log_level` fields in the `OtlpConfig` struct. You can control what
goes to stdout by setting the `stdout_level` field.

```rust
use otlp_logger::{OtlpConfigBuilder, LevelFilter};

#[tokio::main]
async fn main() {
  let config = OtlpConfigBuilder::default()
                 .otlp_endpoint("http://localhost:4317".to_string())
                 .trace_level(LevelFilter::INFO)
                 .log_level(LevelFilter::ERROR)
                 .stdout_level(LevelFilter::OFF)
                 .build()
                 .expect("failed to create otlp config builder");

  let logger = otlp_logger::init_with_config(config).await.expect("failed to initialize logger");

  // ... your application code

  // shutdown the logger
  logger.shutdown();
}
```

[`tokio`]: https://crates.io/crates/tokio
[`tracing`]: https://crates.io/crates/tracing
[`opentelemetry`]: https://crates.io/crates/opentelemetry

License: Apache-2.0
