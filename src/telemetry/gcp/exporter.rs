use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithTonicConfig};
use tracing::info;
use crate::telemetry::error::TelemetryError;
use crate::telemetry::gcp::auth::GcpAuthInterceptor;

/// Build OTLP exporter configured for GCP Cloud Trace with automatic token refresh
pub async fn build_gcp_exporter(
    project_id: &str,
    endpoint: &str,
) -> Result<SpanExporter, TelemetryError> {
    let auth_interceptor = GcpAuthInterceptor::from_adc(project_id.to_string()).await?;

    let tls_config = tonic::transport::ClientTlsConfig::new().with_native_roots();

    info!("📤 Building OTLP exporter with TLS...");
    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_interceptor(auth_interceptor)
        .with_tls_config(tls_config)
        .build()
        .map_err(|e| TelemetryError::Exporter(e.to_string()))?;

    info!("✅ GCP exporter built successfully");
    Ok(exporter)
}
