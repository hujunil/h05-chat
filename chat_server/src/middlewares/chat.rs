use axum::{
    extract::{FromRequestParts, Path, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};
use chat_core::User;

use crate::{error::AppError, AppState};

pub async fn verify_chat(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();

    let path = Path::<u64>::from_request_parts(&mut parts, &state).await;

    if path.is_err() {
        return AppError::ParseUrlPathError("chat_id should be a number".to_string())
            .into_response();
    }

    let Path(chat_id) = path.unwrap();

    let user = parts.extensions.get::<User>().unwrap();

    if !state
        .is_chat_member(chat_id, user.id as _)
        .await
        .unwrap_or_default()
    {
        let err = AppError::CreateMessageError(format!(
            "User {} are not a member of chat {chat_id}",
            user.id
        ));

        return err.into_response();
    }

    let req = Request::from_parts(parts, body);

    next.run(req).await
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use axum::{
        body::Body, extract::Request, http::StatusCode, middleware::from_fn_with_state,
        response::IntoResponse, routing::get, Router,
    };
    use chat_core::middlewares::verify_token;
    use tower::ServiceExt;

    use crate::AppState;

    use super::verify_chat;

    async fn handler(_req: Request) -> impl IntoResponse {
        (StatusCode::OK, "ok")
    }

    #[tokio::test]
    async fn verify_chat_middlewares_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let user = state.find_user_by_id(1).await?.expect("user should exist");
        let token = state.ek.sign(user)?;

        let app = Router::new()
            .route("/chat/:id/message", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_chat))
            .layer(from_fn_with_state(state.clone(), verify_token::<AppState>))
            .with_state(state);

        // test valid chat id
        let req = Request::builder()
            .uri("/chat/1/message")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;

        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::OK);

        // test invalid chat id
        let req = Request::builder()
            .uri("/chat/5/message")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;

        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        // test invalid chat id
        let req = Request::builder()
            .uri("/chat/invalid_chat_id/message")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;

        let res = app.oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        Ok(())
    }
}
