//! Integration tests for HTTP API endpoints

mod common;

use std::sync::Arc;

use actix_web::{test, web, App, http::StatusCode};
use serde_json::json;

use syt_ek962_security_concepts::auth::{JwtService, LocalAuthProvider, PasswordHasher};
use syt_ek962_security_concepts::handlers::{AppState, configure_routes};
use syt_ek962_security_concepts::models::{User, UserRole};
use syt_ek962_security_concepts::repository::UserRepository;

use common::{MockUserRepository, test_jwt_config};

fn create_test_app_state(repo: Arc<dyn UserRepository>) -> web::Data<AppState> {
    web::Data::new(AppState {
        jwt_service: JwtService::new(test_jwt_config()),
        auth_provider: LocalAuthProvider::new(repo.clone()),
        google_provider: None,
        ldap_provider: None,
        repository: repo,
    })
}

fn create_user_with_password(email: &str, password: &str, role: UserRole) -> User {
    let hasher = PasswordHasher::new();
    let hash = hasher.hash(password).unwrap();
    User::new_local("Test User".to_string(), email.to_string(), hash, role)
}

// ==================== Sign In Tests ====================

#[actix_rt::test]
async fn test_signin_success() {
    let user = create_user_with_password("test@example.com", "secure_password_123", UserRole::User);
    let repo = Arc::new(MockUserRepository::with_user(user));
    let app_state = create_test_app_state(repo);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/signin")
        .set_json(json!({
            "email": "test@example.com",
            "password": "secure_password_123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("token").is_some());
    assert_eq!(body["token_type"], "Bearer");
    assert_eq!(body["user"]["email"], "test@example.com");
}

#[actix_rt::test]
async fn test_signin_wrong_password() {
    let user = create_user_with_password("test@example.com", "correct_password", UserRole::User);
    let repo = Arc::new(MockUserRepository::with_user(user));
    let app_state = create_test_app_state(repo);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/signin")
        .set_json(json!({
            "email": "test@example.com",
            "password": "wrong_password"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn test_signin_nonexistent_user() {
    let repo = Arc::new(MockUserRepository::new());
    let app_state = create_test_app_state(repo);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/signin")
        .set_json(json!({
            "email": "nonexistent@example.com",
            "password": "any_password"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn test_signin_invalid_email_format() {
    let repo = Arc::new(MockUserRepository::new());
    let app_state = create_test_app_state(repo);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/signin")
        .set_json(json!({
            "email": "not_an_email",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn test_signin_empty_password() {
    let repo = Arc::new(MockUserRepository::new());
    let app_state = create_test_app_state(repo);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/signin")
        .set_json(json!({
            "email": "test@example.com",
            "password": ""
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ==================== Token Verification Tests ====================

#[actix_rt::test]
async fn test_verify_valid_token() {
    let user = create_user_with_password("test@example.com", "password123", UserRole::User);
    let user_id = user.id.clone();
    let repo = Arc::new(MockUserRepository::with_user(user));
    let app_state = create_test_app_state(repo);

    // Generate a valid token
    let token = app_state.jwt_service.generate_token(
        &user_id,
        "test@example.com",
        UserRole::User,
    ).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/verify")
        .set_json(json!({
            "token": token
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["valid"], true);
    assert_eq!(body["email"], "test@example.com");
}

#[actix_rt::test]
async fn test_verify_invalid_token() {
    let repo = Arc::new(MockUserRepository::new());
    let app_state = create_test_app_state(repo);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/verify")
        .set_json(json!({
            "token": "invalid.token.here"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["valid"], false);
}

#[actix_rt::test]
async fn test_verify_token_for_deleted_user() {
    let user = create_user_with_password("test@example.com", "password123", UserRole::User);
    let user_id = user.id.clone();

    // Create user but mark as inactive
    let mut inactive_user = user;
    inactive_user.is_active = false;

    let repo = Arc::new(MockUserRepository::with_user(inactive_user));
    let app_state = create_test_app_state(repo);

    let token = app_state.jwt_service.generate_token(
        &user_id,
        "test@example.com",
        UserRole::User,
    ).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/verify")
        .set_json(json!({
            "token": token
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // User not found (inactive)
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ==================== Admin Registration Tests ====================

#[actix_rt::test]
async fn test_register_user_as_admin() {
    let admin = create_user_with_password("admin@example.com", "admin_password", UserRole::Admin);
    let admin_id = admin.id.clone();
    let repo = Arc::new(MockUserRepository::with_user(admin));
    let app_state = create_test_app_state(repo);

    let admin_token = app_state.jwt_service.generate_token(
        &admin_id,
        "admin@example.com",
        UserRole::Admin,
    ).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/admin/register")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(json!({
            "name": "New User",
            "email": "newuser@example.com",
            "password": "secure_password_123",
            "role": "user"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user"]["email"], "newuser@example.com");
}

#[actix_rt::test]
async fn test_register_user_without_admin_token() {
    let user = create_user_with_password("user@example.com", "user_password", UserRole::User);
    let user_id = user.id.clone();
    let repo = Arc::new(MockUserRepository::with_user(user));
    let app_state = create_test_app_state(repo);

    let user_token = app_state.jwt_service.generate_token(
        &user_id,
        "user@example.com",
        UserRole::User,
    ).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/admin/register")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .set_json(json!({
            "name": "New User",
            "email": "newuser@example.com",
            "password": "secure_password_123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn test_register_user_without_token() {
    let repo = Arc::new(MockUserRepository::new());
    let app_state = create_test_app_state(repo);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/admin/register")
        .set_json(json!({
            "name": "New User",
            "email": "newuser@example.com",
            "password": "secure_password_123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn test_register_with_invalid_role() {
    let admin = create_user_with_password("admin@example.com", "admin_password", UserRole::Admin);
    let admin_id = admin.id.clone();
    let repo = Arc::new(MockUserRepository::with_user(admin));
    let app_state = create_test_app_state(repo);

    let admin_token = app_state.jwt_service.generate_token(
        &admin_id,
        "admin@example.com",
        UserRole::Admin,
    ).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/admin/register")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(json!({
            "name": "New User",
            "email": "newuser@example.com",
            "password": "secure_password_123",
            "role": "superuser"  // Invalid role
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn test_register_with_short_password() {
    let admin = create_user_with_password("admin@example.com", "admin_password", UserRole::Admin);
    let admin_id = admin.id.clone();
    let repo = Arc::new(MockUserRepository::with_user(admin));
    let app_state = create_test_app_state(repo);

    let admin_token = app_state.jwt_service.generate_token(
        &admin_id,
        "admin@example.com",
        UserRole::Admin,
    ).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/admin/register")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(json!({
            "name": "New User",
            "email": "newuser@example.com",
            "password": "short"  // Less than 12 characters
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ==================== LDAP Sign In Tests ====================

#[actix_rt::test]
async fn test_ldap_signin_not_configured() {
    let repo = Arc::new(MockUserRepository::new());
    let app_state = create_test_app_state(repo); // LDAP not configured

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/ldap/signin")
        .set_json(json!({
            "username": "testuser",
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ==================== Google OAuth Tests ====================

#[actix_rt::test]
async fn test_google_login_not_configured() {
    let repo = Arc::new(MockUserRepository::new());
    let app_state = create_test_app_state(repo); // Google OAuth not configured

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::get()
        .uri("/auth/google/login")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ==================== Request Body Validation Tests ====================

#[actix_rt::test]
async fn test_signin_missing_email() {
    let repo = Arc::new(MockUserRepository::new());
    let app_state = create_test_app_state(repo);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/signin")
        .set_json(json!({
            "password": "password123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_rt::test]
async fn test_signin_missing_password() {
    let repo = Arc::new(MockUserRepository::new());
    let app_state = create_test_app_state(repo);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/signin")
        .set_json(json!({
            "email": "test@example.com"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ==================== Token Response Format Tests ====================

#[actix_rt::test]
async fn test_signin_response_format() {
    let user = create_user_with_password("test@example.com", "secure_password_123", UserRole::User);
    let repo = Arc::new(MockUserRepository::with_user(user));
    let app_state = create_test_app_state(repo);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .configure(configure_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/signin")
        .set_json(json!({
            "email": "test@example.com",
            "password": "secure_password_123"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;

    // Check all expected fields are present
    assert!(body.get("token").is_some());
    assert!(body.get("token_type").is_some());
    assert!(body.get("expires_in").is_some());
    assert!(body.get("user").is_some());

    // Check user response format
    let user_resp = &body["user"];
    assert!(user_resp.get("id").is_some());
    assert!(user_resp.get("name").is_some());
    assert!(user_resp.get("email").is_some());
    assert!(user_resp.get("role").is_some());

    // Password hash should NOT be in response
    assert!(user_resp.get("password_hash").is_none());
}
