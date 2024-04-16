mod jaeger;

use std::env;
use tracing::*;

use serde_json::Value;

use otlp_logger::OtlpConfigBuilder;
use otlp_logger::LevelFilter;

use testcontainers::clients::Cli;

use crate::jaeger::{Jaeger, JAEGER_PORT, OTLP_PORT};


#[tokio::test]
#[tracing::instrument]
async fn override_level() -> Result<(), Box<dyn std::error::Error + 'static>> {

    if env::consts::OS != "linux" && env::var("GITHUB_ACTIONS").is_ok() {
        println!("Skipping test on OSX in GitHub Actions");
        return Ok(());
    }

    let docker = Cli::default();
    let image = Jaeger::default();
    let container = docker.run(image);

    let port = container.get_host_port_ipv4(OTLP_PORT);
    let endpoint = format!("http://localhost:{}", port);

    std::env::set_var("RUST_LOG", "info,override_level=error");

    let service_name = "init-otlp";

    let config = OtlpConfigBuilder::default()
        .otlp_endpoint(endpoint)
        .service_name(service_name.to_string())
        .trace_level(LevelFilter::TRACE)
        .build()
        .unwrap();
    otlp_logger::init_with_config(config).await.unwrap();

    info!("This is an info message");
    let result = trace_me(5, 2);
    trace!(result, "Result of adding two numbers");
    error!("This is an error message");

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let jaeger_port = container.get_host_port_ipv4(JAEGER_PORT);
    let url = format!(
        "http://localhost:{}/api/traces?service={}",
        jaeger_port,
        service_name
    );
    let res = reqwest::get(url).await.expect("valid HTTP response");
    let traces = res.json::<Value>().await.unwrap();
    
    assert_eq!(traces["data"].as_array().unwrap().len(), 1);

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    Ok(())
}

#[tracing::instrument]
fn trace_me(a: u32, b: u32) -> u32 {
    debug!(a, b, "Adding two numbers");
    a + b
}



