use std::env;

/// Default GCP telemetry endpoint
pub const DEFAULT_ENDPOINT: &str = "https://telemetry.googleapis.com";

/// GCP cloud platforms (maps to cloud.platform semconv values)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GcpPlatform {
    #[default]
    CloudRun,
    CloudFunctions,
    AppEngine,
    ComputeEngine,
    KubernetesEngine,
}

impl GcpPlatform {
    /// Returns the OpenTelemetry semantic convention value
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CloudRun => "gcp_cloud_run",
            Self::CloudFunctions => "gcp_cloud_functions",
            Self::AppEngine => "gcp_app_engine",
            Self::ComputeEngine => "gcp_compute_engine",
            Self::KubernetesEngine => "gcp_kubernetes_engine",
        }
    }

    /// Detect platform from environment variables
    pub fn detect() -> Option<Self> {
        if env::var("K_SERVICE").is_ok() || env::var("K_REVISION").is_ok() {
            Some(Self::CloudRun)
        } else if env::var("FUNCTION_NAME").is_ok() || env::var("FUNCTION_TARGET").is_ok() {
            Some(Self::CloudFunctions)
        } else if env::var("GAE_SERVICE").is_ok() || env::var("GAE_VERSION").is_ok() {
            Some(Self::AppEngine)
        } else {
            None
        }
    }
}

/// GCP-specific configuration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcpConfig {
    pub project_id: String,
    pub endpoint: String,
    pub platform: GcpPlatform,
}

impl GcpConfig {
    pub fn new(project_id: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            endpoint: DEFAULT_ENDPOINT.to_string(),
            platform: GcpPlatform::default(),
        }
    }

    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = endpoint.into();
        self
    }

    pub fn with_platform(mut self, platform: GcpPlatform) -> Self {
        self.platform = platform;
        self
    }

    /// Create from environment variables
    /// - GOOGLE_CLOUD_PROJECT / GCLOUD_PROJECT / GCP_PROJECT for project_id
    /// - OTEL_EXPORTER_OTLP_ENDPOINT for endpoint (defaults to DEFAULT_ENDPOINT)
    /// - Platform auto-detected from K_SERVICE, FUNCTION_NAME, GAE_SERVICE, etc.
    pub fn from_env() -> Option<Self> {
        let project_id = env::var("GOOGLE_CLOUD_PROJECT")
            .or_else(|_| env::var("GCLOUD_PROJECT"))
            .or_else(|_| env::var("GCP_PROJECT"))
            .ok()?;

        let endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| DEFAULT_ENDPOINT.to_string());

        let platform = GcpPlatform::detect().unwrap_or_default();

        Some(Self {
            project_id,
            endpoint,
            platform,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to clean up env vars after tests
    struct EnvGuard {
        vars: Vec<&'static str>,
    }

    impl EnvGuard {
        fn new(vars: &[&'static str]) -> Self {
            Self { vars: vars.to_vec() }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for var in &self.vars {
                env::remove_var(var);
            }
        }
    }

    #[test]
    fn gcp_platform_default_is_cloud_run() {
        assert_eq!(GcpPlatform::default(), GcpPlatform::CloudRun);
    }

    #[test]
    fn gcp_platform_as_str_returns_semconv_values() {
        assert_eq!(GcpPlatform::CloudRun.as_str(), "gcp_cloud_run");
        assert_eq!(GcpPlatform::CloudFunctions.as_str(), "gcp_cloud_functions");
        assert_eq!(GcpPlatform::AppEngine.as_str(), "gcp_app_engine");
        assert_eq!(GcpPlatform::ComputeEngine.as_str(), "gcp_compute_engine");
        assert_eq!(GcpPlatform::KubernetesEngine.as_str(), "gcp_kubernetes_engine");
    }

    #[test]
    fn gcp_config_new_uses_defaults() {
        let config = GcpConfig::new("my-project");

        assert_eq!(config.project_id, "my-project");
        assert_eq!(config.endpoint, DEFAULT_ENDPOINT);
        assert_eq!(config.platform, GcpPlatform::CloudRun);
    }

    #[test]
    fn gcp_config_with_endpoint() {
        let config = GcpConfig::new("proj")
            .with_endpoint("https://custom.example.com");

        assert_eq!(config.endpoint, "https://custom.example.com");
    }

    #[test]
    fn gcp_config_with_platform() {
        let config = GcpConfig::new("proj")
            .with_platform(GcpPlatform::CloudFunctions);

        assert_eq!(config.platform, GcpPlatform::CloudFunctions);
    }

    #[test]
    fn gcp_config_builder_chain() {
        let config = GcpConfig::new("my-project")
            .with_platform(GcpPlatform::AppEngine)
            .with_endpoint("https://trace.example.com");

        assert_eq!(config.project_id, "my-project");
        assert_eq!(config.platform, GcpPlatform::AppEngine);
        assert_eq!(config.endpoint, "https://trace.example.com");
    }

    #[test]
    fn gcp_platform_detect_cloud_run() {
        let _guard = EnvGuard::new(&["K_SERVICE"]);
        env::set_var("K_SERVICE", "my-service");

        assert_eq!(GcpPlatform::detect(), Some(GcpPlatform::CloudRun));
    }

    #[test]
    fn gcp_platform_detect_cloud_run_revision() {
        let _guard = EnvGuard::new(&["K_REVISION"]);
        env::set_var("K_REVISION", "my-service-00001");

        assert_eq!(GcpPlatform::detect(), Some(GcpPlatform::CloudRun));
    }

    #[test]
    fn gcp_platform_detect_cloud_functions() {
        let _guard = EnvGuard::new(&["FUNCTION_NAME"]);
        env::set_var("FUNCTION_NAME", "my-function");

        assert_eq!(GcpPlatform::detect(), Some(GcpPlatform::CloudFunctions));
    }

    #[test]
    fn gcp_platform_detect_app_engine() {
        let _guard = EnvGuard::new(&["GAE_SERVICE"]);
        env::set_var("GAE_SERVICE", "default");

        assert_eq!(GcpPlatform::detect(), Some(GcpPlatform::AppEngine));
    }

    #[test]
    fn gcp_platform_detect_none_when_no_env() {
        // Ensure no GCP env vars are set
        let _guard = EnvGuard::new(&["K_SERVICE", "K_REVISION", "FUNCTION_NAME", "GAE_SERVICE"]);

        assert_eq!(GcpPlatform::detect(), None);
    }

    #[test]
    fn gcp_config_from_env_returns_none_without_project() {
        let _guard = EnvGuard::new(&["GOOGLE_CLOUD_PROJECT", "GCLOUD_PROJECT", "GCP_PROJECT"]);

        assert!(GcpConfig::from_env().is_none());
    }

    #[test]
    fn gcp_config_from_env_with_project() {
        let _guard = EnvGuard::new(&["GOOGLE_CLOUD_PROJECT"]);
        env::set_var("GOOGLE_CLOUD_PROJECT", "test-project");

        let config = GcpConfig::from_env().unwrap();
        assert_eq!(config.project_id, "test-project");
        assert_eq!(config.endpoint, DEFAULT_ENDPOINT);
    }

    #[test]
    fn gcp_config_from_env_with_custom_endpoint() {
        let _guard = EnvGuard::new(&["GOOGLE_CLOUD_PROJECT", "OTEL_EXPORTER_OTLP_ENDPOINT"]);
        env::set_var("GOOGLE_CLOUD_PROJECT", "proj");
        env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://localhost:4317");

        let config = GcpConfig::from_env().unwrap();
        assert_eq!(config.endpoint, "http://localhost:4317");
    }
}
