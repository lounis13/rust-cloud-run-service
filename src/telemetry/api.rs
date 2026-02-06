use opentelemetry_sdk::trace::SdkTracerProvider;

use crate::telemetry::config::{TelemetryBackend, TelemetryConfig};
use crate::telemetry::error::TelemetryError;
use crate::telemetry::trace::init_subscriber;

/// Trait for telemetry providers (GCP, local, etc.)
pub trait TelemetryProvider: Send + Sync {
    /// Build the tracer provider for this backend
    fn build_tracer_provider(
        &self,
        config: &TelemetryConfig,
    ) -> impl std::future::Future<Output = Result<SdkTracerProvider, TelemetryError>> + Send;
}

/// Initialize telemetry with a specific provider
pub async fn init_with_provider<P: TelemetryProvider>(
    provider: &P,
    config: &TelemetryConfig,
) -> Result<(), TelemetryError> {
    let tracer_provider = provider.build_tracer_provider(config).await?;
    init_subscriber(tracer_provider, config);
    Ok(())
}

/// Initialize telemetry with config (uses backend from config)
pub async fn init_with_config(config: &TelemetryConfig) -> Result<(), TelemetryError> {
    match &config.backend {
        TelemetryBackend::Local => {
            let provider = crate::telemetry::default::DefaultProvider;
            init_with_provider(&provider, config).await
        }
        #[cfg(feature = "telemetry-gcp")]
        TelemetryBackend::Gcp(gcp_config) => {
            let provider = crate::telemetry::gcp::GcpProvider::new(gcp_config.clone());
            init_with_provider(&provider, config).await
        }
    }
}

/// Initialize telemetry from environment (Local backend)
pub async fn init() -> Result<(), TelemetryError> {
    let config = TelemetryConfig::from_env();
    init_with_config(&config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn init_with_provider_builds_tracer() {
        use crate::telemetry::default::DefaultProvider;

        let provider = DefaultProvider;
        let config = TelemetryConfig::new("test", "1.0");

        let result = provider.build_tracer_provider(&config).await;

        assert!(result.is_ok());
    }
}
