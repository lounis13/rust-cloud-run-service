use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::{runtime::Tokio, Resource};
use opentelemetry_semantic_conventions::resource::{SERVICE_NAME, SERVICE_VERSION};
use std::env;
use tonic::metadata::MetadataValue;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub async fn init() {
    let otlp_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:4317".to_string());

    let is_gcp = otlp_endpoint.contains("googleapis.com");
    let project_id = env::var("GOOGLE_CLOUD_PROJECT").unwrap_or_default();

    let mut builder = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&otlp_endpoint);

    if is_gcp {
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
        builder = builder.with_metadata(metadata);
    }

    let exporter = builder.build().expect("Failed to create OTLP exporter");

    let resource = Resource::new(vec![
        KeyValue::new(SERVICE_NAME, "rust-cloud-run-service"),
        KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
    ]);

    let provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(exporter, Tokio)
        .with_resource(resource)
        .build();

    let tracer = provider.tracer("rust-cloud-run-service");
    opentelemetry::global::set_tracer_provider(provider);

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer())
        .init();
}
