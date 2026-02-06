use thiserror::Error;

#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Exporter error: {0}")]
    Exporter(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Initialization error: {0}")]
    Init(String),
}

impl From<opentelemetry_sdk::trace::TraceError> for TelemetryError {
    fn from(err: opentelemetry_sdk::trace::TraceError) -> Self {
        Self::Exporter(err.to_string())
    }
}