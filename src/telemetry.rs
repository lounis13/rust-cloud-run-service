use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
use std::env;
use std::fmt::Debug;
use tonic::metadata::MetadataValue;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn init() {
    let otlp_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let is_gcp = otlp_endpoint.contains("googleapis.com");
    let project_id = env::var("GOOGLE_CLOUD_PROJECT").unwrap_or_default();

    let exporter = if is_gcp {
        let provider = gcp_auth::provider()
            .await
            .expect("Failed to create auth provider");
        let token = provider
            .token(&["https://www.googleapis.com/auth/trace.append"])
            .await
            .expect("Failed to get token");

        let mut metadata = tonic::metadata::MetadataMap::new();
        metadata.insert(
            "authorization",
            MetadataValue::try_from(format!("Bearer {}", token.as_str())).unwrap(),
        );
        if !project_id.is_empty() {
            metadata.insert(
                "x-goog-user-project",
                MetadataValue::try_from(&project_id).unwrap(),
            );
        }

        let tls_config = tonic::transport::ClientTlsConfig::new().with_native_roots();

        opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(&otlp_endpoint)
            .with_metadata(metadata)
            .with_tls_config(tls_config)
            .build()
            .expect("Failed to create OTLP exporter")
    } else {
        opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(&otlp_endpoint)
            .build()
            .expect("Failed to create OTLP exporter")
    };

    let resource = Resource::builder()
        .with_attribute(KeyValue::new(SERVICE_NAME, "rust-cloud-run-service"))
        .with_attribute(KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")))
        .with_attribute(KeyValue::new("gcp.project_id", project_id.clone()))
        .build();

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    let tracer = provider.tracer("rust-cloud-run-service");
    opentelemetry::global::set_tracer_provider(provider);

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
 
    // JSON format for GCP Cloud Logging (severity field is recognized)
    let fmt_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_ansi(false)
        .flatten_event(true)
        .with_current_span(true)
        .with_target(true);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(telemetry)
        .with(fmt_layer)
        .init();
}
