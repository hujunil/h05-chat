use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::warn;

#[allow(unused)]
// Middleware to set a request id header if it doesn't exist
pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    // If the request doesn't have a request id header, generate one
    if req.headers().get(super::REQUEST_ID_HEADER).is_none() {
        let request_id = uuid::Uuid::now_v7().to_string();
        // Set the request id header
        match HeaderValue::from_str(&request_id) {
            Ok(header_value) => {
                req.headers_mut()
                    .insert(super::REQUEST_ID_HEADER, header_value);
            }
            Err(e) => {
                warn!("Failed to set request id header: {}", e);
            }
        }
    }

    next.run(req).await
}
