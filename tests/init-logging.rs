use tracing::{info, error};


#[tokio::test]
async fn test_init() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    otlp_logger::init().await;
    info!("This is an info message");
    error!("This is an error message");
}

