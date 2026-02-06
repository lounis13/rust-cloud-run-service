use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::metadata::MetadataValue;
use tonic::service::Interceptor;
use tonic::{Request, Status};

use crate::telemetry::error::TelemetryError;

const TRACE_SCOPE: &str = "https://www.googleapis.com/auth/trace.append";

/// GCP authentication interceptor with automatic token refresh
///
/// Uses `gcp_auth` which automatically handles token caching and refresh.
/// From the gcp_auth documentation:
/// - "TokenProvider handles caching tokens for their lifetime"
/// - "Will not make a request if an appropriate token is already cached"
/// - "The caller should not cache tokens"
#[derive(Clone)]
pub struct GcpAuthInterceptor {
    provider: Arc<Mutex<Arc<dyn gcp_auth::TokenProvider>>>,
    project_id: String,
}

impl GcpAuthInterceptor {
    /// Create a new auth interceptor from Application Default Credentials
    pub async fn from_adc(project_id: String) -> Result<Self, TelemetryError> {
        let provider = gcp_auth::provider()
            .await
            .map_err(|e| TelemetryError::Auth(format!("Failed to create auth provider: {}", e)))?;

        Ok(Self {
            provider: Arc::new(Mutex::new(provider)),
            project_id,
        })
    }
}

impl Interceptor for GcpAuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        // Get token - gcp_auth handles caching and automatic refresh
        let token = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.provider
                    .lock()
                    .await
                    .token(&[TRACE_SCOPE])
                    .await
            })
        })
        .map_err(|e| Status::unauthenticated(format!("Failed to get token: {}", e)))?;

        // Inject authorization header
        let metadata = request.metadata_mut();
        metadata.insert(
            "authorization",
            MetadataValue::try_from(format!("Bearer {}", token.as_str()))
                .map_err(|e| Status::invalid_argument(format!("Invalid token format: {}", e)))?,
        );

        // Inject project ID header
        if !self.project_id.is_empty() {
            metadata.insert(
                "x-goog-user-project",
                MetadataValue::try_from(self.project_id.as_str())
                    .map_err(|e| Status::invalid_argument(format!("Invalid project ID: {}", e)))?,
            );
        }

        Ok(request)
    }
}
