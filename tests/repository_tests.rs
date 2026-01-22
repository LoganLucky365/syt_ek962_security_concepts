//! Tests for UserRepository implementations

mod common;

use syt_ek962_security_concepts::models::{User, UserRole, AuthProviderType};
use syt_ek962_security_concepts::repository::UserRepository;
use syt_ek962_security_concepts::error::AppError;

use common::MockUserRepository;

// ==================== Create Tests ====================

#[tokio::test]
async fn test_create_user() {
    let repo = MockUserRepository::new();

    let user = User::new_local(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hash".to_string(),
        UserRole::User,
    );

    let result = repo.create(&user).await;
    assert!(result.is_ok());

    // Verify user can be found
    let found = repo.find_by_id(&user.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().email, "test@example.com");
}

#[tokio::test]
async fn test_create_duplicate_email_fails() {
    let repo = MockUserRepository::new();

    let user1 = User::new_local(
        "User 1".to_string(),
        "same@example.com".to_string(),
        "hash".to_string(),
        UserRole::User,
    );

    let user2 = User::new_local(
        "User 2".to_string(),
        "same@example.com".to_string(),
        "hash".to_string(),
        UserRole::User,
    );

    repo.create(&user1).await.unwrap();
    let result = repo.create(&user2).await;

    assert!(matches!(result, Err(AppError::Conflict(_))));
}

// ==================== Find By ID Tests ====================

#[tokio::test]
async fn test_find_by_id_existing() {
    let user = User::new_local(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hash".to_string(),
        UserRole::User,
    );
    let user_id = user.id.clone();

    let repo = MockUserRepository::with_user(user);

    let found = repo.find_by_id(&user_id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, user_id);
}

