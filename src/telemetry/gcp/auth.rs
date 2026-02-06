use tonic::metadata::{MetadataMap, MetadataValue};

use crate::telemetry::error::TelemetryError;

const TRACE_SCOPE: &str = "https://www.googleapis.com/auth/trace.append";

/// GCP authentication metadata for OTLP requests
pub struct GcpAuth {
    pub metadata: MetadataMap,
}

impl GcpAuth {
    /// Create auth metadata from Application Default Credentials
    pub async fn from_adc(project_id: &str) -> Result<Self, TelemetryError> {
        let provider = gcp_auth::provider()
            .await
            .map_err(|e| TelemetryError::Auth(format!("Failed to create auth provider: {}", e)))?;

        let token = provider
            .token(&[TRACE_SCOPE])
            .await
            .map_err(|e| TelemetryError::Auth(format!("Failed to get token: {}", e)))?;

        let mut metadata = MetadataMap::new();

        metadata.insert(
            "authorization",
            MetadataValue::try_from(format!("Bearer {}", token.as_str()))
                .map_err(|e| TelemetryError::Auth(format!("Invalid token format: {}", e)))?,
        );

        if !project_id.is_empty() {
            metadata.insert(
                "x-goog-user-project",
                MetadataValue::try_from(project_id)
                    .map_err(|e| TelemetryError::Auth(format!("Invalid project ID: {}", e)))?,
            );
        }

        Ok(Self { metadata })
    }
}
