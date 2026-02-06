use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::fmt::{format::Writer, FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::{LookupSpan, SpanRef};
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

/// Custom JSON formatter that outputs GCP-compatible logs with `severity` at root level
struct GcpJsonFormat;

impl<S, N> FormatEvent<S, N> for GcpJsonFormat
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        use std::fmt::Write;

        // Map tracing level to GCP severity
        let severity = match *event.metadata().level() {
            tracing::Level::ERROR => "ERROR",
            tracing::Level::WARN => "WARNING",
            tracing::Level::INFO => "INFO",
            tracing::Level::DEBUG => "DEBUG",
            tracing::Level::TRACE => "DEBUG",
        };

        // Start JSON object with severity at root
        write!(writer, r#"{{"severity":"{}""#, severity)?;

        // Add timestamp
        write!(
            writer,
            r#","timestamp":"{}""#,
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Micros, true)
        )?;

        // Add target
        write!(writer, r#","target":"{}""#, event.metadata().target())?;

        // Add current span fields (for trace context)
        if let Some(span) = ctx.lookup_current() {
            let ext = span.extensions();
            if let Some(visitor) = ext.get::<tracing_subscriber::fmt::FormattedFields<N>>() {
                if !visitor.is_empty() {
                    write!(writer, r#","span":{{"name":"{}",{}}}"#, span.name(), visitor)?;
                }
            }
        }

        // Add event fields (message, user, etc.)
        let mut visitor = serde_json::Map::new();
        let mut json_visitor = JsonVisitor(&mut visitor);
        event.record(&mut json_visitor);

        if !visitor.is_empty() {
            for (key, value) in visitor.iter() {
                let json_str = serde_json::to_string(value).map_err(|_| std::fmt::Error)?;
                write!(writer, r#","{}":{}"#, key, json_str)?;
            }
        }

        // Close JSON object
        writeln!(writer, "}}")
    }
}

/// Visitor to collect event fields into a JSON map
struct JsonVisitor<'a>(&'a mut serde_json::Map<String, serde_json::Value>);

impl<'a> tracing::field::Visit for JsonVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.0.insert(
            field.name().to_string(),
            serde_json::Value::String(format!("{:?}", value)),
        );
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.0
            .insert(field.name().to_string(), serde_json::Value::String(value.to_string()));
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.0
            .insert(field.name().to_string(), serde_json::Value::Number(value.into()));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.0
            .insert(field.name().to_string(), serde_json::Value::Number(value.into()));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.0
            .insert(field.name().to_string(), serde_json::Value::Bool(value));
    }
}

/// Build the JSON fmt layer for structured logging (cloud environments)
/// Uses custom GCP formatter with `severity` at root level for proper colorization
pub fn build_json_layer<S>() -> impl Layer<S>
where
    S: Subscriber + for<'span> LookupSpan<'span>,
{
    tracing_subscriber::fmt::layer()
        .event_format(GcpJsonFormat)
        .with_ansi(false)
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

/// Initialize the global tracing subscriber with all layers
pub fn init_subscriber(provider: SdkTracerProvider, config: &TelemetryConfig) {
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

    // Note: Not using std::mem::forget() here
    // The warning "OnEnd.AfterShutdown" may appear when Cloud Run scales down,
    // but traces are still exported during normal operation
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
