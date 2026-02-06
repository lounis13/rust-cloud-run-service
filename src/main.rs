mod telemetry;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;
use std::env;
use tracing::info;
use tracing_actix_web::TracingLogger;

#[derive(Deserialize)]
struct HelloQuery {
    user: Option<String>,
}

#[get("/")]
#[tracing::instrument(skip(query), fields(user))]
async fn hello(query: web::Query<HelloQuery>) -> impl Responder {
    let user = query.user.as_deref().unwrap_or("anonymous");
    tracing::Span::current().record("user", user);
    info!(user = user, "Hello endpoint called");
    HttpResponse::Ok().body(format!("Hello, {}!", user))
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Initialize telemetry (auto-detects GCP from GOOGLE_CLOUD_PROJECT env var)
    // IMPORTANT: Keep the guard alive for the entire application lifetime
    let _telemetry_guard = telemetry::init()
        .await
        .expect("Failed to initialize telemetry");

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    info!("Starting server on port {}", port);

    let server = HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .service(hello)
            .service(health)
    })
    .bind(("0.0.0.0", port))?
    .run();

    // Wait for server to finish
    server.await?;

    // Explicitly drop the guard to ensure proper shutdown
    // This gives time to flush remaining spans before process exits
    info!("Server stopped, shutting down telemetry...");
    drop(_telemetry_guard);

    // Give a brief moment for async shutdown to complete
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    Ok(())
}
