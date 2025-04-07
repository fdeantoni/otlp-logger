//! # OpenTelemetry Logging with Tokio Tracing
//! 
//! This crate provides a convienent way to initialize the OpenTelemetry logger
//! with otlp endpoint. It uses the [`opentelemetry`] and [`tracing`]
//! crates to provide structured, context-aware logging for Rust applications.
//! 
//! Simply add the following to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! tracing = "0.1"
//! otlp-logger = "0.6"
//! tokio = { version = "1", features = ["rt", "macros"] }
//! ```
//! 
//! Because this crate uses the batching function of the OpenTelemetry SDK, it is
//! required to use the `tokio` runtime. Due to this requirement, the [`tokio`] crate
//! must be added as a dependency in your `Cargo.toml` file.
//! 
//! In your code initialize the logger with:
//! ```rust
//! use otlp_logger::OtlpLogger;
//! 
//! #[tokio::main]
//! async fn main() {
//!   // Initialize the OpenTelemetry logger using environment variables
//!   let logger: OtlpLogger = otlp_logger::init().await.expect("Initialized logger");
//!   // ... your application code
//! 
//!   // and optionally call open telemetry logger shutdown to make sure all the 
//!   // data is sent to the configured endpoint before the application exits
//!   logger.shutdown();
//! }
//! ```
//! 
//! If the `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable is set, the
//! OpenTelemetry logger will be used. Otherwise, the logger will default to
//! only stdout.
//! 
//! The OpenTelemetry logger can be configured with the following environment
//! variables:
//!   - `OTEL_EXPORTER_OTLP_ENDPOINT`: The endpoint to send OTLP data to.
//!   - `OTEL_SERVICE_NAME`: The name of the service.
//!   - `OTEL_SERVICE_NAMESPACE`: The namespace of the service.
//!   - `OTEL_SERVICE_VERSION`: The version of the service.
//!   - `OTEL_SERVICE_INSTANCE_ID`: The instance ID of the service.
//!   - `OTEL_DEPLOYMENT_ENVIRONMENT`: The deployment environment of the service.
//! 
//! The OpenTelemetry logger can also be configured with the `OtlpConfig` struct, which
//! can be passed to the `init_with_config` function. The `OtlpConfig` struct can be built
//! with the `OtlpConfigBuilder` struct.
//! 
//! Once the logger is initialized, you can use the [`tracing`] macros to log
//! messages. For example:
//! ```rust
//! use tracing::{info, error};
//! 
//! #[tokio::main]
//! async fn main() {
//!    let logger = otlp_logger::init().await.expect("Initialized logger");
//!    info!("This is an info message");
//!    error!("This is an error message");
//! }
//! ```
//! 
//! Traces, metrics, and logs are sent to the configured OTLP endpoint. The traces,  
//! metrics, and log levels are configured via the RUST_LOG environment variable.
//! This behavior can be overridden by setting the `trace_level`, `metrics_level` or
//! `log_level` fields in the `OtlpConfig` struct. You can control what
//! goes to stdout by setting the `stdout_level` field. 
//! ```rust
//! use otlp_logger::{OtlpConfigBuilder, LevelFilter};
//! 
//! #[tokio::main]
//! async fn main() {
//!   let config = OtlpConfigBuilder::default()
//!                  .otlp_endpoint("http://localhost:4317".to_string())
//!                  .trace_level(LevelFilter::INFO)
//!                  .log_level(LevelFilter::ERROR)
//!                  .stdout_level(LevelFilter::OFF)
//!                  .build()
//!                  .expect("failed to create otlp config builder");
//! 
//!   let logger = otlp_logger::init_with_config(config).await.expect("failed to initialize logger");
//! 
//!   // ... your application code
//! 
//!   // shutdown the logger
//!   logger.shutdown();
//! }
//! ```
//! 
//! [`tokio`]: https://crates.io/crates/tokio
//! [`tracing`]: https://crates.io/crates/tracing
//! [`opentelemetry`]: https://crates.io/crates/opentelemetry
//!
use derive_builder::*;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use thiserror::Error;

use anyhow::{Context, Result};

use opentelemetry_otlp::OTEL_EXPORTER_OTLP_ENDPOINT;
use opentelemetry_sdk::{error::{OTelSdkError, OTelSdkResult}, logs::SdkLoggerProvider, metrics::SdkMeterProvider, propagation::TraceContextPropagator, trace::SdkTracerProvider};
use opentelemetry::trace::TracerProvider as _;

