pub mod persist;
// pub mod app_state;

use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use axum::{http::HeaderValue, routing::post, Router};
use tracing::debug;

#[axum::debug_handler]
async fn root_rpc_endpoint() -> &'static str {
    "Hello, World!"
}

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

    let router = Router::new()
        .route("/rpc", post(root_rpc_endpoint))
        .nest_service("/assets", ServeDir::new("../frontend/assets"))
        .layer(cors_layer)
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .expect("serving");
}
