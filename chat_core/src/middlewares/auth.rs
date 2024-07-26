use axum::{
    extract::{FromRequestParts, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::Deserialize;

use super::TokenVerify;

#[derive(Deserialize, Debug)]
struct Params {
    access_token: String,
}

pub async fn verify_token<T>(State(state): State<T>, req: Request, next: Next) -> Response
where
    T: TokenVerify + Sync + Send + Clone + 'static,
{
    let (mut parts, body) = req.into_parts();
    let token =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(auth) => auth.0.token().to_string(),
            Err(e) => {
                // If the Authorization header is missing, try to parse the query params
                if e.is_missing() {
                    match Query::<Params>::from_request_parts(&mut parts, &state).await {
                        Ok(params) => params.access_token.clone(),
                        Err(e) => {
                            let msg = format!("parse query params failed: {}", e);
                            return (StatusCode::UNAUTHORIZED, msg).into_response();
                        }
                    }
                } else {
                    let msg = format!("parse Authorization header failed: {}", e);
                    return (StatusCode::BAD_REQUEST, msg).into_response();
                }
            }
        };

    let req = match state.verify(&token) {
        Ok(user) => {
            let mut req = Request::from_parts(parts, body);
            req.extensions_mut().insert(user);
            req
        }
        Err(e) => {
            let msg = format!("verify token failed: {:?}", e);
            tracing::warn!(msg);
            return (StatusCode::FORBIDDEN, msg).into_response();
        }
    };

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::sync::Arc;
    use tower::ServiceExt;

    use crate::{
        middlewares::TokenVerify,
        utils::{DecodingKey, EncodingKey},
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware::from_fn_with_state,
        response::IntoResponse,
        routing::get,
        Router,
    };

    #[derive(Clone)]
    struct AppState(Arc<AppStateInner>);

    struct AppStateInner {
        ek: EncodingKey,
        dk: DecodingKey,
    }

    impl TokenVerify for AppState {
        type Error = jwt_simple::Error;

        fn verify(&self, token: &str) -> Result<crate::User, Self::Error> {
            self.0.dk.verify(token)
        }
    }

    async fn handler() -> impl IntoResponse {
        (StatusCode::OK, "ok")
    }

    #[tokio::test]
    async fn verify_token_should_work() -> Result<()> {
        let encoded_pem = include_str!("../../fixtures/encoding.pem");
        let decoded_pem = include_str!("../../fixtures/decoding.pem");

        let ek = EncodingKey::load(encoded_pem).unwrap();
        let dk = DecodingKey::load(decoded_pem).unwrap();

        let state = AppState(Arc::new(AppStateInner { ek, dk }));

        let user = crate::User::new(1, "tester", "tester@example.com");
        let token = state.0.ek.sign(user)?;

        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
            .with_state(state);

        let req = Request::builder()
            .uri("/")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::OK);

        let req = Request::builder()
            .uri(format!("/?access_token={}", token))
            .body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::OK);

        let req = Request::builder().uri("/").body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        let req = Request::builder()
            .uri("/")
            .header("Authorization", "Bearer bad-token")
            .body(Body::empty())?;

        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);

        let req = Request::builder()
            .uri("/?access_token=bad-token")
            .body(Body::empty())?;
        let res = app.oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);

        Ok(())
    }
}
