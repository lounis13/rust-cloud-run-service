use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::SdkTracerProvider;

use crate::telemetry::api::TelemetryProvider;
use crate::telemetry::config::TelemetryConfig;
use crate::telemetry::error::TelemetryError;
use crate::telemetry::resource::build_base_resource;

/// Default provider for local development
/// - Exports to local OTLP collector if configured
/// - Falls back to no-op if no endpoint
pub struct DefaultProvider;

impl TelemetryProvider for DefaultProvider {
    async fn build_tracer_provider(
        &self,
        config: &TelemetryConfig,
    ) -> Result<SdkTracerProvider, TelemetryError> {
        let resource = build_base_resource(config);

        let provider = match &config.otlp_endpoint {
            Some(endpoint) => {
                let exporter = opentelemetry_otlp::SpanExporter::builder()
                    .with_tonic()
                    .with_endpoint(endpoint)
                    .build()
                    .map_err(|e: opentelemetry_otlp::ExporterBuildError| {
                        TelemetryError::Exporter(e.to_string())
                    })?;

                SdkTracerProvider::builder()
                    .with_batch_exporter(exporter)
                    .with_resource(resource)
                    .build()
            }
            None => {
                // No-op provider for local dev without collector
                SdkTracerProvider::builder()
                    .with_resource(resource)
                    .build()
            }
        };

        Ok(provider)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn default_provider_without_endpoint_succeeds() {
        let provider = DefaultProvider;
        let config = TelemetryConfig::new("test-service", "1.0.0");

        let result = provider.build_tracer_provider(&config).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn default_provider_with_otlp_endpoint_succeeds() {
        let provider = DefaultProvider;
        let config = TelemetryConfig::new("test-service", "1.0.0")
            .with_otlp_endpoint("http://localhost:4317");

        let result = provider.build_tracer_provider(&config).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn default_provider_with_invalid_endpoint_succeeds_build() {
        // Note: Invalid URL format doesn't fail at build time, only at runtime when connecting
        let provider = DefaultProvider;
        let config = TelemetryConfig::new("test-service", "1.0.0")
            .with_otlp_endpoint("invalid-url");

        let result = provider.build_tracer_provider(&config).await;

        // Build succeeds, connection would fail later
        assert!(result.is_ok());
    }
}