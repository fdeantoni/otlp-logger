use std::collections::HashMap;

use testcontainers::{core::WaitFor, Image};

const NAME: &str = "otel/opentelemetry-collector-contrib";
const TAG: &str = "0.98.0";
const DEFAULT_WAIT: u64 = 3000;

pub const OTLP_PORT: u16 = 4317;
pub const PROM_METRICS_PORT: u16 = 55679;

const DEFAULT_CONFIG: &str = r#"
receivers:
  otlp:
    protocols:
      grpc:
      http:

exporters:
  debug:
    verbosity: detailed
  prometheus:
    endpoint: ":55679"

service:
  pipelines:
    traces:
      receivers: [otlp]
      processors: []
      exporters: [debug]
    metrics:
      receivers: [otlp]
      processors: []
      exporters: [debug, prometheus]
    logs:
      receivers: [otlp]
      processors: []
      exporters: [debug]
"#;

#[derive(Debug, Clone)]
pub struct Collector {
    env_vars: HashMap<String, String>,
}

impl Default for Collector {
    fn default() -> Self {
        let mut env_vars = HashMap::new();
        env_vars.insert("CONFIG".to_owned(), DEFAULT_CONFIG.to_owned());
        Collector { env_vars }
    }
}

impl Image for Collector {

    fn name(&self) -> &str {
        NAME
    }

    fn tag(&self) -> &str {
        TAG
    }

    fn cmd(&self) -> impl IntoIterator<Item = impl Into<std::borrow::Cow<'_, str>>> {
        vec!["--config=env:CONFIG".to_owned()]
    }

    #[allow(refining_impl_trait)]
    fn env_vars(&self) -> Box<dyn Iterator<Item = (&String, &String)> + '_> {
        Box::new(self.env_vars.iter())
    }    

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![
            WaitFor::message_on_stderr("Everything is ready. Begin running and processing data."),
            WaitFor::millis(DEFAULT_WAIT),
        ]
    }
}