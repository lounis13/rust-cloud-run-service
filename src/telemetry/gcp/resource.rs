use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::{
    CLOUD_ACCOUNT_ID, CLOUD_PLATFORM, CLOUD_PROVIDER, CLOUD_REGION, FAAS_NAME, FAAS_VERSION,
};

use crate::telemetry::config::TelemetryConfig;
use crate::telemetry::gcp::config::GcpPlatform;
use crate::telemetry::resource::build_resource;

/// GCP cloud provider value (semconv)
pub const CLOUD_PROVIDER_GCP: &str = "gcp";

/// GCP project ID attribute (required by Cloud Trace)
pub const GCP_PROJECT_ID: &str = "gcp.project_id";

/// GCP-specific resource attributes
pub struct GcpResourceBuilder {
    project_id: String,
    platform: GcpPlatform,
    region: Option<String>,
    service_id: Option<String>,
    revision: Option<String>,
}

impl GcpResourceBuilder {
    pub fn new(project_id: impl Into<String>, platform: GcpPlatform) -> Self {
        Self {
            project_id: project_id.into(),
            platform,
            region: std::env::var("CLOUD_RUN_REGION")
                .or_else(|_| std::env::var("FUNCTION_REGION"))
                .or_else(|_| std::env::var("GAE_REGION"))
                .ok(),
            service_id: std::env::var("K_SERVICE")
                .or_else(|_| std::env::var("FUNCTION_NAME"))
                .or_else(|_| std::env::var("GAE_SERVICE"))
                .ok(),
            revision: std::env::var("K_REVISION")
                .or_else(|_| std::env::var("GAE_VERSION"))
                .ok(),
        }
    }

    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    pub fn with_service(mut self, service_id: impl Into<String>) -> Self {
        self.service_id = Some(service_id.into());
        self
    }

    pub fn with_revision(mut self, revision: impl Into<String>) -> Self {
        self.revision = Some(revision.into());
        self
    }

    pub fn build(self, config: &TelemetryConfig) -> Resource {
        let mut attrs = vec![
            KeyValue::new(CLOUD_PROVIDER, CLOUD_PROVIDER_GCP),
            KeyValue::new(CLOUD_PLATFORM, self.platform.as_str()),
            KeyValue::new(CLOUD_ACCOUNT_ID, self.project_id.clone()),
            KeyValue::new(GCP_PROJECT_ID, self.project_id),
        ];

        if let Some(region) = self.region {
            attrs.push(KeyValue::new(CLOUD_REGION, region));
        }

        if let Some(service_id) = self.service_id {
            attrs.push(KeyValue::new(FAAS_NAME, service_id));
        }

        if let Some(revision) = self.revision {
            attrs.push(KeyValue::new(FAAS_VERSION, revision));
        }

        build_resource(config, attrs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> TelemetryConfig {
        TelemetryConfig::new("test-service", "1.0.0")
    }

    #[test]
    fn gcp_resource_builder_includes_base_attributes() {
        let config = test_config();
        let builder = GcpResourceBuilder::new("my-project", GcpPlatform::CloudRun);

        let resource = builder.build(&config);

        assert!(!resource.is_empty());
    }

    #[test]
    fn gcp_resource_builder_with_region() {
        let config = test_config();
        let builder = GcpResourceBuilder::new("my-project", GcpPlatform::CloudRun)
            .with_region("us-central1");

        let resource = builder.build(&config);

        assert!(!resource.is_empty());
    }

    #[test]
    fn gcp_resource_builder_with_service_and_revision() {
        let config = test_config();
        let builder = GcpResourceBuilder::new("my-project", GcpPlatform::CloudRun)
            .with_service("my-service")
            .with_revision("rev-001");

        let resource = builder.build(&config);

        assert!(!resource.is_empty());
    }

    #[test]
    fn gcp_resource_builder_chain_methods() {
        let config = test_config();
        let builder = GcpResourceBuilder::new("proj", GcpPlatform::CloudFunctions)
            .with_region("europe-west1")
            .with_service("my-function")
            .with_revision("v2");

        let resource = builder.build(&config);

        assert!(!resource.is_empty());
    }
}
