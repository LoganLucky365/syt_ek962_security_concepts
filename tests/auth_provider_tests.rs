//! Tests for LocalAuthProvider

mod common;

use std::sync::Arc;

use syt_ek962_security_concepts::auth::{LocalAuthProvider, AuthProvider};
use syt_ek962_security_concepts::models::{User, UserRole, AuthProviderType};
use syt_ek962_security_concepts::error::AppError;

use common::MockUserRepository;

// ==================== Registration Tests ====================

#[tokio::test]
async fn test_register_new_user() {
    let repo = Arc::new(MockUserRepository::new());
    let provider = LocalAuthProvider::new(repo.clone());

    let user = provider
        .register("Test User", "test@example.com", "secure_password_123", UserRole::User)
        .await
        .unwrap();

    assert_eq!(user.name, "Test User");
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.role, UserRole::User);
    assert_eq!(user.auth_provider, AuthProviderType::Local);
    assert!(user.is_active);
}

#[tokio::test]
async fn test_register_admin_user() {
    let repo = Arc::new(MockUserRepository::new());
    let provider = LocalAuthProvider::new(repo.clone());

    let user = provider
        .register("Admin User", "admin@example.com", "secure_password_123", UserRole::Admin)
        .await
        .unwrap();

    assert_eq!(user.role, UserRole::Admin);
    assert!(user.is_admin());
}

#[tokio::test]
async fn test_register_normalizes_email_to_lowercase() {
    let repo = Arc::new(MockUserRepository::new());
    let provider = LocalAuthProvider::new(repo.clone());

    let user = provider
        .register("Test User", "TEST@EXAMPLE.COM", "secure_password_123", UserRole::User)
        .await
        .unwrap();

    assert_eq!(user.email, "test@example.com");
}

