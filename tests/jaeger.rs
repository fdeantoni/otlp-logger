use testcontainers::{core::WaitFor, Image};

const NAME: &str = "jaegertracing/all-in-one";
const TAG: &str = "1.56";
const DEFAULT_WAIT: u64 = 3000;
pub const OTLP_PORT: u16 = 4317;
pub const JAEGER_PORT: u16 = 16686;

#[derive(Debug, Default, Clone)]
pub struct Jaeger;


impl Image for Jaeger {

    fn name(&self) -> &str {
        NAME
    }

    fn tag(&self) -> &str {
        TAG
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            WaitFor::message_on_stderr("Channel Connectivity change to READY"),
            WaitFor::millis(DEFAULT_WAIT),
        ]
    }
}