use tracing::{info, error};


#[tokio::test]
async fn test_init() {
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    let logger = otlp_logger::init().await.expect("Initialized logger");
    info!("This is an info message");
    error!("This is an error message");

    logger.shutdown();
}

