mod jaeger;

use std::env;
use tracing::*;

use serde_json::Value;

use otlp_logger::OtlpConfigBuilder;
use otlp_logger::LevelFilter;

use testcontainers::runners::AsyncRunner;

use crate::jaeger::{Jaeger, JAEGER_PORT, OTLP_PORT};


#[tokio::test]
#[tracing::instrument]
async fn override_level() -> Result<(), Box<dyn std::error::Error + 'static>> {

    if env::consts::OS != "linux" && env::var("GITHUB_ACTIONS").is_ok() {
        println!("Skipping test on OSX in GitHub Actions");
        return Ok(());
    }

    let image = Jaeger::default();
    let container = image.start().await?;

    let port = container.get_host_port_ipv4(OTLP_PORT).await?;
    let endpoint = format!("http://localhost:{}", port);

    unsafe {
        std::env::set_var("RUST_LOG", "info,override_level=info");
    }

    let service_name = "override-level";

    let config = OtlpConfigBuilder::default()
        .otlp_endpoint(endpoint)
        .service_name(service_name.to_string())
        .trace_level(LevelFilter::TRACE)
        .log_level(LevelFilter::OFF)
        .build()
        .unwrap();
    let logger = otlp_logger::OtlpLogger::init_with_config(config).await.expect("Failed to initialize logger");

    info!("This is an info message");
    let result = trace_me(5, 2);
    trace!(result, "Result of adding two numbers");
    error!("This is an error message");

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let jaeger_port = container.get_host_port_ipv4(JAEGER_PORT).await?;
    let url = format!(
        "http://localhost:{}/api/traces?service={}",
        jaeger_port,
        service_name
    );

    let mut pass = false;
    let mut retry = 0;

    while !pass && retry < 5 {
        let res = reqwest::get(&url).await;
        match res {
            Ok(response) => {
                let traces = response.json::<Value>().await?;
                if traces["data"].as_array().unwrap().len() > 0 {
                    pass = true;
                }
            }
            Err(_) => {
                println!("Failed to fetch traces, retrying...");
            }
        }
        retry += 1;
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    assert!(pass, "No traces found after 5 retries");

    logger.shutdown();

    Ok(())
}

#[tracing::instrument]
fn trace_me(a: u32, b: u32) -> u32 {
    debug!(a, b, "Adding two numbers");
    a + b
}



