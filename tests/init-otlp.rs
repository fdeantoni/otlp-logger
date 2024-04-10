mod jaeger;

use std::env;
use tracing::*;
use testcontainers::clients::Cli;

use crate::jaeger::Jaeger;


#[tokio::test]
#[tracing::instrument]
async fn test_otlp() -> Result<(), Box<dyn std::error::Error + 'static>> {

    if env::consts::OS == "macos" && env::var("GITHUB_ACTIONS").is_ok() {
        println!("Skipping test on OSX in GitHub Actions");
        return Ok(());
    }

    let docker = Cli::default();
    let image = Jaeger::default();
    let container = docker.run(image);

    let port = container.get_host_port_ipv4(4317);
    let endpoint = format!("http://localhost:{}", port);

    std::env::set_var("RUST_LOG", "info,init_otlp=trace");
    std::env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", endpoint);

    otlp_logger::init();
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    info!("This is an info message");
    let result = trace_me(5, 2);
    trace!(result, "Result of adding two numbers");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    Ok(())
}

#[tracing::instrument]
fn trace_me(a: u32, b: u32) -> u32 {
    debug!(a, b, "Adding two numbers");
    a + b
}



