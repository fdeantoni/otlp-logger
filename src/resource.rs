use std::time::Duration;

use opentelemetry::KeyValue;
use opentelemetry_sdk::{resource::{OsResourceDetector, ProcessResourceDetector, ResourceDetector, SdkProvidedResourceDetector, TelemetryResourceDetector}, Resource};
use opentelemetry_semantic_conventions::resource as otel_resource;

use crate::OtlpConfig;

pub fn otel_resource(config: &OtlpConfig) -> Resource {
    let os_resource = OsResourceDetector.detect(Duration::from_secs(0));
    let process_resource = ProcessResourceDetector.detect(Duration::from_secs(0));
    let telemetry_resource = TelemetryResourceDetector.detect(Duration::from_secs(0));
    let sdk_resource = SdkProvidedResourceDetector.detect(Duration::from_secs(0));

    let mut provided = Vec::new();
    if let Some(service_name) = &config.service_name {
        provided.push(KeyValue::new(otel_resource::SERVICE_NAME, service_name.clone()));
    }
    if let Some(service_namespace) = &config.service_namespace {
        provided.push(KeyValue::new(otel_resource::SERVICE_NAMESPACE, service_namespace.clone()));
    }
    if let Some(service_version) = &config.service_version {
        provided.push(KeyValue::new(otel_resource::SERVICE_VERSION, service_version.clone()));
    }
    if let Some(service_instant_id) = &config.service_instant_id {
        provided.push(KeyValue::new(otel_resource::SERVICE_INSTANCE_ID, service_instant_id.clone()));
    }
    if let Some(deployment_environment) = &config.deployment_environment {
        provided.push(KeyValue::new(otel_resource::DEPLOYMENT_ENVIRONMENT, deployment_environment.clone()));
    }

    let app = Resource::new(provided);

    sdk_resource
        .merge(&telemetry_resource)
        .merge(&os_resource)
        .merge(&process_resource)
        .merge(&app)
}