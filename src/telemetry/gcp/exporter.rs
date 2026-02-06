use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithTonicConfig};

use crate::telemetry::error::TelemetryError;
use crate::telemetry::gcp::auth::GcpAuth;

/// Build OTLP exporter configured for GCP Cloud Trace
pub async fn build_gcp_exporter(
    project_id: &str,
    endpoint: &str,
) -> Result<SpanExporter, TelemetryError> {
    let auth = GcpAuth::from_adc(project_id).await?;

    let tls_config = tonic::transport::ClientTlsConfig::new().with_native_roots();

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_metadata(auth.metadata)
        .with_tls_config(tls_config)
        .build()
        .map_err(|e| TelemetryError::Exporter(e.to_string()))?;

    Ok(exporter)
}