#[tokio::test]
async fn test_register_duplicate_email_fails() {
    let repo = Arc::new(MockUserRepository::new());
    let provider = LocalAuthProvider::new(repo.clone());

    // Register first user
    provider
        .register("User 1", "test@example.com", "password_123456", UserRole::User)
        .await
        .unwrap();

    // Attempt to register with same email
    let result = provider
        .register("User 2", "test@example.com", "different_password", UserRole::User)
        .await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

#[tokio::test]
async fn test_register_creates_password_hash() {
    let repo = Arc::new(MockUserRepository::new());
    let provider = LocalAuthProvider::new(repo.clone());

    let user = provider
        .register("Test User", "test@example.com", "secure_password_123", UserRole::User)
        .await
        .unwrap();

    // Password hash should be set and not equal to plain password
    assert!(user.password_hash.is_some());
    assert_ne!(user.password_hash.as_ref().unwrap(), "secure_password_123");
    assert!(user.password_hash.as_ref().unwrap().starts_with("$argon2id$"));
}

// ==================== Authentication Tests ====================

#[tokio::test]
async fn test_authenticate_valid_credentials() {
    let repo = Arc::new(MockUserRepository::new());
    let provider = LocalAuthProvider::new(repo.clone());

    // Register user first
    provider
        .register("Test User", "test@example.com", "secure_password_123", UserRole::User)
        .await
        .unwrap();

    // Authenticate
    let result = provider
        .authenticate("test@example.com", "secure_password_123")
        .await
        .unwrap();

    assert_eq!(result.user.email, "test@example.com");
}

#[tokio::test]
async fn test_authenticate_wrong_password() {
    let repo = Arc::new(MockUserRepository::new());
    let provider = LocalAuthProvider::new(repo.clone());

    provider
        .register("Test User", "test@example.com", "correct_password_123", UserRole::User)
        .await
        .unwrap();

    let result = provider
        .authenticate("test@example.com", "wrong_password")
        .await;

    assert!(matches!(result, Err(AppError::Unauthorized(_))));
}

#[tokio::test]
async fn test_authenticate_nonexistent_user() {
    let repo = Arc::new(MockUserRepository::new());
    let provider = LocalAuthProvider::new(repo.clone());

    let result = provider
        .authenticate("nonexistent@example.com", "password123")
        .await;

    assert!(matches!(result, Err(AppError::Unauthorized(_))));
}

#[tokio::test]
async fn test_authenticate_case_insensitive_email() {
    let repo = Arc::new(MockUserRepository::new());
    let provider = LocalAuthProvider::new(repo.clone());

    provider
        .register("Test User", "test@example.com", "secure_password_123", UserRole::User)
        .await
        .unwrap();

    // Authenticate with different case
    let result = provider
        .authenticate("TEST@EXAMPLE.COM", "secure_password_123")
        .await
        .unwrap();

    assert_eq!(result.user.email, "test@example.com");
}

#[tokio::test]
async fn test_authenticate_external_user_without_password_fails() {
    // Create an external user (Google) without password
    let external_user = User::new_external(
        "Google User".to_string(),
        "google@example.com".to_string(),
        AuthProviderType::Google,
        "google-sub-123".to_string(),
        UserRole::User,
    );

    let repo = Arc::new(MockUserRepository::with_user(external_user));
    let provider = LocalAuthProvider::new(repo.clone());

    let result = provider
        .authenticate("google@example.com", "any_password")
        .await;

    assert!(matches!(result, Err(AppError::Unauthorized(_))));
}

// ==================== Provider Name Test ====================

#[tokio::test]
async fn test_provider_name() {
    let repo = Arc::new(MockUserRepository::new());
    let provider = LocalAuthProvider::new(repo);

    assert_eq!(provider.name(), "local");
}

// ==================== Password Hasher Access Test ====================

#[tokio::test]
async fn test_password_hasher_access() {
    let repo = Arc::new(MockUserRepository::new());
    let provider = LocalAuthProvider::new(repo);

    let hasher = provider.password_hasher();
    let hash = hasher.hash("test_password").unwrap();

    assert!(hasher.verify("test_password", &hash).unwrap());
}

// ==================== Edge Cases ====================

#[tokio::test]
async fn test_register_with_unicode_name() {
    let repo = Arc::new(MockUserRepository::new());
    let provider = LocalAuthProvider::new(repo.clone());

    let user = provider
        .register("Müller Größe", "unicode@example.com", "password_123456", UserRole::User)
        .await
        .unwrap();

    assert_eq!(user.name, "Müller Größe");
}

#[tokio::test]
async fn test_authenticate_with_special_password_chars() {
    let repo = Arc::new(MockUserRepository::new());
    let provider = LocalAuthProvider::new(repo.clone());

    let special_password = "P@$$w0rd!#%^&*()";

    provider
        .register("Test User", "test@example.com", special_password, UserRole::User)
        .await
        .unwrap();

    let result = provider
        .authenticate("test@example.com", special_password)
        .await
        .unwrap();

    assert_eq!(result.user.email, "test@example.com");
}

#[tokio::test]
async fn test_multiple_users_independent() {
    let repo = Arc::new(MockUserRepository::new());
    let provider = LocalAuthProvider::new(repo.clone());

    // Register multiple users
    provider
        .register("User 1", "user1@example.com", "password_user_1", UserRole::User)
        .await
        .unwrap();

    provider
        .register("User 2", "user2@example.com", "password_user_2", UserRole::Admin)
        .await
        .unwrap();

    // Authenticate each with their own password
    let result1 = provider.authenticate("user1@example.com", "password_user_1").await;
    let result2 = provider.authenticate("user2@example.com", "password_user_2").await;

    assert!(result1.is_ok());
    assert!(result2.is_ok());

    // Cross-authentication should fail
    let cross1 = provider.authenticate("user1@example.com", "password_user_2").await;
    let cross2 = provider.authenticate("user2@example.com", "password_user_1").await;

    assert!(cross1.is_err());
    assert!(cross2.is_err());
}
