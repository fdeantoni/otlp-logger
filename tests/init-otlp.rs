mod jaeger;

use std::env;
use tracing::*;

use serde_json::Value;

use testcontainers::runners::AsyncRunner;

use crate::jaeger::{Jaeger, JAEGER_PORT, OTLP_PORT};


#[tokio::test]
#[tracing::instrument]
async fn test_otlp() -> Result<(), Box<dyn std::error::Error + 'static>> {

    if env::consts::OS != "linux" && env::var("GITHUB_ACTIONS").is_ok() {
        println!("Skipping test on OSX in GitHub Actions");
        return Ok(());
    }

    let image = Jaeger::default();
    let container = image.start().await;

    let port = container.get_host_port_ipv4(OTLP_PORT).await;
    let endpoint = format!("http://localhost:{}", port);

    std::env::set_var("RUST_LOG", "info,init_otlp=trace");
    std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", endpoint);

    let service_name = "init-otlp";
    std::env::set_var("OTEL_RESOURCE_ATTRIBUTES", format!("service.name={}", service_name));

    otlp_logger::init().await;

    info!("This is an info message");
    let result = trace_me(5, 2);
    trace!(result, "Result of adding two numbers");

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let jaeger_port = container.get_host_port_ipv4(JAEGER_PORT).await;
    let url = format!(
        "http://localhost:{}/api/traces?service={}",
        jaeger_port,
        service_name
    );
    let res = reqwest::get(url).await.expect("valid HTTP response");
    let traces = res.json::<Value>().await.unwrap();
    
    println!("Response: {:?}", traces);    

    assert_eq!(traces["data"].as_array().unwrap().len(), 1);

    Ok(())
}

#[tracing::instrument]
fn trace_me(a: u32, b: u32) -> u32 {
    debug!(a, b, "Adding two numbers");
    a + b
}