#[tokio::test]
async fn test_find_by_id_nonexistent() {
    let repo = MockUserRepository::new();

    let found = repo.find_by_id("nonexistent-id").await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_find_by_id_inactive_user() {
    let mut user = User::new_local(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hash".to_string(),
        UserRole::User,
    );
    let user_id = user.id.clone();
    user.is_active = false;

    let repo = MockUserRepository::with_user(user);

    let found = repo.find_by_id(&user_id).await.unwrap();
    assert!(found.is_none());
}

// ==================== Find By Email Tests ====================

#[tokio::test]
async fn test_find_by_email_existing() {
    let user = User::new_local(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hash".to_string(),
        UserRole::User,
    );

    let repo = MockUserRepository::with_user(user);

    let found = repo.find_by_email("test@example.com").await.unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn test_find_by_email_case_insensitive() {
    let user = User::new_local(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hash".to_string(),
        UserRole::User,
    );

    let repo = MockUserRepository::with_user(user);

    let found = repo.find_by_email("TEST@EXAMPLE.COM").await.unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn test_find_by_email_nonexistent() {
    let repo = MockUserRepository::new();

    let found = repo.find_by_email("nonexistent@example.com").await.unwrap();
    assert!(found.is_none());
}

// ==================== Find By External ID Tests ====================

#[tokio::test]
async fn test_find_by_external_id_google() {
    let user = User::new_external(
        "Google User".to_string(),
        "google@example.com".to_string(),
        AuthProviderType::Google,
        "google-sub-123".to_string(),
        UserRole::User,
    );

    let repo = MockUserRepository::with_user(user);

    let found = repo.find_by_external_id("google", "google-sub-123").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().email, "google@example.com");
}

#[tokio::test]
async fn test_find_by_external_id_active_directory() {
    let user = User::new_external(
        "AD User".to_string(),
        "ad@company.com".to_string(),
        AuthProviderType::ActiveDirectory,
        "ad-username".to_string(),
        UserRole::User,
    );

    let repo = MockUserRepository::with_user(user);

    let found = repo.find_by_external_id("activedirectory", "ad-username").await.unwrap();
    assert!(found.is_some());
}

#[tokio::test]
async fn test_find_by_external_id_wrong_provider() {
    let user = User::new_external(
        "Google User".to_string(),
        "google@example.com".to_string(),
        AuthProviderType::Google,
        "google-sub-123".to_string(),
        UserRole::User,
    );

    let repo = MockUserRepository::with_user(user);

    // Search with wrong provider
    let found = repo.find_by_external_id("activedirectory", "google-sub-123").await.unwrap();
    assert!(found.is_none());
}

// ==================== Update Tests ====================

#[tokio::test]
async fn test_update_user() {
    let mut user = User::new_local(
        "Original Name".to_string(),
        "test@example.com".to_string(),
        "hash".to_string(),
        UserRole::User,
    );
    let user_id = user.id.clone();

    let repo = MockUserRepository::with_user(user.clone());

    user.name = "Updated Name".to_string();
    repo.update(&user).await.unwrap();

    let found = repo.find_by_id(&user_id).await.unwrap().unwrap();
    assert_eq!(found.name, "Updated Name");
}

#[tokio::test]
async fn test_update_nonexistent_user() {
    let repo = MockUserRepository::new();

    let user = User::new_local(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hash".to_string(),
        UserRole::User,
    );

    let result = repo.update(&user).await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

// ==================== Delete Tests ====================

#[tokio::test]
async fn test_delete_user_soft_delete() {
    let user = User::new_local(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hash".to_string(),
        UserRole::User,
    );
    let user_id = user.id.clone();

    let repo = MockUserRepository::with_user(user);

    // Delete user
    repo.delete(&user_id).await.unwrap();

    // User should no longer be found (soft deleted)
    let found = repo.find_by_id(&user_id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_delete_nonexistent_user() {
    let repo = MockUserRepository::new();

    let result = repo.delete("nonexistent-id").await;
    assert!(matches!(result, Err(AppError::NotFound(_))));
}

// ==================== Count and Empty Tests ====================

#[tokio::test]
async fn test_is_empty_true() {
    let repo = MockUserRepository::new();

    assert!(repo.is_empty().await.unwrap());
}

#[tokio::test]
async fn test_is_empty_false() {
    let user = User::new_local(
        "Test User".to_string(),
        "test@example.com".to_string(),
        "hash".to_string(),
        UserRole::User,
    );

    let repo = MockUserRepository::with_user(user);

    assert!(!repo.is_empty().await.unwrap());
}

#[tokio::test]
async fn test_count_zero() {
    let repo = MockUserRepository::new();

    assert_eq!(repo.count().await.unwrap(), 0);
}

#[tokio::test]
async fn test_count_multiple_users() {
    let users = vec![
        User::new_local(
            "User 1".to_string(),
            "user1@example.com".to_string(),
            "hash".to_string(),
            UserRole::User,
        ),
        User::new_local(
            "User 2".to_string(),
            "user2@example.com".to_string(),
            "hash".to_string(),
            UserRole::User,
        ),
        User::new_local(
            "User 3".to_string(),
            "user3@example.com".to_string(),
            "hash".to_string(),
            UserRole::Admin,
        ),
    ];

    let repo = MockUserRepository::with_users(users);

    assert_eq!(repo.count().await.unwrap(), 3);
}

#[tokio::test]
async fn test_count_excludes_inactive() {
    let user1 = User::new_local(
        "Active User".to_string(),
        "active@example.com".to_string(),
        "hash".to_string(),
        UserRole::User,
    );

    let mut user2 = User::new_local(
        "Inactive User".to_string(),
        "inactive@example.com".to_string(),
        "hash".to_string(),
        UserRole::User,
    );
    user2.is_active = false;

    let repo = MockUserRepository::with_users(vec![user1, user2]);

    assert_eq!(repo.count().await.unwrap(), 1);
}

// ==================== Multiple Provider Tests ====================

#[tokio::test]
async fn test_multiple_providers_same_email_different_external_id() {
    let google_user = User::new_external(
        "User".to_string(),
        "user@example.com".to_string(),
        AuthProviderType::Google,
        "google-123".to_string(),
        UserRole::User,
    );

    let repo = MockUserRepository::with_user(google_user);

    // Find by Google external ID
    let found = repo.find_by_external_id("google", "google-123").await.unwrap();
    assert!(found.is_some());

    // Should not find with different provider
    let not_found = repo.find_by_external_id("activedirectory", "google-123").await.unwrap();
    assert!(not_found.is_none());
}
