//! GCP Cloud Trace telemetry provider.
//!
//! This module provides OpenTelemetry integration with Google Cloud Trace
//! using OTLP/gRPC export with automatic GCP authentication.
//!
//! # Features
//!
//! - Automatic authentication via Application Default Credentials (ADC)
//! - Support for multiple GCP platforms (Cloud Run, Cloud Functions, App Engine, etc.)
//! - Semantic conventions for GCP resource attributes
//!
//! # Example
//!
//! ```rust,ignore
//! use telemetry::gcp::{GcpConfig, GcpPlatform};
//!
//! // Simple configuration
//! let config = GcpConfig::new("my-project-id");
//!
//! // With custom platform
//! let config = GcpConfig::new("my-project-id")
//!     .with_platform(GcpPlatform::CloudFunctions)
//!     .with_endpoint("https://custom-endpoint.example.com");
//!
//! // From environment variables
//! let config = GcpConfig::from_env().expect("GCP config from env");
//! ```
//!
//! # Environment Variables
//!
//! - `GOOGLE_CLOUD_PROJECT` / `GCLOUD_PROJECT` / `GCP_PROJECT`: Project ID
//! - `OTEL_EXPORTER_OTLP_ENDPOINT`: Custom OTLP endpoint
//! - `K_SERVICE`, `FUNCTION_NAME`, `GAE_SERVICE`: Platform auto-detection

mod auth;
pub mod config;
pub mod exporter;
pub mod resource;

use opentelemetry_sdk::trace::SdkTracerProvider;

use crate::telemetry::api::TelemetryProvider;
use crate::telemetry::config::TelemetryConfig;
use crate::telemetry::error::TelemetryError;

pub use config::{GcpConfig, GcpPlatform};
pub use exporter::build_gcp_exporter;
pub use resource::GcpResourceBuilder;

/// GCP Cloud Trace telemetry provider.
///
/// Exports traces to Google Cloud Trace using OTLP/gRPC with
/// automatic authentication via Application Default Credentials.
pub struct GcpProvider {
    config: GcpConfig,
}

impl GcpProvider {
    /// Create a new GCP provider with the given configuration.
    pub fn new(config: GcpConfig) -> Self {
        Self { config }
    }
}

impl TelemetryProvider for GcpProvider {
    async fn build_tracer_provider(
        &self,
        config: &TelemetryConfig,
    ) -> Result<SdkTracerProvider, TelemetryError> {
        let exporter =
            build_gcp_exporter(&self.config.project_id, &self.config.endpoint).await?;

        let resource =
            GcpResourceBuilder::new(&self.config.project_id, self.config.platform).build(config);

        let provider = SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(resource)
            .build();

        Ok(provider)
    }
}
