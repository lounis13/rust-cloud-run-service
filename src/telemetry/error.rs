use std::fmt;

#[derive(Debug)]
pub enum TelemetryError {
    Auth(String),
    Exporter(String),
    Config(String),
    Init(String),
}

impl fmt::Display for TelemetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Auth(msg) => write!(f, "Authentication error: {}", msg),
            Self::Exporter(msg) => write!(f, "Exporter error: {}", msg),
            Self::Config(msg) => write!(f, "Configuration error: {}", msg),
            Self::Init(msg) => write!(f, "Initialization error: {}", msg),
        }
    }
}

impl std::error::Error for TelemetryError {}

impl From<opentelemetry_sdk::trace::TraceError> for TelemetryError {
    fn from(err: opentelemetry_sdk::trace::TraceError) -> Self {
        Self::Exporter(err.to_string())
    }
}