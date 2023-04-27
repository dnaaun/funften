use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use axum::{http::HeaderValue, Router};
use tracing::debug;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 2001));
    debug!("Listening on {}", addr);

    let cors_layer = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .allow_origin("http://localhost:1001".parse::<HeaderValue>().unwrap());

    let trace_layer = TraceLayer::new_for_http();

    let router = Router::new().nest_service("/assets", ServeDir::new("../frontend/assets"));

    let router = router.layer(cors_layer).layer(trace_layer);

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .expect("serving");
}
