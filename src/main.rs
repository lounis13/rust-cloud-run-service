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
    telemetry::init()
        .await
        .expect("Failed to initialize telemetry");

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    info!("Starting server on port {}", port);

    HttpServer::new(|| {
        App::new()
            .wrap(TracingLogger::default())
            .service(hello)
            .service(health)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
