use std::time::Duration;

use std::env::args_os;
use std::process::id;

use opentelemetry::{KeyValue, StringValue, Value};
use opentelemetry_sdk::{
    resource::{ResourceDetector, SdkProvidedResourceDetector, TelemetryResourceDetector},
    Resource,
};
use opentelemetry_semantic_conventions::resource as otel_resource;

use crate::OtlpConfig;

pub fn otel_resource(config: &OtlpConfig) -> Resource {
    let os_resource = detect_os();
    let process_resource = detect_process();
    let telemetry_resource = TelemetryResourceDetector.detect(Duration::from_secs(0));
    let sdk_resource = SdkProvidedResourceDetector.detect(Duration::from_secs(0));

    let mut provided = Vec::new();
    if let Some(service_name) = &config.service_name {
        provided.push(KeyValue::new(
            otel_resource::SERVICE_NAME,
            service_name.clone(),
        ));
    }
    if let Some(service_namespace) = &config.service_namespace {
        provided.push(KeyValue::new(
            otel_resource::SERVICE_NAMESPACE,
            service_namespace.clone(),
        ));
    }
    if let Some(service_version) = &config.service_version {
        provided.push(KeyValue::new(
            otel_resource::SERVICE_VERSION,
            service_version.clone(),
        ));
    }
    if let Some(service_instant_id) = &config.service_instant_id {
        provided.push(KeyValue::new(
            otel_resource::SERVICE_INSTANCE_ID,
            service_instant_id.clone(),
        ));
    }
    if let Some(deployment_environment) = &config.deployment_environment {
        provided.push(KeyValue::new(
            otel_resource::DEPLOYMENT_ENVIRONMENT_NAME,
            deployment_environment.clone(),
        ));
    }

    let app = Resource::new(provided);

    sdk_resource
        .merge(&telemetry_resource)
        .merge(&os_resource)
        .merge(&process_resource)
        .merge(&app)
}

fn detect_os() -> Resource {
    Resource::new(vec![KeyValue::new(
        otel_resource::OS_TYPE,
        std::env::consts::OS,
    )])
}

fn detect_process() -> Resource {
    let arguments = args_os();
    let cmd_arg_val = arguments
        .into_iter()
        .map(|arg| arg.to_string_lossy().into_owned().into())
        .collect::<Vec<StringValue>>();
    let current_exe = std::env::current_exe()
        .map(|exe| exe.display().to_string())
        .unwrap_or_default();
    Resource::new(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::PROCESS_COMMAND_ARGS,
            Value::Array(cmd_arg_val.into()),
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::PROCESS_PID,
            id() as i64,
        ),
        KeyValue::new(otel_resource::PROCESS_EXECUTABLE_NAME, current_exe),
    ])
}
