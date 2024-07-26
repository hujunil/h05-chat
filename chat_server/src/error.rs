use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}

#[derive(Error, Debug)]
pub enum AppError {
    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),

    #[error("create chat error: {0}")]
    CreateChatError(String),

    #[error("{0}")]
    ChatFileError(String),

    #[error("parse url path param error: {0}")]
    ParseUrlPathError(String),

    #[error("create message error: {0}")]
    CreateMessageError(String),

    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),

    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("workspace already exists: {0}")]
    WorkspaceAlreadyExists(String),

    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("upload file error: {0}")]
    UploadFileError(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response<axum::body::Body> {
        let status = match self {
            AppError::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            AppError::CreateChatError(_) => StatusCode::BAD_REQUEST,
            AppError::ChatFileError(_) => StatusCode::BAD_REQUEST,
            AppError::ParseUrlPathError(_) => StatusCode::BAD_REQUEST,
            AppError::CreateMessageError(_) => StatusCode::BAD_REQUEST,
            AppError::UploadFileError(_) => StatusCode::BAD_REQUEST,
            AppError::JwtError(_) => StatusCode::FORBIDDEN,
            AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::IoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::WorkspaceAlreadyExists(_) => StatusCode::CONFLICT,
            AppError::PasswordHashError(_) => StatusCode::UNPROCESSABLE_ENTITY,
        };
        (status, axum::response::Json(self.to_string())).into_response()
    }
}
