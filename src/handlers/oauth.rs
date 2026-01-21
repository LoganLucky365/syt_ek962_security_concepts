use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::UserResponse;
use super::auth::AppState;

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct OAuthInitResponse {
    pub authorization_url: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct OAuthCallbackResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
    pub is_new_user: bool,
}

pub async fn google_login(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let google_provider = state.google_provider.as_ref().ok_or_else(|| {
        // Deplyoment debugging
        AppError::OAuthError("Not configured".to_string())
    })?;

    let (authorization_url, csrf_token) = google_provider.authorization_url();

    tracing::info!("flow started");

    Ok(HttpResponse::Ok().json(OAuthInitResponse {
        authorization_url,
        state: csrf_token.secret().to_string(),
    }))
}

pub async fn google_callback(state: web::Data<AppState>, query: web::Query<OAuthCallbackQuery>) -> Result<HttpResponse, AppError> {
    let google_provider = state.google_provider.as_ref().ok_or_else(|| {
        // Deployment debuging
        AppError::OAuthError("not configured".to_string())
    })?;

    tracing::info!("started callback procesing");

    let google_user = google_provider.exchange_code(&query.code).await?;

    let (auth_result, is_new_user) = google_provider
        .authenticate_or_create(google_user)
        .await?;

    let token = state.jwt_service.generate_token(
        &auth_result.user.id,
        &auth_result.user.email,
        auth_result.user.role,
    )?;

    Ok(HttpResponse::Ok().json(OAuthCallbackResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        user: auth_result.user.into(),
        is_new_user,
    }))
}
