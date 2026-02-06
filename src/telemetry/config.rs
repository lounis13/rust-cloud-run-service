use std::env;

/// Log output format
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum LogFormat {
    /// Pretty human-readable format with colors (for local dev)
    #[default]
    Pretty,
    /// JSON structured format (for cloud environments)
    Json,
}

/// Telemetry backend selection
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum TelemetryBackend {
    /// Local development (OTLP to local collector or no-op)
    #[default]
    Local,
    /// Google Cloud Platform (Cloud Trace)
    #[cfg(feature = "telemetry-gcp")]
    Gcp(crate::telemetry::gcp::GcpConfig),
}

impl TelemetryBackend {
    /// Auto-detect backend from environment variables
    /// Returns GCP backend if GCP project is configured, otherwise Local
    pub fn from_env() -> Self {
        #[cfg(feature = "telemetry-gcp")]
        {
            if let Some(gcp_config) = crate::telemetry::gcp::GcpConfig::from_env() {
                return Self::Gcp(gcp_config);
            }
        }
        Self::Local
    }
}

/// Main telemetry configuration
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub service_name: String,
    pub service_version: String,
    pub otlp_endpoint: Option<String>,
    pub log_level: String,
    pub log_format: LogFormat,
    pub backend: TelemetryBackend,
}

impl TelemetryConfig {
    /// Create config from environment variables with auto-detected backend
    /// - Detects GCP if GOOGLE_CLOUD_PROJECT is set
    /// - Falls back to Local backend otherwise
    pub fn from_env() -> Self {
        let log_format = match env::var("LOG_FORMAT").as_deref() {
            Ok("json") => LogFormat::Json,
            Ok("pretty") => LogFormat::Pretty,
            _ => LogFormat::Pretty,
        };

        Self {
            service_name: env::var("OTEL_SERVICE_NAME")
                .unwrap_or_else(|_| env!("CARGO_PKG_NAME").to_string()),
            service_version: env::var("OTEL_SERVICE_VERSION")
                .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string()),
            otlp_endpoint: env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok(),
            log_level: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            log_format,
            backend: TelemetryBackend::from_env(),
        }
    }

    /// Create a new config with explicit values
    pub fn new(service_name: impl Into<String>, service_version: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            service_version: service_version.into(),
            otlp_endpoint: None,
            log_level: "info".to_string(),
            log_format: LogFormat::Pretty,
            backend: TelemetryBackend::Local,
        }
    }

    pub fn with_log_format(mut self, format: LogFormat) -> Self {
        self.log_format = format;
        self
    }

    pub fn builder() -> TelemetryConfigBuilder {
        TelemetryConfigBuilder::default()
    }

    pub fn with_backend(mut self, backend: TelemetryBackend) -> Self {
        self.backend = backend;
        self
    }

    pub fn with_otlp_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.otlp_endpoint = Some(endpoint.into());
        self
    }

    pub fn with_log_level(mut self, level: impl Into<String>) -> Self {
        self.log_level = level.into();
        self
    }
}

#[derive(Default)]
pub struct TelemetryConfigBuilder {
    service_name: Option<String>,
    service_version: Option<String>,
    otlp_endpoint: Option<String>,
    log_level: Option<String>,
    log_format: Option<LogFormat>,
    backend: Option<TelemetryBackend>,
}

impl TelemetryConfigBuilder {
    pub fn service_name(mut self, name: impl Into<String>) -> Self {
        self.service_name = Some(name.into());
        self
    }

    pub fn service_version(mut self, version: impl Into<String>) -> Self {
        self.service_version = Some(version.into());
        self
    }

