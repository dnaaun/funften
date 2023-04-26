use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;
use std::net::SocketAddr;

use axum::{Router, http::HeaderValue};
use tracing::debug;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let router = Router::new();

    let addr = SocketAddr::from(([127, 0, 0, 1], 2001));
    debug!("Listening on {}", addr);

    let cors_layer = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods(tower_http::cors::Any)
        .allow_headers(tower_http::cors::Any)
        .allow_origin("http://localhost:1001".parse::<HeaderValue>().unwrap());

    let trace_layer = TraceLayer::new_for_http();

    let router = router.layer(cors_layer).layer(trace_layer);


    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .expect("serving");
}
