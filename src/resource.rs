use std::env::args_os;
use std::process::id;

use opentelemetry::{KeyValue, StringValue, Value};
use opentelemetry_sdk::resource::Resource;
use opentelemetry_semantic_conventions::resource as otel_resource;

use crate::OtlpConfig;

pub fn otel_resource(config: &OtlpConfig) -> Resource {

    let mut builder = Resource::builder()
        .with_attributes(detect_os())
        .with_attributes(detect_process());

    if let Some(service_name) = &config.service_name {
        builder = builder.with_attribute(KeyValue::new(otel_resource::SERVICE_NAME, service_name.clone()));
    }
    if let Some(service_namespace) = &config.service_namespace {
        builder = builder.with_attribute(KeyValue::new(otel_resource::SERVICE_NAMESPACE, service_namespace.clone()));
    }
    if let Some(service_version) = &config.service_version {
        builder = builder.with_attribute(KeyValue::new(otel_resource::SERVICE_VERSION, service_version.clone()));
    }
    if let Some(service_instant_id) = &config.service_instant_id {
        builder = builder.with_attribute(KeyValue::new(otel_resource::SERVICE_INSTANCE_ID, service_instant_id.clone()));
    }
    if let Some(deployment_environment) = &config.deployment_environment {
        builder = builder.with_attribute(KeyValue::new(otel_resource::DEPLOYMENT_ENVIRONMENT_NAME, deployment_environment.clone()));
    }

    builder.build()
}

fn detect_os() -> Vec<KeyValue> {
    vec![KeyValue::new(otel_resource::OS_TYPE, std::env::consts::OS)]
}

fn detect_process() -> Vec<KeyValue> {
    let arguments = args_os();
    let cmd_arg_val = arguments
        .into_iter()
        .map(|arg| arg.to_string_lossy().into_owned().into())
        .collect::<Vec<StringValue>>();
    let current_exe = std::env::current_exe().map(|exe| exe.display().to_string()).unwrap_or_default();
    vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::PROCESS_COMMAND_ARGS,
            Value::Array(cmd_arg_val.into()),
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::PROCESS_PID,
            id() as i64,
        ),
        KeyValue::new(
            otel_resource::PROCESS_EXECUTABLE_NAME, 
            current_exe
        ),
    ]
}