use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
pub use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, *};

mod resource;
mod trace;
mod metrics;
mod logs;

use resource::*;
use trace::*;


#[derive(Debug, Default, Builder)]
#[builder(setter(into), default)]
pub struct OtlpConfig {    
    service_name: Option<String>,
    service_namespace: Option<String>,
    service_version: Option<String>,
    service_instant_id: Option<String>,
    deployment_environment: Option<String>,  
    otlp_endpoint: Option<String>,   
    trace_level: Option<LevelFilter>,   
    metrics_level: Option<LevelFilter>,
    log_level: Option<LevelFilter>,
    stdout_level: Option<LevelFilter>,
}

impl OtlpConfig {
    pub fn builder() -> OtlpConfigBuilder {
        OtlpConfigBuilder::default()
    }
}

#[derive(Debug)]
pub struct EndpointLogger {
    tracer_provider: SdkTracerProvider,
    logger_provider: SdkLoggerProvider,
    meter_provider: SdkMeterProvider
}

impl EndpointLogger {
    pub async fn init(config: OtlpConfig) -> Result<Self> {

        let otlp_endpoint = config.otlp_endpoint.as_ref().context("OTLP endpoint not set")?;
        let resource = otel_resource(&config);
        
        let logger_provider = logs::otel_logs(otlp_endpoint, resource.clone())?;
        let tracer_provider = otel_tracer(otlp_endpoint, resource.clone())?;
        let meter_provider = metrics::otel_metrics(otlp_endpoint, resource.clone())?;   

        let logs_layer = OpenTelemetryTracingBridge::new(&logger_provider)
            .with_filter(define_filter_level(config.log_level));

        let tracer = tracer_provider.tracer("otlp-tracing");
        let tracer_layer = OpenTelemetryLayer::new(tracer)
            .with_filter(define_filter_level(config.trace_level));

        let metrics_layer = MetricsLayer::new(meter_provider.clone())
            .with_filter(define_filter_level(config.metrics_level));

        let stdout_layer = tracing_subscriber::fmt::layer()
            .compact()
            .with_file(true)
            .with_line_number(true)
            .with_filter(define_filter_level(config.stdout_level.or_else(||config.log_level)));
        
        tracing_subscriber::registry()
            .with(stdout_layer)
            .with(tracer_layer)
            .with(metrics_layer)
            .with(logs_layer)
            .try_init()
            .context("Could not init tracing registry")?;

        Ok(EndpointLogger {
            tracer_provider,
            logger_provider,
            meter_provider
        }) 

    }

    pub fn shutdown(&self) {
        let mut shutdown_errors = Vec::new();
        if let Some(err) = shutdown_helper(self.tracer_provider.shutdown()) {
            shutdown_errors.push(err);
        }
        if let Some(err) = shutdown_helper(self.logger_provider.shutdown()) {
            shutdown_errors.push(err);
        }
        if let Some(err) = shutdown_helper(self.meter_provider.shutdown()) {
            shutdown_errors.push(err);
        }
        if !shutdown_errors.is_empty() {
            eprintln!("Errors shutting down providers: {:?}", shutdown_errors);
        }
    }
}

fn shutdown_helper(result: OTelSdkResult) -> Option<OTelSdkError> {
    match result {
        Ok(_) | Err(OTelSdkError::AlreadyShutdown) => None,
        Err(err) => {
            Some(err)
        }         
    }
}

#[derive(Debug)]
pub struct StdoutOnlyLogger;

impl StdoutOnlyLogger {
    pub fn init() -> Result<Self> {
        let stdout_layer = tracing_subscriber::fmt::layer()
            .compact()
            .with_file(true)
            .with_line_number(true)
            .with_filter(define_filter_level(None));

        tracing_subscriber::registry()
            .with(stdout_layer)
            .try_init()
            .context("Could not init tracing registry")?;

        opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
        Ok(StdoutOnlyLogger)
    }
}


fn define_filter_level(level: Option<LevelFilter>) -> EnvFilter {
    match level {
        Some(l) => EnvFilter::default().add_directive(l.into()),
        None => EnvFilter::from_default_env(),
    }
}

#[derive(Debug)]
pub enum OtlpLogger {
    WithEndpoint(EndpointLogger),
    StdoutOnly(StdoutOnlyLogger),
}

