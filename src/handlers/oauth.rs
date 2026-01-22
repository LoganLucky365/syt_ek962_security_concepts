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
        AppError::OAuthError("not configured".to_string())
    })?;

    tracing::info!("started callback processing");

    let google_user = google_provider.exchange_code(&query.code).await?;

    let (auth_result, is_new_user) = google_provider
        .authenticate_or_create(google_user)
        .await?;

    let token = state.jwt_service.generate_token(
        &auth_result.user.id,
        &auth_result.user.email,
        auth_result.user.role,
    )?;

    let user_response: UserResponse = auth_result.user.into();
    let user_json = serde_json::to_string(&user_response).unwrap_or_default();

    // Return HTML that stores token and redirects to frontend
    let html = format!(r#"<!DOCTYPE html>
<html>
<head><title>Login erfolgreich</title></head>
<body>
<script>
    localStorage.setItem('auth_token', '{}');
    localStorage.setItem('auth_user', '{}');
    window.location.href = '/';
</script>
<p>Login erfolgreich, Weiterleitung...</p>
</body>
</html>"#, token, user_json.replace('\'', "\\'").replace('\n', ""));

    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html))
}
