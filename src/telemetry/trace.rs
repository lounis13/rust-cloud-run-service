use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

use crate::telemetry::config::{LogFormat, TelemetryConfig};

/// Build the OpenTelemetry tracing layer
pub fn build_otel_layer<S>(
    provider: &SdkTracerProvider,
    service_name: &str,
) -> OpenTelemetryLayer<S, opentelemetry_sdk::trace::Tracer>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    let tracer = provider.tracer(service_name.to_string());
    tracing_opentelemetry::layer().with_tracer(tracer)
}

/// Build the JSON fmt layer for structured logging (cloud environments)
pub fn build_json_layer<S>() -> impl Layer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false)
        .flatten_event(true)
        .with_current_span(true)
        .with_target(true)
}

/// Build the pretty fmt layer for human-readable output (local dev)
pub fn build_pretty_layer<S>() -> impl Layer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    tracing_subscriber::fmt::layer()
        .pretty()
        .with_ansi(true)
        .with_target(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE)
}

/// Build the env filter from config
pub fn build_filter(config: &TelemetryConfig) -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.log_level))
}

/// Wrapper to manage telemetry lifecycle (currently traces, will include logging/metrics later)
/// Keep this guard alive for the entire application lifetime to prevent premature shutdown
pub struct TelemetryGuard {
    provider: SdkTracerProvider,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        tracing::info!("🔴 Telemetry shutting down gracefully");
        // Explicitly shutdown to flush remaining spans
        if let Err(e) = self.provider.shutdown() {
            tracing::error!("Failed to shutdown TracerProvider: {:?}", e);
        }
    }
}

/// Initialize the global tracing subscriber with all layers
/// Returns a guard that MUST be kept alive for the application lifetime
pub fn init_subscriber(provider: SdkTracerProvider, config: &TelemetryConfig) -> TelemetryGuard {
    // Set the global tracer provider BEFORE creating layers
    opentelemetry::global::set_tracer_provider(provider.clone());

    let otel_layer = build_otel_layer(&provider, &config.service_name);
    let filter = build_filter(config);

    match config.log_format {
        LogFormat::Pretty => {
            let fmt_layer = build_pretty_layer();
            tracing_subscriber::registry()
                .with(filter)
                .with(otel_layer)
                .with(fmt_layer)
                .init();
        }
        LogFormat::Json => {
            let fmt_layer = build_json_layer();
            tracing_subscriber::registry()
                .with(filter)
                .with(otel_layer)
                .with(fmt_layer)
                .init();
        }
    }

    tracing::info!("✅ Telemetry initialized - keep the returned guard alive!");

    // Return guard - caller MUST keep it alive
    TelemetryGuard { provider }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_filter_uses_config_log_level() {
        let config = TelemetryConfig::new("test", "1.0")
            .with_log_level("debug");

        let filter = build_filter(&config);

        assert_eq!(filter.to_string(), "debug");
    }

    #[test]
    fn build_filter_defaults_to_info() {
        let config = TelemetryConfig::new("test", "1.0");

        let filter = build_filter(&config);

        assert_eq!(filter.to_string(), "info");
    }

    #[test]
    fn build_otel_layer_creates_layer() {
        use tracing_subscriber::Registry;

        let provider = SdkTracerProvider::builder().build();

        let _layer = build_otel_layer::<Registry>(&provider, "test-service");

        // Layer creation should not panic
    }
}
