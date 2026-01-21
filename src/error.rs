use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    Unauthorized(String),
    Forbidden(String),
    NotFound(String),
    ValidationError(String),
    Conflict(String),
    InternalError(String),
    DatabaseError(String),
    OAuthError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            AppError::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::Conflict(msg) => write!(f, "Conflict: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::OAuthError(msg) => write!(f, "OAuth error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl ResponseError for AppError {
    fn error_response(&self) -> HttpResponse {
        let (status, message) = match self {
            AppError::Unauthorized(_) => {
                (StatusCode::UNAUTHORIZED, "Du musst dich scho einlogen burschi")
            }
            AppError::Forbidden(_) => {
                (StatusCode::FORBIDDEN, "Access denied -> hehe")
            }
            AppError::NotFound(_) => {
                (StatusCode::NOT_FOUND, "Ressource not there")
            }
            AppError::ValidationError(msg) => {
                (StatusCode::BAD_REQUEST, msg.as_str())
            }
            AppError::Conflict(_) => {
                (StatusCode::CONFLICT, "Resource already there")
            }
            AppError::InternalError(_) | AppError::DatabaseError(_) => {
                tracing::error!("Internal error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal malfunction -> its always your fault")
            }
            AppError::OAuthError(msg) => {
                tracing::error!("OAuth error: {}", self);
                (StatusCode::BAD_REQUEST, msg.as_str())
            }
        };

        HttpResponse::build(status).json(serde_json::json!({
            "error": message
        }))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        AppError::DatabaseError(err.to_string())
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        tracing::warn!("JWT error: {:?}", err);
        AppError::Unauthorized("Invalid token".to_string())
    }
}

impl From<argon2::password_hash::Error> for AppError {
    fn from(err: argon2::password_hash::Error) -> Self {
        tracing::error!("Password hashing error: {:?}", err);
        AppError::InternalError("Password processing failed".to_string())
    }
}