    pub fn otlp_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.otlp_endpoint = Some(endpoint.into());
        self
    }

    pub fn log_level(mut self, level: impl Into<String>) -> Self {
        self.log_level = Some(level.into());
        self
    }

    pub fn log_format(mut self, format: LogFormat) -> Self {
        self.log_format = Some(format);
        self
    }

    pub fn json(self) -> Self {
        self.log_format(LogFormat::Json)
    }

    pub fn pretty(self) -> Self {
        self.log_format(LogFormat::Pretty)
    }

    pub fn backend(mut self, backend: TelemetryBackend) -> Self {
        self.backend = Some(backend);
        self
    }

    #[cfg(feature = "telemetry-gcp")]
    pub fn gcp(self, gcp_config: crate::telemetry::gcp::GcpConfig) -> Self {
        self.backend(TelemetryBackend::Gcp(gcp_config))
    }

    pub fn build(self) -> TelemetryConfig {
        TelemetryConfig {
            service_name: self
                .service_name
                .unwrap_or_else(|| env!("CARGO_PKG_NAME").to_string()),
            service_version: self
                .service_version
                .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string()),
            otlp_endpoint: self.otlp_endpoint,
            log_level: self.log_level.unwrap_or_else(|| "info".to_string()),
            log_format: self.log_format.unwrap_or_default(),
            backend: self.backend.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_format_default_is_pretty() {
        assert_eq!(LogFormat::default(), LogFormat::Pretty);
    }

    #[test]
    fn telemetry_backend_default_is_local() {
        assert_eq!(TelemetryBackend::default(), TelemetryBackend::Local);
    }

    #[test]
    fn config_new_sets_defaults() {
        let config = TelemetryConfig::new("test-service", "1.0.0");

        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.service_version, "1.0.0");
        assert_eq!(config.log_level, "info");
        assert_eq!(config.log_format, LogFormat::Pretty);
        assert_eq!(config.backend, TelemetryBackend::Local);
        assert!(config.otlp_endpoint.is_none());
    }

    #[test]
    fn config_with_methods_chain() {
        let config = TelemetryConfig::new("svc", "1.0")
            .with_log_level("debug")
            .with_log_format(LogFormat::Json)
            .with_otlp_endpoint("http://localhost:4317");

        assert_eq!(config.log_level, "debug");
        assert_eq!(config.log_format, LogFormat::Json);
        assert_eq!(config.otlp_endpoint, Some("http://localhost:4317".to_string()));
    }

    #[test]
    fn builder_sets_all_fields() {
        let config = TelemetryConfigBuilder::default()
            .service_name("my-service")
            .service_version("2.0.0")
            .log_level("warn")
            .otlp_endpoint("http://collector:4317")
            .json()
            .build();

        assert_eq!(config.service_name, "my-service");
        assert_eq!(config.service_version, "2.0.0");
        assert_eq!(config.log_level, "warn");
        assert_eq!(config.log_format, LogFormat::Json);
        assert_eq!(config.otlp_endpoint, Some("http://collector:4317".to_string()));
    }

    #[test]
    fn builder_pretty_sets_log_format() {
        let config = TelemetryConfig::builder().pretty().build();
        assert_eq!(config.log_format, LogFormat::Pretty);
    }

    #[test]
    fn builder_json_sets_log_format() {
        let config = TelemetryConfig::builder().json().build();
        assert_eq!(config.log_format, LogFormat::Json);
    }

    #[test]
    fn builder_uses_defaults_when_not_set() {
        let config = TelemetryConfig::builder().build();

        assert_eq!(config.log_level, "info");
        assert_eq!(config.log_format, LogFormat::Pretty);
        assert_eq!(config.backend, TelemetryBackend::Local);
    }

    #[test]
    fn backend_from_env_returns_local_without_gcp_project() {
        // Ensure no GCP project env var is set
        std::env::remove_var("GOOGLE_CLOUD_PROJECT");
        std::env::remove_var("GCLOUD_PROJECT");
        std::env::remove_var("GCP_PROJECT");

        let backend = TelemetryBackend::from_env();

        assert_eq!(backend, TelemetryBackend::Local);
    }

    #[cfg(feature = "telemetry-gcp")]
    #[test]
    fn backend_from_env_returns_gcp_with_project() {
        std::env::set_var("GOOGLE_CLOUD_PROJECT", "test-project");

        let backend = TelemetryBackend::from_env();

        assert!(matches!(backend, TelemetryBackend::Gcp(_)));

        std::env::remove_var("GOOGLE_CLOUD_PROJECT");
    }

    #[cfg(feature = "telemetry-gcp")]
    #[test]
    fn config_from_env_auto_detects_gcp() {
        std::env::set_var("GOOGLE_CLOUD_PROJECT", "auto-detect-project");
        std::env::set_var("LOG_FORMAT", "json");

        let config = TelemetryConfig::from_env();

        assert!(matches!(config.backend, TelemetryBackend::Gcp(_)));
        assert_eq!(config.log_format, LogFormat::Json);

        std::env::remove_var("GOOGLE_CLOUD_PROJECT");
        std::env::remove_var("LOG_FORMAT");
    }
}
