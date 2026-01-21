//! Common test utilities and mocks

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

use syt_ek962_security_concepts::error::AppError;
use syt_ek962_security_concepts::models::{User, UserRole, AuthProviderType};
use syt_ek962_security_concepts::repository::UserRepository;

/// In-memory mock repository for testing
pub struct MockUserRepository {
    users: RwLock<HashMap<String, User>>,
}

impl MockUserRepository {
    pub fn new() -> Self {
        Self {
            users: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_user(user: User) -> Self {
        let repo = Self::new();
        repo.users.write().unwrap().insert(user.id.clone(), user);
        repo
    }

    pub fn with_users(users: Vec<User>) -> Self {
        let repo = Self::new();
        {
            let mut map = repo.users.write().unwrap();
            for user in users {
                map.insert(user.id.clone(), user);
            }
        }
        repo
    }
}

impl Default for MockUserRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserRepository for MockUserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, AppError> {
        let users = self.users.read().unwrap();
        Ok(users.get(id).filter(|u| u.is_active).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let users = self.users.read().unwrap();
        let email_lower = email.to_lowercase();
        Ok(users
            .values()
            .find(|u| u.email.to_lowercase() == email_lower && u.is_active)
            .cloned())
    }

    async fn find_by_external_id(
        &self,
        provider: &str,
        external_id: &str,
    ) -> Result<Option<User>, AppError> {
        let users = self.users.read().unwrap();
        Ok(users
            .values()
            .find(|u| {
                u.auth_provider.to_string() == provider
                    && u.external_id.as_deref() == Some(external_id)
                    && u.is_active
            })
            .cloned())
    }

    async fn create(&self, user: &User) -> Result<(), AppError> {
        let mut users = self.users.write().unwrap();

        // Check for duplicate email
        if users.values().any(|u| u.email.to_lowercase() == user.email.to_lowercase()) {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        users.insert(user.id.clone(), user.clone());
        Ok(())
    }

    async fn update(&self, user: &User) -> Result<(), AppError> {
        let mut users = self.users.write().unwrap();
        if users.contains_key(&user.id) {
            users.insert(user.id.clone(), user.clone());
            Ok(())
        } else {
            Err(AppError::NotFound("User not found".to_string()))
        }
    }

    async fn delete(&self, id: &str) -> Result<(), AppError> {
        let mut users = self.users.write().unwrap();
        if let Some(user) = users.get_mut(id) {
            user.is_active = false;
            Ok(())
        } else {
            Err(AppError::NotFound("User not found".to_string()))
        }
    }

    async fn is_empty(&self) -> Result<bool, AppError> {
        let users = self.users.read().unwrap();
        Ok(users.values().filter(|u| u.is_active).count() == 0)
    }

    async fn count(&self) -> Result<i64, AppError> {
        let users = self.users.read().unwrap();
        Ok(users.values().filter(|u| u.is_active).count() as i64)
    }
}

/// Helper to create a test user with password hash
#[allow(dead_code)]
pub fn create_test_user(email: &str, password_hash: &str, role: UserRole) -> User {
    User::new_local(
        "Test User".to_string(),
        email.to_string(),
        password_hash.to_string(),
        role,
    )
}

/// Helper to create an external user (Google/AD)
#[allow(dead_code)]
pub fn create_external_user(
    email: &str,
    provider: AuthProviderType,
    external_id: &str,
    role: UserRole,
) -> User {
    User::new_external(
        "External User".to_string(),
        email.to_string(),
        provider,
        external_id.to_string(),
        role,
    )
}

/// Test JWT configuration
pub fn test_jwt_config() -> syt_ek962_security_concepts::config::JwtConfig {
    syt_ek962_security_concepts::config::JwtConfig {
        secret: "test_secret_key_for_testing_at_least_32_chars".to_string(),
        expiration_secs: 3600,
        issuer: "test-auth-service".to_string(),
    }
}
