mod collector;

use std::env;
use std::time::Instant;
use tracing::*;

use otlp_logger::OtlpConfig;
use otlp_logger::MetricsAggregationConfig;
use otlp_logger::MetricsAggregation;

use testcontainers::runners::AsyncRunner;

use crate::collector::{Collector, OTLP_PORT, PROM_METRICS_PORT};


#[tokio::test]
#[tracing::instrument]
async fn override_metrics() -> Result<(), Box<dyn std::error::Error + 'static>> {

    if env::consts::OS != "linux" && env::var("GITHUB_ACTIONS").is_ok() {
        println!("Skipping test on OSX in GitHub Actions");
        return Ok(());
    }

    let image = Collector::default();
    let container = image.start().await?;

    let port = container.get_host_port_ipv4(OTLP_PORT).await?;
    let endpoint = format!("http://localhost:{}", port);

    std::env::set_var("RUST_LOG", "info,override_metrics=trace");

    let service_name = "override-metrics";

    let aggregation = MetricsAggregationConfig::builder()
        .histogram(MetricsAggregation::ExplicitBucketHistogram {
            boundaries: vec![
                0.0, 5.0, 10.0,
            ],
            record_min_max: true,
        }
    ).build()?;

    let config = OtlpConfig::builder()
        .otlp_endpoint(endpoint)
        .service_name(service_name.to_string())
        .metrics_aggregation(aggregation)
        .build()
        .unwrap();
    otlp_logger::init_with_config(config).await?;

    info!("This is an info message");
    let result = trace_me(5, 2);
    trace!(result, "Result of adding two numbers");
    error!("This is an error message");

    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    let prom_port = container.get_host_port_ipv4(PROM_METRICS_PORT).await?;
    let url = format!(
        "http://localhost:{}/metrics",
        prom_port,
    );
    let res = reqwest::get(url).await?;
    let metrics = res.text().await?;
    
    assert!(metrics.contains("adding_some_result_total{a=\"5\",b=\"2\",job=\"override-metrics\"} 7"));
    assert!(metrics.contains("adding_some_timing_count{a=\"5\",b=\"2\",job=\"override-metrics\"} 1"));

    Ok(())
}

#[tracing::instrument]
fn trace_me(a: u32, b: u32) -> u32 {
    debug!(a, b, "Adding two numbers");
    let timer = Instant::now();
    let result = a + b;
    trace!(monotonic_counter.adding.some.result = result, a, b);
    trace!(histogram.adding.some.timing = timer.elapsed().as_secs_f64(), a, b);
    result
}



