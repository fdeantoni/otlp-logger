mod collector;

use std::env;
use tracing::*;

use testcontainers::runners::AsyncRunner;
use crate::collector::{Collector, OTLP_PORT};

#[tokio::test]
#[tracing::instrument]
async fn test_otlp() -> Result<(), Box<dyn std::error::Error + 'static>> {

    if env::consts::OS != "linux" && env::var("GITHUB_ACTIONS").is_ok() {
        println!("Skipping test on OSX in GitHub Actions");
        return Ok(());
    }

    let image = Collector::default();
    let container = image.start().await?;

    let port = container.get_host_port_ipv4(OTLP_PORT).await?;
    let endpoint = format!("http://localhost:{}", port);

    unsafe {
        std::env::set_var("RUST_LOG", "info,init_otlp=trace");
        std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", endpoint);

        let service_name = "init-otlp";
        std::env::set_var("OTEL_RESOURCE_ATTRIBUTES", format!("service.name={}", service_name));
        std::env::set_var("OTEL_METRIC_EXPORT_INTERVAL", "1000");
    }

    let provider = otlp_logger::init().await?;

    info!("This is an info message");
    let result = trace_me(5, 2);
    trace!(result, "Result of adding two numbers");
    info!(monotonic_counter.foo = 1);

    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let test_logs = r#""otelcol.component.kind": "exporter", "otelcol.signal": "logs""#;
    let test_traces = r#""otelcol.component.kind": "exporter", "otelcol.signal": "traces""#;
    let test_metrics = r#""otelcol.component.kind": "exporter", "otelcol.signal": "metrics""#;

    let mut pass = false;
    let mut retry = 0;

    while !pass && retry < 5 {
        let res = container.stderr_to_vec().await;
        match res {
            Ok(response) => {
                let logs = String::from_utf8_lossy(&response);
                println!("Logs: {}", logs);

                if logs.contains(test_logs) && logs.contains(test_traces) && logs.contains(test_metrics) {
                    pass = true;
                    println!("Found traces/logs/metrics in logs");
                } else {
                    println!("No traces/logs/metrics found, retrying...");
                }
            }
            Err(_) => {
                println!("Failed to fetch traces/logs/metrics, retrying...");
            }
        }
        retry += 1;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    assert!(pass, "No traces, logs, and/or metrics found after 5 retries");

    provider.shutdown();

    Ok(())
}

#[tracing::instrument]
fn trace_me(a: u32, b: u32) -> u32 {
    debug!(a, b, "Adding two numbers");
    a + b
}



