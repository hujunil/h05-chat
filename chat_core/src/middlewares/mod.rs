use core::fmt;

mod auth;
mod request_id;
mod server_time;

pub use auth::verify_token;
use axum::{middleware::from_fn, Router};
use request_id::set_request_id;
use server_time::ServerTimeLayer;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

const REQUEST_ID_HEADER: &str = "X-Request-ID";

#[allow(unused)]
const SERVER_TIME_HEADER: &str = "X-Server-Time";

pub trait TokenVerify {
    type Error: fmt::Debug;
    fn verify(&self, token: &str) -> Result<crate::User, Self::Error>;
}

pub fn set_layer(app: Router) -> Router {
    app.layer(
        ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(tower_http::LatencyUnit::Micros),
                    ),
            )
            .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
            .layer(from_fn(set_request_id))
            .layer(ServerTimeLayer),
    )
}