impl OtlpLogger {
    pub async fn init_with_config(config: OtlpConfig) -> Result<Self, TryInitError> {
        if config.otlp_endpoint.is_some() {
            let logger = EndpointLogger::init(config).await.map_err(|e| TryInitError {
                msg: "Failed to initialize OTLP Endpoint Logger".to_string(),
                source: e,
            })?;
            Ok(OtlpLogger::WithEndpoint(logger))
        } else {
            let logger = StdoutOnlyLogger::init().map_err(|e| TryInitError {
                msg: "Failed to initialize Stdout Only Logger".to_string(),
                source: e,
            })?;
            Ok(OtlpLogger::StdoutOnly(logger))
        }
    }

    pub async fn try_init() -> Result<Self, TryInitError> {
        let endpoint = std::env::var(OTEL_EXPORTER_OTLP_ENDPOINT).ok();
        let config = OtlpConfigBuilder::default()
            .otlp_endpoint(endpoint)
            .build()
            .map_err(|e| TryInitError {
                msg: "Failed to configure endpoint from environment".to_string(),
                source: e.into(),
            })?;
        Self::init_with_config(config).await
    }

    pub fn shutdown(&self) {
        match self {
            OtlpLogger::WithEndpoint(logger) => logger.shutdown(),
            OtlpLogger::StdoutOnly(_) => {}
        }
    }
}

impl Drop for OtlpLogger {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub async fn init() -> Result<OtlpLogger, TryInitError> {
    OtlpLogger::try_init().await
}

pub async fn init_with_config(config: OtlpConfig) -> Result<OtlpLogger, TryInitError> {
    OtlpLogger::init_with_config(config).await
}

#[derive(Error, Debug)]
pub struct TryInitError {
    msg: String,
    source: anyhow::Error,
}

impl std::fmt::Display for TryInitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error initializing OtlpLogger: {}", self.msg)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_config_builder_all() {
        let config = OtlpConfig::builder()
            .service_name("test-service".to_string())
            .service_namespace("test-namespace".to_string())
            .service_version("test-version".to_string())
            .service_instant_id("test-instant-id".to_string())
            .deployment_environment("test-environment".to_string())
            .otlp_endpoint(Some("http://localhost:4317".to_string()))
            .trace_level(LevelFilter::DEBUG)
            .stdout_level(LevelFilter::WARN)
            .build()
            .unwrap();

        assert_eq!(config.service_name, Some("test-service".to_string()));
        assert_eq!(config.service_namespace, Some("test-namespace".to_string()));
        assert_eq!(config.service_version, Some("test-version".to_string()));
        assert_eq!(config.service_instant_id, Some("test-instant-id".to_string()));
        assert_eq!(config.deployment_environment, Some("test-environment".to_string()));
        assert_eq!(config.otlp_endpoint, Some("http://localhost:4317".to_string()));        
        assert_eq!(config.trace_level, Some(LevelFilter::DEBUG));
        assert_eq!(config.stdout_level, Some(LevelFilter::WARN));
    }

    #[test]
    fn test_config_builder_some() {
        let config = OtlpConfig::builder()
             .otlp_endpoint("http://localhost:4317".to_string())
             .trace_level(LevelFilter::INFO)
             .stdout_level(LevelFilter::ERROR)
             .build()
             .expect("failed to configure otlp-logger");

        assert_eq!(config.service_name, None);
        assert_eq!(config.service_namespace, None);
        assert_eq!(config.service_version, None);
        assert_eq!(config.service_instant_id, None);
        assert_eq!(config.deployment_environment, None);
        assert_eq!(config.otlp_endpoint, Some("http://localhost:4317".to_string()));
        assert_eq!(config.trace_level, Some(LevelFilter::INFO));
        assert_eq!(config.stdout_level, Some(LevelFilter::ERROR));        
    }

    #[test]
    fn test_config_builder_none() {
        let config = OtlpConfig::builder()
            .build()
            .unwrap();

        assert_eq!(config.service_name, None);
        assert_eq!(config.service_namespace, None);
        assert_eq!(config.service_version, None);
        assert_eq!(config.service_instant_id, None);
        assert_eq!(config.deployment_environment, None);
        assert_eq!(config.otlp_endpoint, None);      
        assert_eq!(config.trace_level, None);
        assert_eq!(config.stdout_level, None); 
    }
}