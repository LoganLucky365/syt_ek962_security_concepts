use actix_web::{web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use crate::auth::{JwtService, LocalAuthProvider, Claims, AuthProvider};
use crate::error::AppError;
use crate::models::{CreateUser, UserResponse, UserRole};
use crate::repository::UserRepository;

pub struct AppState {
    pub jwt_service: JwtService,
    pub auth_provider: LocalAuthProvider,
    pub repository: Arc<dyn UserRepository>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(length(min = 1, max = 100, message = "name between 1 and 100 char"))]
    pub name: String,

    #[validate(email(message = "Invalid email"))]
    pub email: String,

    #[validate(length(min = 12, message = "pw at least 12 char"))]
    pub password: String,

    pub role: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub message: String,
    pub user: UserResponse,
}

#[derive(Debug, Deserialize, Validate)]
pub struct SignInRequest {
    #[validate(email(message = "Invalid email"))]
    pub email: String,

    #[validate(length(min = 1, message = "Password required"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct SignInResponse {
    pub token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub user: UserResponse,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub user_id: String,
    pub email: String,
    pub role: String,
    pub message: String,
}

fn extract_bearer_token(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

async fn require_admin(req: &HttpRequest, state: &web::Data<AppState>) -> Result<Claims, AppError> {
    let token = extract_bearer_token(req).ok_or_else(|| {
        AppError::Unauthorized("Authorization header required".to_string())
    })?;

    let claims = state.jwt_service.validate_token(&token)?;

    if claims.role != "admin" {
        tracing::warn!(
            user_id = %claims.sub,
            "Non-admin user attempted admin operation"
        );
        return Err(AppError::Forbidden(
            "Administrator privileges required".to_string(),
        ));
    }

    Ok(claims)
}

pub async fn register_user(
    req: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<RegisterRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let admin_claims = require_admin(&req, &state).await?;

    let role = match body.role.as_deref() {
        Some("admin") => UserRole::Admin,
        Some("user") | None => UserRole::User,
        Some(other) => {
            return Err(AppError::ValidationError(format!(
                "Invalid role: {}. Must be 'admin' or 'user'",
                other
            )));
        }
    };

    tracing::info!(
        admin_id = %admin_claims.sub,
        new_user_email = %body.email,
        role = %role,
        "Admin registering new user"
    );

    let user = state
        .auth_provider
        .register(&body.name, &body.email, &body.password, role)
        .await?;

    Ok(HttpResponse::Created().json(RegisterResponse {
        message: "User registered successfully".to_string(),
        user: user.into(),
    }))
}

pub async fn signin(
    state: web::Data<AppState>,
    body: web::Json<SignInRequest>,
) -> Result<HttpResponse, AppError> {
    body.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    let result = state
        .auth_provider
        .authenticate(&body.email, &body.password)
        .await?;

    let token = state.jwt_service.generate_token(
        &result.user.id,
        &result.user.email,
        result.user.role,
    )?;

    Ok(HttpResponse::Ok().json(SignInResponse {
        token,
        token_type: "Bearer".to_string(),
        expires_in: 3600,
        user: result.user.into(),
    }))
}

pub async fn verify_token(
    state: web::Data<AppState>,
    body: web::Json<VerifyRequest>,
) -> Result<HttpResponse, AppError> {
    let claims = match state.jwt_service.validate_token(&body.token) {
        Ok(claims) => claims,
        Err(_) => {
            return Ok(HttpResponse::Forbidden().json(serde_json::json!({
                "valid": false,
                "error": "Invalid or expired token"
            })));
        }
    };

    let user = state
        .repository
        .find_by_id(&claims.sub)
        .await?
        .ok_or_else(|| {
            tracing::warn!(user_id = %claims.sub, "Token valid but user not found");
            AppError::Forbidden("User not found".to_string())
        })?;

    if !user.is_active {
        return Ok(HttpResponse::Forbidden().json(serde_json::json!({
            "valid": false,
            "error": "User account is deactivated"
        })));
    }

    Ok(HttpResponse::Ok().json(VerifyResponse {
        valid: true,
        user_id: claims.sub,
        email: claims.email,
        role: claims.role,
        message: "User is registered and token is valid".to_string(),
    }))
}

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/admin/register", web::post().to(register_user))
            .route("/signin", web::post().to(signin))
            .route("/verify", web::post().to(verify_token)),
    );
}
