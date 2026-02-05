mod telemetry;

use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use std::env;
use tracing::info;
use tracing_actix_web::TracingLogger;

#[get("/")]
#[tracing::instrument]
async fn hello() -> impl Responder {
    tracing::info!("Hello endpoint called");
    HttpResponse::Ok().body("Hello, World!")
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().body("ok")
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    info!(
        "Starting"
    );
    telemetry::init().await;

    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("PORT must be a number");

    tracing::info!("Starting server on port {}", port);

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
